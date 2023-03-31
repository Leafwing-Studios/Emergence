//! The patterns and mechanisms of how organisms change over time.

use bevy::prelude::*;
use leafwing_abilities::prelude::Pool;

use crate::{
    asset_management::manifest::Id,
    player_interaction::clipboard::ClipboardData,
    simulation::{
        geometry::{Facing, MapGeometry, TilePos},
        time::{Days, TimePool},
    },
    structures::{commands::StructureCommandsExt, structure_manifest::StructureManifest},
    terrain::terrain_manifest::Terrain,
    units::{
        unit_assets::UnitHandles,
        unit_manifest::{Unit, UnitManifest},
        UnitBundle,
    },
};

use super::{
    energy::{Energy, EnergyPool},
    OrganismId,
};

/// How this organism can grow, change and transform over time.
///
/// This represents a local view of the graph.
#[derive(Component, Debug, Clone)]
pub(crate) struct Lifecycle {
    /// The forms that this organism can turn into, and their triggering conditions.
    life_paths: Vec<LifePath>,
}

impl Lifecycle {
    /// The simplest lifecycle: nothing ever changes.
    pub(crate) const STATIC: Lifecycle = Lifecycle {
        life_paths: Vec::new(),
    };

    /// Creates a new [`Lifecycle`] from an ordered list of [`LifePath`].
    ///
    /// Earlier lifepaths will be prioritized for transformation if multiple conditions are met simultaneously.
    pub(crate) fn new(life_paths: Vec<LifePath>) -> Self {
        Lifecycle { life_paths }
    }

    /// Returns the [`OrganismId`] the list of completed [`LifePath`], if any.
    ///
    /// These are prioritized in the order they were added to the lifecycle.
    pub(crate) fn new_forms(&self) -> Vec<OrganismId> {
        self.life_paths
            .iter()
            .filter(|life_path| life_path.is_complete())
            .map(|life_path| life_path.new_form)
            .collect()
    }

    /// Records any energy gained, storing the results in any [`LifePath`]s that care about this.
    ///
    /// Energy gained will always count, even if it overflows the organism's cap.
    pub(crate) fn record_energy_gained(&mut self, energy: Energy) {
        for life_path in &mut self.life_paths {
            if let Some(energy_pool) = &mut life_path.energy_required {
                let proposed = energy_pool.current() + energy;
                energy_pool.set_current(proposed);
            }
        }
    }

    /// Records any elapsed in-game time, storing the results in any [`LifePath`]s that care about this.
    pub(crate) fn record_elapsed_time(&mut self, delta_days: Days) {
        for life_path in &mut self.life_paths {
            if let Some(time_pool) = &mut life_path.time_required {
                let proposed = time_pool.current() + delta_days;
                time_pool.set_current(proposed);
            }
        }
    }

    /// Pretty formatting for this type
    pub(crate) fn display(
        &self,
        structure_manifest: &StructureManifest,
        unit_manifest: &UnitManifest,
    ) -> String {
        let mut string = String::new();
        for life_path in &self.life_paths {
            string += &format!("\n{}", life_path.display(structure_manifest, unit_manifest));
        }
        string
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Lifecycle::STATIC
    }
}

/// A path from one organism to another form.
///
/// Units will transform once all of their non-`None` conditions are met.
#[derive(Debug, Clone)]
pub(crate) struct LifePath {
    /// The form that this organism will take once all of the conditions are met.
    pub(crate) new_form: OrganismId,
    /// The amount of energy that must be produced before we can transform.
    pub(crate) energy_required: Option<EnergyPool>,
    /// The amount of time that must pass before we can transform.
    pub(crate) time_required: Option<TimePool>,
}

impl LifePath {
    /// Have all of the prerequisites been met to transform?
    pub(crate) fn is_complete(&self) -> bool {
        // All conditions must be true in order for the life path to be complete
        let mut ready = true;
        if let Some(energy_pool) = &self.energy_required {
            ready &= energy_pool.is_full();
        };

        if let Some(time_pool) = &self.time_required {
            ready &= time_pool.is_full();
        };

        ready
    }

    /// Pretty formatting for this type
    pub(crate) fn display(
        &self,
        structure_manifest: &StructureManifest,
        unit_manifest: &UnitManifest,
    ) -> String {
        let mut string = String::new();

        if let Some(energy_pool) = &self.energy_required {
            string += &format!("{}/{} energy", energy_pool.current(), energy_pool.max());
        }

        if let Some(time_pool) = &self.time_required {
            string += &format!("{:.2}/{:.2} days", time_pool.current().0, time_pool.max().0);
        }

        string += &format!(
            "-> {}",
            self.new_form.display(structure_manifest, unit_manifest)
        );

        string
    }
}

/// Checks if lifecycles are complete, and transitions the organism to that form.
pub(super) fn transform_when_lifecycle_complete(
    query: Query<(Entity, &Lifecycle, &TilePos, &Facing, Option<&Id<Unit>>)>,
    structure_manifest: Res<StructureManifest>,
    unit_manifest: Res<UnitManifest>,
    unit_handles: Res<UnitHandles>,
    map_geometry: Res<MapGeometry>,
    terrain_query: Query<&Id<Terrain>>,
    mut commands: Commands,
) {
    for (entity, lifecycle, &tile_pos, &facing, maybe_unit) in query.iter() {
        for new_form in lifecycle.new_forms() {
            // Make sure that there's a valid place to spawn the new form.
            if let OrganismId::Structure(structure_id) = new_form {
                let variety = structure_manifest.get(structure_id);
                let footprint = variety.footprint.rotated(facing);
                let allowed_terrain_types = &variety.allowed_terrain_types();

                if map_geometry.can_build(
                    tile_pos,
                    footprint,
                    &terrain_query,
                    allowed_terrain_types,
                ) {
                    // Look for another viable form to transform into.
                    continue;
                }
            }

            // Cleanup is handled on the basis of what this organism *currently* is.
            if maybe_unit.is_some() {
                commands.entity(entity).despawn_recursive();
            } else {
                commands.despawn_structure(tile_pos);
            }

            match new_form {
                OrganismId::Structure(structure_id) => {
                    let data = ClipboardData {
                        structure_id,
                        facing,
                        active_recipe: structure_manifest
                            .get(structure_id)
                            .starting_recipe()
                            .clone(),
                    };
                    commands.spawn_structure(tile_pos, data);
                }
                OrganismId::Unit(unit_id) => {
                    let unit_data = unit_manifest.get(unit_id).clone();

                    commands.spawn(UnitBundle::new(
                        unit_id,
                        tile_pos,
                        unit_data,
                        &unit_handles,
                        &map_geometry,
                    ));
                }
            }

            // We only want to transform once, so we break out of the loop.
            break;
        }
    }
}
