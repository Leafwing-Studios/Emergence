//! Manages the game world's grid and data tied to that grid

mod indexing;
use hexx::HexLayout;
pub use indexing::MapGeometry;

mod meshes;
pub(crate) use meshes::hexagonal_column;

mod position;
pub use position::{DiscreteHeight, Height, Volume, VoxelPos};

mod rotation;
pub(crate) use rotation::{sync_rotation_to_facing, Facing, RotationDirection};

mod voxels;
pub(crate) use voxels::{VoxelKind, VoxelObject};

pub(crate) const MAP_LAYOUT: HexLayout = HexLayout {
    orientation: hexx::HexOrientation::Flat,
    origin: hexx::Vec2::ZERO,
    hex_size: hexx::Vec2::ONE,
};
