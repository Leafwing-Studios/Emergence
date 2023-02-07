use crate::structures::StructureId;
use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::AssetCollection;

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
