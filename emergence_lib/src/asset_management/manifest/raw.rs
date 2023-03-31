//! The raw manifest data before it has been processed.
//!
//! The processing will primarily remove the string IDs and replace them by numbers.

use bevy::reflect::TypeUuid;
use serde::Deserialize;

use super::Manifest;

/// A utility trait to ensure that all trait bounds are satisfied.
pub trait RawManifest:
    std::fmt::Debug + TypeUuid + Send + Sync + for<'de> Deserialize<'de> + 'static
{
    /// The marker type for the manifest ID.
    type Marker: 'static + Send + Sync;

    /// The type of the processed manifest data.
    type Data: std::fmt::Debug + Send + Sync;

    /// The path of the asset.
    fn path() -> &'static str;

    /// Process the raw manifest from the asset file to the manifest data used in-game.
    fn process(&self) -> Manifest<Self::Marker, Self::Data>;
}
