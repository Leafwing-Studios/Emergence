//! Generating and representing terrain as game objects.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate as emergence_lib;

use crate::asset_management::terrain::TerrainHandles;
use crate::player_interaction::zoning::Zoning;
use crate::simulation::geometry::{MapGeometry, TilePos};
use bevy::ecs::component::Component;
use derive_more::Display;

use emergence_macros::IterableEnum;

/// Available terrain types.
#[derive(Component, Clone, Copy, Hash, Eq, PartialEq, IterableEnum, Debug, Display)]
pub(crate) enum Terrain {
    /// Terrain with no distinguishing characteristics.
    Plain,
    /// Terrain that is rocky, and thus difficult to traverse.
    Rocky,
    /// Terrain that is unusually muddy.
    Muddy,
}

impl Terrain {
    /// The walking speed multiplier associated with this terrain type.
    ///
    /// These values should always be strictly positive.
    /// Higher values make units walk faster.
    pub(crate) const fn walking_speed(&self) -> f32 {
        match self {
            Terrain::Plain => 1.0,
            Terrain::Rocky => 2.0,
            Terrain::Muddy => 0.5,
        }
    }

    /// The rendering material associated with this terrain type.
    pub(crate) fn material(&self) -> StandardMaterial {
        let base_color = match self {
            Terrain::Plain => Color::BEIGE,
            Terrain::Rocky => Color::GRAY,
            Terrain::Muddy => Color::BISQUE,
        };

        StandardMaterial {
            base_color,
            perceptual_roughness: 0.6,
            metallic: 0.01,
            ..Default::default()
        }
    }
}

/// All of the components needed to define a piece of terrain.
#[derive(Bundle)]
pub(crate) struct TerrainBundle {
    /// The type of terrain
    terrain_type: Terrain,
    /// The location of this terrain hex
    tile_pos: TilePos,
    /// Makes the tiles pickable
    raycast_mesh: RaycastMesh<Terrain>,
    /// The structure that should be built here.
    zoning: Zoning,
    /// The mesh and material used
    pbr_bundle: PbrBundle,
}

impl TerrainBundle {
    /// Creates a new Terrain entity.
    pub(crate) fn new(
        terrain_type: Terrain,
        tile_pos: TilePos,
        handles: &TerrainHandles,
        map_geometry: &MapGeometry,
    ) -> Self {
        let world_pos = tile_pos.into_world_pos(map_geometry);

        let pbr_bundle = PbrBundle {
            mesh: handles.mesh.clone_weak(),
            material: handles
                .terrain_materials
                .get(&terrain_type)
                .unwrap()
                .clone_weak(),
            transform: Transform::from_xyz(world_pos.x, 0.0, world_pos.z).with_scale(Vec3 {
                x: 1.,
                y: world_pos.y,
                z: 1.,
            }),
            ..default()
        };

        TerrainBundle {
            terrain_type,
            tile_pos,
            raycast_mesh: RaycastMesh::<Terrain>::default(),
            zoning: Zoning::None,
            pbr_bundle,
        }
    }
}
