//! Generating and representing terrain as game objects.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate::asset_management::manifest::{Id, Terrain, TerrainManifest};
use crate::player_interaction::zoning::Zoning;
use crate::simulation::geometry::{Height, MapGeometry, TilePos};

/// All logic and initialization needed for terrain.
pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainManifest>()
            .add_system(respond_to_height_changes);
    }
}

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
    /// Constructs a new [`TerrainData`] object
    pub(crate) fn new(walking_speed: f32) -> Self {
        TerrainData { walking_speed }
    }

    /// Returns the relative walking speed of units on this terrain
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
    /// The height of this terrain hex
    height: Height,
    /// Makes the tiles pickable
    raycast_mesh: RaycastMesh<Terrain>,
    /// The structure that should be built here.
    zoning: Zoning,
    /// The scene used to construct the terrain tile
    scene_bundle: SceneBundle,
}

impl TerrainBundle {
    /// Creates a new Terrain entity.
    pub(crate) fn new(
        terrain_id: Id<Terrain>,
        tile_pos: TilePos,
        scene_handle: &Handle<Scene>,
        map_geometry: &MapGeometry,
    ) -> Self {
        let world_pos = tile_pos.into_world_pos(map_geometry);
        let scene_bundle = SceneBundle {
            scene: scene_handle.clone_weak(),
            transform: Transform::from_translation(world_pos),
            ..Default::default()
        };

        let height = map_geometry.get_height(tile_pos).unwrap();

        TerrainBundle {
            terrain_id,
            tile_pos,
            height,
            raycast_mesh: RaycastMesh::<Terrain>::default(),
            zoning: Zoning::None,
            scene_bundle,
        }
    }
}

/// Updates the game state appropriately whenever the height of a tile is changed.
fn respond_to_height_changes(
    mut terrain_query: Query<(Ref<Height>, &TilePos, &mut Transform)>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (height, &tile_pos, mut transform) in terrain_query.iter_mut() {
        if height.is_changed() {
            map_geometry.update_height(tile_pos, *height);
            transform.translation.y = height.into_world_pos();
        }
    }
}
