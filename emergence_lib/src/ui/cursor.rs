//! Controls the appearance of the player's cursor.

use bevy::prelude::*;

use crate::{
    asset_management::AssetState, construction::terraform::TerraformingTool,
    player_interaction::clipboard::Tool, ui::ui_assets::CHOICE_ICON_SIZE,
};

use super::ui_assets::Icons;

/// The plugin that adds the cursor to the UI and controls its appearance.
pub(super) struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, track_cursor.pipe(ignore))
            .add_systems(Update, set_cursor.run_if(in_state(AssetState::FullyLoaded)));
    }
}

/// Marker component for the cursor entity
#[derive(Component, Debug, Default, Clone, Copy)]
struct Cursor;

/// Changes the cursor's UI element based on the current [`Tool`] contents
fn set_cursor(
    tool: Res<Tool>,
    mut cursor_query: Query<&mut UiImage, With<Cursor>>,
    terraforming_icons: Res<Icons<TerraformingTool>>,
    mut commands: Commands,
) {
    if let Ok(mut cursor_image) = cursor_query.get_single_mut() {
        if tool.is_changed() {
            *cursor_image = match *tool {
                // Use the matching icon for the terraforming tool
                Tool::Terraform(terraforming_tool) => terraforming_icons.get(terraforming_tool),
                // Ghosts are used instead for structures
                Tool::Structures(_) => Handle::default(),
                // No need to show a custom cursor if we have nothing selected
                Tool::None => Handle::default(),
            }
            .into()
        }
    } else {
        commands.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Px(CHOICE_ICON_SIZE),
                    height: Val::Px(CHOICE_ICON_SIZE),
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                image: UiImage {
                    texture: Handle::default(),
                    ..default()
                },
                ..Default::default()
            },
            Cursor,
        ));
    }
}

/// Moves the cursor to follow the mouse position
fn track_cursor(
    mut cursor_query: Query<&mut Style, With<Cursor>>,
    window_query: Query<&Window>,
) -> Option<()> {
    let window = window_query.get_single().ok()?;
    let mut cursor_style = cursor_query.get_single_mut().ok()?;
    let mouse_position = window.cursor_position()?;
    // Center the cursor icon on the mouse position
    cursor_style.left = Val::Px(mouse_position.x - CHOICE_ICON_SIZE / 2.);
    cursor_style.bottom = Val::Px(mouse_position.y - CHOICE_ICON_SIZE / 2.);
    Some(())
}
