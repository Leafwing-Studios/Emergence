//! Rendering and animation logic.

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use hexx::{Hex, HexLayout, MeshInfo};

use crate::{
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

fn populate_terrain(
    new_terrain: Query<(Entity, &TilePos, &Terrain), Added<Terrain>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_geometry: Res<MapGeometry>,
) {
    // mesh
    let mesh = hexagonal_plane(&map_geometry.layout);
    let mesh_handle = meshes.add(mesh);

    for (terrain_entity, tile_pos, terrain) in new_terrain.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);

        let color = match terrain {
            Terrain::Plain => Color::WHITE,
            Terrain::High => Color::YELLOW,
            Terrain::Rocky => Color::RED,
        };

        // PERF: this is wildly inefficient and lazy. Store the handles instead!
        let material = materials.add(color.into());

        commands.entity(terrain_entity).insert(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(pos.x, 0.0, pos.y),
            ..default()
        });
    }
}

fn populate_structures(
    new_structures: Query<(Entity, &TilePos), Added<Structure>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_geometry: Res<MapGeometry>,
) {
    const SIZE: f32 = 1.0;
    const OFFSET: f32 = -SIZE / 2.;

    let mesh = Mesh::from(shape::Cube { size: SIZE });
    let mesh_handle = meshes.add(mesh);

    for (entity, tile_pos) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);

        // PERF: this is wildly inefficient and lazy. Store the handles instead!
        let material = materials.add(Color::PINK.into());

        commands.entity(entity).insert(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(pos.x, OFFSET, pos.y),
            ..default()
        });
    }
}

fn populate_units(
    new_structures: Query<(Entity, &TilePos), Added<Unit>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_geometry: Res<MapGeometry>,
) {
    const SIZE: f32 = 0.5;
    const OFFSET: f32 = -SIZE / 2.;

    let mesh = Mesh::from(shape::Cube { size: SIZE });
    let mesh_handle = meshes.add(mesh);

    for (entity, tile_pos) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);

        // PERF: this is wildly inefficient and lazy. Store the handles instead!
        let material = materials.add(Color::BLACK.into());

        commands.entity(entity).insert(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material.clone(),
            transform: Transform::from_xyz(pos.x, OFFSET, pos.y),
            ..default()
        });
    }
}

fn hexagonal_plane(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = MeshInfo::hexagonal_column(hex_layout, Hex::ZERO, 1.0);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs.to_vec());
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
}
