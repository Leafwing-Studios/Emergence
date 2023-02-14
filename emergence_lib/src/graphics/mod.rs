//! Rendering and animation logic.

use bevy::prelude::*;

use crate::{
    asset_management::{
        structures::StructureHandles, terrain::TerrainHandles, units::UnitHandles, AssetState,
    },
    organisms::units::UnitId,
    player_interaction::InteractionSystem,
    simulation::geometry::{MapGeometry, TilePos},
    structures::{ghost::Ghost, StructureId},
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
            .add_system_set(
                SystemSet::on_update(AssetState::Ready)
                    .with_system(populate_terrain)
                    .with_system(populate_units)
                    .with_system(populate_structures)
                    // We need to avoid attempting to insert bundles into entities that no longer exist
                    .with_system(mesh_ghosts.before(InteractionSystem::ManageGhosts)),
            )
            .add_system_to_stage(CoreStage::PostUpdate, change_ghost_material);
    }
}

/// Adds rendering components to every spawned terrain tile
fn populate_terrain(
    new_terrain: Query<(Entity, &TilePos, &Terrain), Added<Terrain>>,
    mut commands: Commands,
    handles: Res<TerrainHandles>,
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
    new_structures: Query<(Entity, &TilePos, &StructureId), (Added<StructureId>, Without<Ghost>)>,
    mut commands: Commands,
    structure_handles: Res<StructureHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (entity, tile_pos, structure_id) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let terrain_height = map_geometry.height_index.get(tile_pos).unwrap();

        let scene_handle = structure_handles.scenes.get(structure_id).unwrap();

        commands.entity(entity).insert(SceneBundle {
            scene: scene_handle.clone_weak(),
            transform: Transform::from_xyz(pos.x, terrain_height + StructureId::OFFSET, pos.y),
            ..default()
        });
    }
}

/// Adds rendering components to every spawned ghost
fn mesh_ghosts(
    new_structures: Query<(Entity, &TilePos, &StructureId), (Added<StructureId>, With<Ghost>)>,
    mut commands: Commands,
    map_geometry: Res<MapGeometry>,
    structure_handles: Res<StructureHandles>,
) {
    // TODO: vary ghost mesh based on structure_id
    for (entity, tile_pos, structure_id) in new_structures.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let terrain_height = map_geometry.height_index.get(tile_pos).unwrap();

        let scene_handle = structure_handles.scenes.get(structure_id).unwrap();

        // Spawn scene as a child of the root ghost
        commands.entity(entity).insert(SceneBundle {
            scene: scene_handle.clone_weak(),
            transform: Transform::from_xyz(pos.x, terrain_height + StructureId::OFFSET, pos.y),
            ..default()
        });
    }
}

/// Modifies the material of any entities spawned due to a ghost structure.
fn change_ghost_material(
    ghost_query: Query<Entity, With<Ghost>>,
    children: Query<&Children>,
    mut material_query: Query<&mut Handle<StandardMaterial>>,
    structure_handles: Res<StructureHandles>,
) {
    for ghost_entity in ghost_query.iter() {
        for child in children.iter_descendants(ghost_entity) {
            if let Ok(mut material) = material_query.get_mut(child) {
                *material = structure_handles.ghost_material.clone_weak();
            }
        }
    }
}

/// Adds rendering components to every spawned unit
fn populate_units(
    new_units: Query<(Entity, &TilePos, &UnitId), Added<UnitId>>,
    mut commands: Commands,
    unit_handles: Res<UnitHandles>,
    map_geometry: Res<MapGeometry>,
) {
    for (entity, tile_pos, unit_id) in new_units.iter() {
        let pos = map_geometry.layout.hex_to_world_pos(tile_pos.hex);
        let terrain_height = *map_geometry.height_index.get(tile_pos).unwrap();
        let scene_handle = unit_handles.scenes.get(unit_id).unwrap();

        commands.entity(entity).insert(SceneBundle {
            scene: scene_handle.clone_weak(),
            transform: Transform::from_xyz(pos.x, terrain_height, pos.y),
            ..default()
        });
    }
}
