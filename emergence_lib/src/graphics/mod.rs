//! Rendering and animation logic.

use bevy::prelude::*;

use crate::{
    asset_management::{AssetState, StructureHandles, TileHandles},
    organisms::units::Unit,
    simulation::geometry::{MapGeometry, TilePos},
    structures::StructureId,
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
        app.add_plugin(LightingPlugin).add_system_set(
            SystemSet::on_update(AssetState::Ready)
                .with_system(populate_terrain)
                .with_system(populate_units)
                .with_system(populate_structures),
        );
    }
}

/// Adds rendering components to every spawned terrain tile
fn populate_terrain(
    new_terrain: Query<(Entity, &TilePos, &Terrain), Added<Terrain>>,
    mut commands: Commands,
    handles: Res<TileHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (terrain_entity, tile_pos, terrain) in new_terrain.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let hex_height = *map_geometry.height_index.get(tile_pos).unwrap();

        commands.entity(terrain_entity).insert(PbrBundle {
            mesh: handles.mesh.clone_weak(),
            material: handles.terrain_materials.get(terrain).unwrap().clone_weak(),
            transform: Transform::from_xyz(pos.x, 0.0, pos.y).with_scale(Vec3 {
                x: 1.,
                y: hex_height,
                z: 1.,
            }),
            ..default()
        });
    }
}

/// Adds rendering components to every spawned structure
fn populate_structures(
    new_structures: Query<(Entity, &TilePos, &StructureId), Added<StructureId>>,
    mut commands: Commands,
    structure_handles: Res<StructureHandles>,
    map_geometry: Res<MapGeometry>,
    materials: Res<Assets<StandardMaterial>>,
) {
    /// The size of a single structure
    const SIZE: f32 = 1.0;
    /// The offset required to have a structure sit on top of the tile correctly
    const OFFSET: f32 = SIZE / 2.0;

    let material_handle = structure_handles.get_material();
    let material = materials.get(&material_handle).unwrap();
    dbg!(material);

    for (entity, tile_pos, structure_id) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let terrain_height = map_geometry.height_index.get(tile_pos).unwrap();

        if let Some(mesh) = structure_handles.get_mesh(structure_id) {
            commands.entity(entity).insert(PbrBundle {
                mesh: mesh.clone_weak(),
                material: material_handle.clone_weak(),
                transform: Transform::from_xyz(pos.x, terrain_height + OFFSET, pos.y),
                ..default()
            });
        } else {
            error!("No mesh loaded for {structure_id:?}!");
        }
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
