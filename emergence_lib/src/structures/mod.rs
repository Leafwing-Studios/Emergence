//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate::{
    asset_management::{
        manifest::{plugin::ManifestPlugin, Id},
        AssetCollectionExt,
    },
    player_interaction::{clipboard::ClipboardData, selection::ObjectInteraction},
    simulation::{
        geometry::{Facing, TilePos},
        SimulationSet,
    },
};

use self::{
    construction::{ghost_lifecycle, ghost_signals, validate_ghosts},
    structure_assets::StructureHandles,
    structure_manifest::{RawStructureManifest, Structure},
};

pub(crate) mod commands;
pub mod construction;
mod structure_assets;
pub mod structure_manifest;

/// The systems that make structures tick.
pub(super) struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ManifestPlugin::<RawStructureManifest>::new())
            .add_asset_collection::<StructureHandles>()
            .add_systems(
                (
                    validate_ghosts,
                    ghost_signals.after(validate_ghosts),
                    ghost_lifecycle.after(validate_ghosts),
                )
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );
    }
}

/// The data needed to build a structure
#[derive(Bundle)]
struct StructureBundle {
    /// Unique identifier of structure variety
    structure: Id<Structure>,
    /// The direction this structure is facing
    facing: Facing,
    /// The location of this structure
    tile_pos: TilePos,
    /// Makes structures pickable
    raycast_mesh: RaycastMesh<Id<Structure>>,
    /// How is this structure being interacted with
    object_interaction: ObjectInteraction,
    /// The mesh used for raycasting
    picking_mesh: Handle<Mesh>,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
}

impl StructureBundle {
    /// Creates a new structure
    fn new(
        tile_pos: TilePos,
        data: ClipboardData,
        picking_mesh: Handle<Mesh>,
        scene_handle: Handle<Scene>,
        world_pos: Vec3,
    ) -> Self {
        StructureBundle {
            structure: data.structure_id,
            facing: data.facing,
            tile_pos,
            raycast_mesh: RaycastMesh::default(),
            object_interaction: ObjectInteraction::None,
            picking_mesh,
            scene_bundle: SceneBundle {
                scene: scene_handle,
                transform: Transform::from_translation(world_pos),
                ..Default::default()
            },
        }
    }
}
