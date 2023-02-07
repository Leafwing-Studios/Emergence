//! Code related to loading, storing and tracking assets

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use self::{structures::StructureHandles, terrain::TerrainHandles};

pub(crate) mod structures;
pub(crate) mod terrain;

/// Collects asset management systems and resources.
pub struct AssetManagementPlugin;

impl Plugin for AssetManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainHandles>()
            .add_state(AssetState::Loading)
            .add_loading_state(
                LoadingState::new(AssetState::Loading)
                    .continue_to_state(AssetState::Ready)
                    .with_collection::<StructureHandles>(),
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
