//! Controls what is being visualized on the terrain by the [`TileOverlay`].

use crate::{
    asset_management::{
        manifest::{ItemManifest, StructureManifest, UnitManifest},
        AssetState,
    },
    infovis::TileOverlay,
    signals::Signals,
};
use bevy::prelude::*;

use super::{FiraSansFontFamily, LeftPanel};

pub(super) struct OverlayMenuPlugin;

impl Plugin for OverlayMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(select_overlay)
            .add_startup_system(setup_overlay_menu)
            .add_system(update_signal_type_display.run_if(in_state(AssetState::Ready)));
    }
}

/// Records the structure of the overlay menu. for easy lookup.
#[derive(Resource)]
struct OverlayMenu {
    /// The entity that tracks the [`SignalType`] being displayed.
    signal_type_entity: Entity,
}

/// Controls the overlay that is currently being displayed based on UI interactions.
fn select_overlay(
    // FIXME: use an actual UI widget for this...
    keyboard_input: Res<Input<KeyCode>>,
    mut tile_overlay: ResMut<TileOverlay>,
    signals: Res<Signals>,
) {
    if keyboard_input.just_pressed(KeyCode::F1) {
        // FIXME: this is very silly, but it's the easiest way to get and cycle signal types
        tile_overlay.visualized_signal = signals.random_signal_type();
    }
}

/// Creates the UI needed to display the overlay.
fn setup_overlay_menu(
    mut commands: Commands,
    left_panel_query: Query<Entity, With<LeftPanel>>,
    tile_overlay: Res<TileOverlay>,
    fonts: Res<FiraSansFontFamily>,
) {
    let left_panel_entity = left_panel_query.single();
    let text_style = TextStyle {
        font: fonts.regular.clone_weak(),
        font_size: 20.0,
        color: Color::WHITE,
    };

    let signal_type_entity = commands
        .spawn(TextBundle {
            text: Text::from_section("SIGNAL_TYPE".to_string(), text_style),
            ..Default::default()
        })
        .id();
    commands
        .entity(left_panel_entity)
        .add_child(signal_type_entity);

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
            ..Default::default()
        })
        .id();
    commands.entity(left_panel_entity).add_child(legend_entity);

    commands.insert_resource(OverlayMenu { signal_type_entity });
}

fn update_signal_type_display(
    mut query: Query<&mut Text>,
    overlay_menu: Res<OverlayMenu>,
    tile_overlay: Res<TileOverlay>,
    item_manifest: Res<ItemManifest>,
    structure_manifest: Res<StructureManifest>,
    unit_manifest: Res<UnitManifest>,
) {
    let mut text = query.get_mut(overlay_menu.signal_type_entity).unwrap();

    text.sections[0].value = match tile_overlay.visualized_signal {
        Some(signal_type) => format!(
            "{}",
            signal_type.display(&item_manifest, &structure_manifest, &unit_manifest)
        ),
        None => "No signal".to_string(),
    };
}
