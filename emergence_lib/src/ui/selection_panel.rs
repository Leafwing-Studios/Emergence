//! Displays information about the currently hovered entity.

use bevy::prelude::*;

use crate::{
    items::recipe::RecipeManifest,
    player_interaction::{details::SelectionDetails, InteractionSystem},
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

    let structure_details = populate_structure_details(&mut commands, &key_text_style);

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
    mut structure_details_query: Query<
        (&mut Style, &mut Text),
        (With<StructureDetails>, Without<UnitDetails>),
    >,
    mut unit_details_query: Query<
        (&mut Style, &mut Text),
        (With<UnitDetails>, Without<StructureDetails>),
    >,
    recipe_manifest: Res<RecipeManifest>,
) {
    let mut parent_visibility = hover_panel_query.single_mut();
    let (mut structure_style, mut structure_text) = structure_details_query.single_mut();
    let (mut unit_style, mut unit_text) = unit_details_query.single_mut();

    match &*selection_details {
        SelectionDetails::Structure(details) => {
            *parent_visibility = Visibility::VISIBLE;
            structure_style.display = Display::Flex;
            unit_style.display = Display::None;

            // Details
            structure_text.sections[0].value = format!("{details}");
            // Recipe info
            structure_text.sections[1].value =
                if let Some(crafting_details) = &details.crafting_details {
                    if let Some(recipe_id) = &crafting_details.active_recipe {
                        let recipe_info = recipe_manifest.get(&recipe_id);
                        format!("\n{recipe_info}")
                    } else {
                        String::default()
                    }
                } else {
                    String::default()
                }
        }
        SelectionDetails::Unit(details) => {
            *parent_visibility = Visibility::VISIBLE;
            unit_style.display = Display::Flex;
            structure_style.display = Display::None;

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
fn populate_structure_details(commands: &mut Commands, key_text_style: &TextStyle) -> Entity {
    commands
        .spawn((
            TextBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                text: Text::from_sections([
                    TextSection::new("", key_text_style.clone()),
                    TextSection::new("", key_text_style.clone()),
                ]),
                ..default()
            },
            StructureDetails,
        ))
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
