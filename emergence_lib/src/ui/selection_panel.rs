//! Displays information about the currently hovered entity.

use bevy::prelude::*;

use crate::{
    items::recipe::RecipeManifest,
    player_interaction::{details::SelectionDetails, InteractionSystem},
    structures::crafting::CraftingState,
};

use super::{FiraSansFontFamily, RightPanel, UiStage};

/// Initializes and updates the hover details panel.
pub(super) struct HoverDetailsPlugin;

impl Plugin for HoverDetailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(UiStage::LayoutPopulation, populate_hover_panel)
            .add_system(update_hover_details.after(InteractionSystem::HoverDetails));
    }
}

/// The root node for the hover panel.
#[derive(Component)]
struct HoverPanel;

/// The UI node that stores all structure details.
#[derive(Component)]
struct StructureDetails;

/// The UI node that stores all unit details.
#[derive(Component)]
struct UnitDetails;

/// The text that displays the position of the hovered entity.
#[derive(Component)]
struct PositionText;

/// The text that displays the identity of the hovered entity.
#[derive(Component)]
struct IdText;

/// The text that displays the crafting status and settings of the hovered entity.
#[derive(Component)]
struct CraftingText;

/// Estabilishes UI elements for hover details.
fn populate_hover_panel(
    mut commands: Commands,
    font_family: Res<FiraSansFontFamily>,
    parent_query: Query<Entity, With<RightPanel>>,
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

    let right_panel = parent_query.single();

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
        .id();

    let structure_details =
        populate_structure_details(&mut commands, &key_text_style, &value_text_style);

    let unit_details = populate_unit_details(&mut commands, &key_text_style);

    commands.entity(right_panel).add_child(hover_panel);
    commands
        .entity(hover_panel)
        .add_child(structure_details)
        .add_child(unit_details);
}

/// Updates UI elements for hover details based on new information.
fn update_hover_details(
    selection_details: Res<SelectionDetails>,
    mut hover_panel_query: Query<&mut Visibility, With<HoverPanel>>,
    mut position_query: Query<&mut Text, (With<PositionText>, Without<UnitDetails>)>,
    mut structure_details_query: Query<&mut Style, (With<StructureDetails>, Without<UnitDetails>)>,
    mut unit_details_query: Query<&mut Style, (With<UnitDetails>, Without<StructureDetails>)>,
    mut organism_query: Query<
        &mut Text,
        (
            With<IdText>,
            // Avoid conflicting queries
            Without<PositionText>,
            Without<HoverPanel>,
            Without<UnitDetails>,
        ),
    >,
    mut crafting_query: Query<
        (&mut Text, &mut Visibility),
        (
            // Avoid conflicting queries
            With<CraftingText>,
            Without<PositionText>,
            Without<HoverPanel>,
            Without<IdText>,
            Without<UnitDetails>,
        ),
    >,
    mut unit_text_query: Query<&mut Text, With<UnitDetails>>,
    recipe_manifest: Res<RecipeManifest>,
) {
    let mut parent_visibility = hover_panel_query.single_mut();
    let structure_details_display = &mut structure_details_query.single_mut().display;
    let unit_details_display = &mut unit_details_query.single_mut().display;

    match &*selection_details {
        SelectionDetails::Structure(details) => {
            *parent_visibility = Visibility::VISIBLE;
            *structure_details_display = Display::Flex;
            *unit_details_display = Display::None;

            position_query.single_mut().sections[1].value = format!("{:?}", details.tile_pos);
            organism_query.single_mut().sections[1].value =
                format!("Variety: {}", details.structure_id.id);

            let (mut crafting_text, mut crafting_visibility) = crafting_query.single_mut();

            if let Some(crafting_details) = &details.crafting_details {
                *crafting_visibility = Visibility::VISIBLE;
                crafting_text.sections[1].value = format!("{}", crafting_details.input_inventory);
                crafting_text.sections[3].value = format!("{}", crafting_details.output_inventory);

                crafting_text.sections[5].value =
                    if let Some(recipe_id) = &crafting_details.active_recipe {
                        let recipe_details = recipe_manifest.get(recipe_id);
                        format!("{recipe_id}: \n {recipe_details}")
                    } else {
                        "None".to_string()
                    };
                crafting_text.sections[7].value = match crafting_details.state {
                    CraftingState::WaitingForInput => "Waiting for input".to_string(),
                    CraftingState::InProgress => {
                        format!("Crafting ({:.2}s)", crafting_details.timer.remaining_secs())
                    }
                    CraftingState::Finished => "Waiting for space in output".to_string(),
                };
            } else {
                *crafting_visibility = Visibility::INVISIBLE;
            }
        }
        SelectionDetails::Unit(details) => {
            *parent_visibility = Visibility::VISIBLE;
            *unit_details_display = Display::Flex;
            *structure_details_display = Display::None;

            let mut unit_text = unit_text_query.single_mut();
            unit_text.sections[0].value = format!("{details}");
        }
        SelectionDetails::None => {
            *parent_visibility = Visibility::INVISIBLE;
        }
    };
}

/// Generates the [`StructureDetails`] node and its children.
///
/// The returned [`Entity`] is for the root node.
fn populate_structure_details(
    commands: &mut Commands,
    key_text_style: &TextStyle,
    value_text_style: &TextStyle,
) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            StructureDetails,
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
                IdText,
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
        .id()
}

/// Generates the [`UnitDetails`] node and its children.
///
/// The returned [`Entity`] is for the root node.
fn populate_unit_details(commands: &mut Commands, key_text_style: &TextStyle) -> Entity {
    commands
        .spawn((
            TextBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                text: Text::from_section("", key_text_style.clone()),
                ..default()
            },
            UnitDetails,
        ))
        .id()
}
