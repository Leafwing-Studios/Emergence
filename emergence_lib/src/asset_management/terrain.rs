//! Asset loading for terrain

use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use crate::{
    enum_iter::IterableEnum, player_interaction::selection::ObjectInteraction, terrain::TerrainData,
};

use super::{
    manifest::{Id, Terrain, TerrainManifest},
    Loadable,
};

/// All logic and initialization needed for terrain.
pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainManifest>();
    }
}

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct TerrainHandles {
    /// The scene used for each type of terrain
    pub(crate) scenes: HashMap<Id<Terrain>, Handle<Scene>>,
}

impl FromWorld for TerrainHandles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let mut scenes = HashMap::new();
        let variants: [Id<Terrain>; 3] = [Id::new("loam"), Id::new("muddy"), Id::new("rocky")];
        for id in variants {
            let path_string = format!("terrain/{id}.gltf#Scene0");
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

        TerrainHandles { scenes }
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

impl Default for TerrainManifest {
    // TODO: load this from file
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(Id::new("rocky"), TerrainData::new(2.0));
        map.insert(Id::new("loam"), TerrainData::new(1.0));
        map.insert(Id::new("muddy"), TerrainData::new(0.5));

        TerrainManifest::new(map)
    }
}
