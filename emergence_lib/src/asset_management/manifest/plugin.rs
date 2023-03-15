//! The plugin to handle loading of manifest assets.

use bevy::prelude::*;

use super::{
    loader::{RawManifest, RawManifestLoader},
    raw::RawItemManifest,
};

/// A plugin to load and process manifest assets.
pub struct ManifestPlugin;

impl Plugin for ManifestPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RawManifestLoader<RawItemManifest>>()
            .add_asset::<RawItemManifest>()
            .add_startup_system(initiate_manifest_loading::<RawItemManifest>)
            .add_system(process_raw_manifest::<RawItemManifest>);
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

/// Wait for the manifest to be fully loaded and then process it.
fn process_raw_manifest<M>(mut ev_asset: EventReader<AssetEvent<M>>, raw_manifests: Res<Assets<M>>)
where
    M: RawManifest,
{
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let Some(raw_manifest) = raw_manifests.get(handle) else { continue };
                info!("Raw manifest loaded! {raw_manifest:?}");
            }
            AssetEvent::Modified { handle } => {
                let Some(raw_manifest) = raw_manifests.get(handle) else { continue };
                info!("Raw manifest modified! {raw_manifest:?}");
            }
            AssetEvent::Removed { handle: _ } => {
                info!("Raw manifest removed!");
            }
        }
    }
}
