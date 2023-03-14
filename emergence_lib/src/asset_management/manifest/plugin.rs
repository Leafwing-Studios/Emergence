//! The plugin to handle loading of manifest assets.

use bevy::prelude::*;

use super::{loader::RawManifestLoader, raw::RawItemManifest};

/// A plugin to load and process manifest assets.
struct ManifestPlugin;

impl Plugin for ManifestPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RawManifestLoader<RawItemManifest>>()
            .add_asset::<RawItemManifest>();
    }
}
