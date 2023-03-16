//! Generating and representing terrain as game objects.

use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate::asset_management::manifest::{Id, Terrain, TerrainManifest};
use crate::asset_management::terrain::TerrainHandles;
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
struct TerrainBundle {
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
    fn new(
        terrain_id: Id<Terrain>,
        tile_pos: TilePos,
        scene: Handle<Scene>,
        map_geometry: &MapGeometry,
    ) -> Self {
        let world_pos = tile_pos.into_world_pos(map_geometry);
        let scene_bundle = SceneBundle {
            scene,
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
    mut terrain_query: Query<(Ref<Height>, &TilePos, &mut Transform, &Children)>,
    mut column_query: Query<&mut Transform, (With<Parent>, Without<Height>)>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (height, &tile_pos, mut transform, children) in terrain_query.iter_mut() {
        if height.is_changed() {
            map_geometry.update_height(tile_pos, *height);
            transform.translation.y = height.into_world_pos();
            // During terrain initialization we ensure that the column is always the 0th child
            let column_child = children[0];
            let mut column_transform = column_query.get_mut(column_child).unwrap();
            *column_transform = height.column_transform();
        }
    }
}

pub(crate) struct SpawnTerrainCommand {
    pub(crate) tile_pos: TilePos,
    pub(crate) height: Height,
    pub(crate) terrain_id: Id<Terrain>,
}

impl Command for SpawnTerrainCommand {
    fn write(self, world: &mut World) {
        let handles = world.resource::<TerrainHandles>();
        let scene_handle = handles.scenes.get(&self.terrain_id).unwrap().clone_weak();

        let mut map_geometry = world.resource_mut::<MapGeometry>();

        // Store the height, so it can be used below
        map_geometry.update_height(self.tile_pos, self.height);

        // Drop the borrow so the borrow checker is happy
        let map_geometry = world.resource::<MapGeometry>();

        // Spawn the terrain entity
        let terrain_entity = world
            .spawn(TerrainBundle::new(
                self.terrain_id,
                self.tile_pos,
                scene_handle,
                &map_geometry,
            ))
            .id();

        // Spawn the column as the 0th child of the tile entity
        // The scene bundle will be added as the first child
        let handles = world.resource::<TerrainHandles>();
        let bundle = PbrBundle {
            mesh: handles.column_mesh.clone_weak(),
            material: handles.column_material.clone_weak(),
            ..Default::default()
        };

        let hex_column = world.spawn(bundle).id();
        world.entity_mut(terrain_entity).add_child(hex_column);

        // Update the index of what terrain is where
        let mut map_geometry = world.resource_mut::<MapGeometry>();
        map_geometry
            .terrain_index
            .insert(self.tile_pos, terrain_entity);
    }
}
