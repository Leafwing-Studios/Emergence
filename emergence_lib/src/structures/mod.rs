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
    geometry::{DiscreteHeight, Facing, Height, MapGeometry, VoxelPos},
    player_interaction::{
        clipboard::ClipboardData, picking::PickableVoxel, selection::ObjectInteraction,
    },
};

use self::{
    logistic_buildings::LogisticsPlugin,
    structure_assets::StructureHandles,
    structure_manifest::{RawStructureManifest, Structure},
};

pub(crate) mod commands;
pub(crate) mod logistic_buildings;
mod structure_assets;
pub mod structure_manifest;

/// The systems that make structures tick.
pub(super) struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ManifestPlugin::<RawStructureManifest>::new())
            .add_plugin(LogisticsPlugin)
            .add_asset_collection::<StructureHandles>();
    }
}

/// The data needed to build a structure
#[derive(Bundle)]
struct StructureBundle {
    /// Unique identifier of structure variety
    structure: Id<Structure>,
    /// The footprint of this structure
    footprint: Footprint,
    /// The direction this structure is facing
    facing: Facing,
    /// The location of this structure
    voxel_pos: VoxelPos,
    /// Makes structures pickable
    raycast_mesh: RaycastMesh<PickableVoxel>,
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
        voxel_pos: VoxelPos,
        footprint: Footprint,
        data: ClipboardData,
        picking_mesh: Handle<Mesh>,
        scene_handle: Handle<Scene>,
        world_pos: Vec3,
    ) -> Self {
        StructureBundle {
            structure: data.structure_id,
            footprint,
            facing: data.facing,
            voxel_pos,
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
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    /// The set of tiles is taken up by this structure.
    pub(crate) set: HashSet<VoxelPos>,
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
            set: HashSet::from_iter(vec![VoxelPos::ZERO]),
        }
    }

    /// A footprint that occupies a set of tiles in a solid hexagon.
    pub fn hexagon(radius: u32) -> Self {
        let mut set = HashSet::new();
        for hex in hexagon(Hex::ZERO, radius) {
            set.insert(VoxelPos {
                hex,
                height: DiscreteHeight::ZERO,
            });
        }

        Footprint { set }
    }

    /// Computes the set of tiles that this footprint occupies in world space, when centered at `center`.
    fn in_world_space(&self, center: VoxelPos) -> HashSet<VoxelPos> {
        self.set
            .iter()
            .map(|&offset| center + offset)
            .collect::<HashSet<_>>()
    }

    /// Rotates the footprint by the provided [`Facing`].
    fn rotated(&self, facing: Facing) -> Self {
        let mut set = HashSet::new();
        for &voxel_pos in self.set.iter() {
            set.insert(voxel_pos.rotated(facing));
        }

        Footprint { set }
    }

    /// Returns this footprint after correcting for offset and rotation.
    pub(crate) fn normalized(&self, facing: Facing, center: VoxelPos) -> HashSet<VoxelPos> {
        let rotated = self.rotated(facing);
        rotated.in_world_space(center)
    }

    /// Returns the highest height of tiles in this footprint after normalization.
    ///
    /// Returns [`Height::ZERO`] if the footprint is empty or no valid tiles are found.
    pub(crate) fn height(
        &self,
        facing: Facing,
        center: VoxelPos,
        map_geometry: &MapGeometry,
    ) -> Option<DiscreteHeight> {
        self.normalized(facing, center)
            .iter()
            .map(|&voxel_pos| map_geometry.get_height(voxel_pos.hex).unwrap_or_default())
            .reduce(|a, b| a.max(b))
    }

    /// Returns the highest normalized height of tiles in this footprint.
    pub(crate) fn max_height(&self) -> DiscreteHeight {
        self.set
            .iter()
            .map(|&voxel_pos| voxel_pos.height)
            .reduce(|a, b| a.max(b))
            .unwrap_or_default()
    }

    /// Computes the translation (in world space) of the center of this footprint.
    ///
    /// Uses the height of the first tile in the footprint.
    pub(crate) fn world_pos(
        &self,
        facing: Facing,
        center: VoxelPos,
        map_geometry: &MapGeometry,
    ) -> Option<Vec3> {
        let mut transform_of_center = center.into_world_pos();

        let structure_height = self.height(facing, center, map_geometry)?;

        // Adjust the height in case the structure does not cover the origin tile of the footprint.
        // This occurs in bridges, which are elevated above the ground.
        transform_of_center.y = structure_height.into_world_pos() + Height::TOPPER_THICKNESS;

        Some(transform_of_center)
    }
}

/// A special structure used to create interest in the game world.
///
/// Landmarks cannot be created or destroyed by players.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct Landmark;

#[cfg(test)]
mod tests {
    use super::*;

    /// A footprint that occupies a line of two adjacent tiles,
    /// beginning at the origin and moving right one.
    ///
    /// This is a horizontal line, in the axial coordinate diagram shown here:
    /// <https://www.redblobgames.com/grids/hexagons/#coordinates>
    fn two_tile_footprint() -> Footprint {
        let mut set = HashSet::new();
        set.insert(VoxelPos::ZERO);
        set.insert(VoxelPos::from_xy(1, 0));

        Footprint { set }
    }

    #[test]
    fn hexagon_footprint_matches() {
        let footprint = Footprint::hexagon(1);
        let expected = HashSet::from_iter(vec![
            VoxelPos::ZERO,
            VoxelPos::from_xy(0, 1),
            VoxelPos::from_xy(1, 0),
            VoxelPos::from_xy(1, -1),
            VoxelPos::from_xy(0, -1),
            VoxelPos::from_xy(-1, 0),
            VoxelPos::from_xy(-1, 1),
        ]);

        assert_eq!(footprint.set, expected);
    }

    #[test]
    fn footprint_in_world_space_is_correct_at_origin() {
        let footprint = two_tile_footprint();
        let expected = HashSet::from_iter(vec![VoxelPos::from_xy(0, 0), VoxelPos::from_xy(1, 0)]);

        assert_eq!(footprint.in_world_space(VoxelPos::ZERO), expected);
    }

    #[test]
    fn footprint_in_world_space_is_correct_at_non_origin() {
        let footprint = two_tile_footprint();

        let expected = HashSet::from_iter(vec![VoxelPos::from_xy(1, 0), VoxelPos::from_xy(2, 0)]);
        assert_eq!(footprint.in_world_space(VoxelPos::from_xy(1, 0)), expected);

        let expected = HashSet::from_iter(vec![VoxelPos::from_xy(0, 1), VoxelPos::from_xy(1, 1)]);
        assert_eq!(footprint.in_world_space(VoxelPos::from_xy(0, 1)), expected);
    }

    #[test]
    fn footprint_rotated_by_0_is_unchanged() {
        let footprint = two_tile_footprint();
        assert_eq!(footprint.rotated(Facing::default()), footprint);
    }

    #[test]
    fn footprint_rotated_by_60_is_correct() {
        let footprint = two_tile_footprint();
        let expected = Footprint {
            set: HashSet::from_iter(vec![VoxelPos::from_xy(0, 0), VoxelPos::from_xy(0, 1)]),
        };

        let mut facing = Facing::default();
        facing.rotate_clockwise();

        assert_eq!(footprint.rotated(facing), expected);

        let mut facing = Facing::default();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();

        assert_eq!(footprint.rotated(facing), expected);
    }

    #[test]
    fn footprint_rotated_by_120_is_correct() {
        let footprint = two_tile_footprint();
        let expected = Footprint {
            set: HashSet::from_iter(vec![VoxelPos::from_xy(0, 0), VoxelPos::from_xy(-1, 1)]),
        };

        let mut facing = Facing::default();
        facing.rotate_clockwise();
        facing.rotate_clockwise();

        assert_eq!(footprint.rotated(facing), expected);

        let mut facing = Facing::default();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();

        assert_eq!(footprint.rotated(facing), expected);
    }

    #[test]
    fn footprint_rotated_by_180_is_correct() {
        let footprint = two_tile_footprint();
        let expected = Footprint {
            set: HashSet::from_iter(vec![VoxelPos::from_xy(0, 0), VoxelPos::from_xy(-1, 0)]),
        };

        let mut facing = Facing::default();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();

        assert_eq!(footprint.rotated(facing), expected);

        let mut facing = Facing::default();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();

        assert_eq!(footprint.rotated(facing), expected);
    }

    #[test]
    fn footprint_rotated_by_240_is_correct() {
        let footprint = two_tile_footprint();
        let expected = Footprint {
            set: HashSet::from_iter(vec![VoxelPos::from_xy(0, 0), VoxelPos::from_xy(0, -1)]),
        };

        let mut facing = Facing::default();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();

        assert_eq!(footprint.rotated(facing), expected);

        let mut facing = Facing::default();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();

        assert_eq!(footprint.rotated(facing), expected);
    }

    #[test]
    fn footprint_rotated_by_300_is_correct() {
        let footprint = two_tile_footprint();
        let expected = Footprint {
            set: HashSet::from_iter(vec![VoxelPos::from_xy(0, 0), VoxelPos::from_xy(1, -1)]),
        };

        let mut facing = Facing::default();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();

        assert_eq!(footprint.rotated(facing), expected);

        let mut facing = Facing::default();
        facing.rotate_counterclockwise();

        assert_eq!(footprint.rotated(facing), expected);
    }

    #[test]
    fn footprint_rotated_by_360_is_unchanged() {
        let footprint = two_tile_footprint();
        let mut facing = Facing::default();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();
        facing.rotate_clockwise();

        assert_eq!(footprint.rotated(facing), footprint);

        let mut facing = Facing::default();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();
        facing.rotate_counterclockwise();

        assert_eq!(footprint.rotated(facing), footprint);
    }
}
