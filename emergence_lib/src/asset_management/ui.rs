//! Loads and manages asset state for in-game UI

use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use super::{
    manifest::{Id, Structure},
    Loadable,
};

/// Stores all structural elements of the UI: buttons, frames, widgets and so on
#[derive(Resource)]
pub(crate) struct UiElements {
    /// The background image used by hex menus
    pub(crate) hex_menu_background: Handle<Image>,
}

impl FromWorld for UiElements {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        UiElements {
            hex_menu_background: asset_server.load("ui/hex-menu-background.png"),
        }
    }
}

impl Loadable for UiElements {
    fn load_state(&self, asset_server: &AssetServer) -> bevy::asset::LoadState {
        asset_server.get_load_state(&self.hex_menu_background)
    }
}

/// Stores all structural elements of the UI: buttons, frames, widgets and so on
#[derive(Resource)]
pub(crate) struct Icons {
    /// The background image used by hex menus
    pub(crate) structures: HashMap<Id<Structure>, Handle<Image>>,
}

impl FromWorld for Icons {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut structures = HashMap::new();

        // TODO: discover this from the file directory
        let structure_names = vec!["acacia", "leuco", "ant_hive", "hatchery"];

        for id in structure_names {
            let structure_id = Id::from_string_id(id);
            let structure_path = format!("structures/{id}.gltf#Scene0");
            let scene = asset_server.load(structure_path);
            structures.insert(structure_id, scene);
        }

        Icons { structures }
    }
}

impl Loadable for Icons {
    fn load_state(&self, asset_server: &AssetServer) -> bevy::asset::LoadState {
        for (structure, icon_handle) in &self.structures {
            let load_state = asset_server.get_load_state(icon_handle);
            info!("{structure:?}'s icon is {load_state:?}");

            if load_state != LoadState::Loaded {
                return load_state;
            }
        }

        LoadState::Loaded
    }
}
