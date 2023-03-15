//! The plugin to handle loading of manifest assets.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    loader::RawManifestLoader,
    raw::{RawItemManifest, RawManifest},
    Manifest,
};

/// A plugin to handle the creation of all manifest resources.
pub struct ManifestPlugin;

impl Plugin for ManifestPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RawManifestPlugin::<RawItemManifest>::new());
    }
}

/// A plugin to load and process raw manifest assets.
pub(crate) struct RawManifestPlugin<M>
where
    M: RawManifest,
{
    /// Make the compiler happy and use the generic argument.
    _phantom_data: PhantomData<M>,
}

impl<M> RawManifestPlugin<M>
where
    M: RawManifest,
{
    /// Create a new raw manifest plugin.
    pub(crate) fn new() -> Self {
        Self {
            _phantom_data: PhantomData::default(),
        }
    }
}

impl<M> Plugin for RawManifestPlugin<M>
where
    M: RawManifest,
{
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RawManifestLoader<M>>()
            .add_asset::<M>()
            .add_startup_system(initiate_manifest_loading::<M>)
            .add_system(detect_manifest_creation::<M>)
            .add_system(
                detect_manifest_modification::<M>
                    .run_if(resource_exists::<Manifest<M::Marker, M::Data>>()),
            );
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
    ///
    /// We mainly need this for the asset to not be unloaded.
    #[allow(dead_code)]
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
fn detect_manifest_creation<M>(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<M>>,
    raw_manifests: Res<Assets<M>>,
) where
    M: RawManifest,
{
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            let Some(raw_manifest) = raw_manifests.get(handle) else {
                warn!("Raw manifest created, but asset not available!");
                continue;
            };

            info!("Manifest asset {} loaded!", M::path());

            // Create the manifest and insert it as a resource
            commands.insert_resource(raw_manifest.process());
        }
    }
}

/// Update the manifest after the asset has been changed.
fn detect_manifest_modification<M>(
    mut ev_asset: EventReader<AssetEvent<M>>,
    raw_manifests: Res<Assets<M>>,
    mut manifest: ResMut<Manifest<M::Marker, M::Data>>,
) where
    M: RawManifest,
{
    for ev in ev_asset.iter() {
        if let AssetEvent::Modified { handle } = ev {
            let Some(raw_manifest) = raw_manifests.get(handle) else {
                warn!("Raw manifest modified, but asset not available!");
                continue;
            };

            debug!("Manifest asset {} modified.", M::path());

            // Update the manifest resource
            *manifest = raw_manifest.process();
        }
    }
}
