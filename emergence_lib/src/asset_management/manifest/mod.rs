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
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

mod deserialize;

/// Write-once data definitions.
///
/// These are intended to be created a single time, via [`Manifest::new`].
#[derive(Debug, Resource, Serialize)]
pub(crate) struct Manifest<Id, Data>
where
    Id: Debug
        + Display
        + PartialEq
        + Eq
        + Hash
        + Send
        + Sync
        + TypeUuid
        + for<'d> Deserialize<'d>
        + 'static,
    Data: Debug + Send + Sync + TypeUuid + 'static + for<'d> Deserialize<'d>,
{
    /// The internal mapping.
    map: HashMap<Id, Data>,
}

impl<Id, Data> Manifest<Id, Data>
where
    Id: Debug
        + Display
        + PartialEq
        + Eq
        + Hash
        + Send
        + Sync
        + TypeUuid
        + for<'d> Deserialize<'d>
        + 'static,
    Data: Debug + Send + Sync + TypeUuid + 'static + for<'d> Deserialize<'d>,
{
    /// Create a new manifest with the given definitions.
    pub fn new(map: HashMap<Id, Data>) -> Self {
        Self { map }
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
    Id: Debug
        + Display
        + PartialEq
        + Eq
        + Hash
        + Send
        + Sync
        + TypeUuid
        + for<'d> Deserialize<'d>
        + 'static,
    Data: Debug + Send + Sync + TypeUuid + 'static + for<'d> Deserialize<'d>,
{
    // TODO: Find a better / safer way to generate this
    // Perhaps we need a combination of the ID and Data UUID
    const TYPE_UUID: bevy::utils::Uuid = Data::TYPE_UUID;
}

pub struct ManifestAssetLoader<Id, Data>
where
    Id: Debug
        + Display
        + PartialEq
        + Eq
        + Hash
        + Send
        + Sync
        + TypeUuid
        + Deserialize<'static>
        + 'static,
    Data: Debug + Send + Sync + TypeUuid + 'static + Deserialize<'static>,
{
    id: PhantomData<Id>,
    data: PhantomData<Data>,
}

impl<Id, Data> AssetLoader for ManifestAssetLoader<Id, Data>
where
    Id: Debug
        + Display
        + PartialEq
        + Eq
        + Hash
        + Send
        + Sync
        + TypeUuid
        + Deserialize<'static>
        + 'static,
    Data: Debug + Send + Sync + TypeUuid + 'static + Deserialize<'static>,
{
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let toml_str = String::from_utf8(Vec::from(bytes))?;
            let manifest: Manifest<Id, Data> = toml::from_str(&toml_str)?;
            load_context.set_default_asset(LoadedAsset::new(manifest));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["manifest.toml"]
    }
}
