//! Logic and data related to marking structures for demolition.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    asset_management::manifest::Id,
    geometry::{Height, MapGeometry, VoxelPos},
    signals::{Emitter, SignalStrength, SignalType},
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
        current: VoxelPos,
        target: VoxelPos,
        structure_id: Id<Structure>,
        map_geometry: &MapGeometry,
    ) -> Option<Entity> {
        // This is only a viable target if the unit can reach it!
        if current.abs_height_diff(target) > Height::MAX_STEP {
            return None;
        }

        let entity = map_geometry.get_structure(target)?;

        let &found_structure_id = self.query.get(entity).ok()?;

        match found_structure_id == structure_id {
            true => Some(entity),
            false => None,
        }
    }
}

/// Keeps marked tiles clear by sending removal signals from structures that are marked for removal
pub(super) fn set_emitter_for_structures_to_be_demolished(
    mut structure_query: Query<(&mut Emitter, &Id<Structure>), With<MarkedForDemolition>>,
) {
    for (mut doomed_emitter, &structure_id) in structure_query.iter_mut() {
        doomed_emitter.signals = vec![(
            SignalType::Demolish(structure_id),
            SignalStrength::new(100.),
        )];
    }
}
