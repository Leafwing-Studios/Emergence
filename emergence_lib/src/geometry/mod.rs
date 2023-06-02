//! Manages the game world's grid and data tied to that grid

mod indexing;
pub use indexing::MapGeometry;

mod meshes;
pub(crate) use meshes::hexagonal_column;

mod position;
pub use position::{Height, TilePos, Volume};

mod rotation;
pub(crate) use rotation::{
    direction_from_angle, sync_rotation_to_facing, Facing, RotationDirection,
};

mod voxels;
