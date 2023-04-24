//! Code to display the status of each unit and crafting structure.

use bevy::prelude::{shape::Quad, *};
use bevy_mod_billboard::{
    prelude::{BillboardMeshHandle, BillboardPlugin, BillboardTexture},
    BillboardDepth, BillboardTextureBundle,
};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    asset_management::{manifest::Id, AssetState},
    construction::terraform::TerraformingAction,
    crafting::components::CraftingState,
    player_interaction::PlayerAction,
    units::{
        goals::{Goal, GoalKind},
        unit_manifest::Unit,
    },
};

use super::ui_assets::Icons;

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
    /// Display all statuses.
    All,
}

impl StatusVisualization {
    /// Cycles to the next option.
    fn cycle(&mut self) {
        *self = match self {
            StatusVisualization::Off => StatusVisualization::Structures,
            StatusVisualization::Structures => StatusVisualization::Units,
            StatusVisualization::Units => StatusVisualization::All,
            StatusVisualization::All => StatusVisualization::Off,
        };
    }

    /// Returns true if the status of structures should be displayed.
    fn structures_enabled(&self) -> bool {
        match self {
            StatusVisualization::Off => false,
            StatusVisualization::Structures => true,
            StatusVisualization::Units => false,
            StatusVisualization::All => true,
        }
    }

    /// Returns true if the status of units should be displayed.
    fn units_enabled(&self) -> bool {
        match self {
            StatusVisualization::Off => false,
            StatusVisualization::Structures => false,
            StatusVisualization::Units => true,
            StatusVisualization::All => true,
        }
    }
}

/// How far along a crafting structure is in its current crafting process.
///
/// This is used to display the progress of the crafting process,
/// and is a visual indicator of [`CraftingState`].
///
/// This isn't a one-to-one mapping of [`CraftingState`] because
/// some states are ephemeral and don't need to be displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum CraftingProgress {
    /// Crafting is stopped because the input inventory is empty.
    NeedsInput,
    /// Crafting is stopped because the output inventory is full.
    FullAndBlocked,
    /// Crafting is in progress: n/6ths of the way to completion.
    InProgress(u8),
    /// No recipe has been selected.
    NoRecipe,
}

impl From<&CraftingState> for CraftingProgress {
    fn from(state: &CraftingState) -> Self {
        match state {
            CraftingState::NeedsInput => CraftingProgress::NeedsInput,
            CraftingState::InProgress { progress, required } => {
                debug_assert!(progress <= required);
                let fraction = progress.as_secs_f32() / required.as_secs_f32();
                // Round to the nearest 1/6th.
                // This allows for a maximum of 6 segments, and represents both 0 and 6 segments differently.
                let n_segments = (fraction * 6.0).round() as u8;
                CraftingProgress::InProgress(n_segments)
            }
            CraftingState::FullAndBlocked => CraftingProgress::FullAndBlocked,
            CraftingState::RecipeComplete => CraftingProgress::InProgress(6),
            CraftingState::Overproduction => CraftingProgress::InProgress(6),
            CraftingState::NoRecipe => CraftingProgress::NoRecipe,
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
    mut meshes: ResMut<Assets<Mesh>>,
) {
    /// The scale of the icons.
    ///
    /// This converts pixels to world units.
    const ICON_SCALE: f32 = 2.0;

    /// The transform of the status display.
    const STATUS_TRANSFORM: Transform = Transform {
        // Float above the parent entity.
        translation: Vec3::new(0.0, 3.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::new(ICON_SCALE, ICON_SCALE, ICON_SCALE),
    };

    let mesh: Mesh = Quad::new(Vec2::new(1., 1.)).into();
    // PERF: we could cache this mesh somewhere rather than recreating it every time
    let mesh_handle = meshes.add(mesh);

    for parent_entity in query.iter() {
        let status_entity = commands
            .spawn(BillboardTextureBundle {
                billboard_depth: BillboardDepth(false),
                transform: STATUS_TRANSFORM,
                mesh: BillboardMeshHandle(mesh_handle.clone()),
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
    mut status_icon_query: Query<
        (&mut Handle<BillboardTexture>, &mut Visibility),
        With<StatusDisplay>,
    >,
    mut billboard_textures: ResMut<Assets<BillboardTexture>>,
    crafting_progress_icons: Res<Icons<CraftingProgress>>,
    goal_icons: Res<Icons<GoalKind>>,
) {
    if status_visualization.structures_enabled() {
        for (crafting_state, status) in crafting_query.iter() {
            let (mut status_icon, mut visibility) =
                status_icon_query.get_mut(status.entity).unwrap();

            let crafting_progress = CraftingProgress::from(crafting_state);
            let image_handle = crafting_progress_icons.get(crafting_progress);

            // PERF: this is dumb to reinsert every frame
            *status_icon =
                billboard_textures.add(BillboardTexture::Single(image_handle.clone_weak()));

            *visibility = Visibility::Inherited;
        }
    } else {
        for (.., status) in crafting_query.iter() {
            let (_, mut visibility) = status_icon_query.get_mut(status.entity).unwrap();
            *visibility = Visibility::Hidden;
        }
    }

    if status_visualization.units_enabled() {
        for (goal, status) in unit_query.iter() {
            let (mut status_icon, mut visibility) =
                status_icon_query.get_mut(status.entity).unwrap();

            let goal_kind = GoalKind::from(goal);
            let image_handle = goal_icons.get(goal_kind);

            // PERF: this is dumb to reinsert every frame
            *status_icon =
                billboard_textures.add(BillboardTexture::Single(image_handle.clone_weak()));

            *visibility = Visibility::Inherited;
        }
    } else {
        for (.., status) in unit_query.iter() {
            let (_, mut visibility) = status_icon_query.get_mut(status.entity).unwrap();
            *visibility = Visibility::Hidden;
        }
    }
}
