//! The plugin to handle loading of manifest assets.

use std::marker::PhantomData;

use bevy::{prelude::*, reflect::TypePath};

use crate::asset_management::{AssetCollectionExt, AssetState, Loadable};

use super::{
    loader::{IsRawManifest, RawManifestLoader},
    Manifest,
};

/// A plugin to load and process [`Manifest`] types from disk.
pub(crate) struct ManifestPlugin<M>
where
    M: IsRawManifest,
{
    /// Make the compiler happy and use the generic argument.
    _phantom_data: PhantomData<M>,
}

impl<M> ManifestPlugin<M>
where
    M: IsRawManifest,
{
    /// Create a new raw manifest plugin.
    pub(crate) fn new() -> Self {
        Self {
            _phantom_data: PhantomData::default(),
        }
    }
}

/// System set for all [`detect_manifest_creation`] systems
#[derive(Debug, PartialEq, Eq, Hash, Clone, SystemSet)]
pub struct DetectManifestCreationSet;

impl<M> Plugin for ManifestPlugin<M>
where
    M: IsRawManifest,
{
    fn build(&self, app: &mut App) {
        info!("Building RawManifestPlugin for {}", M::path().display());

        app.init_asset_loader::<RawManifestLoader<M>>()
            .add_asset::<M>()
            .add_asset_collection::<RawManifestHandle<M>>()
            .add_systems(
                Update,
                detect_manifest_creation::<M>
                    .in_set(DetectManifestCreationSet)
                    .in_schedule(OnExit(AssetState::LoadManifests)),
            )
            .add_systems(
                Update,
                detect_manifest_modification::<M>
                    .run_if(resource_exists::<Manifest<M::Marker, M::Data>>()),
            );
    }
}

/// Resource to store the handle to a [`IsRawManifest`] type while it is being loaded.
///
/// This is necessary to stop the asset from being discarded.
#[derive(Debug, Clone, Resource)]
pub struct RawManifestHandle<M: TypePath>
where
    M: IsRawManifest,
{
    /// The handle to the raw manifest asset.
    ///
    /// We mainly need this for the asset to not be unloaded.
    handle: Handle<M>,
}

impl<M: TypePath> Loadable for RawManifestHandle<M>
where
    M: IsRawManifest,
{
    const STAGE: AssetState = AssetState::LoadManifests;

    fn initialize(world: &mut World) {
        let asset_server = world.resource::<AssetServer>();
        let handle: Handle<M> = asset_server.load(M::path());

        world.insert_resource(Self { handle });
    }

    fn load_state(&self, asset_server: &AssetServer) -> bevy::asset::LoadState {
        let load_state = asset_server.get_load_state(self.handle.clone_weak());

        debug!("Load state: {load_state:?}");

        load_state
    }
}

/// Wait for the manifest to be fully loaded and then process it.
pub fn detect_manifest_creation<M: TypePath>(
    mut commands: Commands,
    raw_manifest_handle: Res<RawManifestHandle<M>>,
    raw_manifests: Res<Assets<M>>,
) where
    M: IsRawManifest,
{
    let Some(raw_manifest) = raw_manifests.get(&raw_manifest_handle.handle) else {
        error!(
            "Raw manifest for {} created, but asset not available!",
            M::path().display()
        );
        return;
    };

    info!("Manifest asset {} loaded!", M::path().display());

    // Create the manifest and insert it as a resource
    commands.insert_resource(raw_manifest.process());
}

/// Update the manifest after the asset has been changed.
fn detect_manifest_modification<M: TypePath>(
    mut ev_asset: EventReader<AssetEvent<M>>,
    raw_manifests: Res<Assets<M>>,
    mut manifest: ResMut<Manifest<M::Marker, M::Data>>,
) where
    M: IsRawManifest,
{
    for ev in ev_asset.iter() {
        if let AssetEvent::Modified { handle } = ev {
            let Some(raw_manifest) = raw_manifests.get(handle) else {
                warn!("Raw manifest modified, but asset not available!");
                continue;
            };

            debug!("Manifest asset {} modified.", M::path().display());

            // Update the manifest resource
            *manifest = raw_manifest.process();
        }
    }
}
