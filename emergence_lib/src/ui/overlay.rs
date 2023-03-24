//! Controls what is being visualized on the terrain by the [`TileOverlay`].

use bevy::prelude::*;
use bevy::utils::HashMap;
use derive_more::Display;
use emergence_macros::IterableEnum;

use crate as emergence_lib;

use crate::enum_iter::IterableEnum;
use crate::{
    asset_management::manifest::{Id, ItemManifest, StructureManifest, UnitManifest},
    infovis::TileOverlay,
    signals::SignalType,
};

use super::{FiraSansFontFamily, LeftPanel};

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
    /// The panel that contains all of the buttons.
    button_panel: Entity,
    /// The buttons that store each [`SignalKind`].
    signal_kind_buttons: HashMap<SignalKind, Entity>,
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

/// The data-less equivalent of a [`SignalType`].
#[derive(Component, Display, IterableEnum, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum SignalKind {
    Push,
    Pull,
    Stores,
    Contains,
    Work,
    Demolish,
    Unit,
}

impl From<SignalType> for SignalKind {
    fn from(signal_type: SignalType) -> Self {
        match signal_type {
            SignalType::Push(_) => Self::Push,
            SignalType::Pull(_) => Self::Pull,
            SignalType::Stores(_) => Self::Stores,
            SignalType::Contains(_) => Self::Contains,
            SignalType::Work(_) => Self::Work,
            SignalType::Demolish(_) => Self::Demolish,
            SignalType::Unit(_) => Self::Unit,
        }
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

    let button_panel = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Percent(100.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::PINK),
            ..Default::default()
        })
        .id();
    commands.entity(left_panel_entity).add_child(button_panel);

    let signal_kind_column = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Px(50.0)),
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::RED),
            ..Default::default()
        })
        .id();
    let signal_id_column = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Px(50.0)),
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLUE),
            ..Default::default()
        })
        .id();
    commands
        .entity(button_panel)
        .push_children(&[signal_kind_column, signal_id_column]);

    let text_style = TextStyle {
        font: fonts.regular.clone_weak(),
        font_size: 20.0,
        color: Color::BLACK,
    };

    let mut signal_kind_buttons = HashMap::new();
    for signal_kind in SignalKind::variants() {
        let signal_kind_button_entity = commands
            .spawn(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(100.0), Val::Px(50.0)),
                    // Vertically center the text in the button.
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::GREEN),
                ..Default::default()
            })
            .insert(signal_kind)
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(format!("{signal_kind}"), text_style.clone()),
                    ..Default::default()
                });
            })
            .id();

        commands
            .entity(button_panel)
            .add_child(signal_kind_button_entity);

        signal_kind_buttons.insert(signal_kind, signal_kind_button_entity);
    }

    commands.insert_resource(OverlayMenu {
        legend_entity,
        button_panel,
        signal_kind_buttons,
    });
}
