//! Code related to loading, storing and tracking assets

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
};
use hexx::{Hex, HexLayout, MeshInfo};

use crate::{
    enum_iter::IterableEnum, player_interaction::selection::ObjectInteraction,
    simulation::geometry::MapGeometry, structures::StructureId, terrain::Terrain,
};

/// Collects asset management systems and resources.
pub struct AssetManagementPlugin;

impl Plugin for AssetManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileHandles>()
            .init_resource::<StructureHandles>();
    }
}

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct TileHandles {
    /// The material used for each type of terrain
    pub(crate) terrain_materials: HashMap<Terrain, Handle<StandardMaterial>>,
    /// The mesh used for each type of structure
    pub(crate) mesh: Handle<Mesh>,
    /// The materials used for tiles when they are selected or otherwise interacted with
    pub(crate) interaction_materials: HashMap<ObjectInteraction, Handle<StandardMaterial>>,
}

impl FromWorld for TileHandles {
    fn from_world(world: &mut World) -> Self {
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        let mut terrain_materials = HashMap::new();
        for variant in Terrain::variants() {
            let material_handle = material_assets.add(variant.material());
            terrain_materials.insert(variant, material_handle);
        }

        let mut interaction_materials = HashMap::new();
        for variant in ObjectInteraction::variants() {
            let material_handle = material_assets.add(variant.material());
            interaction_materials.insert(variant, material_handle);
        }

        let map_geometry = world.resource::<MapGeometry>();
        let mesh_object = hexagonal_column(&map_geometry.layout, 1.0);
        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
        let mesh = mesh_assets.add(mesh_object);

        TileHandles {
            terrain_materials,
            mesh,
            interaction_materials,
        }
    }
}

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct StructureHandles {
    /// The material used for each type of structures
    pub(crate) materials: HashMap<StructureId, Handle<StandardMaterial>>,
    /// The mesh used for each type of structure
    pub(crate) meshes: HashMap<StructureId, Handle<Mesh>>,
}

/// The base size of structures
pub(crate) const STRUCTURE_SCALE: f32 = 1.0;

impl FromWorld for StructureHandles {
    fn from_world(world: &mut World) -> Self {
        let mut materials_assets = world.resource_mut::<Assets<StandardMaterial>>();
        let mut materials = HashMap::new();
        materials.insert(
            StructureId::new("leuco"),
            materials_assets.add(Color::PURPLE.into()),
        );
        materials.insert(
            StructureId::new("acacia"),
            materials_assets.add(Color::DARK_GREEN.into()),
        );

        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
        let mut meshes = HashMap::new();
        meshes.insert(
            StructureId::new("leuco"),
            mesh_assets.add(Mesh::from(shape::Cube {
                size: STRUCTURE_SCALE,
            })),
        );
        meshes.insert(
            StructureId::new("acacia"),
            mesh_assets.add(Mesh::from(shape::Cube {
                size: STRUCTURE_SCALE,
            })),
        );

        StructureHandles { materials, meshes }
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
