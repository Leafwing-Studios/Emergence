//! Logic and data related to marking structures for demolition.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    asset_management::manifest::Id,
    simulation::geometry::{MapGeometry, TilePos},
    structures::structure_manifest::Structure,
};

/// Marker component for structures that are intended to be deconstructed
#[derive(Component, Debug)]
pub(crate) struct MarkedForDemolition;

/// A query for the structures that need to be demolished.
#[derive(SystemParam)]
pub(crate) struct DemolitionQuery<'w, 's> {
    /// The contained query type.
    query: Query<'w, 's, &'static Id<Structure>, With<MarkedForDemolition>>,
}

impl<'w, 's> DemolitionQuery<'w, 's> {
    /// Is there a structure of type `structure_id` at `structure_pos` that needs to be demolished?
    ///
    /// If so, returns `Some(matching_structure_entity_that_needs_to_be_demolished)`.
    pub(crate) fn needs_demolition(
        &self,
        structure_pos: TilePos,
        structure_id: Id<Structure>,
        map_geometry: &MapGeometry,
    ) -> Option<Entity> {
        let entity = map_geometry.get_structure(structure_pos)?;

        let &found_structure_id = self.query.get(entity).ok()?;

        match found_structure_id == structure_id {
            true => Some(entity),
            false => None,
        }
    }
}
