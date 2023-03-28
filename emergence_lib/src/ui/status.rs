//! Code to display the status of each unit and crafting structure.

use bevy::prelude::*;
use bevy_mod_billboard::{BillboardPlugin, BillboardTextBundle};

use crate::{
    asset_management::AssetState, structures::crafting::CraftingState, units::goals::Goal,
};

use super::FiraSansFontFamily;

pub(super) struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusVisualization>()
            .add_system(toggle_status_visualization.before(display_status))
            .add_system(display_status.run_if(in_state(AssetState::Ready)))
            .add_plugin(BillboardPlugin);
    }
}

/// Marker component for entities that display the status of a unit or crafting structure.
#[derive(Component)]
struct StatusDisplay;

/// Controls the visibility of the status display.
#[derive(Resource, Default)]
struct StatusVisualization {
    enabled: bool,
}

/// Toggles the status display on and off.
fn toggle_status_visualization(
    mut status_visualization: ResMut<StatusVisualization>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F2) {
        status_visualization.enabled = !status_visualization.enabled;
    }
}

/// Displays the status of each unit and crafting structure.
fn display_status(
    status_visualization: Res<StatusVisualization>,
    unit_query: Query<(&Transform, &Goal)>,
    crafting_query: Query<(&Transform, &CraftingState)>,
    status_display_query: Query<Entity, With<StatusDisplay>>,
    fonts: Res<FiraSansFontFamily>,
    mut commands: Commands,
) {
    // PERF: immediate mode for now
    for entity in status_display_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if status_visualization.enabled {
        for (transform, goal) in unit_query.iter() {
            commands
                .spawn(BillboardTextBundle {
                    text: Text::from_section(
                        format!("Goal: {:?}", goal),
                        TextStyle {
                            font: fonts.regular.clone_weak(),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    ),
                    transform: Transform::from_translation(Vec3::new(
                        transform.translation.x,
                        transform.translation.y + 0.5,
                        transform.translation.z,
                    )),
                    ..Default::default()
                })
                .insert(StatusDisplay);
        }

        for (transform, crafting_state) in crafting_query.iter() {
            commands
                .spawn(BillboardTextBundle {
                    text: Text::from_section(
                        format!("Crafting: {:?}", crafting_state),
                        TextStyle {
                            font: fonts.regular.clone_weak(),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    ),
                    transform: Transform::from_translation(Vec3::new(
                        transform.translation.x,
                        transform.translation.y + 0.5,
                        transform.translation.z,
                    )),
                    ..Default::default()
                })
                .insert(StatusDisplay);
        }
    }
}
