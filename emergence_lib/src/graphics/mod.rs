//! Rendering and animation logic.

use bevy::prelude::*;

use crate::asset_management::AssetState;

use self::{
    atmosphere::AtmospherePlugin, lighting::LightingPlugin, structures::remove_ghostly_shadows,
    terrain::manage_litter_piles,
};

mod atmosphere;
pub(crate) mod lighting;
pub(crate) mod palette;
mod structures;
mod terrain;
mod units;

/// Adds all logic required to render the game.
///
/// The game should be able to run and function without this plugin: no gameplay logic allowed!
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LightingPlugin)
            .add_plugin(AtmospherePlugin)
            .add_system(manage_litter_piles.run_if(in_state(AssetState::FullyLoaded)))
            // Run these after Update to avoid panics due to despawned entities
            .add_systems(
                (inherit_materials, remove_ghostly_shadows).in_base_set(CoreSet::PostUpdate),
            );
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
