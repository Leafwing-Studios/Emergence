//! The plugin to handle loading of manifest assets.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    asset_management::{AssetCollectionExt, AssetState, Loadable},
    items::item_manifest::RawItemManifest,
    structures::structure_manifest::StructureManifest,
    terrain::terrain_manifest::TerrainManifest,
};

use super::{loader::RawManifestLoader, raw::RawManifest, Manifest};

/// A plugin to handle the creation of all manifest resources.
pub struct ManifestPlugin;

impl Plugin for ManifestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StructureManifest>()
            .init_resource::<TerrainManifest>()
            .add_plugin(RawManifestPlugin::<RawItemManifest>::new())
            // This is needed to ensure that the manifest resources are actually created in time for AssetState::Ready
            .add_system(
                apply_system_buffers
                    .after(DetectManifestCreationSet)
                    .in_schedule(OnExit(AssetState::Loading)),
            );
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

/// System set for all [`detect_manifest_creation`] systems
#[derive(Debug, PartialEq, Eq, Hash, Clone, SystemSet)]
struct DetectManifestCreationSet;

impl<M> Plugin for RawManifestPlugin<M>
where
    M: RawManifest,
{
    fn build(&self, app: &mut App) {
        info!("Building RawManifestPlugin for {}", M::path());

        app.init_asset_loader::<RawManifestLoader<M>>()
            .add_asset::<M>()
            .add_asset_collection::<RawManifestHandle<M>>()
            .add_system(
                detect_manifest_creation::<M>
                    .in_set(DetectManifestCreationSet)
                    .in_schedule(OnExit(AssetState::Loading)),
            )
            .add_system(
                detect_manifest_modification::<M>
                    .run_if(resource_exists::<Manifest<M::Marker, M::Data>>()),
            );
    }
}

/// Resource to store the handle to a [`RawManifest`] while it is being loaded.
///
/// This is necessary to stop the asset from being discarded.
#[derive(Debug, Clone, Resource)]
struct RawManifestHandle<M>
where
    M: RawManifest,
{
    /// The handle to the raw manifest asset.
    ///
    /// We mainly need this for the asset to not be unloaded.
    handle: Handle<M>,
}

impl<M> FromWorld for RawManifestHandle<M>
where
    M: RawManifest,
{
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let handle: Handle<M> = asset_server.load(M::path());

        Self { handle }
    }
}

impl<M> Loadable for RawManifestHandle<M>
where
    M: RawManifest,
{
    fn load_state(&self, asset_server: &AssetServer) -> bevy::asset::LoadState {
        let load_state = asset_server.get_load_state(self.handle.clone_weak());

        debug!("Load state: {load_state:?}");

        load_state
    }
}

/// Wait for the manifest to be fully loaded and then process it.
fn detect_manifest_creation<M>(
    mut commands: Commands,
    raw_manifest_handle: Res<RawManifestHandle<M>>,
    raw_manifests: Res<Assets<M>>,
) where
    M: RawManifest,
{
    let Some(raw_manifest) = raw_manifests.get(&raw_manifest_handle.handle) else {
        error!("Raw manifest for {} created, but asset not available!", M::path());
        return;
    };

    info!("Manifest asset {} loaded!", M::path());

    // Create the manifest and insert it as a resource
    commands.insert_resource(raw_manifest.process());
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
