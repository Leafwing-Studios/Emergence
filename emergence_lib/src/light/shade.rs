//! Shade is cast by structures and terrain based on their height and the position of the sun.

use crate::{
    asset_management::manifest::Id,
    construction::ghosts::Ghost,
    simulation::{
        geometry::{Height, MapGeometry, TilePos},
        time::{InGameTime, TimeOfDay},
    },
    structures::{
        structure_manifest::{Structure, StructureManifest},
        Footprint,
    },
};
use bevy::prelude::*;

use super::{NormalizedIlluminance, TotalLight};

use std::fmt::Display;

/// The amount of shade on a tile.
#[derive(Component, Clone, Debug, Default)]
pub(crate) struct Shade {
    /// How much shade is cast on this tile.
    ///
    /// This is a ratio from 0 to 1, where 0 is full shade and 1 is full sun.
    pub(crate) light_fraction: f32,
}

impl Display for Shade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.light_fraction)
    }
}

/// The amount of light currently received by a tile.
#[derive(Component, Clone, Debug, Default)]
pub(crate) struct ReceivedLight {
    /// The amount of light received by this tile.
    pub(crate) normalized_illuminance: NormalizedIlluminance,
}

impl Display for ReceivedLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.normalized_illuminance)
    }
}

/// Computes the amount of shade on each tile.
pub(super) fn compute_shade(
    mut terrain_query: Query<&mut Shade>,
    // FIXME: previews cast shadows in the game, but we only want them to be previewed to the player
    structure_query: Query<(&TilePos, &Id<Structure>), Without<Ghost>>,
    map_geometry: Res<MapGeometry>,
    in_game_time: Res<InGameTime>,
    structure_manifest: Res<StructureManifest>,
) {
    /// The fraction of light that is blocked by a single shadow.
    ///
    /// Should be between 0 and 1.
    /// A value of 0 means all light is blocked, and a value of 1 means no light is blocked.
    const SHADE_FRACTION: f32 = 0.5;

    // PERF: we can be much less aggressive about computing these values
    // They only need to be recomputed when the map geometry changes, or when the time of day changes

    // Reset the shade for all tiles
    for mut shade in terrain_query.iter_mut() {
        shade.light_fraction = 1.0;
    }

    if in_game_time.time_of_day() == TimeOfDay::Night {
        return;
    }

    // Cast shade from structures to nearby tiles
    // TODO: vary this by Footprint
    for (&center, &structure_id) in structure_query.iter() {
        let structure_data = structure_manifest.get(structure_id);
        let tiles_in_footprint = structure_data.footprint.in_world_space(center);

        for tile_pos in &tiles_in_footprint {
            for shaded_tile_pos in shaded_area(*tile_pos, &map_geometry, structure_data.height) {
                // Don't shade yourself
                if tiles_in_footprint.contains(&shaded_tile_pos) {
                    continue;
                }

                let shaded_terrain_entity = map_geometry.get_terrain(shaded_tile_pos).unwrap();
                let mut shade = terrain_query.get_mut(shaded_terrain_entity).unwrap();
                shade.light_fraction *= SHADE_FRACTION;
            }
        }
    }

    for tile_pos in map_geometry.valid_tile_positions() {
        for shaded_tile_pos in shaded_area(tile_pos, &map_geometry, Height::ZERO) {
            let shaded_terrain_entity = map_geometry.get_terrain(shaded_tile_pos).unwrap();
            let mut shade = terrain_query.get_mut(shaded_terrain_entity).unwrap();
            shade.light_fraction *= SHADE_FRACTION;
        }
    }
}

/// Computes the set of tiles that are shaded by a given object.
fn shaded_area(
    tile_pos: TilePos,
    map_geometry: &MapGeometry,
    height_of_caster: Height,
) -> Vec<TilePos> {
    let mut shaded_tiles = Vec::new();

    let originating_terrain_height = map_geometry.get_height(tile_pos).unwrap();
    // TODO: account for height of originating structure
    for neighbor in tile_pos.all_valid_neighbors(map_geometry) {
        let neighbor_terrain_height = map_geometry.get_height(neighbor).unwrap();
        if neighbor_terrain_height < originating_terrain_height + height_of_caster {
            shaded_tiles.push(neighbor);
        }
    }
    shaded_tiles
}

/// Computes the amount of light received by each tile.
pub(super) fn compute_received_light(
    mut terrain_query: Query<(&mut ReceivedLight, &Shade)>,
    total_light: Res<TotalLight>,
) {
    let overall_light = total_light.normalized_illuminance();

    for (mut received_light, shade) in terrain_query.iter_mut() {
        received_light.normalized_illuminance = overall_light * shade.light_fraction;
    }
}
