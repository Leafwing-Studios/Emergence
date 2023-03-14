//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::{
    prelude::*,
    utils::{Duration, HashMap, HashSet},
};
use bevy_mod_raycast::RaycastMesh;
use leafwing_abilities::prelude::Pool;

use crate::{
    asset_management::manifest::{Id, Structure, StructureManifest, Terrain},
    items::{inventory::Inventory, ItemCount},
    organisms::{
        energy::{Energy, EnergyPool},
        OrganismVariety,
    },
    player_interaction::{clipboard::ClipboardData, selection::ObjectInteraction},
    simulation::geometry::{Facing, TilePos},
};

use self::{
    crafting::{ActiveRecipe, CraftingPlugin, InputInventory},
    ghost::{ghost_lifecyle, ghost_signals},
};

pub(crate) mod commands;
pub(crate) mod crafting;
pub(crate) mod ghost;

/// Information about a single [`Id<Structure>`] variety of structure.
#[derive(Debug, Clone)]
pub(crate) struct StructureData {
    /// Data needed for living structures
    organism: Option<OrganismVariety>,
    /// Can this structure make things?
    crafts: bool,
    /// Does this structure start with a recipe pre-selected?
    starting_recipe: ActiveRecipe,
    /// The amount of work by units required to complete the construction of this building.
    ///
    /// If this is [`Duration::ZERO`], no work will be needed at all.
    build_duration: Duration,
    /// The set of items needed to create a new copy of this structure
    construction_materials: InputInventory,
    /// The set of terrain types that this structure can be built on
    pub(crate) allowed_terrain_types: HashSet<Id<Terrain>>,
    /// The color associated with this structure
    pub(crate) color: Color,
}

impl StructureData {
    /// Returns the starting recipe of the structure
    pub fn starting_recipe(&self) -> &ActiveRecipe {
        &self.starting_recipe
    }

    /// Returns the set of terrain types that this structure can be built on
    pub fn allowed_terrain_types(&self) -> &HashSet<Id<Terrain>> {
        &self.allowed_terrain_types
    }
}

impl Default for StructureManifest {
    fn default() -> Self {
        let mut map = HashMap::default();

        let leuco_construction_materials = InputInventory {
            inventory: Inventory::new_from_item(ItemCount::new(Id::leuco_chunk(), 1)),
        };

        // TODO: read these from files
        map.insert(
            Id::new("leuco"),
            StructureData {
                organism: Some(OrganismVariety {
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                }),
                crafts: true,
                starting_recipe: ActiveRecipe::new(Id::leuco_chunk_production()),
                build_duration: Duration::from_secs(5),
                construction_materials: leuco_construction_materials,
                allowed_terrain_types: HashSet::from_iter([Id::new("loam"), Id::new("muddy")]),
                color: Color::ORANGE_RED,
            },
        );

        let acacia_construction_materials = InputInventory {
            inventory: Inventory::new_from_item(ItemCount::new(Id::acacia_leaf(), 2)),
        };

        map.insert(
            Id::new("acacia"),
            StructureData {
                organism: Some(OrganismVariety {
                    energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                }),
                crafts: true,
                starting_recipe: ActiveRecipe::new(Id::acacia_leaf_production()),
                build_duration: Duration::ZERO,
                construction_materials: acacia_construction_materials,
                allowed_terrain_types: HashSet::from_iter([Id::new("loam"), Id::new("muddy")]),
                color: Color::GREEN,
            },
        );

        map.insert(
            Id::new("ant_hive"),
            StructureData {
                organism: None,
                crafts: true,
                starting_recipe: ActiveRecipe::new(Id::ant_egg_production()),
                construction_materials: InputInventory::default(),
                build_duration: Duration::from_secs(10),
                allowed_terrain_types: HashSet::from_iter([
                    Id::new("loam"),
                    Id::new("muddy"),
                    Id::new("rocky"),
                ]),
                color: Color::BEIGE,
            },
        );

        map.insert(
            Id::new("hatchery"),
            StructureData {
                organism: None,
                crafts: true,
                starting_recipe: ActiveRecipe::new(Id::hatch_ants()),
                construction_materials: InputInventory::default(),
                build_duration: Duration::from_secs(5),
                allowed_terrain_types: HashSet::from_iter([Id::new("loam"), Id::new("rocky")]),
                color: Color::BLUE,
            },
        );

        StructureManifest::new(map)
    }
}

/// The data needed to build a structure
#[derive(Bundle)]
struct StructureBundle {
    /// Unique identifier of structure variety
    structure: Id<Structure>,
    /// The direction this structure is facing
    facing: Facing,
    /// The location of this structure
    tile_pos: TilePos,
    /// Makes structures pickable
    raycast_mesh: RaycastMesh<Id<Structure>>,
    /// How is this structure being interacted with
    object_interaction: ObjectInteraction,
    /// The mesh used for raycasting
    picking_mesh: Handle<Mesh>,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
}

impl StructureBundle {
    /// Creates a new structure
    fn new(
        tile_pos: TilePos,
        data: ClipboardData,
        picking_mesh: Handle<Mesh>,
        scene_handle: Handle<Scene>,
        world_pos: Vec3,
    ) -> Self {
        StructureBundle {
            structure: data.structure_id,
            facing: data.facing,
            tile_pos,
            raycast_mesh: RaycastMesh::default(),
            object_interaction: ObjectInteraction::None,
            picking_mesh,
            scene_bundle: SceneBundle {
                scene: scene_handle,
                transform: Transform::from_translation(world_pos),
                ..default()
            },
        }
    }
}

/// The systems that make structures tick.
pub(super) struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CraftingPlugin)
            .init_resource::<StructureManifest>()
            .add_system(ghost_signals)
            .add_system(ghost_lifecyle);
    }
}
