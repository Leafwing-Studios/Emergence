//! Asset loading for structures

use crate::{
    asset_management::{
        hexagonal_column,
        palette::{GHOST_ALPHA, GHOST_COLOR, HOVER_HUE, HOVER_SATURATION, SELECTION_COLOR},
    },
    simulation::geometry::MapGeometry,
    structures::StructureId,
};
use bevy::{asset::LoadState, prelude::*, utils::HashMap};

use super::Loadable;

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct StructureHandles {
    /// The scene for each type of structure
    pub(crate) scenes: HashMap<StructureId, Handle<Scene>>,
    /// The material to be used for all ghosts
    pub(crate) ghost_material: Handle<StandardMaterial>,
    /// The material to be used for all previews
    pub(crate) preview_material: Handle<StandardMaterial>,
    /// The material to be used for selected structures
    pub(crate) selected_material: Handle<StandardMaterial>,
    /// The material to be used for selected ghosts
    pub(crate) selected_ghost_material: Handle<StandardMaterial>,
    /// The raycasting mesh used to select structures
    pub(crate) picking_mesh: Handle<Mesh>,
}

impl FromWorld for StructureHandles {
    fn from_world(world: &mut World) -> Self {
        /// The height of the picking box for a single structure.
        ///
        /// Hex tiles always have a diameter of 1.0.
        const PICKING_HEIGHT: f32 = 1.0;

        let map_geometry = world.resource::<MapGeometry>();
        let picking_mesh_object = hexagonal_column(&map_geometry.layout, PICKING_HEIGHT);
        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
        let picking_mesh = mesh_assets.add(picking_mesh_object);

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

        let ghost_material = materials.add(StandardMaterial {
            base_color: GHOST_COLOR,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        let preview_material = materials.add(StandardMaterial {
            base_color: Color::Hsla {
                hue: HOVER_HUE,
                saturation: HOVER_SATURATION,
                lightness: HOVER_SATURATION,
                alpha: GHOST_ALPHA,
            },
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        let selected_material = materials.add(StandardMaterial {
            base_color: SELECTION_COLOR,
            ..Default::default()
        });

        let selected_ghost_material = materials.add(StandardMaterial {
            base_color: GHOST_COLOR,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        let mut handles = StructureHandles {
            scenes: HashMap::default(),
            ghost_material,
            preview_material,
            selected_material,
            selected_ghost_material,
            picking_mesh,
        };

        let asset_server = world.resource::<AssetServer>();

        // TODO: discover this from the file directory
        let structure_names = vec!["acacia", "leuco"];

        for id in structure_names {
            let structure_id = StructureId { id };
            let structure_path = format!("structures/{id}.gltf#Scene0");
            let scene = asset_server.load(structure_path);
            handles.scenes.insert(structure_id, scene);
        }

        handles
    }
}

impl Loadable for StructureHandles {
    fn load_state(&self, asset_server: &AssetServer) -> LoadState {
        for (structure, scene_handle) in &self.scenes {
            let scene_load_state = asset_server.get_load_state(scene_handle);
            info!("{structure:?}'s scene is {scene_load_state:?}");

            if scene_load_state != LoadState::Loaded {
                return scene_load_state;
            }
        }

        LoadState::Loaded
    }
}
