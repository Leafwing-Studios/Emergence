//! Graphics and animation code for structures.

use bevy::prelude::*;

use crate::{
    asset_management::{
        manifest::{Id, Structure},
        structures::StructureHandles,
    },
    player_interaction::selection::ObjectInteraction,
    structures::ghost::Ghostly,
};

/// Modifies the material of any structures based on their interaction state.
pub(super) fn change_structure_material(
    root_structure_query: Query<
        (Entity, &ObjectInteraction, Option<&Ghostly>),
        With<Id<Structure>>,
    >,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    structure_handles: Res<StructureHandles>,
) {
    for (root_entity, object_interaction, maybe_ghostly) in root_structure_query.iter() {
        for child in children.iter_descendants(root_entity) {
            if let Ok(mut material) = material_query.get_mut(child) {
                let maybe_material_handle = match maybe_ghostly {
                    Some(..) => structure_handles.ghost_materials.get(object_interaction),
                    None => structure_handles
                        .interaction_materials
                        .get(object_interaction),
                };

                // FIXME: how do we restore the materials back to their original form??
                if let Some(new_handle) = maybe_material_handle {
                    *material = new_handle.clone_weak();
                }
            }
        }
    }
}
