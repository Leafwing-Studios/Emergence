//! Controls what is being visualized on the terrain by the [`TileOverlay`].

use bevy::prelude::*;

use crate::{
    asset_management::manifest::{Id, ItemManifest, StructureManifest, UnitManifest},
    infovis::TileOverlay,
    signals::SignalType,
};

pub(super) struct OverlayMenuPlugin;

impl Plugin for OverlayMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(select_overlay);
    }
}

#[derive(Resource)]
struct OverlayMenu {}

/// Controls the overlay that is currently being displayed based on UI interactions.
fn select_overlay(
    // FIXME: use an actual UI widget for this...
    keyboard_input: Res<Input<KeyCode>>,
    mut tile_overlay: ResMut<TileOverlay>,
) {
    if keyboard_input.just_pressed(KeyCode::Grave) {
        tile_overlay.visualized_signal = Some(SignalType::Push(Id::from_name("acacia_leaf")));
    }
}
