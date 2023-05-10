//! Shade is cast by structures and terrain based on their height and the position of the sun.

use crate::{
    asset_management::manifest::Id,
    construction::ghosts::Ghost,
    simulation::{
        geometry::{Facing, Height, MapGeometry, TilePos},
        time::{InGameTime, TimeOfDay},
    },
    structures::structure_manifest::{Structure, StructureManifest},
};
use bevy::prelude::*;

use super::{Illuminance, TotalLight};

use std::fmt::Display;

/// The amount of shade on a tile.
#[derive(Component, Clone, Debug, Default)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Shade {
    /// This tile is not shaded.
    #[default]
    FullSun,
    /// This tile is partially shaded.
    PartialSun,
    /// This tile is fully shaded.
    FullShade,
}

impl Shade {
    /// Adds one level of shade to this tile.
    fn add_shade(&mut self) {
        *self = match self {
            Shade::FullSun => Shade::PartialSun,
            Shade::PartialSun => Shade::FullShade,
            Shade::FullShade => Shade::FullShade,
        };
    }

    /// Computes the amount of light recieved by a tile given the shade and total light.
    pub(crate) fn received_light(&self, total_light: &TotalLight) -> Illuminance {
        match (total_light.0, self) {
            (Illuminance::Dark, _) => Illuminance::Dark,
            (Illuminance::DimlyLit, Shade::FullSun) => Illuminance::DimlyLit,
            (Illuminance::DimlyLit, Shade::PartialSun) => Illuminance::Dark,
            (Illuminance::DimlyLit, Shade::FullShade) => Illuminance::Dark,
            (Illuminance::BrightlyLit, Shade::FullSun) => Illuminance::BrightlyLit,
            (Illuminance::BrightlyLit, Shade::PartialSun) => Illuminance::DimlyLit,
            (Illuminance::BrightlyLit, Shade::FullShade) => Illuminance::Dark,
        }
    }
}

impl Display for Shade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shade::FullSun => write!(f, "Full Sun"),
            Shade::PartialSun => write!(f, "Partial Sun"),
            Shade::FullShade => write!(f, "Full Shade"),
        }
    }
}

/// The amount of light currently received by a tile.
#[derive(Component, Clone, Debug, Default)]
pub(crate) struct ReceivedLight(pub(crate) Illuminance);

impl Display for ReceivedLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Computes the amount of shade on each tile.
pub(super) fn compute_shade(
    mut terrain_query: Query<&mut Shade>,
    // FIXME: previews cast shadows in the game, but we only want them to be previewed to the player
    structure_query: Query<(&TilePos, &Id<Structure>, &Facing), Without<Ghost>>,
    map_geometry: Res<MapGeometry>,
    in_game_time: Res<InGameTime>,
    structure_manifest: Res<StructureManifest>,
) {
    // PERF: we can be much less aggressive about computing these values
    // They only need to be recomputed when the map geometry changes, or when the time of day changes

    // Reset the shade for all tiles
    for mut shade in terrain_query.iter_mut() {
        *shade = Shade::FullSun;
    }

    if in_game_time.time_of_day() == TimeOfDay::Night {
        return;
    }

    // Cast shade from structures to nearby tiles
    for (&center, &structure_id, &facing) in structure_query.iter() {
        let structure_data = structure_manifest.get(structure_id);
        let tiles_in_footprint = structure_data
            .footprint
            .rotated(facing)
            .in_world_space(center);

        for tile_pos in &tiles_in_footprint {
            for shaded_tile_pos in shaded_area(*tile_pos, &map_geometry, structure_data.height) {
                // Don't shade yourself
                if tiles_in_footprint.contains(&shaded_tile_pos) {
                    continue;
                }

                let shaded_terrain_entity = map_geometry.get_terrain(shaded_tile_pos).unwrap();
                let mut shade = terrain_query.get_mut(shaded_terrain_entity).unwrap();
                shade.add_shade();
            }
        }
    }

    for tile_pos in map_geometry.valid_tile_positions() {
        // Don't double-count shade from tiles with structures
        if map_geometry.get_structure(tile_pos).is_some() {
            continue;
        }

        for shaded_tile_pos in shaded_area(tile_pos, &map_geometry, Height::ZERO) {
            let shaded_terrain_entity = map_geometry.get_terrain(shaded_tile_pos).unwrap();
            let mut shade = terrain_query.get_mut(shaded_terrain_entity).unwrap();
            shade.add_shade();
        }
    }
}

/// Computes the set of tiles that are shaded by a given object.
fn shaded_area(
    tile_pos: TilePos,
    map_geometry: &MapGeometry,
    height_of_caster: Height,
) -> Vec<TilePos> {
    /// The unit vector pointing away from the sun.
    const SHADOW_DIRECTION: TilePos = TilePos::new(0, 1);

    let mut shaded_tiles = Vec::new();

    let Ok(originating_terrain_height) = map_geometry.get_height(tile_pos) else { return Vec::new() };
    let total_height = originating_terrain_height + height_of_caster;
    let total_height = total_height.0.round() as i32;

    for distance_from_caster in 1..=total_height {
        let candidate = tile_pos + SHADOW_DIRECTION * distance_from_caster;

        let Ok(candidate_terrain_height) = map_geometry.get_height(candidate) else {
			continue;
		};

        // The height that a shadow can reach decreases as the distance from the caster increases
        if candidate_terrain_height.0.round() as i32 + distance_from_caster <= total_height {
            shaded_tiles.push(candidate);
        }
    }
    shaded_tiles
}

/// Computes the amount of light received by each tile.
pub(super) fn compute_received_light(
    mut terrain_query: Query<(&mut ReceivedLight, &Shade)>,
    total_light: Res<TotalLight>,
) {
    for (mut received_light, shade) in terrain_query.iter_mut() {
        received_light.0 = shade.received_light(&total_light);
    }
}
