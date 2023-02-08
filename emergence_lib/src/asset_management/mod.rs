//! Code related to loading, storing and tracking assets

use self::{structures::StructureHandles, terrain::TerrainHandles};
use bevy::prelude::*;

pub(crate) mod structures;
pub(crate) mod terrain;

/// Collects asset management systems and resources.
pub struct AssetManagementPlugin;

impl Plugin for AssetManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainHandles>()
            .init_resource::<StructureHandles>()
            .add_state(AssetState::Ready);
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
