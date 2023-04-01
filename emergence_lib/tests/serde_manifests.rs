use bevy::utils::HashMap;
use emergence_lib::{
    items::item_manifest::{ItemData, RawItemManifest},
    terrain::terrain_manifest::{RawTerrainManifest, TerrainData},
};

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
    print!("{}", &serialized);

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
    print!("{}", &serialized);

    // Deserialize it
    let deserialized: RawTerrainManifest = serde_json::from_str(&serialized).unwrap();

    // Check that the deserialized version is the same as the original
    assert_eq!(raw_terrain_manifest, deserialized);
}
