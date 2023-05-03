//! Datastructures and mechanics for roots, which draw water from the nearby water table.

use std::fmt::{Display, Formatter};

use hexx::shapes::hexagon;
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::Id,
    crafting::components::{CraftingState, InputInventory},
    items::{item_manifest::ItemManifest, ItemCount},
    simulation::geometry::{Height, MapGeometry, TilePos, Volume},
    structures::structure_manifest::{Structure, StructureManifest},
};
use bevy::prelude::*;

use super::{WaterConfig, WaterTable};

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

            match water_table.relative_water_depth(tile_pos, map_geometry) {
                super::WaterDepth::Flooded(..) => relevant_tiles.push(tile_pos),
                super::WaterDepth::Dry => (),
                super::WaterDepth::Underground(depth) => {
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
    water_config: Res<WaterConfig>,
    mut structure_query: Query<(
        &TilePos,
        &Id<Structure>,
        &CraftingState,
        &mut InputInventory,
    )>,
    structure_manifest: Res<StructureManifest>,
    item_manifest: Res<ItemManifest>,
    map_geometry: Res<MapGeometry>,
) {
    // TODO: only do this during CraftingState::NeedsInput
    for (&center, &structure_id, crafting_state, mut input_inventory) in structure_query.iter_mut()
    {
        if crafting_state != &CraftingState::NeedsInput {
            continue;
        };

        let water_items_requested = input_inventory
            .inventory()
            .remaining_space_for_item(Id::water(), &item_manifest);

        if water_items_requested == 0 {
            continue;
        };

        let water_tiles_requested = water_config.items_to_tiles(water_items_requested);

        let root_zone = match &structure_manifest.get(structure_id).root_zone {
            Some(root_zone) => root_zone,
            None => continue,
        };

        let relevant_tiles = root_zone.relevant_tiles(center, &water_table, &map_geometry);
        let n = relevant_tiles.len() as f32;
        let water_per_tile = water_tiles_requested / n;

        let mut total_water = Volume::ZERO;

        for tile_pos in relevant_tiles {
            // This can ever so slightly overdraw water, but that's fine.
            // Accounting for this would significantly complicate the code.
            // Pretend it's capillary action or something!
            total_water += water_table.remove(tile_pos, water_per_tile);
        }

        let water_items_produced = water_config.tiles_to_items(total_water);
        let _ = input_inventory.fill_with_items(
            &ItemCount::new(Id::water(), water_items_produced),
            &item_manifest,
        );
    }
}
