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
        for (unit_transform, goal) in unit_query.iter() {
            let transform = Transform {
                translation: Vec3::new(
                    unit_transform.translation.x,
                    unit_transform.translation.y + 0.5,
                    unit_transform.translation.z,
                ),
                scale: Vec3::splat(0.0085),
                ..Default::default()
            };

            commands
                .spawn(BillboardTextBundle {
                    transform,
                    text: Text::from_section(
                        format!("{:?}", goal),
                        TextStyle {
                            font_size: 60.0,
                            font: fonts.regular.clone_weak(),
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    ..default()
                })
                .insert(StatusDisplay);
        }

        for (structure_transform, crafting_state) in crafting_query.iter() {
            let transform = Transform {
                translation: Vec3::new(
                    structure_transform.translation.x,
                    structure_transform.translation.y + 1.0,
                    structure_transform.translation.z,
                ),
                scale: Vec3::splat(0.0085),
                ..Default::default()
            };

            commands
                .spawn(BillboardTextBundle {
                    transform,
                    text: Text::from_section(
                        format!("{:?}", crafting_state),
                        TextStyle {
                            font_size: 60.0,
                            font: fonts.regular.clone_weak(),
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    ..default()
                })
                .insert(StatusDisplay);
        }
    }
}
