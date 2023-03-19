//! Displays information about the currently hovered entity.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::{
        ItemManifest, RecipeManifest, StructureManifest, TerrainManifest, UnitManifest,
    },
    player_interaction::{selection::SelectionDetails, InteractionSystem},
};

use super::{FiraSansFontFamily, RightPanel};

/// Initializes and updates the hover details panel.
pub(super) struct HoverDetailsPlugin;

impl Plugin for HoverDetailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(populate_hover_panel)
            .add_system(update_hover_details.after(InteractionSystem::HoverDetails));
    }
}

/// The root node for the hover panel.
#[derive(Component)]
struct HoverPanel;

/// The UI node that stores all ghost details.
#[derive(Component, Default)]
struct GhostDetailsMarker;

/// The UI node that stores all structure details.
#[derive(Component, Default)]
struct StructureDetailsMarker;

/// The UI node that stores all terrain details.
#[derive(Component, Default)]
struct TerrainDetailsMarker;

/// The UI node that stores all unit details.
#[derive(Component, Default)]
struct UnitDetailsMarker;

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
                    size: Size::new(Val::Percent(100.), Val::Px(500.)),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                background_color: Color::rgba(0., 0., 0., 0.9).into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            HoverPanel,
        ))
        .id();

    let ghost_details = populate_details::<GhostDetailsMarker>(&mut commands, &key_text_style);
    let structure_details =
        populate_details::<StructureDetailsMarker>(&mut commands, &key_text_style);
    let terrain_details = populate_details::<TerrainDetailsMarker>(&mut commands, &key_text_style);
    let unit_details = populate_details::<UnitDetailsMarker>(&mut commands, &key_text_style);

    commands.entity(right_panel).add_child(hover_panel);
    commands
        .entity(hover_panel)
        .add_child(ghost_details)
        .add_child(structure_details)
        .add_child(terrain_details)
        .add_child(unit_details);
}

/// Updates UI elements for hover details based on new information.
#[allow(clippy::too_many_arguments)]
fn update_hover_details(
    selection_details: Res<SelectionDetails>,
    mut hover_panel_query: Query<&mut Visibility, With<HoverPanel>>,
    mut ghost_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<GhostDetailsMarker>,
            Without<StructureDetailsMarker>,
            Without<TerrainDetailsMarker>,
            Without<UnitDetailsMarker>,
        ),
    >,
    mut structure_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<StructureDetailsMarker>,
            Without<GhostDetailsMarker>,
            Without<TerrainDetailsMarker>,
            Without<UnitDetailsMarker>,
        ),
    >,
    mut unit_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<UnitDetailsMarker>,
            Without<GhostDetailsMarker>,
            Without<StructureDetailsMarker>,
            Without<TerrainDetailsMarker>,
        ),
    >,
    mut terrain_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<TerrainDetailsMarker>,
            Without<GhostDetailsMarker>,
            Without<StructureDetailsMarker>,
            Without<UnitDetailsMarker>,
        ),
    >,
    structure_manifest: Res<StructureManifest>,
    unit_manifest: Res<UnitManifest>,
    terrain_manifest: Res<TerrainManifest>,
    recipe_manifest: Res<RecipeManifest>,
    item_manifest: Res<ItemManifest>,
) {
    let mut parent_visibility = hover_panel_query.single_mut();
    let (mut ghost_style, mut ghost_text) = ghost_details_query.single_mut();
    let (mut structure_style, mut structure_text) = structure_details_query.single_mut();
    let (mut unit_style, mut unit_text) = unit_details_query.single_mut();
    let (mut terrain_style, mut terrain_text) = terrain_details_query.single_mut();

    match *selection_details {
        SelectionDetails::Ghost(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_style.display = Display::Flex;
            structure_style.display = Display::None;
            terrain_style.display = Display::None;
            unit_style.display = Display::None;
        }
        SelectionDetails::Structure(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_style.display = Display::None;
            structure_style.display = Display::Flex;
            terrain_style.display = Display::None;
            unit_style.display = Display::None;
        }
        SelectionDetails::Terrain(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_style.display = Display::None;
            structure_style.display = Display::None;
            terrain_style.display = Display::Flex;
            unit_style.display = Display::None;
        }
        SelectionDetails::Unit(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_style.display = Display::None;
            structure_style.display = Display::None;
            terrain_style.display = Display::None;
            unit_style.display = Display::Flex;
        }
        SelectionDetails::None => {
            // Don't bother messing with Display here to avoid triggering a pointless relayout
            *parent_visibility = Visibility::Hidden;
        }
    }

    match &*selection_details {
        SelectionDetails::Ghost(details) => {
            ghost_text.sections[0].value =
                details.display(&item_manifest, &structure_manifest, &recipe_manifest);
        }
        SelectionDetails::Structure(details) => {
            structure_text.sections[0].value = details.display(&structure_manifest, &item_manifest);
        }
        SelectionDetails::Terrain(details) => {
            terrain_text.sections[0].value =
                details.display(&terrain_manifest, &structure_manifest, &item_manifest);
        }
        SelectionDetails::Unit(details) => {
            unit_text.sections[0].value =
                details.display(&unit_manifest, &item_manifest, &structure_manifest);
        }
        SelectionDetails::None => (),
    };
}

/// Generates the details node with the marker component `T` and its children.
///
/// The returned [`Entity`] is for the root node.
fn populate_details<T: Component + Default>(
    commands: &mut Commands,
    key_text_style: &TextStyle,
) -> Entity {
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
            T::default(),
        ))
        .id()
}
