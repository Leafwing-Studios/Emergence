//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::{prelude::*, utils::HashSet};
use bevy_mod_raycast::RaycastMesh;
use hexx::{shapes::hexagon, Hex};
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::{
        manifest::{plugin::ManifestPlugin, Id},
        AssetCollectionExt,
    },
    player_interaction::{clipboard::ClipboardData, selection::ObjectInteraction},
    simulation::geometry::{Facing, TilePos},
};

use self::{
    structure_assets::StructureHandles,
    structure_manifest::{RawStructureManifest, Structure},
};

pub(crate) mod commands;
mod structure_assets;
pub mod structure_manifest;

/// The systems that make structures tick.
pub(super) struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ManifestPlugin::<RawStructureManifest>::new())
            .add_asset_collection::<StructureHandles>();
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
    raycast_mesh: RaycastMesh<Structure>,
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

/// The set of tiles taken up by a structure.
///
/// Structures are always "centered" on 0, 0, so these coordinates are relative to that.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    /// The set of tiles is taken up by this structure.
    pub(crate) set: HashSet<TilePos>,
}

impl Default for Footprint {
    fn default() -> Self {
        Self::single()
    }
}

impl Footprint {
    /// A footprint that occupies a single tile.
    pub fn single() -> Self {
        Self {
            set: HashSet::from_iter(vec![TilePos::ZERO]),
        }
    }

    /// A footprint that occupies a single tile and allows units to pass over it.
    pub fn path() -> Self {
        Self {
            set: HashSet::from_iter(vec![TilePos::ZERO]),
        }
    }

    /// A footprint that occupies a set of tiles in a solid hexagon.
    pub fn hexagon(radius: u32) -> Self {
        let mut set = HashSet::new();
        for hex in hexagon(Hex::ZERO, radius) {
            set.insert(TilePos { hex });
        }

        Footprint { set }
    }

    /// Computes the set of tiles that this footprint occupies in world space, when centered at `center`.
    pub(crate) fn in_world_space(&self, center: TilePos) -> HashSet<TilePos> {
        self.set
            .iter()
            .map(|&offset| center + offset)
            .collect::<HashSet<_>>()
    }

    /// Rotates the footprint by the provided [`Facing`].
    pub(crate) fn rotated(&self, facing: &Facing) -> Self {
        let mut set = HashSet::new();
        for &tile_pos in self.set.iter() {
            set.insert(tile_pos.rotated(*facing));
        }

        Footprint { set }
    }
}
