//! Create and update a panel to display info about the hovered tile.
use bevy::prelude::*;

use crate::cursor::CursorTilePos;

use super::RightPanel;

/// The panel to display information on hover.
#[derive(Debug, Component)]
pub struct HoverPanel;

/// The text to display the position of the tile.
#[derive(Debug, Component)]
pub struct PositionText;

/// Create the hover panel in the UI.
pub fn setup_hover_panel(mut commands: Commands, query: Query<Entity, With<RightPanel>>) {
    let right_panel = query.single();

    let hover_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                background_color: Color::rgba(0., 0., 0., 0.7).into(),
                visibility: Visibility::INVISIBLE,
                ..default()
            },
            HoverPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("Position: ", TextStyle::default()),
                    TextSection::from_style(TextStyle::default()),
                ]),
                PositionText,
            ));
        })
        .id();

    commands.entity(right_panel).add_child(hover_panel);
}

/// Update the information displayed in the hover panel.
pub fn update_hover_panel(
    cursor_tile_pos: Res<CursorTilePos>,
    mut panel_query: Query<&mut Visibility, With<HoverPanel>>,
    mut position_query: Query<&mut Text, With<PositionText>>,
) {
    if let Some(cursor_tile_pos) = cursor_tile_pos.0 {
        // Update visibility of the whole panel
        *panel_query.single_mut() = Visibility::VISIBLE;

        // Update position text
        position_query.single_mut().sections[1].value =
            format!("{}, {}", cursor_tile_pos.x, cursor_tile_pos.y);
    } else {
        *panel_query.single_mut() = Visibility::INVISIBLE;
    }
}
