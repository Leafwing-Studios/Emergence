//! Controls the appearance of the player's cursor.

use bevy::prelude::*;

use crate::{
    asset_management::AssetState, construction::terraform::TerraformingTool,
    player_interaction::clipboard::Clipboard,
};

use super::ui_assets::Icons;

pub(super) struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_cursor.run_if(in_state(AssetState::FullyLoaded)));
    }
}

/// Marker component for the cursor entity
#[derive(Component, Debug, Default, Clone, Copy)]
struct Cursor;

/// Changes the cursor's UI element based on the current [`Clipboard`] contents
fn set_cursor(
    clipboard: Res<Clipboard>,
    mut cursor_query: Query<&mut UiImage, With<Cursor>>,
    terraforming_icons: Res<Icons<TerraformingTool>>,
    mut commands: Commands,
) {
    if let Ok(mut cursor_image) = cursor_query.get_single_mut() {
        if cursor_image.is_changed() {
            *cursor_image = match *clipboard {
                // Use the matching icon for the terraforming tool
                Clipboard::Terraform(terraforming_tool) => {
                    terraforming_icons.get(terraforming_tool)
                }
                // Ghosts are used instead for structures
                Clipboard::Structures(_) => Handle::default(),
                // No need to show a custom cursor if we have nothing selected
                Clipboard::Empty => Handle::default(),
            }
            .into()
        }
    } else {
        commands.spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(32.0), Val::Px(32.0)),
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                ..Default::default()
            },
            Cursor::default(),
        ));
    }
}
