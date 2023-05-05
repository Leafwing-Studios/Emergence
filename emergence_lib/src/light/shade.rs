//! Shade is cast by structures and terrain based on their height and the position of the sun.

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{MapGeometry, TilePos},
    structures::structure_manifest::Structure,
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
    structure_query: Query<&TilePos, With<Id<Structure>>>,
    map_geometry: Res<MapGeometry>,
) {
    // Reset the shade for all tiles
    for mut shade in terrain_query.iter_mut() {
        shade.light_fraction = 1.0;
    }

    // Cast shade from structures to nearby tiles
    // TODO: vary this by Footprint
    for tile_pos in structure_query.iter() {
        let neighbors = tile_pos.all_valid_neighbors(&map_geometry);
        for neighbor in neighbors {
            let terrain_entity = map_geometry.get_terrain(neighbor).unwrap();
            let mut shade = terrain_query.get_mut(terrain_entity).unwrap();
            // TODO: vary this by structure type
            shade.light_fraction *= 0.5;
        }
    }

    // TODO: account for height

    // TODO: account for time of day

    // TODO: cast shade from terrain to nearby tiles
}

/// Computes the amount of light received by each tile.
pub(super) fn compute_recieved_light(
    mut terrain_query: Query<(&mut ReceivedLight, &Shade)>,
    total_light: Res<TotalLight>,
) {
    let overall_light = total_light.normalized_illuminance();

    for (mut received_light, shade) in terrain_query.iter_mut() {
        received_light.normalized_illuminance = overall_light * shade.light_fraction;
    }
}
