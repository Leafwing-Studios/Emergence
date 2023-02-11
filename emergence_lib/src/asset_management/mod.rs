//! Code related to loading, storing and tracking assets

use self::{structures::StructureHandles, terrain::TerrainHandles};
use bevy::{asset::LoadState, prelude::*};

pub(crate) mod manifest;
pub(crate) mod structures;
pub(crate) mod terrain;

/// Collects asset management systems and resources.
pub struct AssetManagementPlugin;

impl Plugin for AssetManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainHandles>()
            .init_resource::<StructureHandles>()
            .add_state(AssetState::Loading)
            .add_system_set(
                SystemSet::on_update(AssetState::Loading)
                    .with_system(StructureHandles::check_loaded),
            );
    }
}

/// Tracks the progress of asset loading.
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum AssetState {
    #[default]
    /// Assets still need to be loaded.
    Loading,
    /// All assets are loaded.
    Ready,
}

/// An asset collection that must be loaded before the game can start.
pub trait Loadable: Resource + Sized {
    /// How far along are we in loading these assets?
    fn load_state(&self, asset_server: &AssetServer) -> LoadState;

    /// A system that checks if these assets are loaded.
    fn check_loaded(
        asset_collection: Res<Self>,
        asset_server: Res<AssetServer>,
        mut asset_state: ResMut<State<AssetState>>,
    ) {
        let overall_load_state = asset_collection.load_state(&asset_server);

        if overall_load_state == LoadState::Loaded {
            info!("Transitioning to AssetState::Ready");
            asset_state.set(AssetState::Ready).unwrap();
        }
    }
}
