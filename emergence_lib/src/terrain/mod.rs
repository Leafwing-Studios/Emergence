//! Generating and representing terrain as game objects.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate::asset_management::manifest::{Id, Terrain};
use crate::asset_management::terrain::TerrainHandles;
use crate::player_interaction::zoning::Zoning;
use crate::simulation::geometry::{MapGeometry, TilePos};

#[derive(Debug)]
pub(crate) struct TerrainData {
    /// The walking speed multiplier associated with this terrain type.
    ///
    /// These values should always be strictly positive.
    /// Higher values make units walk faster.
    /// 1.0 is "normal speed".
    walking_speed: f32,
}

impl TerrainData {
    pub(crate) fn walking_speed(&self) -> f32 {
        self.walking_speed
    }
}

/// All of the components needed to define a piece of terrain.
#[derive(Bundle)]
pub(crate) struct TerrainBundle {
    /// The type of terrain
    terrain_id: Id<Terrain>,
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
        terrain_id: Id<Terrain>,
        tile_pos: TilePos,
        handles: &TerrainHandles,
        map_geometry: &MapGeometry,
    ) -> Self {
        let world_pos = tile_pos.into_world_pos(map_geometry);

        let pbr_bundle = PbrBundle {
            mesh: handles.mesh.clone_weak(),
            material: handles
                .terrain_materials
                .get(&terrain_id)
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
            terrain_id,
            tile_pos,
            raycast_mesh: RaycastMesh::<Terrain>::default(),
            zoning: Zoning::None,
            pbr_bundle,
        }
    }
}
