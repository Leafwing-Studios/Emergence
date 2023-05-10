//! Organisms that are underwater should eventually drown and die.
use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{Height, TilePos},
    structures::{commands::StructureCommandsExt, structure_manifest::Structure},
    units::unit_manifest::Unit,
    water::WaterTable,
};

use super::Organism;

/// Kills all organisms that are underwater.
pub(super) fn drown(
    unit_query: Query<(Entity, &TilePos), With<Id<Unit>>>,
    structure_query: Query<&TilePos, (With<Id<Structure>>, With<Organism>)>,
    water_table: Res<WaterTable>,
    mut commands: Commands,
) {
    for (entity, &tile_pos) in unit_query.iter() {
        let water_depth = water_table.surface_water_depth(tile_pos);
        if water_depth > Height::WADING_DEPTH {
            commands.entity(entity).despawn_recursive();
        }
    }

    for &tile_pos in structure_query.iter() {
        let water_depth = water_table.surface_water_depth(tile_pos);
        if water_depth > Height::WADING_DEPTH {
            commands.despawn_structure(tile_pos);
        }
    }
}
