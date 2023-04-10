//! Displays available intent and selected ability option.

use crate::asset_management::AssetState;
use bevy::prelude::*;

use super::LeftPanel;

/// Logic used to display the intent bar.
pub(super) struct IntentPlugin;

impl Plugin for IntentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_intent_bar.in_schedule(OnEnter(AssetState::FullyLoaded)))
            .add_system(update_intent_bar.run_if(in_state(AssetState::FullyLoaded)));
    }
}

/// Marker component for the intent bar.
#[derive(Component)]
struct IntentBar;

/// Initializes the intent bar UI element.
fn spawn_intent_bar(left_panel_query: Query<Entity, With<LeftPanel>>, mut commands: Commands) {
    let left_panel = left_panel_query.single();

    commands
        .entity(left_panel)
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    ..Default::default()
                },
                background_color: Color::PINK.into(),
                ..Default::default()
            });
        })
        .insert(IntentBar);
}

/// Updates the intent bar to display the current intent level.
fn update_intent_bar() {}
