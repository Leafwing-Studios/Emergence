use bevy::reflect::{FromReflect, Reflect};

use crate::units::UnitData;

use super::Manifest;

/// The marker type for [`Id<Unit>`](super::Id).
#[derive(Reflect, FromReflect, Clone, Copy, PartialEq, Eq)]
pub struct Unit;
/// Stores the read-only definitions for all units.
pub(crate) type UnitManifest = Manifest<Unit, UnitData>;
