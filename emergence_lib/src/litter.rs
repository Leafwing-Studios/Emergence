//! Logic and components for littered items.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::utils::Duration;
use hexx::{Direction, HexLayout};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

use crate::{
    crafting::{inventories::StorageInventory, item_tags::ItemKind},
    geometry::{direction_from_angle, DiscreteHeight, Height, MapGeometry, VoxelPos},
    items::item_manifest::ItemManifest,
    signals::{Emitter, SignalStrength, SignalType},
    structures::{logistic_buildings::AbsorbsItems, Footprint},
    water::{FlowVelocity, WaterDepth},
};

/// The components needed to track littered items in a single [`VoxelPos`].
#[derive(Bundle)]
struct LitterBundle {
    /// The items that are littered.
    litter: Litter,
    /// The position of the litter.
    voxel_pos: VoxelPos,
    /// The scene used to display the litter.
    scene_bundle: SceneBundle,
    /// Is this litter currently floating?
    floating: Floating,
}

/// Items that are littered without a container.
///
/// This component is tracked on a per-tile basis.
#[derive(Component, Clone, Debug, Deref, DerefMut)]
pub(crate) struct Litter {
    /// The items that are littered on the ground.
    pub(crate) contents: StorageInventory,
}

/// Is this litter currently floating?
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Floating(bool);

impl Litter {
    /// The pretty formatting for the litter stored here.
    pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
        let mut display = String::new();

        let item_slot = self.contents.iter().next().unwrap();

        display.push_str(&item_slot.display(item_manifest));

        display
    }
}

impl Default for Litter {
    fn default() -> Self {
        Litter {
            contents: StorageInventory::new(1, None),
        }
    }
}

/// Updates the signals produced by litter.
pub(super) fn set_litter_emitters(mut query: Query<(&mut Emitter, Ref<Litter>)>) {
    for (mut emitter, litter) in query.iter_mut() {
        if litter.is_changed() {
            emitter.signals.clear();
            for item_slot in litter.contents.iter() {
                let item_kind = ItemKind::Single(item_slot.item_id());

                let signal_type = match litter.contents.is_full() {
                    true => SignalType::Push(item_kind),
                    false => SignalType::Contains(item_kind),
                };
                let signal_strength = SignalStrength::new(10.);

                emitter.signals.push((signal_type, signal_strength));
            }
        }
    }
}

/// The set of systems that update terrain emitters.
#[derive(SystemSet, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct LitterEmitters;

/// Litter entities with empty content should be despawned.
pub(super) fn clear_empty_litter(query: Query<(Entity, &Litter)>, mut commands: Commands) {
    for (entity, litter) in query.iter() {
        if litter.contents.is_empty() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Make litter in tiles submerged by water float (and stop it from floating when there's no water).
pub(super) fn make_litter_float(
    mut query: Query<(&mut Floating, &mut VoxelPos), With<Litter>>,
    terrain_query: Query<(&VoxelPos, &WaterDepth), Without<Litter>>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (mut floating, voxel_pos) in query.iter_mut() {
        let terrain_entity = map_geometry.get_terrain(voxel_pos.hex).unwrap();
        let (terrain_pos, water_depth) = terrain_query.get(terrain_entity).unwrap();

        if let WaterDepth::Flooded(..) = water_depth {
            // TODO: branch based on the item's density
            floating.set_if_neq(Floating(true));
            let water_table_height = water_depth.water_table_height(terrain_pos.height());

            // We need to go one tile higher, otherwise we'd share a tile with the terrain when the water depth approaches zero
            let top_of_water = DiscreteHeight::from(water_table_height).above();

            let proposed = VoxelPos {
                hex: voxel_pos.hex,
                height: top_of_water,
            };
            map_geometry.move_litter(voxel_pos, proposed);
        } else {
            floating.set_if_neq(Floating(false));
            let proposed = VoxelPos {
                hex: voxel_pos.hex,
                // Place litter on top of the terrain, not inside of it
                height: terrain_pos.above().height,
            };
            map_geometry.move_litter(voxel_pos, proposed);
        }
    }
}

/// The direction and time remaining for a piece of litter to drift with the current.
#[derive(Component, Default)]
pub(super) struct LitterDrift {
    /// The direction the litter is drifting.
    pub(super) direction: Option<Direction>,
    /// The time remaining for the litter to drift.
    pub(super) timer: Timer,
}

impl LitterDrift {
    /// Starts the timer for the litter to drift with the current.
    pub(super) fn start(&mut self, direction: Direction, required_time: Duration) {
        // This attempts to avoid allocations by reusing the timer
        self.direction = Some(direction);
        self.timer.set_duration(required_time);
        self.timer.reset();
    }

    /// Ends the timer for the litter to drift with the current.
    pub(super) fn finish(&mut self) {
        self.direction = None;
        // The timer is reset when starting, so we don't need to do anything here
    }
}

/// Carries floating litter along with the surface water current.
pub(super) fn carry_floating_litter_with_current(
    mut terrain_query: Query<(
        &mut VoxelPos,
        &mut LitterDrift,
        &WaterDepth,
        &FlowVelocity,
        &Floating,
    )>,
    water_height_query: Query<(&VoxelPos, &WaterDepth), Without<LitterDrift>>,
    net_query: Query<&Footprint, With<AbsorbsItems>>,
    fixed_time: Res<FixedTime>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    /// Controls how fast litter drifts with the current
    ///
    /// Higher values mean litter drifts faster.
    /// This must be greater than 0.
    const ITEM_DRIFT_RATE: f32 = 1e2;

    /// Controls how much litter varies relative to the current direction
    ///
    /// This is the standard deviation of the normal distribution used to determine the drift angle, and is in units of radians.
    /// This must be greater than 0.
    const DRIFT_DEVIATION: f32 = 0.5;

    /// The maximum amount of time in seconds litter can take to drift a single step
    ///
    /// Must be greater than 0.
    /// If this is larger than the maximum number of seconds that can be stored in a Duration, the app will panic.
    const MAX_DRIFT_TIME: f32 = 10.0;

    let delta_time = fixed_time.period;
    let rng = &mut thread_rng();
    let normal_distribution = Normal::new(0.0, DRIFT_DEVIATION).unwrap();

    for (voxel_pos, mut litter_drift, water_depth, flow_velocity, floating) in
        terrain_query.iter_mut()
    {
        // Don't both computing drift if it's not floating
        if !floating.0 {
            continue;
        }

        // Don't cause items to drift out of overfull nets
        if let Some(structure_entity) = map_geometry.get_structure(*voxel_pos) {
            if net_query.get(structure_entity).is_ok() {
                continue;
            }
        }

        if let WaterDepth::Flooded(surface_water_depth) = water_depth {
            litter_drift.timer.tick(delta_time);

            let flow_direction = flow_velocity.direction()
                // Truncate the noise so it never makes the litter drift more than 90 degrees off-course
                // to prevent goods flowing upstream
                + normal_distribution.sample(rng).clamp(-TAU / 4., TAU / 4.);

            // Volume transferred = cross-sectional area * water speed * time
            // Cross-sectional area = water depth * tile area
            // Volume transferred = water depth * tile area * water speed * time
            // Water speed = volume transferred / time * tile area / water depth
            // Tile area is 1 (because we're computing on a per-tile basis), and flow_velocity is in tiles per second
            // Therefore: water speed = flow velocity / water depth
            let water_speed = flow_velocity.magnitude().0 / surface_water_depth.0;

            // If the litter is not already drifting, start it drifting
            if litter_drift.direction.is_none() {
                let direction =
                    direction_from_angle(flow_direction, HexLayout::default().orientation);
                let time_to_drift = (1. / (ITEM_DRIFT_RATE * water_speed)).min(MAX_DRIFT_TIME);

                litter_drift.start(direction, Duration::from_secs_f32(time_to_drift));
            }

            // If the litter has finished drifting, stop it drifting and move it
            if litter_drift.timer.finished() {
                if let Some(direction) = litter_drift.direction {
                    let new_voxel_pos = voxel_pos.neighbor(direction);
                    let source_height = water_depth.surface_height(voxel_pos.height());
                    let Ok(target_entity) = map_geometry.get_terrain(new_voxel_pos.hex) else { continue };

                    let Ok((target_tile_pos, target_water_depth)) =
                        water_height_query.get(target_entity) else { continue };
                    let target_height = target_water_depth.surface_height(target_tile_pos.height());

                    // Verify that we're not trying to deposit goods up a cliff or waterfall
                    // Note that this is a one-way check; we don't care if the source is higher than the target
                    if target_height - source_height <= Height::MAX_STEP {
                        map_geometry.move_litter(voxel_pos, new_voxel_pos);
                    }
                }

                litter_drift.finish();
            }
        } else {
            // PERF: we could probably avoid doing any work in this branch by more carefully tracking when things start floating
            litter_drift.finish();
        }
    }
}
