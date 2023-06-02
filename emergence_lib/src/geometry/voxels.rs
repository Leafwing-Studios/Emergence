use bevy::prelude::*;

/// A single object stored in a voxel.
///
/// Each voxel can contain at most one object.
pub(crate) struct VoxelObject {
    /// The entity that represents this object in the ECS.
    entity: Entity,
    /// The kind of object stored in this voxel.
    object_kind: VoxelKind,
}

/// A variety of object stored in the voxel grid.
///
/// Each voxel can contain at most one object.
pub(crate) enum VoxelKind {
    Litter {
        full: bool,
    },
    Terrain,
    Structure {
        can_walk_on_top_of: bool,
        can_walk_through: bool,
    },
    GhostStructure,
    GhostTerrain,
}

impl VoxelKind {
    /// Can units walk over over the voxel on top of this object?
    pub(super) fn can_walk_on_top_of(&self) -> bool {
        match self {
            VoxelKind::Litter { .. } => false,
            VoxelKind::Terrain => true,
            VoxelKind::Structure {
                can_walk_on_top_of, ..
            } => *can_walk_on_top_of,
            VoxelKind::GhostStructure => false,
            VoxelKind::GhostTerrain => false,
        }
    }

    /// Can units walk through the voxel occupied by this object?
    pub(super) fn can_walk_through(&self) -> bool {
        match self {
            VoxelKind::Litter { full } => !full,
            VoxelKind::Terrain => false,
            VoxelKind::Structure {
                can_walk_through, ..
            } => *can_walk_through,
            VoxelKind::GhostStructure => true,
            VoxelKind::GhostTerrain => true,
        }
    }
}
