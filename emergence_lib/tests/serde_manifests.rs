use std::time::Duration;

use bevy::utils::{HashMap, HashSet};
use emergence_lib::{
    asset_management::manifest::RawId,
    items::{
        item_manifest::{ItemData, RawItemManifest},
        recipe::{RawRecipeData, RawRecipeManifest, RecipeConditions, Threshold},
    },
    organisms::{
        energy::{Energy, EnergyPool},
        lifecycle::{RawLifePath, RawLifecycle},
        RawOrganismId, RawOrganismVariety,
    },
    simulation::light::Illuminance,
    structures::{
        construction::Footprint,
        crafting::RawActiveRecipe,
        structure_manifest::{
            RawConstructionStrategy, RawStructureData, RawStructureKind, RawStructureManifest,
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
                        lifecycle: RawLifecycle::STATIC,
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
                        lifecycle: RawLifecycle::STATIC,
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
                RawRecipeData {
                    inputs: HashMap::new(),
                    outputs: HashMap::from_iter([("acacia_leaf".to_string(), 1)]),
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
                RawRecipeData {
                    inputs: HashMap::from_iter([("acacia_leaf".to_string(), 1)]),
                    outputs: HashMap::from_iter([("leuco_chunk".to_string(), 1)]),
                    craft_time: Duration::from_secs(2),
                    conditions: RecipeConditions::NONE,
                    energy: Some(Energy(40.)),
                },
            ),
            (
                RawId::new("ant_egg_production"),
                RawRecipeData {
                    inputs: HashMap::from_iter([("leuco_chunk".to_string(), 1)]),
                    outputs: HashMap::from_iter([("ant_egg".to_string(), 1)]),
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
                RawRecipeData {
                    inputs: HashMap::from_iter([("ant_egg".to_string(), 1)]),
                    outputs: HashMap::new(),
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
    let acacia_construction_strategy = RawConstructionStrategy {
        seedling: Some(RawId::new("acacia_seed")),
        work: Duration::ZERO,
        materials: HashMap::from_iter([(RawId::new("acacia_leaf".to_string()), 1)]),
        allowed_terrain_types: HashSet::from_iter([RawId::new("loam"), RawId::new("muddy")]),
    };

    // Create a new raw structure manifest
    let raw_structure_manifest = RawStructureManifest {
        structure_types: HashMap::from_iter(vec![
            (
                RawId::new("leuco"),
                RawStructureData {
                    organism_variety: Some(RawOrganismVariety {
                        prototypical_form: RawOrganismId::structure("leuco"),
                        lifecycle: RawLifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                    }),
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("leuco_chunk_production"),
                    },
                    construction_strategy: RawConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(3),
                        materials: HashMap::from_iter([(RawId::new("leuco_chunk".to_string()), 1)]),
                        allowed_terrain_types: HashSet::from_iter([
                            RawId::new("loam"),
                            RawId::new("muddy"),
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
                        lifecycle: RawLifecycle::new(vec![RawLifePath {
                            new_form: RawOrganismId::structure("acacia_sprout"),
                            energy_required: None,
                            time_required: Some(1.),
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
                        lifecycle: RawLifecycle::new(vec![RawLifePath {
                            new_form: RawOrganismId::structure("acacia"),
                            energy_required: Some(500.),
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
                        lifecycle: RawLifecycle::STATIC,
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
                    construction_strategy: RawConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(10),
                        materials: HashMap::new(),
                        allowed_terrain_types: HashSet::from_iter([
                            RawId::new("loam"),
                            RawId::new("muddy"),
                            RawId::new("rocky"),
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
                    construction_strategy: RawConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(5),
                        materials: HashMap::new(),
                        allowed_terrain_types: HashSet::from_iter([
                            RawId::new("loam"),
                            RawId::new("muddy"),
                            RawId::new("rocky"),
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
                    construction_strategy: RawConstructionStrategy {
                        seedling: None,
                        work: Duration::from_secs(10),
                        materials: HashMap::from_iter([(RawId::new("leuco_chunk".to_string()), 1)]),
                        allowed_terrain_types: HashSet::from_iter([
                            RawId::new("loam"),
                            RawId::new("muddy"),
                            RawId::new("rocky"),
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
