use bevy::prelude::*;

use crate::items::inventory::InventoryState;

/// A single object stored in a voxel.
///
/// Each voxel can contain at most one object.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VoxelObject {
    /// The entity that represents this object in the ECS.
    pub entity: Entity,
    /// The kind of object stored in this voxel.
    pub object_kind: VoxelKind,
}

/// A variety of object stored in the voxel grid.
///
/// Each voxel can contain at most one object.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VoxelKind {
    Litter {
        inventory_state: InventoryState,
    },
    Terrain,
    Structure {
        can_walk_on_roof: bool,
        can_walk_through: bool,
    },
    GhostStructure,
}

impl VoxelKind {
    /// Can units walk over over the voxel on top of this object?
    pub(super) fn can_walk_on_roof(&self) -> bool {
        match self {
            VoxelKind::Litter { inventory_state } => match inventory_state {
                InventoryState::Empty => false,
                InventoryState::Partial { .. } => false,
                InventoryState::Full => true,
            },
            VoxelKind::Terrain => true,
            VoxelKind::Structure {
                can_walk_on_roof, ..
            } => *can_walk_on_roof,
            VoxelKind::GhostStructure => false,
        }
    }

    /// Can units walk through the voxel occupied by this object?
    pub(super) fn can_walk_through(&self) -> bool {
        match self {
            VoxelKind::Litter { inventory_state } => match inventory_state {
                InventoryState::Empty => true,
                InventoryState::Partial { .. } => true,
                InventoryState::Full => false,
            },
            VoxelKind::Terrain => false,
            VoxelKind::Structure {
                can_walk_through, ..
            } => *can_walk_through,
            VoxelKind::GhostStructure => true,
        }
    }

    /// Does this object block light?
    pub(crate) fn blocks_light(&self) -> bool {
        match self {
            VoxelKind::Litter { inventory_state } => match inventory_state {
                InventoryState::Empty => false,
                InventoryState::Partial { .. } => false,
                InventoryState::Full => true,
            },
            VoxelKind::Terrain => true,
            VoxelKind::Structure { .. } => true,
            VoxelKind::GhostStructure => false,
        }
    }

    /// Can objects be dropped off at this voxel?
    pub(crate) fn can_drop_off(&self) -> bool {
        match self {
            VoxelKind::Litter { .. } => false,
            VoxelKind::Terrain => false,
            VoxelKind::Structure { .. } => true,
            VoxelKind::GhostStructure => true,
        }
    }

    /// Can objects be picked up from this voxel?
    pub(crate) fn can_pick_up(&self) -> bool {
        match self {
            VoxelKind::Litter { .. } => true,
            VoxelKind::Terrain => false,
            VoxelKind::Structure { .. } => true,
            VoxelKind::GhostStructure => false,
        }
    }

    /// Can units perform work at this voxel?
    pub(crate) fn can_work_at(&self) -> bool {
        match self {
            VoxelKind::Litter { .. } => false,
            VoxelKind::Terrain => false,
            VoxelKind::Structure { .. } => true,
            VoxelKind::GhostStructure => true,
        }
    }
}
