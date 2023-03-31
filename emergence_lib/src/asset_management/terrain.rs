//! Asset loading for terrain

use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use crate::{
    enum_iter::IterableEnum,
    player_interaction::selection::ObjectInteraction,
    simulation::geometry::{hexagonal_column, Height, MapGeometry},
    terrain::terrain_manifest::{Terrain, TerrainManifest},
};

use super::{manifest::Id, palette::environment::COLUMN_COLOR, Loadable};

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct TerrainHandles {
    /// The scene used for each type of terrain
    pub(crate) scenes: HashMap<Id<Terrain>, Handle<Scene>>,
    /// The mesh used for raycasting the terrain topper
    pub(crate) topper_mesh: Handle<Mesh>,
    /// The mesh of the column underneath each terrain topper
    pub(crate) column_mesh: Handle<Mesh>,
    /// The material of the column underneath each terrain topper
    pub(crate) column_material: Handle<StandardMaterial>,
    /// The materials used to display player interaction with terrain tiles
    pub(crate) interaction_materials: HashMap<ObjectInteraction, Handle<StandardMaterial>>,
}

impl FromWorld for TerrainHandles {
    fn from_world(world: &mut World) -> Self {
        let names = world.resource::<TerrainManifest>().names();
        let asset_server = world.resource::<AssetServer>();

        let mut scenes = HashMap::new();
        for name in names {
            let path_string = format!("terrain/{name}.gltf#Scene0");
            let scene = asset_server.load(path_string);
            scenes.insert(Id::from_name(name), scene);
        }

        let map_geometry = world.resource::<MapGeometry>();
        let column_mesh_object = hexagonal_column(&map_geometry.layout, 1.0);
        let topper_mesh_object = hexagonal_column(&map_geometry.layout, Height::TOPPER_THICKNESS);
        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
        let column_mesh = mesh_assets.add(column_mesh_object);
        let topper_mesh = mesh_assets.add(topper_mesh_object);

        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
        let mut interaction_materials = HashMap::new();
        for variant in ObjectInteraction::variants() {
            if let Some(material) = variant.material() {
                let material_handle = material_assets.add(material);
                interaction_materials.insert(variant, material_handle);
            }
        }
        let column_material = material_assets.add(StandardMaterial {
            base_color: COLUMN_COLOR,
            perceptual_roughness: 1.0,
            ..default()
        });

        TerrainHandles {
            scenes,
            topper_mesh,
            column_mesh,
            column_material,
            interaction_materials,
        }
    }
}

impl Loadable for TerrainHandles {
    fn load_state(&self, asset_server: &AssetServer) -> LoadState {
        for (terrain, scene_handle) in &self.scenes {
            let scene_load_state = asset_server.get_load_state(scene_handle);

            if scene_load_state != LoadState::Loaded {
                info!("Terrain {terrain:?}'s scene is {scene_load_state:?}");
                return scene_load_state;
            }
        }

        LoadState::Loaded
    }
}
