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

use std::any::TypeId;

use self::manifest::plugin::ManifestPlugin;
use bevy::{asset::LoadState, prelude::*, utils::HashSet};

pub mod manifest;

/// Collects asset management systems and resources.
pub struct AssetManagementPlugin;

impl Plugin for AssetManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AssetState>().add_plugin(ManifestPlugin);
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
    Ready,
}

/// The set of all assets that need to be loaded.
#[derive(Resource, Debug, Default)]
struct AssetsToLoad {
    /// The set of [`Loadable`] types that still need to be loaded
    set: HashSet<TypeId>,
}

impl AssetsToLoad {
    /// Registers that `T` still needs to be loaded.
    fn insert<T: Loadable>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.set.insert(type_id);
    }

    /// Registers that `T` is done loading.
    fn remove<T: Loadable>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.set.remove(&type_id);
    }

    /// A system that checks if the asset collection of type `T` loaded.
    fn check_loaded<T: Loadable>(
        asset_collection: Res<T>,
        asset_server: Res<AssetServer>,
        mut assets_to_load: ResMut<AssetsToLoad>,
    ) {
        if asset_collection.load_state(&asset_server) == LoadState::Loaded {
            assets_to_load.remove::<T>();
        }
    }

    /// A system that moves into [`AssetState::Ready`] when all assets are loaded.
    fn transition_when_complete(
        assets_to_load: Res<AssetsToLoad>,
        mut asset_state: ResMut<NextState<AssetState>>,
    ) {
        if assets_to_load.set.is_empty() {
            asset_state.set(AssetState::Ready);
        }
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

    /// How far along are we in loading these assets?
    fn load_state(&self, asset_server: &AssetServer) -> LoadState;
}

/// An [`App`] extension trait to add and setup [`Loadable`] collections.
pub trait AssetCollectionExt {
    /// Sets up all resources and systems needed to load the asset collection of type `T` to the app.
    fn add_asset_collection<T: Loadable>(&mut self) -> &mut Self;
}

impl AssetCollectionExt for App {
    fn add_asset_collection<T: Loadable>(&mut self) -> &mut Self {
        if let Some(mut assets_to_load) = self.world.get_resource_mut::<AssetsToLoad>() {
            assets_to_load.insert::<T>();
        } else {
            // Only called for the first asset collection added.
            self.add_system(
                AssetsToLoad::transition_when_complete.run_if(in_state(AssetState::LoadAssets)),
            );
            self.init_resource::<AssetsToLoad>();
        }

        // Begin the loading process
        self.add_system(T::initialize.in_schedule(OnEnter(T::STAGE)));

        // Poll each asset collection
        self.add_system(AssetsToLoad::check_loaded::<T>.run_if(in_state(AssetState::LoadAssets)));

        self
    }
}
