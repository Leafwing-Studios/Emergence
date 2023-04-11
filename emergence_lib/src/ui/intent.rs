//! Displays available intent and selected ability option.

use crate::{asset_management::AssetState, player_interaction::abilities::IntentPool};
use bevy::prelude::*;
use leafwing_abilities::prelude::Pool;

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

/// The height of the intent bar, in pixels.
const INTENT_BAR_HEIGHT: Val = Val::Px(300.0);

/// Initializes the intent bar UI element.
fn spawn_intent_bar(left_panel_query: Query<Entity, With<LeftPanel>>, mut commands: Commands) {
    let left_panel = left_panel_query.single();

    commands.entity(left_panel).with_children(|parent| {
        parent
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), INTENT_BAR_HEIGHT),
                    ..Default::default()
                },
                background_color: Color::PURPLE.into(),
                ..Default::default()
            })
            .insert(IntentBar);
    });
}

/// Updates the intent bar to display the current intent level.
fn update_intent_bar(
    intent_pool: Res<IntentPool>,
    mut intent_bar_ui_query: Query<&mut Style, With<IntentBar>>,
) {
    if let Ok(mut intent_bar_style) = intent_bar_ui_query.get_single_mut() {
        intent_bar_style.size.height =
            INTENT_BAR_HEIGHT * intent_pool.current().0 / intent_pool.max().0;
    }
}
