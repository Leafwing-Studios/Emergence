//! Code to display the status of each unit and crafting structure.

use bevy::prelude::*;
use bevy_mod_billboard::{prelude::BillboardPlugin, BillboardDepth, BillboardTextBundle};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::AssetState,
    construction::terraform::TerraformingAction,
    crafting::components::{CraftingState, InputInventory, OutputInventory},
    items::item_manifest::ItemManifest,
    player_interaction::PlayerAction,
    structures::structure_manifest::StructureManifest,
    terrain::terrain_manifest::TerrainManifest,
    units::goals::Goal,
};

use super::FiraSansFontFamily;

/// Plugin that displays the status of each unit and crafting structure.
pub(super) struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StatusVisualization>()
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

/// Displays the status of each unit and crafting structure.
fn display_status(
    status_visualization: Res<StatusVisualization>,
    unit_query: Query<(&Transform, &Goal)>,
    crafting_query: Query<(&Transform, &CraftingState)>,
    terraforming_query: Query<
        (&Transform, &InputInventory, &OutputInventory),
        With<TerraformingAction>,
    >,
    status_display_query: Query<Entity, With<StatusDisplay>>,
    fonts: Res<FiraSansFontFamily>,
    item_manifest: Res<ItemManifest>,
    structure_manifest: Res<StructureManifest>,
    terrain_manifest: Res<TerrainManifest>,
    mut commands: Commands,
) {
    // PERF: immediate mode for now
    for entity in status_display_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if status_visualization.structures_enabled() {
        for (structure_transform, crafting_state) in crafting_query.iter() {
            let transform = Transform {
                translation: Vec3::new(
                    structure_transform.translation.x,
                    structure_transform.translation.y + 1.0,
                    structure_transform.translation.z,
                ),
                scale: Vec3::splat(0.0085),
                ..Default::default()
            };

            commands
                .spawn(BillboardTextBundle {
                    transform,
                    text: Text::from_section(
                        format!("{crafting_state}"),
                        TextStyle {
                            font_size: 60.0,
                            font: fonts.regular.clone_weak(),
                            color: crafting_state.color(),
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    billboard_depth: BillboardDepth(false),
                    ..default()
                })
                .insert(StatusDisplay);
        }
    }

    if status_visualization.units_enabled() {
        for (unit_transform, goal) in unit_query.iter() {
            let transform = Transform {
                translation: Vec3::new(
                    unit_transform.translation.x,
                    unit_transform.translation.y + 0.5,
                    unit_transform.translation.z,
                ),
                scale: Vec3::splat(0.0085),
                ..Default::default()
            };

            commands
                .spawn(BillboardTextBundle {
                    transform,
                    text: Text::from_section(
                        goal.display(&item_manifest, &structure_manifest, &terrain_manifest),
                        TextStyle {
                            font_size: 60.0,
                            font: fonts.regular.clone_weak(),
                            color: goal.color(),
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    billboard_depth: BillboardDepth(false),
                    ..default()
                })
                .insert(StatusDisplay);
        }
    }

    if status_visualization.terraforming_enabled() {
        for (unit_transform, input_inventory, output_inventory) in terraforming_query.iter() {
            let transform = Transform {
                translation: Vec3::new(
                    unit_transform.translation.x,
                    unit_transform.translation.y + 0.5,
                    unit_transform.translation.z,
                ),
                scale: Vec3::splat(0.0085),
                ..Default::default()
            };

            // Clippy is wrong.
            // The semantics here are different: an empty inventory has no items currently,
            // but an inventory with zero length is a placeholder for an inventory that does not accept items.
            #[allow(clippy::len_zero)]
            let string = match (input_inventory.len() > 0, output_inventory.len() > 0) {
                (true, true) => format!(
                    "Deliver {} + Remove {}",
                    input_inventory.display(&item_manifest),
                    output_inventory.display(&item_manifest)
                ),
                (true, false) => format!("Deliver {}", input_inventory.display(&item_manifest)),
                (false, true) => format!("Remove {}", output_inventory.display(&item_manifest)),
                (false, false) => String::new(),
            };

            commands
                .spawn(BillboardTextBundle {
                    transform,
                    text: Text::from_section(
                        string,
                        TextStyle {
                            font_size: 60.0,
                            font: fonts.regular.clone_weak(),
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment::Center),
                    billboard_depth: BillboardDepth(false),
                    ..default()
                })
                .insert(StatusDisplay);
        }
    }
}
