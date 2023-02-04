//! Code related to loading, storing and tracking assets

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
};
use hexx::{Hex, HexLayout, MeshInfo};

use crate::{simulation::geometry::MapGeometry, structures::StructureId, terrain::Terrain};

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
    pub(crate) materials: HashMap<Terrain, Handle<StandardMaterial>>,
    /// The mesh used for each type of structure
    pub(crate) mesh: Handle<Mesh>,
    /// The material used for tiles when they are selected
    pub(crate) selected_tile_handle: Handle<StandardMaterial>,
}

impl FromWorld for TileHandles {
    fn from_world(world: &mut World) -> Self {
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
        let mut materials = HashMap::new();
        materials.insert(Terrain::Plain, material_assets.add(Color::BEIGE.into()));
        materials.insert(Terrain::Rocky, material_assets.add(Color::GRAY.into()));
        materials.insert(Terrain::High, material_assets.add(Color::RED.into()));

        let selected_tile_handle = material_assets.add(Color::SEA_GREEN.into());

        let map_geometry = world.resource::<MapGeometry>();

        let mesh_object = hexagonal_column(&map_geometry.layout, 1.0);
        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
        let mesh = mesh_assets.add(mesh_object);

        TileHandles {
            materials,
            mesh,
            selected_tile_handle,
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
