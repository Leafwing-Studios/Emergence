use bevy::utils::HashMap;
use emergence_lib::{
    construction::RawConstructionStrategy,
    crafting::{
        item_tags::ItemTag,
        recipe::{
            RawActiveRecipe, RawRecipeData, RawRecipeInput, RawRecipeManifest, RecipeConditions,
            Threshold,
        },
    },
    items::item_manifest::{RawItemData, RawItemManifest},
    light::Illuminance,
    organisms::{
        energy::{Energy, EnergyPool},
        lifecycle::{RawLifePath, RawLifecycle},
        RawOrganismId, RawOrganismVariety,
    },
    simulation::geometry::Height,
    structures::{
        structure_manifest::{RawStructureData, RawStructureKind, RawStructureManifest},
        Footprint,
    },
    terrain::terrain_manifest::{RawTerrainManifest, TerrainData},
    units::{
        basic_needs::RawDiet,
        unit_manifest::{RawUnitData, RawUnitManifest},
        WanderingBehavior,
    },
    water::{
        roots::RootZone,
        water_dynamics::{SoilWaterEvaporationRate, SoilWaterFlowRate},
        SoilWaterCapacity,
    },
};
use leafwing_abilities::prelude::Pool;

#[test]
fn can_serialize_item_manifest() {
    // Create a new raw item manifest
    let raw_item_manifest = RawItemManifest {
        items: HashMap::from_iter(vec![
            (
                "test_item".to_string(),
                RawItemData {
                    stack_size: 1,
                    compostable: true,
                    fluid: false,
                    buoyant: true,
                    seed: None,
                },
            ),
            (
                "test_item_2".to_string(),
                RawItemData {
                    stack_size: 2,
                    compostable: false,
                    fluid: false,
                    buoyant: false,
                    seed: Some(RawOrganismId::Structure("test_organism".to_string())),
                },
            ),
            (
                "water".to_string(),
                RawItemData {
                    stack_size: 100,
                    compostable: false,
                    fluid: true,
                    buoyant: false,
                    seed: None,
                },
            ),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_item_manifest).unwrap();
    println!("{}", &serialized);

    // Deserialize it
    let deserialized: RawItemManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_item_manifest, deserialized);
}

#[test]
fn can_serialize_terrain_manifest() {
    // Create a new raw terrain manifest
    let raw_terrain_manifest = RawTerrainManifest {
        terrain_types: HashMap::from_iter(vec![(
            "test_terrain".to_string(),
            TerrainData {
                walking_speed: 1.0,
                soil_water_capacity: SoilWaterCapacity(0.3),
                soil_water_flow_rate: SoilWaterFlowRate(0.1),
                soil_water_evaporation_rate: SoilWaterEvaporationRate(0.2),
            },
        )]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_terrain_manifest).unwrap();
    println!("{}", &serialized);

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
                    max_age: 10.,
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
                    max_age: 0.2,
                },
            ),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_unit_manifest).unwrap();
    println!("{}", &serialized);

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
                    inputs: RawRecipeInput::empty(),
                    outputs: HashMap::from_iter([
                        ("acacia_leaf".to_string(), 1.),
                        // Output can be stochastic
                        ("acacia_seed".to_string(), 0.1),
                    ]),
                    craft_time: 3.,
                    conditions: Some(RecipeConditions::new(
                        0,
                        Threshold::new(Illuminance::DimlyLit, Illuminance::BrightlyLit),
                    )),
                    energy: Some(Energy(20.)),
                },
            ),
            (
                "leuco_chunk_production".to_string(),
                RawRecipeData {
                    inputs: RawRecipeInput::Flexible {
                        tag: ItemTag::Compostable,
                        count: 1,
                    },
                    outputs: HashMap::from_iter([("leuco_chunk".to_string(), 1.)]),
                    craft_time: 2.,
                    conditions: None,
                    energy: Some(Energy(40.)),
                },
            ),
            (
                "ant_egg_production".to_string(),
                RawRecipeData {
                    inputs: RawRecipeInput::single("leuco_chunk", 1),
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
                    inputs: RawRecipeInput::single("ant_egg", 1),
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
    println!("{}", &serialized);

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
                    },
                    max_workers: 6,
                    height: 1,
                    footprint: Some(Footprint::single()),
                    root_zone: None,
                    passable: false,
                },
            ),
            (
                "path".to_string(),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Path,
                    construction_strategy: RawConstructionStrategy::Direct {
                        work: Some(3.),
                        materials: HashMap::new(),
                    },
                    max_workers: 1,
                    height: 0,
                    footprint: Some(Footprint::single()),
                    root_zone: None,
                    passable: true,
                },
            ),
            (
                "spring".to_string(),
                RawStructureData {
                    organism_variety: None,
                    kind: RawStructureKind::Landmark,
                    construction_strategy: RawConstructionStrategy::Landmark,
                    max_workers: 0,
                    footprint: None,
                    root_zone: None,
                    height: 1,
                    passable: false,
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
                    },
                    max_workers: 1,
                    footprint: Some(Footprint::single()),
                    root_zone: None,
                    height: 0,
                    passable: false,
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
                    height: 1,
                    footprint: Some(Footprint::single()),
                    root_zone: Some(RootZone {
                        max_depth: Height(1.5),
                        radius: 1,
                    }),
                    passable: false,
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
                    height: 3,
                    footprint: Some(Footprint::single()),
                    root_zone: Some(RootZone {
                        max_depth: Height(3.0),
                        radius: 2,
                    }),
                    passable: false,
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
                    },
                    max_workers: 3,
                    height: 2,
                    footprint: Some(Footprint::hexagon(1)),
                    root_zone: None,
                    passable: false,
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
                    },
                    max_workers: 6,
                    height: 1,
                    footprint: Some(Footprint::single()),
                    root_zone: None,
                    passable: false,
                },
            ),
        ]),
    };

    // Serialize it
    let serialized = serde_json::to_string(&raw_structure_manifest).unwrap();
    println!("{}", &serialized);

    // Deserialize it
    let deserialized: RawStructureManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_structure_manifest, deserialized);
}
