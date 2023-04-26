//! Organisms that are underwater should eventually drown and die.
use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{Height, MapGeometry, TilePos},
    structures::{commands::StructureCommandsExt, structure_manifest::Structure},
    units::unit_manifest::Unit,
};

use super::Organism;

/// Kills all organisms that are underwater.
pub(super) fn drown(
    unit_query: Query<(Entity, &TilePos), With<Id<Unit>>>,
    structure_query: Query<&TilePos, (With<Id<Structure>>, With<Organism>)>,
    map_geometry: Res<MapGeometry>,
    mut commands: Commands,
) {
    /// The water height at which units and structures drown.
    // TODO: make drowning characteristics customizable on a per strain basis
    const DROWNING_HEIGHT: Height = Height(2);

    for (entity, &tile_pos) in unit_query.iter() {
        if let Some(water_height) = map_geometry.get_surface_water_height(tile_pos) {
            if water_height >= DROWNING_HEIGHT {
                commands.entity(entity).despawn_recursive();
            }
        }
    }

    for &tile_pos in structure_query.iter() {
        if let Some(water_height) = map_geometry.get_surface_water_height(tile_pos) {
            if water_height >= DROWNING_HEIGHT {
                commands.despawn_structure(tile_pos);
            }
        }
    }
}
