//! Read-only definitions for game objects.
//!
//! These are intended to be loaded from a file or dynamically generated via gameplay.
//! Other systems should look up the data contained here,
//! in order to populate the properties of in-game entities.

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::{BoxedFuture, HashMap},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

type RawManifest<Data>
where
    Data: Send + Sync + TypeUuid + 'static + DeserializeOwned,
= HashMap<String, Data>;

/// Write-once data definitions.
///
/// These are intended to be created a single time, via [`Manifest::new`].
#[derive(Debug, Resource, Serialize)]
pub(crate) struct Manifest<Id, Data>
where
    Id: PartialEq + Eq + Hash + Send + Sync + TypeUuid + From<u32> + 'static,
    Data: Send + Sync + TypeUuid + 'static + DeserializeOwned,
{
    /// Lookup table to obtain the ID, given the String identifier used in the assets.
    str_to_id: HashMap<String, Id>,

    /// Lookup table to obtain the String identifier used in the assets, given the ID.
    id_to_str: HashMap<Id, String>,

    /// The internal mapping.
    map: HashMap<Id, Data>,
}

impl<Id, Data> Manifest<Id, Data>
where
    Id: Display + PartialEq + Eq + Hash + Send + Sync + TypeUuid + From<u32> + 'static,
    Data: Send + Sync + TypeUuid + 'static + DeserializeOwned,
{
    pub fn from_raw(raw_manifest: RawManifest<Data>) -> Self {
        let mut str_to_id = HashMap::<String, Id>::new();
        let mut id_to_str = HashMap::<Id, String>::new();
        let mut map = HashMap::<Id, Data>::new();

        let mut id_counter = 0u32;

        // Convert the string identifiers used in the manifest to u32s
        for (str, data) in raw_manifest.drain() {
            assert!(
                !str_to_id.contains_key(&str),
                "Duplicate identifier '{str}'"
            );

            str_to_id.insert(str.clone(), Id::from(id_counter));
            id_to_str.insert(Id::from(id_counter), str);
            map.insert(Id::from(id_counter), data);
        }

        Self {
            str_to_id,
            id_to_str,
            map,
        }
    }

    /// Get the data entry for the given ID.
    ///
    /// # Panics
    ///
    /// This function panics when the given ID does not exist in the manifest.
    /// We assume that all IDs are valid and the manifests are complete.
    pub fn get(&self, id: &Id) -> &Data {
        self.map
            .get(id)
            .unwrap_or_else(|| panic!("ID {id} not found in manifest"))
    }

    /// The complete list of loaded options.
    ///
    /// The order is arbitrary.
    pub fn variants(&self) -> impl IntoIterator<Item = &Id> {
        self.map.keys()
    }
}

impl<Id, Data> TypeUuid for Manifest<Id, Data>
where
    Id: PartialEq + Eq + Hash + Send + Sync + TypeUuid + From<u32> + 'static,
    Data: Send + Sync + TypeUuid + 'static + DeserializeOwned,
{
    // TODO: Find a better / safer way to generate this
    // Perhaps we need a combination of the ID and Data UUID
    const TYPE_UUID: bevy::utils::Uuid = Data::TYPE_UUID;
}

pub struct ManifestAssetLoader<Id, Data>
where
    Id: PartialEq + Eq + Hash + Send + Sync + TypeUuid + From<u32> + 'static,
    Data: Send + Sync + TypeUuid + 'static + DeserializeOwned,
{
    id: PhantomData<Id>,
    data: PhantomData<Data>,
}

impl<Id, Data> AssetLoader for ManifestAssetLoader<Id, Data>
where
    Id: Display + PartialEq + Eq + Hash + Send + Sync + TypeUuid + From<u32> + 'static,
    Data: Send + Sync + TypeUuid + 'static + DeserializeOwned,
{
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let raw_manifest: RawManifest<Data> = serde_json::from_slice(bytes)?;
            let manifest = Manifest::<Id, Data>::from_raw(raw_manifest);
            load_context.set_default_asset(LoadedAsset::new(manifest));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["manifest.json"]
    }
}
