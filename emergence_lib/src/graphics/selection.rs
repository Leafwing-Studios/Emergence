//! Graphics code to display the currently selected game object.

use bevy::prelude::*;

use crate::{
    asset_management::{
        manifest::{Id, Terrain},
        terrain::TerrainHandles,
    },
    player_interaction::selection::ObjectInteraction,
};

/// Displays the overlay of the tile
pub(super) fn display_tile_overlay(
    terrain_query: Query<
        (&Children, &ObjectInteraction),
        (With<Id<Terrain>>, Changed<ObjectInteraction>),
    >,
    mut overlay_query: Query<(&mut Handle<StandardMaterial>, &mut Visibility)>,
    terrain_handles: Res<TerrainHandles>,
) {
    for (children, object_interaction) in terrain_query.iter() {
        // This is promised to be the correct entity in the initialization of the terrain's children
        let overlay_entity = children[1];

        let (mut overlay_material, mut overlay_visibility) =
            overlay_query.get_mut(overlay_entity).unwrap();

        match object_interaction {
            ObjectInteraction::None => {
                *overlay_visibility = Visibility::Hidden;
            }
            _ => {
                *overlay_visibility = Visibility::Visible;
                let new_material = terrain_handles
                    .interaction_materials
                    .get(object_interaction)
                    .unwrap()
                    .clone_weak();

                *overlay_material = new_material;
            }
        }
    }
}
