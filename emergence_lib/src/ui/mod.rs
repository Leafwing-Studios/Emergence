//! Manages all UI across the entire game.

use crate::{
    asset_management::{manifest::Id, AssetCollectionExt},
    construction::terraform::TerraformingTool,
    structures::structure_manifest::Structure,
    ui::{
        cursor::CursorPlugin,
        overlay::OverlayMenuPlugin,
        production_statistics::ProductionStatisticsPlugin,
        select_structure::SelectStructurePlugin,
        select_terraforming::SelectTerraformingPlugin,
        selection_details::SelectionDetailsPlugin,
        status::{CraftingProgress, StatusPlugin},
        ui_assets::{Icons, UiElements},
    },
    units::{goals::GoalKind, unit_manifest::Unit},
};
use bevy::prelude::*;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

mod cursor;
mod overlay;
mod production_statistics;
mod select_structure;
mod select_terraforming;
mod selection_details;
mod status;
mod ui_assets;
mod wheel_menu;

/// The font handles for the `FiraSans` font family.
///
/// This is cached in a resource to improve performance.
#[derive(Debug, Resource)]
struct FiraSansFontFamily {
    /// The font to use for regular text.
    regular: Handle<Font>,
}

/// Struct to build the UI plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        info!("Building UI plugin...");

        let asset_server = app.world.get_resource::<AssetServer>().unwrap();

        app.insert_resource(FiraSansFontFamily {
            regular: asset_server.load("fonts/FiraSans-Medium.ttf"),
        })
        .add_asset_collection::<UiElements>()
        .add_asset_collection::<Icons<Id<Structure>>>()
        .add_asset_collection::<Icons<Id<Unit>>>()
        .add_asset_collection::<Icons<TerraformingTool>>()
        .add_asset_collection::<Icons<CraftingProgress>>()
        .add_asset_collection::<Icons<GoalKind>>()
        .add_systems(PreStartup, setup_ui)
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(SelectionDetailsPlugin)
        .add_plugin(ProductionStatisticsPlugin)
        .add_plugin(StatusPlugin)
        .add_plugin(OverlayMenuPlugin)
        .add_plugin(SelectStructurePlugin)
        .add_plugin(SelectTerraformingPlugin);
    }
}

/// The UI panel on the left side.
#[derive(Debug, Component)]
struct LeftPanel;

/// The UI panel on the right side.
#[derive(Debug, Component)]
struct RightPanel;

/// Create the basic UI layout.
fn setup_ui(mut commands: Commands) {
    // UI layout
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // UI panel on the left side
            parent.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        width: Val::Px(200.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                },
                LeftPanel,
            ));

            // UI panel on the right side
            parent.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(400.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                },
                RightPanel,
            ));
        });
}
