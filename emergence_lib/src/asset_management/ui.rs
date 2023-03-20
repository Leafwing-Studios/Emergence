//! Loads and manages asset state for in-game UI

use bevy::{asset::LoadState, prelude::*, utils::HashMap};
use core::fmt::Debug;
use core::hash::Hash;

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

/// Stores the icons of type `D`.
#[derive(Resource)]
pub(crate) struct Icons<D: Send + Sync + 'static> {
    /// The map used to look-up handles
    map: HashMap<D, Handle<Image>>,
}

impl<D: Send + Sync + 'static + Hash + Eq> Icons<D> {
    /// Returns a weakly cloned handle to the image of the icon corresponding to `structure_id`.
    pub(crate) fn get(&self, structure_id: D) -> Handle<Image> {
        self.map.get(&structure_id).unwrap().clone_weak()
    }
}

impl FromWorld for Icons<Id<Structure>> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut map = HashMap::new();

        // TODO: discover this from the file directory
        let structure_names = vec!["acacia", "leuco", "ant_hive", "hatchery"];

        for id in structure_names {
            let structure_id = Id::from_name(id);
            let structure_path = format!("icons/structures/{id}.png");
            let scene = asset_server.load(structure_path);
            map.insert(structure_id, scene);
        }

        Icons { map }
    }
}

impl<D: Send + Sync + Debug + 'static> Loadable for Icons<D>
where
    Icons<D>: FromWorld,
{
    fn load_state(&self, asset_server: &AssetServer) -> bevy::asset::LoadState {
        for (structure, icon_handle) in &self.map {
            let load_state = asset_server.get_load_state(icon_handle);
            info!("{structure:?}'s icon is {load_state:?}");

            if load_state != LoadState::Loaded {
                return load_state;
            }
        }

        LoadState::Loaded
    }
}
