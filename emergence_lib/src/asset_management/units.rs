//! Asset loading for units

use crate::organisms::units::UnitId;
use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use super::Loadable;

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct UnitHandles {
    /// The scene for each type of structure
    pub(crate) scenes: HashMap<UnitId, Handle<Scene>>,
}

impl FromWorld for UnitHandles {
    fn from_world(world: &mut World) -> Self {
        let mut handles = UnitHandles {
            scenes: HashMap::default(),
        };

        let asset_server = world.resource::<AssetServer>();

        // TODO: discover this from the file directory
        let structure_names = vec!["ant"];

        for id in structure_names {
            let structure_id = UnitId { id };
            let structure_path = format!("units/{id}.gltf#Scene0");
            let scene = asset_server.load(structure_path);
            handles.scenes.insert(structure_id, scene);
        }

        handles
    }
}

impl Loadable for UnitHandles {
    fn load_state(&self, asset_server: &AssetServer) -> LoadState {
        for (unit, scene_handle) in &self.scenes {
            let scene_load_state = asset_server.get_load_state(scene_handle);
            info!("{unit:?}'s scene is {scene_load_state:?}");

            if scene_load_state != LoadState::Loaded {
                return scene_load_state;
            }
        }

        LoadState::Loaded
    }
}
