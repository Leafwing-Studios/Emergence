//! Asset loading for terrain

use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use crate::{
    asset_management::{manifest::Id, AssetState, Loadable},
    enum_iter::IterableEnum,
    geometry::{hexagonal_column, Height, MapGeometry},
    graphics::palette::environment::COLUMN_COLOR,
    items::inventory::InventoryState,
    player_interaction::selection::ObjectInteraction,
    terrain::terrain_manifest::{Terrain, TerrainManifest},
};

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
    /// Models used to depict litter on tiles.
    pub(crate) litter_models: HashMap<InventoryState, Handle<Scene>>,
}

impl Loadable for TerrainHandles {
    const STAGE: AssetState = AssetState::LoadAssets;

    fn initialize(world: &mut World) {
        let names = world.resource::<TerrainManifest>().names();
        let asset_server = world.resource::<AssetServer>();

        let mut scenes = HashMap::new();
        for name in names {
            let path_string = format!("terrain/{name}.gltf#Scene0");
            let scene = asset_server.load(path_string);
            scenes.insert(Id::from_name(name.to_string()), scene);
        }

        let mut litter_models = HashMap::new();
        // TODO: these should probably be procedurally generated to match the stored item type.
        litter_models.insert(
            InventoryState::Partial,
            asset_server.load("litter/partial.gltf#Scene0"),
        );

        litter_models.insert(
            InventoryState::Full,
            asset_server.load("litter/pile.gltf#Scene0"),
        );

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

        world.insert_resource(TerrainHandles {
            scenes,
            topper_mesh,
            column_mesh,
            column_material,
            interaction_materials,
            litter_models,
        });
    }

    fn load_state(&self, asset_server: &AssetServer) -> LoadState {
        for (terrain, scene_handle) in &self.scenes {
            let scene_load_state = asset_server.get_load_state(scene_handle);

            if scene_load_state != LoadState::Loaded {
                let maybe_path = asset_server.get_handle_path(scene_handle);
                let path = maybe_path
                    .map(|p| format!("{:?}", p.path()))
                    .unwrap_or("unknown_path".to_string());

                info!("Terrain {terrain:?}'s scene at {path} is {scene_load_state:?}");
                return scene_load_state;
            }
        }

        LoadState::Loaded
    }
}
