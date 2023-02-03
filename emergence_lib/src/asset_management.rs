//! Code related to loading, storing and tracking assets

use bevy::{prelude::*, utils::HashMap};

use crate::{structures::StructureId, terrain::Terrain};

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
    pub(crate) terrain_handles: HashMap<Terrain, Handle<StandardMaterial>>,
    /// The material used for tiles when they are selected
    pub(crate) selected_tile_handle: Handle<StandardMaterial>,
}

impl FromWorld for TileHandles {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let mut terrain_handles = HashMap::new();
        terrain_handles.insert(Terrain::Plain, materials.add(Color::BEIGE.into()));
        terrain_handles.insert(Terrain::Rocky, materials.add(Color::GRAY.into()));
        terrain_handles.insert(Terrain::High, materials.add(Color::RED.into()));

        let selected_tile_handle = materials.add(Color::SEA_GREEN.into());

        TileHandles {
            terrain_handles,
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
