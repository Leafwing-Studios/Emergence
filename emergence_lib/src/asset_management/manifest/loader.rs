//! A loader for manifest assets.

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use thiserror::Error;

use bevy::{
    asset::{Asset, AssetLoader, AsyncReadExt, LoadContext},
    reflect::TypePath,
    utils::BoxedFuture,
};

use bevy::reflect::TypeUuid;
use serde::Deserialize;

use super::Manifest;

/// The raw manifest data before it has been processed.
///
/// The processing will primarily remove the string IDs and replace them by numbers.
pub trait IsRawManifest:
    Asset + std::fmt::Debug + TypePath + TypeUuid + Send + Sync + for<'de> Deserialize<'de> + 'static
{
    /// The file extension of this manifest type.
    ///
    /// This is used to determine which manifest loader to use.
    /// Note that this must be unique across all manifest types,
    /// otherwise the wrong loader will be used.
    const EXTENSION: &'static str;

    /// The marker type for the manifest ID.
    type Marker: 'static + Send + Sync;

    /// The type of the processed manifest data.
    type Data: std::fmt::Debug + Send + Sync;

    /// Returns the path to the manifest file.
    fn path() -> PathBuf {
        Path::new("manifests/base_game").with_extension(Self::EXTENSION)
    }

    /// Process the raw manifest from the asset file to the manifest data used in-game.
    fn process(&self) -> Manifest<Self::Marker, Self::Data>;
}

/// A loader for `.manifest.json` files.
#[derive(Debug, Clone)]
pub(crate) struct RawManifestLoader<M>
where
    M: IsRawManifest,
{
    /// Use the generic to make the compiler happy.
    _phantom_manifest: PhantomData<M>,
}

/// An erorr produced when loading a raw manifest.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RawManifestError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [serde_json](serde_json) Error
    #[error("Could not parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl<M> AssetLoader for RawManifestLoader<M>
where
    M: IsRawManifest,
{
    type Asset = M;
    type Settings = ();
    type Error = RawManifestError;

    fn extensions(&self) -> &[&str] {
        &[<M as IsRawManifest>::EXTENSION]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let custom_asset = serde_json::from_slice::<M>(&bytes)?;
            Ok(custom_asset)
        })
    }
}

impl<M> Default for RawManifestLoader<M>
where
    M: IsRawManifest,
{
    fn default() -> Self {
        Self {
            _phantom_manifest: PhantomData,
        }
    }
}
