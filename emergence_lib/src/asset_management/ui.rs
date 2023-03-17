//! Loads and manages asset state for in-game UI

use bevy::prelude::*;

use super::Loadable;

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
