//! Rendering and animation logic.

use bevy::prelude::*;

use crate::{asset_management::AssetState, player_interaction::InteractionSystem};

use self::{icons::spawn_icon_camera, lighting::LightingPlugin};

pub(crate) mod icons;
mod lighting;
mod selection;
mod structures;
mod units;

/// Adds all logic required to render the game.
///
/// The game should be able to run and function without this plugin: no gameplay logic allowed!
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LightingPlugin)
            .add_startup_system(spawn_icon_camera)
            .add_system_set(
                SystemSet::on_update(AssetState::Ready).with_system(units::display_held_item),
            )
            .add_system_to_stage(CoreStage::PostUpdate, inherit_materials)
            .add_system_to_stage(CoreStage::Last, icons::generate_icons)
            .add_system(selection::display_tile_interactions.after(InteractionSystem::SelectTiles));
    }
}

/// A material that will be inherited by all children in the scene.
#[derive(Component, Debug, Deref)]
pub(crate) struct InheritedMaterial(pub(crate) Handle<StandardMaterial>);

/// Applies [`InheritedMaterial`] to all child entities recursively.
pub(super) fn inherit_materials(
    root_structure_query: Query<(Entity, &InheritedMaterial)>,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
) {
    for (root_entity, inherited_material) in root_structure_query.iter() {
        for child in children.iter_descendants(root_entity) {
            if let Ok(mut child_material) = material_query.get_mut(child) {
                *child_material = inherited_material.clone_weak();
            }
        }
    }
}
