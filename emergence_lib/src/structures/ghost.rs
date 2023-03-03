//! Ghosts are translucent phantom structures, used to show structures that could be or are planned to be built.
//!
//! There is an important distinction between "ghosts" and "previews", even though they appear similarly to players.
//! Ghosts are buildings that are genuinely planned to be built.
//! Previews are simply hovered, and used as a visual aid to show placement.

use crate::{
    self as emergence_lib, asset_management::manifest::StructureManifest,
    graphics::InheritedMaterial,
};
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_mod_raycast::RaycastMesh;
use emergence_macros::IterableEnum;

use crate::{
    asset_management::manifest::{Id, Structure},
    player_interaction::clipboard::ClipboardData,
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{Facing, TilePos},
};

use super::{
    commands::StructureCommandsExt,
    crafting::{ActiveRecipe, CraftingState, InputInventory},
};

/// A marker component that indicates that this structure is planned to be built, rather than actually existing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghost;

/// A marker component indicating that this structure should be rendered in a transparent style.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghostly;

/// The set of components needed to spawn a ghost.
#[derive(Bundle)]
pub(super) struct GhostBundle {
    /// Marker component
    ghost: Ghost,
    /// The location of the ghost
    tile_pos: TilePos,
    /// The variety of structure
    structure_id: Id<Structure>,
    /// The direction the ghost is facing
    facing: Facing,
    /// The items required to actually seed this item
    construction_materials: InputInventory,
    /// Tracks work that needs to be done on this building
    crafting_state: CraftingState,
    /// What should the structure craft when it is first built?
    active_recipe: ActiveRecipe,
    /// Makes structures pickable
    raycast_mesh: RaycastMesh<Ghost>,
    /// The mesh used for raycasting
    picking_mesh: Handle<Mesh>,
    /// The material to be used by all children in the scene
    inherited_material: InheritedMaterial,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
    /// Emits signals, drawing units towards this ghost to build it
    emitter: Emitter,
}

impl GhostBundle {
    /// Creates a new [`GhostBundle`].
    pub(super) fn new(
        tile_pos: TilePos,
        clipboard_data: ClipboardData,
        structure_manifest: &StructureManifest,
        picking_mesh: Handle<Mesh>,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        let structure_id = clipboard_data.structure_id;
        let structure_data = structure_manifest.get(structure_id);

        GhostBundle {
            ghost: Ghost,
            tile_pos,
            structure_id,
            facing: clipboard_data.facing,
            construction_materials: structure_data.construction_materials.clone(),
            crafting_state: CraftingState::NeedsInput,
            active_recipe: clipboard_data.active_recipe,
            raycast_mesh: RaycastMesh::default(),
            picking_mesh,
            inherited_material,
            scene_bundle: SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(world_pos),
                ..default()
            },
            emitter: Emitter::default(),
        }
    }
}

/// The variety of ghostly structure.
#[derive(IterableEnum, Debug, PartialEq, Eq, Hash)]
pub(crate) enum GhostKind {
    /// A structure that is going to be built.
    Ghost,
    /// A ghost, but currently selected
    SelectedGhost,
    /// A structure that players are holding in their clipboard and planning to place
    Preview,
    /// A preview that cannot be built in its current location
    ForbiddenPreview,
}

impl GhostKind {
    /// The material associated with each ghostly structure.
    pub(crate) fn material(&self) -> StandardMaterial {
        use crate::asset_management::palette::{
            FORBIDDEN_PREVIEW_COLOR, GHOST_COLOR, PREVIEW_COLOR, SELECTED_GHOST_COLOR,
        };

        let base_color = match self {
            GhostKind::Ghost => GHOST_COLOR,
            GhostKind::SelectedGhost => SELECTED_GHOST_COLOR,
            GhostKind::Preview => PREVIEW_COLOR,
            GhostKind::ForbiddenPreview => FORBIDDEN_PREVIEW_COLOR,
        };

        StandardMaterial {
            base_color,
            ..Default::default()
        }
    }
}

/// A marker component that indicates that this structure is planned to be built, rather than actually existing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Preview;

/// The set of components needed to spawn a structure preview.
#[derive(Bundle)]
pub(super) struct PreviewBundle {
    /// Marker component
    preview: Preview,
    /// The location of the preview
    tile_pos: TilePos,
    /// The variety of structure
    structure_id: Id<Structure>,
    /// The direction the preview is facing
    facing: Facing,
    /// The material to be used by all children in the scene
    inherited_material: InheritedMaterial,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
}

impl PreviewBundle {
    /// Creates a new [`PreviewBundle`].
    pub(super) fn new(
        tile_pos: TilePos,
        data: ClipboardData,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        PreviewBundle {
            preview: Preview,
            tile_pos,
            structure_id: data.structure_id,
            facing: data.facing,
            inherited_material,
            scene_bundle: SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(world_pos),
                ..default()
            },
        }
    }
}

/// Computes the correct signals for ghosts to send throughout their lifecycle
// TODO: use a `Ref` instead of &mut in Bevy 0.10
pub(super) fn ghost_signals(
    mut ghost_query: Query<
        (
            &Id<Structure>,
            &mut Emitter,
            &mut CraftingState,
            &InputInventory,
        ),
        With<Ghost>,
    >,
) {
    /// The rate at which neglect grows for each cycle
    ///
    /// Should be positive.
    const NEGLECT_RATE: f32 = 0.05;

    // Ghosts that are ignored will slowly become more important to build.
    for (&structure_id, mut emitter, crafting_state, input_inventory) in ghost_query.iter_mut() {
        // Always increase neglect over time
        emitter.neglect_multiplier += NEGLECT_RATE;

        if crafting_state.is_changed() {
            match *crafting_state {
                CraftingState::NeedsInput => {
                    // Emit signals to cause workers to bring the correct item to this ghost
                    for item_slot in input_inventory.iter() {
                        let signal_type = SignalType::Pull(item_slot.item_id());
                        let signal_strength = SignalStrength::new(10.);
                        emitter.signals.push((signal_type, signal_strength))
                    }
                }
                CraftingState::InProgress {
                    progress: _,
                    required: _,
                    work_required,
                    worker_present: _,
                } => {
                    // Wipe out any pull signals as we've already got enough stuff.
                    emitter.signals.clear();

                    if work_required {
                        let signal_type = SignalType::Work(structure_id);
                        let signal_strength = SignalStrength::new(10.);
                        emitter.signals.push((signal_type, signal_strength))
                    }
                }
                _ => (),
            }
        }
    }
}

/// Manages the progression of ghosts from input needed -> work needed -> built.
///
/// Transforms ghosts into structures once all of their construction materials have been supplied and enough work has been performed.
pub(super) fn ghost_lifecyle(
    mut ghost_query: Query<
        (
            &mut CraftingState,
            &InputInventory,
            &TilePos,
            &Id<Structure>,
            &Facing,
            &ActiveRecipe,
        ),
        With<Ghost>,
    >,
    structure_manifest: Res<StructureManifest>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut crafting_state, input_inventory, &tile_pos, &structure_id, &facing, active_recipe) in
        ghost_query.iter_mut()
    {
        match *crafting_state {
            CraftingState::NeedsInput => {
                *crafting_state = match input_inventory.is_full() {
                    true => {
                        let structure_details = structure_manifest.get(structure_id);
                        CraftingState::InProgress {
                            progress: Duration::ZERO,
                            required: structure_details.build_duration,
                            work_required: structure_details.build_duration > Duration::ZERO,
                            worker_present: false,
                        }
                    }
                    false => CraftingState::NeedsInput,
                };
            }
            CraftingState::InProgress {
                progress,
                required,
                work_required,
                worker_present,
            } => {
                let mut updated_progress = progress;

                if !work_required || worker_present {
                    updated_progress += time.delta();
                }

                *crafting_state = if updated_progress >= required {
                    CraftingState::RecipeComplete
                } else {
                    CraftingState::InProgress {
                        progress: updated_progress,
                        required,
                        work_required,
                        worker_present,
                    }
                }
            }
            CraftingState::RecipeComplete => {
                commands.despawn_ghost(tile_pos);
                commands.spawn_structure(
                    tile_pos,
                    ClipboardData {
                        structure_id,
                        facing,
                        active_recipe: active_recipe.clone(),
                    },
                );
            }
            _ => unreachable!(),
        }
    }
}
