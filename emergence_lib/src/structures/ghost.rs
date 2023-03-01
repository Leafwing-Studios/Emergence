//! Ghosts are translucent phantom structures, used to show structures that could be or are planned to be built.
//!
//! There is an important distinction between "ghosts" and "previews", even though they appear similarly to players.
//! Ghosts are buildings that are genuinely planned to be built.
//! Previews are simply hovered, and used as a visual aid to show placement.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate::{
    asset_management::manifest::{Id, Structure},
    player_interaction::{clipboard::ClipboardData, selection::ObjectInteraction},
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{Facing, TilePos},
};

use super::crafting::{ActiveRecipe, InputInventory};

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
    /// Render this entity in a translucent style
    ghostly: Ghostly,
    /// The location of the ghost
    tile_pos: TilePos,
    /// The variety of structure
    structure_id: Id<Structure>,
    /// The direction the ghost is facing
    facing: Facing,
    /// The items required to actually seed this item
    construction_materials: InputInventory,
    /// What should the structure craft when it is first built?
    active_recipe: ActiveRecipe,
    /// How is this structure being interacted with
    object_interaction: ObjectInteraction,
    /// Makes structures pickable
    raycast_mesh: RaycastMesh<Ghost>,
    /// The mesh used for raycasting
    picking_mesh: Handle<Mesh>,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
    /// Emits signals, drawing units towards this ghost to build it
    emitter: Emitter,
}

impl GhostBundle {
    /// Creates a new [`GhostBundle`].
    pub(super) fn new(
        tile_pos: TilePos,
        data: ClipboardData,
        construction_materials: InputInventory,
        picking_mesh: Handle<Mesh>,
        scene_handle: Handle<Scene>,
        world_pos: Vec3,
    ) -> Self {
        // Emit signals to cause workers to bring the correct item to this ghost
        let mut emitter = Emitter::default();
        for item_slot in construction_materials.iter() {
            let signal_type = SignalType::Pull(item_slot.item_id());
            let signal_strength = SignalStrength::new(10.);
            emitter.signals.push((signal_type, signal_strength))
        }

        GhostBundle {
            ghost: Ghost,
            ghostly: Ghostly,
            tile_pos,
            structure_id: data.structure_id,
            facing: data.facing,
            construction_materials,
            active_recipe: data.active_recipe,
            object_interaction: ObjectInteraction::None,
            raycast_mesh: RaycastMesh::default(),
            picking_mesh,
            scene_bundle: SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(world_pos),
                ..default()
            },
            emitter,
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
    /// Render this entity in a translucent style
    ghostly: Ghostly,
    /// The location of the preview
    tile_pos: TilePos,
    /// The variety of structure
    structure_id: Id<Structure>,
    /// The direction the preview is facing
    facing: Facing,
    /// How is this structure being interacted with
    object_interaction: ObjectInteraction,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
}

impl PreviewBundle {
    /// Creates a new [`PreviewBundle`].
    pub(super) fn new(
        tile_pos: TilePos,
        data: ClipboardData,
        scene_handle: Handle<Scene>,
        world_pos: Vec3,
    ) -> Self {
        PreviewBundle {
            preview: Preview,
            ghostly: Ghostly,
            tile_pos,
            structure_id: data.structure_id,
            facing: data.facing,
            object_interaction: ObjectInteraction::Hovered,
            scene_bundle: SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(world_pos),
                ..default()
            },
        }
    }
}

/// Ghosts that are ignored will slowly become more important to build.
pub(super) fn increase_ghost_neglect(mut ghost_query: Query<&mut Emitter, With<Ghost>>) {
    /// The rate at which neglect grows for each cycle
    ///
    /// Should be positive.
    const NEGLECT_RATE: f32 = 0.05;

    for mut emitter in ghost_query.iter_mut() {
        emitter.neglect_multiplier += NEGLECT_RATE;
    }
}
