//! Rendering and animation logic.

use bevy::prelude::*;

use crate::{asset_management::AssetState, world_gen::WorldGenState};

use self::{
    atmosphere::AtmospherePlugin, lighting::LightingPlugin, litter::render_litter_piles,
    overlay::OverlayPlugin, structures::remove_ghostly_shadows, water::WaterRenderingPlugin,
};

mod atmosphere;
pub(crate) mod lighting;
mod litter;
pub(crate) mod overlay;
pub(crate) mod palette;
mod structures;
mod units;
mod water;

/// Adds all logic required to render the game.
///
/// The game should be able to run and function without this plugin: no gameplay logic allowed!
pub struct GraphicsPlugin;

/// Systems that are used for rendering.
#[derive(SystemSet, Debug, Default, Clone, PartialEq, Eq, Hash)]
struct GraphicsSet;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LightingPlugin)
            .add_plugin(AtmospherePlugin)
            .add_plugin(WaterRenderingPlugin)
            .add_plugin(OverlayPlugin)
            .add_systems(Update, render_litter_piles.in_set(GraphicsSet))
            // Run these after Update to avoid panics due to despawned entities
            .add_systems(PostUpdate, (inherit_materials, remove_ghostly_shadows))
            .configure_set(
                Update,
                GraphicsSet
                    .run_if(in_state(AssetState::FullyLoaded))
                    .run_if(in_state(WorldGenState::Complete)),
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
