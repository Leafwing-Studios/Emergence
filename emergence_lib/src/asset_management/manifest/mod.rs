//! Read-only definitions for game objects.
//!
//! These are intended to be loaded from a file or dynamically generated via gameplay.
//! Other systems should look up the data contained here,
//! in order to populate the properties of in-game entities.

mod identifier;

pub use self::identifier::*;
pub mod loader;
pub mod plugin;

use bevy::{prelude::*, utils::HashMap};
use std::{any::type_name, fmt::Debug};

/// Write-only data definitions.
///
/// These are intended to be created a single time, via [`Manifest::new`].
#[derive(Debug, Resource)]
pub struct Manifest<T, Data>
where
    T: 'static,
    Data: Debug,
{
    /// The internal mapping to the data
    data_map: HashMap<Id<T>, Data>,

    /// The human-readable name associated with each Id.
    name_map: HashMap<Id<T>, String>,
}

impl<T: 'static, Data: Debug> Default for Manifest<T, Data> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, Data> Manifest<T, Data>
where
    Data: Debug,
{
    /// Create a new empty manifest.
    pub fn new() -> Self {
        Self {
            data_map: HashMap::default(),
            name_map: HashMap::default(),
        }
    }

    /// Returns a reference to the internal data map.
    pub fn data_map(&self) -> &HashMap<Id<T>, Data> {
        &self.data_map
    }

    /// Returns a reference to the internal name map.
    pub fn name_map(&self) -> &HashMap<Id<T>, String> {
        &self.name_map
    }

    /// Adds an entry to the manifest by supplying the `name` associated with the [`Id`] type to be constructed.
    ///
    /// Returns any existing `Data` entry if this overwrote the data.
    pub fn insert(&mut self, name: String, data: Data) {
        let id = Id::from_name(name.clone());

        self.data_map.insert(id, data);
        self.name_map.insert(id, name);
    }

    /// Get the data entry for the given ID.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn get(&self, id: Id<T>) -> &Data {
        self.data_map
            .get(&id)
            .unwrap_or_else(|| panic!("ID {id:?} {} not found in manifest", self.name(id)))
    }

    /// Returns the human-readable name associated with the provided `id`.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn name(&self, id: Id<T>) -> &str {
        self.name_map.get(&id).unwrap_or_else(|| {
            panic!(
                "ID {:?} of type {:?} not found in manifest",
                id,
                type_name::<T>()
            )
        })
    }

    /// Returns the complete list of names of the loaded options.
    ///
    /// The order is arbitrary.
    pub fn names(&self) -> impl IntoIterator<Item = &str> {
        let variants = self.variants();
        variants.into_iter().map(|id| self.name(id))
    }

    /// The complete list of loaded options.
    ///
    /// The order is arbitrary.
    pub fn variants(&self) -> impl IntoIterator<Item = Id<T>> + '_ {
        self.data_map.keys().copied()
    }
}

/// A plugin that adds the default manifests to the app.
#[cfg(test)]
pub struct DummyManifestPlugin;

#[cfg(test)]
impl Plugin for DummyManifestPlugin {
    fn build(&self, app: &mut App) {
        use crate::{
            crafting::recipe::RecipeManifest,
            items::item_manifest::ItemManifest,
            structures::structure_manifest::{StructureData, StructureManifest},
            terrain::terrain_manifest::{TerrainData, TerrainManifest},
            units::basic_needs::Diet,
            units::unit_manifest::{UnitData, UnitManifest},
        };

        let mut terrain_manifest = TerrainManifest::default();
        terrain_manifest.insert("grassy".to_string(), TerrainData::default());
        terrain_manifest.insert("rocky".to_string(), TerrainData::default());
        app.insert_resource(terrain_manifest);

        let mut unit_manifest = UnitManifest::default();
        unit_manifest.insert(
            "simple_unit".to_string(),
            UnitData::simple("simple_unit", Diet::simple("food")),
        );
        app.insert_resource(unit_manifest);

        let mut structure_manifest = StructureManifest::default();
        structure_manifest.insert(
            "simple_structure".to_string(),
            StructureData::organism("simple_structure"),
        );
        structure_manifest.insert("passable_structure".to_string(), StructureData::passable());
        structure_manifest.insert("simple_landmark".to_string(), StructureData::impassable());
        app.insert_resource(structure_manifest);

        let item_manifest = ItemManifest::default();
        app.insert_resource(item_manifest);

        let recipe_manifest = RecipeManifest::default();
        app.insert_resource(recipe_manifest);
    }
}
