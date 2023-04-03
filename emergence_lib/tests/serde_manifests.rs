use std::time::Duration;

use bevy::utils::{HashMap, HashSet};
use emergence_lib::{
    asset_management::manifest::{Id, RawId},
    items::{
        inventory::Inventory,
        item_manifest::{ItemData, RawItemManifest},
        recipe::{RawRecipeManifest, RecipeConditions, RecipeData, Threshold},
        ItemCount,
    },
    organisms::{
        energy::{Energy, EnergyPool},
        lifecycle::{LifePath, Lifecycle},
        OrganismId, RawOrganismId, RawOrganismVariety,
    },
    simulation::{light::Illuminance, time::TimePool},
    structures::{
        construction::Footprint,
        crafting::{InputInventory, RawActiveRecipe},
        structure_manifest::{
            ConstructionStrategy, RawStructureData, RawStructureKind, RawStructureManifest,
        },
    },
    terrain::terrain_manifest::{RawTerrainManifest, TerrainData},
    units::{
        hunger::RawDiet,
        unit_manifest::{RawUnitData, RawUnitManifest},
        WanderingBehavior,
    },
};
use leafwing_abilities::prelude::Pool;

#[test]
fn can_serialize_item_manifest() {
    // Create a new raw item manifest
    let raw_item_manifest = RawItemManifest {
        items: HashMap::from_iter(vec![
            (RawId::new("test_item"), ItemData { stack_size: 1 }),
            (RawId::new("test_item_2"), ItemData { stack_size: 2 }),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_item_manifest).unwrap();
    print!("{}\n", &serialized);

    // Deserialize it
    let deserialized: RawItemManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_item_manifest, deserialized);
}

#[test]
fn can_serialize_terrain_manifest() {
    // Create a new raw terrain manifest
    let raw_terrain_manifest = RawTerrainManifest {
        terrain_types: HashMap::from_iter(vec![
            (
                RawId::new("test_terrain"),
                TerrainData { walking_speed: 1.0 },
            ),
            (
                RawId::new("test_terrain2"),
                TerrainData { walking_speed: 2.0 },
            ),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_terrain_manifest).unwrap();
    print!("{}\n", &serialized);

    // Deserialize it
    let deserialized: RawTerrainManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_terrain_manifest, deserialized);
}

#[test]
fn can_serialize_unit_manifest() {
    // Create a new raw unit manifest
    let raw_unit_manifest = RawUnitManifest {
        unit_types: HashMap::from_iter(vec![
            (
                RawId::new("ant"),
                RawUnitData {
                    organism_variety: RawOrganismVariety {
                        prototypical_form: RawOrganismId::unit("ant"),
                        lifecycle: Lifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                    },
                    diet: RawDiet::new("leuco_chunk", 50.),
                    max_impatience: 10,
                    wandering_behavior: WanderingBehavior::from_iter([
                        (1, 0.7),
                        (8, 0.2),
                        (16, 0.1),
                    ]),
                },
            ),
            (
                RawId::new("test_unit"),
                RawUnitData {
                    organism_variety: RawOrganismVariety {
                        prototypical_form: RawOrganismId::unit("test_unit"),
                        lifecycle: Lifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(50.), Energy(0.)),
                    },
                    diet: RawDiet::new("acacia_leaf", 0.),
                    max_impatience: 0,
                    wandering_behavior: WanderingBehavior::from_iter([(0, 0.7), (16, 0.1)]),
                },
            ),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_unit_manifest).unwrap();
    print!("{}\n", &serialized);

    // Deserialize it
    let deserialized: RawUnitManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_unit_manifest, deserialized);
}

#[test]
fn can_serialize_recipe_manifest() {
    // Create a new raw recipe manifest
    let raw_recipe_manifest = RawRecipeManifest {
        recipes: HashMap::from_iter(vec![
            (
                RawId::new("acacia_leaf_production"),
                RecipeData {
                    inputs: Vec::new(),
                    outputs: vec![ItemCount::one(Id::from_name("acacia_leaf"))],
                    craft_time: Duration::from_secs(3),
                    conditions: RecipeConditions::new(
                        0,
                        Threshold::new(Illuminance(5e3), Illuminance(6e4)),
                    ),
                    energy: Some(Energy(20.)),
                },
            ),
            (
                RawId::new("leuco_chunk_production"),
                RecipeData {
                    inputs: vec![ItemCount::one(Id::from_name("acacia_leaf"))],
                    outputs: vec![ItemCount::one(Id::from_name("leuco_chunk"))],
                    craft_time: Duration::from_secs(2),
                    conditions: RecipeConditions::NONE,
                    energy: Some(Energy(40.)),
                },
            ),
            (
                RawId::new("ant_egg_production"),
                RecipeData {
                    inputs: Vec::new(),
                    outputs: vec![ItemCount::one(Id::from_name("ant_egg"))],
                    craft_time: Duration::from_secs(10),
                    conditions: RecipeConditions {
                        workers_required: 2,
                        allowable_light_range: None,
                    },
                    energy: None,
                },
            ),
            (
                RawId::new("hatch_ants"),
                RecipeData {
                    inputs: vec![ItemCount::one(Id::from_name("ant_egg"))],
                    outputs: Vec::new(),
                    craft_time: Duration::from_secs(10),
                    conditions: RecipeConditions {
                        workers_required: 1,
                        allowable_light_range: None,
                    },
                    energy: None,
                },
            ),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_recipe_manifest).unwrap();
    print!("{}\n", &serialized);

    // Deserialize it
    let deserialized: RawRecipeManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_recipe_manifest, deserialized);
}

#[test]
fn can_serialize_structure_manifest() {
    // Shared data
    let acacia_construction_strategy = ConstructionStrategy {
        seedling: Some(Id::from_name("acacia_seed")),
        work: Duration::ZERO,
        materials: InputInventory {
            inventory: Inventory::new_from_item(Id::from_name("acacia_leaf"), 1),
        },
        allowed_terrain_types: HashSet::from_iter([Id::from_name("loam"), Id::from_name("muddy")]),
    };

    // Create a new raw structure manifest
    let raw_structure_manifest = RawStructureManifest {
        structure_types: HashMap::from_iter(vec![
            (
                RawId::new("leuco"),
                RawStructureData {
                    organism_variety: Some(RawOrganismVariety {
                        prototypical_form: RawOrganismId::structure("leuco"),
                        lifecycle: Lifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                    }),
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("leuco_chunk_production"),
                    },
                    construction_strategy: ConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(3),
                        materials: InputInventory {
                            inventory: Inventory::new_from_item(Id::from_name("leuco_chunk"), 1),
                        },
                        allowed_terrain_types: HashSet::from_iter([
                            Id::from_name("loam"),
                            Id::from_name("muddy"),
                        ]),
                    },
                    max_workers: 6,
                    footprint: Footprint::single(),
                },
            ),
            (
                RawId::new("acacia_seed"),
                RawStructureData {
                    organism_variety: Some(RawOrganismVariety {
                        prototypical_form: RawOrganismId::structure("acacia"),
                        lifecycle: Lifecycle::new(vec![LifePath {
                            new_form: OrganismId::Structure(Id::from_name("acacia_sprout")),
                            energy_required: None,
                            time_required: Some(TimePool::simple(1.)),
                        }]),
                        energy_pool: EnergyPool::new_full(Energy(50.), Energy(-1.)),
                    }),
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("acacia_leaf_production"),
                    },
                    construction_strategy: acacia_construction_strategy.clone(),
                    max_workers: 1,
                    footprint: Footprint::single(),
                },
            ),
            (
                RawId::new("acacia_sprout"),
                RawStructureData {
                    organism_variety: Some(RawOrganismVariety {
                        prototypical_form: RawOrganismId::structure("acacia"),
                        lifecycle: Lifecycle::new(vec![LifePath {
                            new_form: OrganismId::Structure(Id::from_name("acacia")),
                            energy_required: Some(EnergyPool::simple(500.)),
                            time_required: None,
                        }]),
                        energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                    }),
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("acacia_leaf_production"),
                    },
                    construction_strategy: acacia_construction_strategy.clone(),
                    max_workers: 1,
                    footprint: Footprint::single(),
                },
            ),
            (
                RawId::new("acacia"),
                RawStructureData {
                    organism_variety: Some(RawOrganismVariety {
                        prototypical_form: RawOrganismId::structure("acacia"),
                        lifecycle: Lifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(300.), Energy(-1.)),
                    }),
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("acacia_leaf_production"),
                    },
                    construction_strategy: acacia_construction_strategy,
                    max_workers: 6,
                    footprint: Footprint::single(),
                },
            ),
            (
                RawId::new("ant_hive"),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("ant_egg_production"),
                    },
                    construction_strategy: ConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(10),
                        materials: InputInventory::default(),
                        allowed_terrain_types: HashSet::from_iter([
                            Id::from_name("loam"),
                            Id::from_name("muddy"),
                            Id::from_name("rocky"),
                        ]),
                    },
                    max_workers: 3,
                    footprint: Footprint::hexagon(1),
                },
            ),
            (
                RawId::new("hatchery"),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("hatch_ants"),
                    },
                    construction_strategy: ConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(5),
                        materials: InputInventory::default(),
                        allowed_terrain_types: HashSet::from_iter([
                            Id::from_name("loam"),
                            Id::from_name("muddy"),
                            Id::from_name("rocky"),
                        ]),
                    },
                    max_workers: 6,
                    // Forms a crescent shape
                    footprint: Footprint::single(),
                },
            ),
            (
                RawId::new("storage"),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Storage {
                        max_slot_count: 3,
                        reserved_for: String::new(),
                    },
                    construction_strategy: ConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(10),
                        materials: InputInventory {
                            inventory: Inventory::new_from_item(Id::from_name("leuco_chunk"), 1),
                        },
                        allowed_terrain_types: HashSet::from_iter([
                            Id::from_name("loam"),
                            Id::from_name("muddy"),
                            Id::from_name("rocky"),
                        ]),
                    },
                    max_workers: 6,
                    footprint: Footprint::single(),
                },
            ),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_structure_manifest).unwrap();
    print!("{}\n", &serialized);

    // Deserialize it
    let deserialized: RawStructureManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_structure_manifest, deserialized);
}
