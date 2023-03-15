//! Asset loading for terrain

use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use crate::{
    enum_iter::IterableEnum, player_interaction::selection::ObjectInteraction,
    simulation::geometry::MapGeometry,
};

use super::{
    hexagonal_column,
    manifest::{Id, Terrain},
    Loadable,
};

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct TerrainHandles {
    /// The scene used for each type of terrain
    pub(crate) scenes: HashMap<Id<Terrain>, Handle<Scene>>,
    /// The mesh used for each type of structure
    pub(crate) mesh: Handle<Mesh>,
    /// The materials used for tiles when they are selected or otherwise interacted with
    pub(crate) interaction_materials: HashMap<ObjectInteraction, Handle<StandardMaterial>>,
}

impl TerrainHandles {
    /// Returns a weakly cloned handle to the correct material for a terrain tile
    pub(crate) fn get_material(
        &self,
        terrain: &Id<Terrain>,
        hovered: bool,
        selected: bool,
    ) -> Handle<StandardMaterial> {
        let maybe_handle = match (hovered, selected) {
            (false, false) => {
                let scene = self.scenes.get(terrain).unwrap_or_default();
            }
            (true, false) => self.interaction_materials.get(&ObjectInteraction::Hovered),
            (false, true) => self.interaction_materials.get(&ObjectInteraction::Selected),
            (true, true) => self
                .interaction_materials
                .get(&ObjectInteraction::HoveredAndSelected),
        };

        maybe_handle.unwrap().clone_weak()
    }
}

impl FromWorld for TerrainHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let mut scenes = HashMap::new();
        let variants: [Id<Terrain>; 3] = [Id::new("loam"), Id::new("muddy"), Id::new("rocky")];
        for id in variants {
            let path_string = format!("structures/{id}.gltf#Scene0");
            let scene = asset_server.load(path_string);
            scenes.insert(id, scene);
        }

        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
        let mut interaction_materials = HashMap::new();
        for variant in ObjectInteraction::variants() {
            if let Some(material) = variant.material() {
                let material_handle = material_assets.add(material);
                interaction_materials.insert(variant, material_handle);
            }
        }

        let map_geometry = world.resource::<MapGeometry>();
        let mesh_object = hexagonal_column(&map_geometry.layout, 1.0);
        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
        let mesh = mesh_assets.add(mesh_object);

        TerrainHandles {
            scenes,
            mesh,
            interaction_materials,
        }
    }
}

impl Loadable for TerrainHandles {
    fn load_state(&self, asset_server: &AssetServer) -> LoadState {
        for (terrain, scene_handle) in &self.scenes {
            let scene_load_state = asset_server.get_load_state(scene_handle);
            info!("{terrain:?}'s scene is {scene_load_state:?}");

            if scene_load_state != LoadState::Loaded {
                return scene_load_state;
            }
        }

        LoadState::Loaded
    }
}
