use crate::structures::StructureId;
use bevy::{
    prelude::{shape::Cube, *},
    utils::HashMap,
};

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct StructureHandles {
    /// The material used for all structures
    pub(crate) material: Handle<StandardMaterial>,
    /// The mesh used for each type of structure
    pub(crate) meshes: HashMap<StructureId, Handle<Mesh>>,
}

impl FromWorld for StructureHandles {
    fn from_world(world: &mut World) -> Self {
        let mut handles = StructureHandles {
            material: Handle::default(),
            meshes: HashMap::default(),
        };

        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();

        // TODO: discover this from the file directory
        let structure_names: Vec<String> = vec!["acacia", "leuco"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        for structure_name in structure_names {
            let structure_id = StructureId { id: structure_name };
            let mesh = mesh_assets.add(
                Cube {
                    size: StructureId::SIZE,
                }
                .into(),
            );
            handles.meshes.insert(structure_id, mesh);
        }

        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        handles.material = material_assets.add(StandardMaterial::default());

        handles
    }
}
