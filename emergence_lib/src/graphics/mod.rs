//! Rendering and animation logic.

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use hexx::{Hex, HexLayout, MeshInfo};
use rand::{distributions::Uniform, thread_rng, Rng};

use crate::{
    asset_management::TileHandles,
    organisms::units::Unit,
    simulation::geometry::{MapGeometry, TilePos},
    structures::Structure,
    terrain::Terrain,
};

use self::lighting::LightingPlugin;

mod lighting;

/// Adds all logic required to render the game.
///
/// The game should be able to run and function without this plugin: no gameplay logic allowed!
pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LightingPlugin)
            .add_system_to_stage(CoreStage::PostUpdate, populate_terrain)
            .add_system_to_stage(CoreStage::PostUpdate, populate_units)
            .add_system_to_stage(CoreStage::PostUpdate, populate_structures);
    }
}

/// Adds rendering components to every spawned terrain tile
fn populate_terrain(
    new_terrain: Query<(Entity, &TilePos, &Terrain), Added<Terrain>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TileHandles>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    let mut rng = thread_rng();

    for (terrain_entity, tile_pos, terrain) in new_terrain.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        // TODO: this should be refactored out of graphics
        let hex_height = rng.sample(Uniform::new(1., 3.));

        let mesh = hexagonal_column(&map_geometry.layout, hex_height);
        let mesh_handle = meshes.add(mesh);

        commands.entity(terrain_entity).insert(PbrBundle {
            mesh: mesh_handle.clone(),
            material: materials.terrain_handles.get(terrain).unwrap().clone_weak(),
            transform: Transform::from_xyz(pos.x, 0.0, pos.y),
            ..default()
        });

        // TODO: this should be refactored out of graphics
        map_geometry.height_index.insert(*tile_pos, hex_height);
        map_geometry.terrain_index.insert(*tile_pos, terrain_entity);
    }
}

/// Adds rendering components to every spawned structure
fn populate_structures(
    new_structures: Query<(Entity, &TilePos), Added<Structure>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_geometry: Res<MapGeometry>,
) {
    /// The size of a single structure
    const SIZE: f32 = 1.0;
    /// The offset required to have a structure sit on top of the tile correctly
    const OFFSET: f32 = SIZE / 2.0;

    let mesh = Mesh::from(shape::Cube { size: SIZE });
    let mesh_handle = meshes.add(mesh);

    for (entity, tile_pos) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let terrain_height = map_geometry.height_index.get(tile_pos).unwrap();

        // PERF: this is wildly inefficient and lazy. Store the handles instead!
        let material = materials.add(Color::PINK.into());

        commands.entity(entity).insert(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(pos.x, terrain_height + OFFSET, pos.y),
            ..default()
        });
    }
}

/// Adds rendering components to every spawned unit
fn populate_units(
    new_structures: Query<(Entity, &TilePos), Added<Unit>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_geometry: Res<MapGeometry>,
) {
    /// The size of a single unit
    const SIZE: f32 = 0.5;
    /// The offset required to have a unit stand on top of the tile correctly
    const OFFSET: f32 = SIZE / 2.0;

    let mesh = Mesh::from(shape::Cube { size: SIZE });
    let mesh_handle = meshes.add(mesh);

    for (entity, tile_pos) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let terrain_height = map_geometry.height_index.get(tile_pos).unwrap();

        // PERF: this is wildly inefficient and lazy. Store the handles instead!
        let material = materials.add(Color::BLACK.into());

        commands.entity(entity).insert(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(pos.x, terrain_height + OFFSET, pos.y),
            ..default()
        });
    }
}

/// Constructs the mesh for a single hexagonal column
fn hexagonal_column(hex_layout: &HexLayout, hex_height: f32) -> Mesh {
    let mesh_info = MeshInfo::partial_hexagonal_column(hex_layout, Hex::ZERO, hex_height);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs.to_vec());
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
}
