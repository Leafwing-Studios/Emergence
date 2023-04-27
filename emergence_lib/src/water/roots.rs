//! Datastructures and mechanics for roots, which draw water from the nearby water table.

use std::fmt::{Display, Formatter};

use hexx::shapes::hexagon;
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{Height, MapGeometry, TilePos},
    structures::structure_manifest::{Structure, StructureManifest},
};
use bevy::prelude::*;

use super::WaterTable;

/// The volume around a tile that roots can draw water from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootZone {
    /// The depth from the surface beyond which roots cannot draw water.
    pub max_depth: Height,
    /// The radius of the root zone.
    ///
    /// Water can only be drawn from tiles within this radius.
    pub radius: u32,
}

impl RootZone {
    /// Returns the set of tiles that this root zone can reach, with water above the max depth.
    fn relevant_tiles(
        &self,
        center: TilePos,
        water_table: &WaterTable,
        map_geometry: &MapGeometry,
    ) -> Vec<TilePos> {
        let hexagon = hexagon(center.hex, self.radius);
        let mut relevant_tiles = Vec::with_capacity(hexagon.len());
        for hex in hexagon {
            let tile_pos = TilePos { hex };
            if !map_geometry.is_valid(tile_pos) {
                continue;
            };

            match water_table.depth_to_water_table(tile_pos, map_geometry) {
                super::DepthToWaterTable::Flooded => relevant_tiles.push(tile_pos),
                super::DepthToWaterTable::Dry => (),
                super::DepthToWaterTable::Depth(depth) => {
                    if depth <= self.max_depth {
                        relevant_tiles.push(tile_pos);
                    }
                }
            }
        }

        relevant_tiles
    }
}

impl Display for RootZone {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Root Zone: {} tiles deep, {} tiles radius",
            self.max_depth, self.radius
        )
    }
}

/// Draws water from the water table if and only if the structure needs more water.
// PERF: we could store RootZone as a component on the structure at the cost of some memory.
// This would give us faster lookups, but force duplication.
pub(super) fn draw_water_from_roots(
    mut water_table: ResMut<WaterTable>,
    structure_query: Query<(&TilePos, &Id<Structure>)>,
    structure_manifest: Res<StructureManifest>,
    map_geometry: Res<MapGeometry>,
    fixed_time: Res<FixedTime>,
) {
    let water_draw_rate = 200.0; // TODO: make this a property of the structure.
    let water_requested = water_draw_rate * fixed_time.period.as_secs_f32();

    // TODO: only do this during CraftingState::NeedsInput
    for (&center, &structure_id) in structure_query.iter() {
        let structure_data = structure_manifest.get(structure_id);
        if let Some(root_zone) = &structure_data.root_zone {
            let relevant_tiles = root_zone.relevant_tiles(center, &water_table, &map_geometry);
            let n = relevant_tiles.len() as f32;
            let water_per_tile = Height(water_requested / n);

            for tile_pos in relevant_tiles {
                // This can ever so slightly overdraw water, but that's fine.
                // Accounting for this would significantly complicate the code.
                // Pretend it's capillary action or something!
                water_table.subtract(tile_pos, water_per_tile);
                // TODO: add water to the structure.
            }
        }
    }
}
