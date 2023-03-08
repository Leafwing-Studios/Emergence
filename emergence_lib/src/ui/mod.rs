//! Creates the UI from all modules.
//!
use crate::ui::{select_structure::SelectStructurePlugin, selection_panel::HoverDetailsPlugin};
use bevy::prelude::*;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

mod intent;
mod select_structure;
mod selection_panel;

/// The different stages of the UI setup.
///
/// Stages are needed here to flush the commands in-between.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
enum UiStage {
    /// Create the initial layout structure.
    LayoutInitialization,

    /// Populate the layout structure with more content.
    LayoutPopulation,
}

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
        .add_startup_stage_before(
            StartupStage::Startup,
            UiStage::LayoutInitialization,
            SystemStage::parallel(),
        )
        .add_startup_stage_after(
            UiStage::LayoutInitialization,
            UiStage::LayoutPopulation,
            SystemStage::parallel(),
        )
        .add_startup_system_to_stage(UiStage::LayoutInitialization, setup_ui)
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_plugin(HoverDetailsPlugin)
        .add_plugin(SelectStructurePlugin);
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
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
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
                        size: Size::new(Val::Px(200.), Val::Percent(100.)),
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
                        size: Size::new(Val::Px(400.), Val::Percent(100.)),
                        ..default()
                    },
                    ..default()
                },
                RightPanel,
            ));
        });
}
