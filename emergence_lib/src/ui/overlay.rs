//! Controls what is being visualized on the terrain by the [`TileOverlay`].

use bevy::prelude::*;

use crate::{
    asset_management::manifest::{Id, ItemManifest, StructureManifest, UnitManifest},
    infovis::TileOverlay,
    signals::SignalType,
};

use super::LeftPanel;

pub(super) struct OverlayMenuPlugin;

impl Plugin for OverlayMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(select_overlay)
            .add_startup_system(setup_overlay_menu);
    }
}

/// Records the structure of the overlay menu. for easy lookup.
#[derive(Resource)]
struct OverlayMenu {
    /// The entity that contains the legend.
    legend_entity: Entity,
}

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

/// Creates the UI needed to display the overlay.
fn setup_overlay_menu(
    mut commands: Commands,
    left_panel_query: Query<Entity, With<LeftPanel>>,
    tile_overlay: Res<TileOverlay>,
) {
    let left_panel_entity = left_panel_query.single();
    let legend_entity = commands
        .spawn(ImageBundle {
            style: Style {
                size: Size::new(
                    Val::Px(TileOverlay::LEGEND_WIDTH as f32),
                    Val::Px(TileOverlay::N_COLORS as f32),
                ),
                ..Default::default()
            },
            image: UiImage {
                texture: tile_overlay.legend_image_handle(),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::PINK),
            ..Default::default()
        })
        .id();
    commands.entity(left_panel_entity).add_child(legend_entity);
    commands.insert_resource(OverlayMenu { legend_entity });
}
