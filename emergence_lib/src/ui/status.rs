//! Code to display the status of each unit and crafting structure.

use bevy::prelude::*;
use bevy_mod_billboard::{prelude::BillboardPlugin, BillboardDepth, BillboardTextBundle};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::{manifest::Id, AssetState},
    construction::terraform::TerraformingAction,
    crafting::components::{CraftingState, InputInventory, OutputInventory},
    items::item_manifest::ItemManifest,
    player_interaction::PlayerAction,
    structures::structure_manifest::StructureManifest,
    terrain::terrain_manifest::TerrainManifest,
    units::{
        goals::Goal,
        unit_manifest::{Unit, UnitManifest},
    },
};

use super::FiraSansFontFamily;

/// Plugin that displays the status of each unit and crafting structure.
pub(super) struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusVisualization>()
            .add_system(add_status_displays.before(display_status))
            .add_system(cycle_status_visualization.before(display_status))
            .add_system(display_status.run_if(in_state(AssetState::FullyLoaded)))
            .add_plugin(BillboardPlugin);
    }
}

/// Marker component for entities that display the status of a unit or crafting structure.
#[derive(Component)]
struct StatusDisplay;

/// Controls the visibility of the status display.
#[derive(Resource, Default)]
enum StatusVisualization {
    /// Don't display the status.
    #[default]
    Off,
    /// Only display the status of structures.
    Structures,
    /// Only display the status of units.
    Units,
    /// Only display the status of terraforming actions.
    Terraforming,
    /// Display all statuses.
    All,
}

impl StatusVisualization {
    /// Cycles to the next option.
    fn cycle(&mut self) {
        *self = match self {
            StatusVisualization::Off => StatusVisualization::Structures,
            StatusVisualization::Structures => StatusVisualization::Units,
            StatusVisualization::Units => StatusVisualization::Terraforming,
            StatusVisualization::Terraforming => StatusVisualization::All,
            StatusVisualization::All => StatusVisualization::Off,
        };
    }

    /// Returns true if the status of structures should be displayed.
    fn structures_enabled(&self) -> bool {
        match self {
            StatusVisualization::Off => false,
            StatusVisualization::Structures => true,
            StatusVisualization::Units => false,
            StatusVisualization::Terraforming => false,
            StatusVisualization::All => true,
        }
    }

    /// Returns true if the status of units should be displayed.
    fn units_enabled(&self) -> bool {
        match self {
            StatusVisualization::Off => false,
            StatusVisualization::Structures => false,
            StatusVisualization::Units => true,
            StatusVisualization::Terraforming => false,
            StatusVisualization::All => true,
        }
    }

    /// Returns true if the status of terraforming actions should be displayed.
    fn terraforming_enabled(&self) -> bool {
        match self {
            StatusVisualization::Off => false,
            StatusVisualization::Structures => false,
            StatusVisualization::Units => false,
            StatusVisualization::Terraforming => true,
            StatusVisualization::All => true,
        }
    }
}

/// Cycles between status display options.
fn cycle_status_visualization(
    mut status_visualization: ResMut<StatusVisualization>,
    player_actions: Res<ActionState<PlayerAction>>,
) {
    if player_actions.just_pressed(PlayerAction::ToggleStatusInfo) {
        status_visualization.cycle();
    }
}

/// A component for text / iconography that displays the status of a unit or crafting structure.
#[derive(Component, Debug)]
struct StatusParent {
    /// The entity that displays the status.
    entity: Entity,
}

/// Adds a status display to each unit and crafting structure when they are spawned.
fn add_status_displays(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            AnyOf<(&Id<Unit>, &CraftingState, &TerraformingAction)>,
            Without<StatusParent>,
        ),
    >,
    fonts: Res<FiraSansFontFamily>,
) {
    /// The scale of the status text.
    ///
    ///  Default settings are way too big.
    const TEXT_SCALE: f32 = 0.1;

    /// The transform of the status display.
    const STATUS_TRANSFORM: Transform = Transform {
        // Float above the parent entity.
        translation: Vec3::new(0.0, 0.5, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::new(TEXT_SCALE, TEXT_SCALE, TEXT_SCALE),
    };

    for parent_entity in query.iter() {
        let status_entity = commands
            .spawn(BillboardTextBundle {
                billboard_depth: BillboardDepth(false),
                // We don't care about setting the text on initialization
                // because it will be set in the `display_status` system.
                text: Text {
                    // Allocate one section now: we'll set the text later.
                    sections: vec![TextSection::from_style(TextStyle {
                        font: fonts.regular.clone_weak(),
                        font_size: 10.0,
                        color: Color::WHITE,
                    })],
                    alignment: TextAlignment::Center,
                    linebreak_behaviour: bevy::text::BreakLineOn::WordBoundary,
                },
                transform: STATUS_TRANSFORM,
                ..Default::default()
            })
            .insert(StatusDisplay)
            .id();

        // By making this a child of the parent entity:
        // - it will be deleted when the parent entity is deleted
        // - it will be moved with the parent entity
        // - it will be hidden when the parent entity is hidden
        commands
            .entity(parent_entity)
            .insert(StatusParent {
                entity: status_entity,
            })
            .add_child(status_entity);
    }
}

/// Displays the status of each unit and crafting structure.
fn display_status(
    status_visualization: Res<StatusVisualization>,
    unit_query: Query<(&Goal, &StatusParent)>,
    crafting_query: Query<(&CraftingState, &StatusParent)>,
    terraforming_query: Query<
        (&InputInventory, &OutputInventory, &StatusParent),
        With<TerraformingAction>,
    >,
    mut status_text_query: Query<(&mut Text, &mut Visibility), With<StatusDisplay>>,
    item_manifest: Res<ItemManifest>,
    structure_manifest: Res<StructureManifest>,
    terrain_manifest: Res<TerrainManifest>,
    unit_manifest: Res<UnitManifest>,
) {
    if status_visualization.structures_enabled() {
        for (crafting_state, status) in crafting_query.iter() {
            let (mut status_text, mut visibility) =
                status_text_query.get_mut(status.entity).unwrap();
            let status_text = &mut status_text.sections[0];

            *visibility = Visibility::Inherited;
            status_text.value = format!("{crafting_state}");
            status_text.style.color = crafting_state.color();
        }
    } else {
        for (.., status) in crafting_query.iter() {
            let (_, mut visibility) = status_text_query.get_mut(status.entity).unwrap();
            *visibility = Visibility::Hidden;
        }
    }

    if status_visualization.units_enabled() {
        for (goal, status) in unit_query.iter() {
            let (mut status_text, mut visibility) =
                status_text_query.get_mut(status.entity).unwrap();
            let status_text = &mut status_text.sections[0];

            *visibility = Visibility::Inherited;
            status_text.value = goal.display(
                &item_manifest,
                &structure_manifest,
                &terrain_manifest,
                &unit_manifest,
            );
            status_text.style.color = goal.color();
        }
    } else {
        for (.., status) in unit_query.iter() {
            let (_, mut visibility) = status_text_query.get_mut(status.entity).unwrap();
            *visibility = Visibility::Hidden;
        }
    }

    if status_visualization.terraforming_enabled() {
        for (input_inventory, output_inventory, status) in terraforming_query.iter() {
            let (mut status_text, mut visibility) =
                status_text_query.get_mut(status.entity).unwrap();
            let status_text = &mut status_text.sections[0];

            *visibility = Visibility::Inherited;
            // Clippy is wrong.
            // The semantics here are different: an empty inventory has no items currently,
            // but an inventory with zero length is a placeholder for an inventory that does not accept items.
            status_text.value = match (input_inventory.len() > 0, output_inventory.len() > 0) {
                (true, true) => format!(
                    "Deliver {} + Remove {}",
                    input_inventory.display(&item_manifest),
                    output_inventory.display(&item_manifest)
                ),
                (true, false) => format!("Deliver {}", input_inventory.display(&item_manifest)),
                (false, true) => format!("Remove {}", output_inventory.display(&item_manifest)),
                (false, false) => String::new(),
            };
        }
    } else {
        for (.., status) in terraforming_query.iter() {
            let (_, mut visibility) = status_text_query.get_mut(status.entity).unwrap();
            *visibility = Visibility::Hidden;
        }
    }
}
