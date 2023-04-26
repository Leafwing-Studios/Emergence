//! Controls what is being visualized on the terrain by the [`TileOverlay`].

use crate::{
    asset_management::AssetState,
    infovis::{OverlayType, TileOverlay},
    items::item_manifest::ItemManifest,
    player_interaction::PlayerAction,
    signals::{SignalKind, Signals},
    structures::structure_manifest::StructureManifest,
    terrain::terrain_manifest::TerrainManifest,
    units::unit_manifest::UnitManifest,
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
            .add_system(update_signal_type_display.run_if(in_state(AssetState::FullyLoaded)));
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
    if player_actions.just_pressed(PlayerAction::ToggleStrongestSignalOverlay) {
        if tile_overlay.overlay_type != OverlayType::StrongestSignal {
            tile_overlay.overlay_type = OverlayType::StrongestSignal;
        } else {
            tile_overlay.overlay_type = OverlayType::None;
        }
    }

    if player_actions.just_pressed(PlayerAction::ToggleSignalOverlay) {
        // FIXME: this is very silly, but it's the easiest way to get and cycle signal types
        tile_overlay.overlay_type = signals.random_signal_type().into();
    }

    if player_actions.just_pressed(PlayerAction::ToggleWaterTableOverlay) {
        if tile_overlay.overlay_type != OverlayType::WaterTable {
            tile_overlay.overlay_type = OverlayType::WaterTable;
        } else {
            tile_overlay.overlay_type = OverlayType::None;
        }
    }
}

/// Creates the UI needed to display the overlay.
fn setup_overlay_menu(
    mut commands: Commands,
    left_panel_query: Query<Entity, With<LeftPanel>>,
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
                texture: Handle::default(),
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
    terrain_manifest: Res<TerrainManifest>,
    unit_manifest: Res<UnitManifest>,
) {
    let mut text = text_query.get_mut(overlay_menu.signal_type_entity).unwrap();
    let mut legend = image_query.get_mut(overlay_menu.legend_entity).unwrap();
    let font_size = 20.0;

    match &tile_overlay.overlay_type {
        OverlayType::None => {
            text.sections = vec![TextSection {
                value: "No overlay".to_string(),
                style: TextStyle {
                    font: fonts.regular.clone_weak(),
                    font_size,
                    color: Color::WHITE,
                },
            }];

            legend.texture = Handle::default();
        }
        OverlayType::Single(signal_type) => {
            let signal_kind: SignalKind = (*signal_type).into();

            text.sections = vec![TextSection {
                value: signal_type.display(
                    &item_manifest,
                    &structure_manifest,
                    &terrain_manifest,
                    &unit_manifest,
                ),
                style: TextStyle {
                    font: fonts.regular.clone_weak(),
                    font_size,
                    color: signal_kind.color(),
                },
            }];

            legend.texture = tile_overlay.legend_image_handle(signal_kind)
        }
        OverlayType::StrongestSignal => {
            text.sections = vec![
                TextSection {
                    value: "Push\n".to_string(),
                    style: TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size,
                        color: SignalKind::Push.color(),
                    },
                },
                TextSection {
                    value: "Pull\n".to_string(),
                    style: TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size,
                        color: SignalKind::Pull.color(),
                    },
                },
                TextSection {
                    value: "Work\n".to_string(),
                    style: TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size,
                        color: SignalKind::Work.color(),
                    },
                },
                TextSection {
                    value: "Demolish\n".to_string(),
                    style: TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size,
                        color: SignalKind::Demolish.color(),
                    },
                },
                TextSection {
                    value: "Lure\n".to_string(),
                    style: TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size,
                        color: SignalKind::Lure.color(),
                    },
                },
                TextSection {
                    value: "Repel\n".to_string(),
                    style: TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size,
                        color: SignalKind::Repel.color(),
                    },
                },
                TextSection {
                    value: "Unit".to_string(),
                    style: TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size,
                        color: SignalKind::Unit.color(),
                    },
                },
            ];

            legend.texture = Handle::default();
        }
        OverlayType::WaterTable => {
            text.sections = vec![TextSection {
                value: "Depth to water table".to_string(),
                style: TextStyle {
                    font: fonts.regular.clone_weak(),
                    font_size,
                    color: Color::WHITE,
                },
            }];

            legend.texture = tile_overlay.water_table_legend_image_handle();
        }
    }
}
