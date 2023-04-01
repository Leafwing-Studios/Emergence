//! A loader for manifest assets.

use std::marker::PhantomData;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    utils::BoxedFuture,
};

use bevy::reflect::TypeUuid;
use serde::Deserialize;

use super::Manifest;

/// The raw manifest data before it has been processed.
///
/// The processing will primarily remove the string IDs and replace them by numbers.
pub trait RawManifest:
    std::fmt::Debug + TypeUuid + Send + Sync + for<'de> Deserialize<'de> + 'static
{
    /// The marker type for the manifest ID.
    type Marker: 'static + Send + Sync;

    /// The type of the processed manifest data.
    type Data: std::fmt::Debug + Send + Sync;

    /// The path of the asset.
    fn path() -> &'static str;

    /// Process the raw manifest from the asset file to the manifest data used in-game.
    fn process(&self) -> Manifest<Self::Marker, Self::Data>;
}

/// A loader for `.manifest.json` files.
#[derive(Debug, Clone)]
pub(crate) struct RawManifestLoader<M>
where
    M: RawManifest,
{
    /// Use the generic to make the compiler happy.
    _phantom_manifest: PhantomData<M>,
}

impl<M> AssetLoader for RawManifestLoader<M>
where
    M: RawManifest,
{
    fn extensions(&self) -> &[&str] {
        &["manifest.json"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let raw_manifest = serde_json::from_slice::<M>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(raw_manifest));
            Ok(())
        })
    }
}

impl<M> Default for RawManifestLoader<M>
where
    M: RawManifest,
{
    fn default() -> Self {
        Self {
            _phantom_manifest: PhantomData::default(),
        }
    }
}
