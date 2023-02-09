//! Asset loading for structures

use crate::structures::StructureId;
use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use super::AssetState;

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct StructureHandles {
    /// The scene for each type of structure
    pub(crate) scenes: HashMap<StructureId, Handle<Scene>>,
}

impl FromWorld for StructureHandles {
    fn from_world(world: &mut World) -> Self {
        let mut handles = StructureHandles {
            scenes: HashMap::default(),
        };

        let asset_server = world.resource::<AssetServer>();

        // TODO: discover this from the file directory
        let structure_names: Vec<String> = vec!["acacia", "leuco"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        for structure_name in structure_names {
            let structure_id = StructureId {
                id: structure_name.clone(),
            };
            let structure_path = format!("structures/{structure_name}.gltf#Scene0");
            let scene = asset_server.load(structure_path);
            handles.scenes.insert(structure_id, scene);
        }

        handles
    }
}

impl StructureHandles {
    /// How far along are we in loading these assets?
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

    /// A system that checks if these assets are loaded.
    pub(super) fn check_loaded(
        structure_handles: Res<StructureHandles>,
        asset_server: Res<AssetServer>,
        mut asset_state: ResMut<State<AssetState>>,
    ) {
        let structure_load_state = structure_handles.load_state(&asset_server);
        info!("Structures are {structure_load_state:?}");

        if structure_load_state == LoadState::Loaded {
            info!("Transitioning to AssetState::Ready");
            asset_state.set(AssetState::Ready).unwrap();
        }
    }
}
