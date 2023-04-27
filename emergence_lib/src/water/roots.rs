//! Datastructures and mechanics for roots, which draw water from the nearby water table.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::simulation::geometry::Height;

/// The volume around a tile that roots can draw water from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootZone {
    /// The depth from the surface beyond which roots cannot draw water.
    pub max_depth: Height,
    /// The radius of the root zone.
    ///
    /// Water can only be drawn from tiles within this radius.
    pub radius: u8,
}

impl Display for RootZone {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Root Zone: {} tiles deep, {} tiles radius",
            self.max_depth, self.radius
        )
    }
}
