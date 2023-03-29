//! Controls what is being visualized on the terrain by the [`TileOverlay`].

use crate::{
    asset_management::{
        manifest::{ItemManifest, StructureManifest, UnitManifest},
        AssetState,
    },
    infovis::TileOverlay,
    player_interaction::PlayerAction,
    signals::{SignalKind, Signals},
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use super::{FiraSansFontFamily, LeftPanel};

/// The plugin that adds the overlay menu to the UI.
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
    /// The entity that tracks the [`SignalType`](crate::signals::SignalType) being displayed.
    signal_type_entity: Entity,
    /// The entity that stores the legend image.
    legend_entity: Entity,
}

/// Controls the overlay that is currently being displayed based on UI interactions.
fn select_overlay(
    // FIXME: use an actual UI widget for this...
    player_actions: Res<ActionState<PlayerAction>>,
    mut tile_overlay: ResMut<TileOverlay>,
    signals: Res<Signals>,
) {
    if player_actions.just_pressed(PlayerAction::ToggleSignalOverlay) {
        // FIXME: this is very silly, but it's the easiest way to get and cycle signal types
        tile_overlay.visualized_signal = signals.random_signal_type().into();
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
                texture: tile_overlay.legend_image_handle(SignalKind::Contains),
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands.entity(left_panel_entity).add_child(legend_entity);

    commands.insert_resource(OverlayMenu {
        signal_type_entity,
        legend_entity,
    });
}

/// Updates the text that displays the [`SignalType`](crate::signals::SignalType) being visualized.
fn update_signal_type_display(
    mut text_query: Query<&mut Text>,
    mut image_query: Query<&mut UiImage>,
    fonts: Res<FiraSansFontFamily>,
    overlay_menu: Res<OverlayMenu>,
    tile_overlay: Res<TileOverlay>,
    item_manifest: Res<ItemManifest>,
    structure_manifest: Res<StructureManifest>,
    unit_manifest: Res<UnitManifest>,
) {
    let mut text = text_query.get_mut(overlay_menu.signal_type_entity).unwrap();
    let mut legend = image_query.get_mut(overlay_menu.legend_entity).unwrap();

    match &tile_overlay.visualized_signal {
        crate::infovis::OverlayType::None => {
            text.sections[0].value = "No signal".to_string();
            text.sections[0].style = TextStyle {
                font: fonts.regular.clone_weak(),
                font_size: 20.0,
                color: Color::WHITE,
            };

            legend.texture = Handle::default();
        }
        crate::infovis::OverlayType::Single(signal_type) => {
            let signal_kind: SignalKind = (*signal_type).into();

            text.sections[0].value =
                signal_type.display(&item_manifest, &structure_manifest, &unit_manifest);
            text.sections[0].style = TextStyle {
                font: text.sections[0].style.font.clone_weak(),
                font_size: 20.0,
                color: signal_kind.color(),
            };

            legend.texture = tile_overlay.legend_image_handle(signal_kind)
        }
        crate::infovis::OverlayType::StrongestSignal => todo!(),
    }
}
