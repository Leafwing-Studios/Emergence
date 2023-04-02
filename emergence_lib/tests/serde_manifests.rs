use bevy::utils::HashMap;
use emergence_lib::{
    asset_management::manifest::Id,
    items::item_manifest::{ItemData, RawItemManifest},
    organisms::{
        energy::{Energy, EnergyPool},
        lifecycle::Lifecycle,
        OrganismId, OrganismVariety,
    },
    terrain::terrain_manifest::{RawTerrainManifest, TerrainData},
    units::{
        hunger::Diet,
        unit_manifest::{RawUnitManifest, UnitData},
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
                UnitData {
                    organism_variety: OrganismVariety {
                        prototypical_form: OrganismId::Unit(Id::from_name("ant")),
                        lifecycle: Lifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(100.), Energy(-1.)),
                    },
                    diet: Diet::new(Id::from_name("leuco_chunk"), Energy(50.)),
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
                UnitData {
                    organism_variety: OrganismVariety {
                        prototypical_form: OrganismId::Unit(Id::from_name("test_unit")),
                        lifecycle: Lifecycle::STATIC,
                        energy_pool: EnergyPool::new_full(Energy(50.), Energy(0.)),
                    },
                    diet: Diet::new(Id::from_name("acacia_leaf"), Energy(0.)),
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
