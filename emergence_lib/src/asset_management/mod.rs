//! Code related to loading, storing and tracking assets
//!
//! # Model conventions
//!
//! In Blender, models should:
//! - be facing towards +x
//! - be centered on the origin
//! - have +Z up (Bevy does the conversion automatically)
//! - scaled such that 1 unit = 1 hex radius
//! - sitting just on top of the XY plane
//! - be exported as embedded gltF files

use std::{
    any::TypeId,
    fmt::{Display, Formatter},
};

use self::manifest::plugin::DetectManifestCreationSet;
use bevy::{
    asset::LoadState,
    prelude::*,
    utils::{get_short_name, HashMap},
};

pub mod manifest;

/// Collects asset management systems and resources.
pub struct AssetManagementPlugin;

impl Plugin for AssetManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AssetState>()
            .init_resource::<AssetsToLoad>()
            .add_systems(
                Update,
                check_manifests_loaded.run_if(in_state(AssetState::LoadManifests)),
            )
            .add_systems(
                Update,
                check_assets_loaded.run_if(in_state(AssetState::LoadAssets)),
            )
            // This is needed to ensure that the manifest resources are actually created in time for AssetState::Loading
            // BLOCKED: this can be removed in Bevy 0.11, as schedules will automatically flush the commands.
            .add_systems(
                OnExit(AssetState::LoadManifests),
                apply_deferred.after(DetectManifestCreationSet),
            );
    }
}

/// Tracks the progress of asset loading.
#[derive(States, Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum AssetState {
    #[default]
    /// Load manifests.
    LoadManifests,
    /// Assets still need to be loaded.
    LoadAssets,
    /// All assets are loaded.
    FullyLoaded,
}

/// The set of all assets that need to be loaded.
#[derive(Resource, Debug, Default)]
pub struct AssetsToLoad {
    /// The set of [`Loadable`] types that still need to be loaded
    remaining: HashMap<TypeId, String>,
}

impl Display for AssetsToLoad {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut remaining: Vec<String> = self.remaining.values().cloned().collect();
        remaining.sort();

        write!(f, "{}", remaining.join("\n"))
    }
}

impl AssetsToLoad {
    /// Registers that `T` still needs to be loaded.
    pub fn insert<T: Loadable>(&mut self) {
        let type_id = TypeId::of::<T>();
        let full_name = std::any::type_name::<T>();
        let short_name = get_short_name(full_name);

        self.remaining.insert(type_id, short_name);
    }

    /// Does `T` still need to be loaded?
    pub fn contains<T: Loadable>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.remaining.contains_key(&type_id)
    }

    /// Registers that `T` is done loading.
    pub fn remove<T: Loadable>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.remaining.remove(&type_id);
    }
}

/// A system that checks if all manifests are loaded.
fn check_manifests_loaded(
    assets_to_load: Res<AssetsToLoad>,
    mut next_state: ResMut<NextState<AssetState>>,
) {
    if assets_to_load.remaining.is_empty() {
        info!("All manifests loaded: transitioning to AssetState::LoadAssets");

        next_state.set(AssetState::LoadAssets);
    } else if assets_to_load.is_changed() {
        info!("Waiting for manifests to load:\n{}", *assets_to_load);
    }
}

/// A system that checks if all assets are loaded.
fn check_assets_loaded(
    assets_to_load: Res<AssetsToLoad>,
    mut next_state: ResMut<NextState<AssetState>>,
) {
    if assets_to_load.remaining.is_empty() {
        info!("All assets loaded: transitioning to AssetState::Ready");

        next_state.set(AssetState::FullyLoaded);
    } else {
        info!("Waiting for assets to load:\n{}", *assets_to_load);
    }
}

/// An asset collection that must be loaded before the game can start.
///
/// This asset collection should begin async asset loading in its [`FromWorld`] implementation.
pub trait Loadable: Resource + Sized {
    /// The stage in which to load the assets.
    const STAGE: AssetState;

    /// Begin loading the assets.
    ///
    /// This system runs during [`Self::STAGE`].
    fn initialize(world: &mut World);

    /// Record that assets of type `T` still need to be loaded.
    fn register_assets_to_load(mut assets_to_load: ResMut<AssetsToLoad>) {
        assets_to_load.insert::<Self>();
    }

    /// How far along are we in loading these assets?
    fn load_state(&self, asset_server: &AssetServer) -> Option<LoadState>;

    /// A system that checks if the asset collection of type `T` loaded.
    fn check_loaded(
        asset_collection: Res<Self>,
        asset_server: Res<AssetServer>,
        mut assets_to_load: ResMut<AssetsToLoad>,
    ) {
        let load_state = asset_collection.load_state(&asset_server);
        if load_state == Some(LoadState::Loaded) && assets_to_load.contains::<Self>() {
            assets_to_load.remove::<Self>();
        }
    }
}

/// An [`App`] extension trait to add and setup [`Loadable`] collections.
pub trait AssetCollectionExt {
    /// Sets up all resources and systems needed to load the asset collection of type `T` to the app.
    fn add_asset_collection<T: Loadable>(&mut self) -> &mut Self;
}

impl AssetCollectionExt for App {
    fn add_asset_collection<T: Loadable>(&mut self) -> &mut Self {
        info!("Adding asset collection: {}", std::any::type_name::<T>());

        // Begin the loading process
        self.add_systems(
            OnEnter(T::STAGE),
            (T::initialize, T::register_assets_to_load),
        );
        // Poll each asset collection
        self.add_systems(Update, T::check_loaded.run_if(in_state(T::STAGE)));

        self
    }
}
