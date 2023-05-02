//! Generating and representing terrain as game objects.

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_mod_raycast::RaycastMesh;
use hexx::ColumnMeshBuilder;

use crate::asset_management::manifest::plugin::ManifestPlugin;
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetCollectionExt;
use crate::construction::zoning::Zoning;
use crate::crafting::components::StorageInventory;
use crate::crafting::item_tags::ItemKind;
use crate::organisms::energy::VigorModifier;
use crate::player_interaction::selection::ObjectInteraction;
use crate::signals::{Emitter, SignalModifier, SignalStrength, SignalType};
use crate::simulation::geometry::{Height, MapGeometry, TilePos};
use crate::simulation::SimulationSet;

use self::terrain_assets::TerrainHandles;
use self::terrain_manifest::{RawTerrainManifest, Terrain};

pub(crate) mod commands;
pub(crate) mod terrain_assets;
pub mod terrain_manifest;

/// All logic and initialization needed for terrain.
pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ManifestPlugin::<RawTerrainManifest>::new())
            .add_asset_collection::<TerrainHandles>()
            .add_systems(
                (
                    respond_to_height_changes,
                    set_terrain_emitters.in_set(TerrainEmitters),
                    update_litter_index,
                )
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );
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
    /// The mesh used for raycasting
    mesh: Handle<Mesh>,
    /// How is the terrain being interacted with?
    object_interaction: ObjectInteraction,
    /// The structure that should be built here.
    zoning: Zoning,
    /// The scene used to construct the terrain tile.
    scene_bundle: SceneBundle,
    /// Modifies the intensity of emitters on this tile.
    signal_modifer: SignalModifier,
    /// Modifies the rate of work and the energy costs on this tile.
    vigor_modifier: VigorModifier,
    /// Controls the signals produced by this terrain tile.
    emitter: Emitter,
    /// Stores littered items
    storage_inventory: StorageInventory,
}

impl TerrainBundle {
    /// Creates a new Terrain entity.
    fn new(
        terrain_id: Id<Terrain>,
        tile_pos: TilePos,
        scene: Handle<Scene>,
        mesh: Handle<Mesh>,
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
            mesh,
            object_interaction: ObjectInteraction::None,
            zoning: Zoning::None,
            scene_bundle,
            signal_modifer: SignalModifier::None,
            vigor_modifier: VigorModifier::None,
            emitter: Emitter::default(),
            storage_inventory: StorageInventory::new(1, None),
        }
    }
}

/// Generates the merged mesh for all columns in the world.
///
/// This needs to be called whenever the height of tiles changes.
fn generate_mesh_for_columns(map_geometry: &MapGeometry) -> Mesh {
    let mut mesh_info_vec = Vec::new();

    for tile_pos in map_geometry.valid_tile_positions() {
        let height = map_geometry.get_height(tile_pos).unwrap().0;
        let mesh_info = ColumnMeshBuilder::new(&map_geometry.layout, height)
            .without_bottom_face()
            .without_top_face()
            .build();
        mesh_info_vec.push(mesh_info);
    }

    let mut merged_mesh_info = mesh_info_vec[0].clone();

    for mesh_info in mesh_info_vec.into_iter().skip(1) {
        merged_mesh_info.merge_with(mesh_info);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, merged_mesh_info.vertices.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, merged_mesh_info.normals.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, merged_mesh_info.uvs.to_vec());
    mesh.set_indices(Some(Indices::U16(merged_mesh_info.indices)));
    mesh
}

/// A marker component for terrain columns.
#[derive(Component)]
struct TerrainColumn;

/// Updates the game state appropriately whenever the height of a tile is changed.
fn respond_to_height_changes(
    mut terrain_topper_query: Query<(Ref<Height>, &TilePos, &mut Transform)>,
    mut map_geometry: ResMut<MapGeometry>,
    mut terrain_column_query: Query<Entity, With<TerrainColumn>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    terrain_handles: Res<TerrainHandles>,
) {
    let mut any_changed = false;

    for (height, &tile_pos, mut transform) in terrain_topper_query.iter_mut() {
        if height.is_changed() {
            any_changed = true;
            map_geometry.update_height(tile_pos, *height);
            // Sets the height of the terrain toppers.
            transform.translation.y = height.into_world_pos();
        }
    }

    if any_changed {
        // Despawn all of the old terrain columns.
        for entity in terrain_column_query.iter_mut() {
            commands.entity(entity).despawn();
        }

        let mesh = generate_mesh_for_columns(&map_geometry);
        let mesh_handle = meshes.add(mesh);

        // Create new terrain columns.
        commands.spawn(MaterialMeshBundle {
            // IMPORTANT: this is inserted by ownership rather than cloned
            // in order to ensure the asset ref-counting despawns the mesh
            mesh: mesh_handle,
            material: terrain_handles.column_material.clone_weak(),
            ..default()
        });
    }
}

/// Updates the signals produced by terrain tiles.
fn set_terrain_emitters(
    mut query: Query<(&mut Emitter, Ref<StorageInventory>), With<Id<Terrain>>>,
) {
    for (mut emitter, storage_inventory) in query.iter_mut() {
        if storage_inventory.is_changed() {
            emitter.signals.clear();
            for item_slot in storage_inventory.iter() {
                let item_kind = ItemKind::Single(item_slot.item_id());

                let signal_type = match storage_inventory.is_full() {
                    true => SignalType::Push(item_kind),
                    false => SignalType::Contains(item_kind),
                };
                let signal_strength = SignalStrength::new(10.);

                emitter.signals.push((signal_type, signal_strength));
            }
        }
    }
}

/// The set of systems that update terrain emitters.
#[derive(SystemSet, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TerrainEmitters;

/// Tracks how much litter is on the ground on each tile.
fn update_litter_index(
    query: Query<(&TilePos, &StorageInventory), (With<Id<Terrain>>, Changed<StorageInventory>)>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (&tile_pos, litter) in query.iter() {
        map_geometry.update_litter_state(tile_pos, litter.state());
    }
}
