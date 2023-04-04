use bevy::utils::{HashMap, HashSet};
use emergence_lib::{
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
            ("test_item".to_string(), ItemData { stack_size: 1 }),
            ("test_item_2".to_string(), ItemData { stack_size: 2 }),
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
                "test_terrain".to_string(),
                TerrainData { walking_speed: 1.0 },
            ),
            (
                "test_terrain2".to_string(),
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
                "ant".to_string(),
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
                "test_unit".to_string(),
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
                "mature_acacia_production".to_string(),
                RawRecipeData {
                    inputs: HashMap::new(),
                    outputs: HashMap::from_iter([
                        ("acacia_leaf".to_string(), 1.),
                        // Output can be stochastic
                        ("acacia_seed".to_string(), 0.1),
                    ]),
                    craft_time: 3.,
                    conditions: Some(RecipeConditions::new(
                        0,
                        Threshold::new(Illuminance(5e3), Illuminance(6e4)),
                    )),
                    energy: Some(Energy(20.)),
                },
            ),
            (
                "leuco_chunk_production".to_string(),
                RawRecipeData {
                    inputs: HashMap::from_iter([("acacia_leaf".to_string(), 1)]),
                    outputs: HashMap::from_iter([("leuco_chunk".to_string(), 1.)]),
                    craft_time: 2.,
                    conditions: None,
                    energy: Some(Energy(40.)),
                },
            ),
            (
                "ant_egg_production".to_string(),
                RawRecipeData {
                    inputs: HashMap::from_iter([("leuco_chunk".to_string(), 1)]),
                    outputs: HashMap::from_iter([("ant_egg".to_string(), 1.)]),
                    craft_time: 10.,
                    conditions: Some(RecipeConditions {
                        workers_required: 2,
                        allowable_light_range: None,
                    }),
                    energy: None,
                },
            ),
            (
                "hatch_ants".to_string(),
                RawRecipeData {
                    inputs: HashMap::from_iter([("ant_egg".to_string(), 1)]),
                    outputs: HashMap::new(),
                    craft_time: 10.,
                    conditions: Some(RecipeConditions {
                        workers_required: 1,
                        allowable_light_range: None,
                    }),
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
    // Create a new raw structure manifest
    let raw_structure_manifest = RawStructureManifest {
        structure_types: HashMap::from_iter(vec![
            (
                "leuco".to_string(),
                RawStructureData {
                    organism_variety: Some(RawOrganismVariety {
                        prototypical_form: RawOrganismId::structure("leuco"),
                        lifecycle: RawLifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                    }),
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("leuco_chunk_production"),
                    },
                    construction_strategy: RawConstructionStrategy::Direct {
                        work: Some(3.),
                        materials: HashMap::from_iter([("leuco_chunk".to_string(), 1)]),
                        allowed_terrain_types: HashSet::from_iter([
                            "loam".to_string(),
                            "muddy".to_string(),
                        ]),
                    },
                    max_workers: 6,
                    footprint: Some(Footprint::single()),
                },
            ),
            (
                "acacia_seedling".to_string(),
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
                    construction_strategy: RawConstructionStrategy::Direct {
                        work: None,
                        materials: HashMap::from_iter([("acacia_seed".to_string(), 1)]),
                        allowed_terrain_types: HashSet::from_iter([
                            "loam".to_string(),
                            "muddy".to_string(),
                        ]),
                    },
                    max_workers: 1,
                    footprint: Some(Footprint::single()),
                },
            ),
            (
                "acacia_sprout".to_string(),
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
                    construction_strategy: RawConstructionStrategy::Seedling(
                        "acacia_seedling".to_string(),
                    ),
                    max_workers: 1,
                    footprint: Some(Footprint::single()),
                },
            ),
            (
                "acacia".to_string(),
                RawStructureData {
                    organism_variety: Some(RawOrganismVariety {
                        prototypical_form: RawOrganismId::structure("acacia"),
                        lifecycle: RawLifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(300.), Energy(-1.)),
                    }),
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("acacia_leaf_production"),
                    },
                    construction_strategy: RawConstructionStrategy::Seedling(
                        "acacia_seedling".to_string(),
                    ),
                    max_workers: 6,
                    footprint: Some(Footprint::single()),
                },
            ),
            (
                "ant_hive".to_string(),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("ant_egg_production"),
                    },
                    construction_strategy: RawConstructionStrategy::Direct {
                        work: Some(10.),
                        materials: HashMap::new(),
                        allowed_terrain_types: HashSet::from_iter([
                            "loam".to_string(),
                            "muddy".to_string(),
                            "rocky".to_string(),
                        ]),
                    },
                    max_workers: 3,
                    footprint: Some(Footprint::hexagon(1)),
                },
            ),
            (
                "hatchery".to_string(),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Crafting {
                        starting_recipe: RawActiveRecipe::new("hatch_ants"),
                    },
                    construction_strategy: RawConstructionStrategy::Direct {
                        work: Some(5.),
                        materials: HashMap::new(),
                        allowed_terrain_types: HashSet::from_iter([
                            "loam".to_string(),
                            "muddy".to_string(),
                            "rocky".to_string(),
                        ]),
                    },
                    max_workers: 6,
                    // Forms a crescent shape
                    footprint: Some(Footprint::single()),
                },
            ),
            (
                "storage".to_string(),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Storage {
                        max_slot_count: 3,
                        reserved_for: None,
                    },
                    construction_strategy: RawConstructionStrategy::Direct {
                        work: Some(10.),
                        materials: HashMap::from_iter([("leuco_chunk".to_string(), 1)]),
                        allowed_terrain_types: HashSet::from_iter([
                            "loam".to_string(),
                            "muddy".to_string(),
                            "rocky".to_string(),
                        ]),
                    },
                    max_workers: 6,
                    footprint: Some(Footprint::single()),
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
