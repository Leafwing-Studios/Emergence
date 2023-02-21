//! Asset loading for terrain

use bevy::{prelude::*, utils::HashMap};

use crate::{
    enum_iter::IterableEnum, player_interaction::selection::ObjectInteraction,
    simulation::geometry::MapGeometry, terrain::Terrain,
};

use super::hexagonal_column;

/// Stores material handles for the different tile types.
#[derive(Resource)]
pub(crate) struct TerrainHandles {
    /// The material used for each type of terrain
    pub(crate) terrain_materials: HashMap<Terrain, Handle<StandardMaterial>>,
    /// The mesh used for each type of structure
    pub(crate) mesh: Handle<Mesh>,
    /// The materials used for tiles when they are selected or otherwise interacted with
    pub(crate) interaction_materials: HashMap<ObjectInteraction, Handle<StandardMaterial>>,
}

impl TerrainHandles {
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

impl FromWorld for TerrainHandles {
    fn from_world(world: &mut World) -> Self {
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        let mut terrain_materials = HashMap::new();
        for variant in Terrain::variants() {
            let material_handle = material_assets.add(variant.material());
            terrain_materials.insert(variant, material_handle);
        }

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
            terrain_materials,
            mesh,
            interaction_materials,
        }
    }
}
