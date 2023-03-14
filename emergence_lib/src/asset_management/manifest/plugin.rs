//! The plugin to handle loading of manifest assets.

use bevy::prelude::*;

use super::{
    loader::{RawManifest, RawManifestLoader},
    raw::RawItemManifest,
};

/// A plugin to load and process manifest assets.
struct ManifestPlugin;

impl Plugin for ManifestPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RawManifestLoader<RawItemManifest>>()
            .add_asset::<RawItemManifest>()
            .add_startup_system(initiate_manifest_loading::<RawItemManifest>);
    }
}

/// Resource to store the handle to a [`RawManifest`] while it is being loaded.
///
/// This is necessary to stop the asset from being discarded.
#[derive(Debug, Default, Clone, Resource)]
struct RawManifestHandle<M>
where
    M: RawManifest,
{
    /// The handle to the raw manifest asset.
    handle: Handle<M>,
}

/// Initiate the loading of the manifest.
fn initiate_manifest_loading<M>(mut commands: Commands, asset_server: Res<AssetServer>)
where
    M: RawManifest,
{
    let handle: Handle<M> = asset_server.load(M::path());

    commands.insert_resource(RawManifestHandle::<M> { handle });
}
