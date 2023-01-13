//! Create and update a panel to display info about the hovered tile.
use bevy::prelude::*;

use crate::{
    cursor::CursorTilePos, organisms::organism_details::HoverDetails,
    structures::crafting::CraftingState,
};

use super::{FiraSansFontFamily, RightPanel, UiStage};

/// The panel to display information on hover.
#[derive(Debug, Component)]
struct HoverPanel;

/// The text to display the position of the tile.
#[derive(Debug, Component)]
struct PositionText;

/// The text to display the type of organism on the tile.
#[derive(Debug, Component)]
struct OrganismText;

/// The text for all details regarding crafting.
#[derive(Debug, Component)]
struct CraftingText;

/// Create the hover panel in the UI.
fn setup_hover_panel(
    mut commands: Commands,
    font_family: Res<FiraSansFontFamily>,
    query: Query<Entity, With<RightPanel>>,
) {
    let key_text_style = TextStyle {
        color: Color::rgb(0.9, 0.9, 0.9),
        font: font_family.regular.clone_weak(),
        font_size: 20.,
    };
    let value_text_style = TextStyle {
        color: Color::WHITE,
        font: font_family.bold.clone_weak(),
        font_size: 20.,
    };

    let right_panel = query.single();

    let hover_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Px(200.)),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                background_color: Color::rgba(0., 0., 0., 0.9).into(),
                visibility: Visibility::INVISIBLE,
                ..default()
            },
            HoverPanel,
        ))
        .with_children(|parent| {
            // Tile position
            parent.spawn((
                TextBundle::from_sections([
                    TextSection::new("Position: ", key_text_style.clone()),
                    TextSection::from_style(value_text_style.clone()),
                ]),
                PositionText,
            ));

            // Organism type
            parent.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new("Organism: ", key_text_style.clone()),
                        TextSection::from_style(value_text_style.clone()),
                    ]),
                    visibility: Visibility::INVISIBLE,
                    ..default()
                },
                OrganismText,
            ));

            // Crafting stuff
            parent.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new("Inputs: ", key_text_style.clone()),
                        TextSection::from_style(value_text_style.clone()),
                        TextSection::new("\nOutputs: ", key_text_style.clone()),
                        TextSection::from_style(value_text_style.clone()),
                        TextSection::new("\nActive recipe: ", key_text_style.clone()),
                        TextSection::from_style(value_text_style.clone()),
                        TextSection::new("\nStatus: ", key_text_style.clone()),
                        TextSection::from_style(value_text_style.clone()),
                    ]),
                    visibility: Visibility::INVISIBLE,
                    ..default()
                },
                CraftingText,
            ));
        })
        .id();

    commands.entity(right_panel).add_child(hover_panel);
}

/// Update the information displayed in the hover panel.
fn update_hover_panel(
    cursor_tile_pos: Res<CursorTilePos>,
    hover_details: Res<HoverDetails>,
    mut panel_query: Query<&mut Visibility, With<HoverPanel>>,
    mut position_query: Query<&mut Text, With<PositionText>>,
    mut organism_query: Query<
        (&mut Text, &mut Visibility),
        (
            With<OrganismText>,
            // Avoid conflicting queries
            Without<PositionText>,
            Without<HoverPanel>,
        ),
    >,
    mut crafting_query: Query<
        (&mut Text, &mut Visibility),
        (
            // Avoid conflicting queries
            With<CraftingText>,
            Without<PositionText>,
            Without<HoverPanel>,
            Without<OrganismText>,
        ),
    >,
) {
    if let Some(cursor_tile_pos) = cursor_tile_pos.maybe_tile_pos() {
        // Update visibility of the whole panel
        *panel_query.single_mut() = Visibility::VISIBLE;

        // Update position text
        position_query.single_mut().sections[1].value =
            format!("{}, {}", cursor_tile_pos.x, cursor_tile_pos.y);

        // Update organism text
        if let Some(organism_details) = &**hover_details {
            let (mut text, mut visibility) = organism_query.single_mut();

            *visibility = Visibility::VISIBLE;
            text.sections[1].value = format!("{}", organism_details.organism_type);

            // Update crafting text
            if let Some(crafting_details) = &organism_details.crafting_details {
                let (mut text, mut visibility) = crafting_query.single_mut();
                *visibility = Visibility::VISIBLE;

                // Update all text entries for crafting
                text.sections[1].value = format!("{}", crafting_details.input_inventory);
                text.sections[3].value = format!("{}", crafting_details.output_inventory);
                text.sections[5].value = if let Some(recipe) = &crafting_details.active_recipe {
                    format!("{}", recipe)
                } else {
                    "None".to_string()
                };
                text.sections[7].value = match crafting_details.state {
                    CraftingState::WaitingForInput => "Waiting for input".to_string(),
                    CraftingState::InProgress => {
                        format!("Crafting ({:.2}s)", crafting_details.timer.remaining_secs())
                    }
                    CraftingState::Finished => "Waiting for space in output".to_string(),
                };
            } else {
                let (_, mut visibility) = crafting_query.single_mut();

                *visibility = Visibility::INVISIBLE;
            }
        } else {
            let (_, mut organism_visibility) = organism_query.single_mut();
            let (_, mut crafting_visibility) = crafting_query.single_mut();

            *organism_visibility = Visibility::INVISIBLE;
            *crafting_visibility = Visibility::INVISIBLE;
        }
    } else {
        *panel_query.single_mut() = Visibility::INVISIBLE;
    }
}

/// Functionality for the info panel on hover.
#[derive(Debug)]
pub struct HoverPanelPlugin;

impl Plugin for HoverPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(UiStage::LayoutPopulation, setup_hover_panel)
            .add_system(update_hover_panel);
    }
}
