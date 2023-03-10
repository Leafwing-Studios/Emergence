//! A loader for manifest assets.

use std::marker::PhantomData;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

/// A utility trait to ensure that all trait bounds are satisfied.
pub trait RawManifest: TypeUuid + Send + Sync + 'static {}

/// A loader for `.manifest.json` files.
#[derive(Debug, Clone, Default)]
pub struct RawManifestLoader<M>
where
    M: RawManifest,
{
    /// Use the generic to make the compiler happy.
    _phantom_manifest: PhantomData<M>,
}

impl<M> AssetLoader for RawManifestLoader<M>
where
    M: RawManifest + for<'de> Deserialize<'de>,
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
