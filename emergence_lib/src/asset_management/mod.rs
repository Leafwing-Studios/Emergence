//! Code related to loading, storing and tracking assets

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::HashMap,
};
use bevy_asset_loader::prelude::*;
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
            .add_state(AssetState::Loading)
            .add_loading_state(
                LoadingState::new(AssetState::Loading)
                    .continue_to_state(AssetState::Ready)
                    .with_collection::<StructureHandles>(),
            );
    }
}

/// Tracks the progress of asset loading.
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum AssetState {
    #[default]
    /// Assets still need to be loaded.
    Loading,
    /// All assets are loaded.
    Ready,
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

impl TileHandles {
    /// Returns a weakly cloned handle to the correct material for a terrain tile
    pub(crate) fn get_material(
        &self,
        terrain: &Terrain,
        hovered: bool,
        selected: bool,
    ) -> Handle<StandardMaterial> {
        let maybe_handle = match (hovered, selected) {
            (false, false) => self.terrain_materials.get(terrain),
            (true, false) => self.interaction_materials.get(&ObjectInteraction::Hovered),
            (false, true) => self.interaction_materials.get(&ObjectInteraction::Selected),
            (true, true) => self
                .interaction_materials
                .get(&ObjectInteraction::HoveredAndSelected),
        };

        maybe_handle.unwrap().clone_weak()
    }
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
#[derive(AssetCollection, Resource)]
pub(crate) struct StructureHandles {
    /// The material used for all structures
    #[asset(standard_material)]
    material: Handle<StandardMaterial>,
    /// The mesh used for each type of structure
    #[asset(path = "structures", collection(typed, mapped))]
    meshes: HashMap<String, Handle<Mesh>>,
}

impl StructureHandles {
    /// Returns a reference to a handle to the appropriate mesh if it exists.
    pub(crate) fn get_mesh(&self, structure_id: &StructureId) -> Option<&Handle<Mesh>> {
        let mut string = structure_id.id.clone();
        string.push_str(".gltf");

        self.meshes.get(&string)
    }

    /// Returns a weakly cloned handle to the material used for structures.
    pub(crate) fn get_material(&self) -> Handle<StandardMaterial> {
        self.material.clone_weak()
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
