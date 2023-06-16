//! Manages the game world's grid and data tied to that grid

mod indexing;
pub use indexing::MapGeometry;

mod meshes;
pub(crate) use meshes::hexagonal_column;

mod position;
pub use position::{DiscreteHeight, Height, Volume, VoxelPos};

mod rotation;
pub(crate) use rotation::{sync_rotation_to_facing, Facing, RotationDirection};

mod voxels;
pub(crate) use voxels::{VoxelKind, VoxelObject};
