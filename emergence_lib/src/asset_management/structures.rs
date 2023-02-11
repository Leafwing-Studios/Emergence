//! Asset loading for structures

use crate::structures::StructureId;
use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use super::Loadable;

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct StructureHandles {
    /// The scene for each type of structure
    pub(crate) scenes: HashMap<StructureId, Handle<Scene>>,
    /// The material to be used for all ghosts
    pub(crate) ghost_material: Handle<StandardMaterial>,
}

impl FromWorld for StructureHandles {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let ghost_material = materials.add(StandardMaterial {
            base_color: Color::hsla(0., 0., 0.9, 0.7),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        let mut handles = StructureHandles {
            scenes: HashMap::default(),
            ghost_material,
        };

        let asset_server = world.resource::<AssetServer>();

        // TODO: discover this from the file directory
        let structure_names = vec!["acacia", "leuco"];

        for id in structure_names {
            let structure_id = StructureId { id };
            let structure_path = format!("structures/{id}.gltf#Scene0");
            let scene = asset_server.load(structure_path);
            handles.scenes.insert(structure_id, scene);
        }

        handles
    }
}

impl Loadable for StructureHandles {
    fn load_state(&self, asset_server: &AssetServer) -> LoadState {
        for (structure, scene_handle) in &self.scenes {
            let scene_load_state = asset_server.get_load_state(scene_handle);
            info!("{structure:?}'s scene is {scene_load_state:?}");

            if scene_load_state != LoadState::Loaded {
                return scene_load_state;
            }
        }

        LoadState::Loaded
    }
}
