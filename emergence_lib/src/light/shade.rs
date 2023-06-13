//! Shade is cast by structures and terrain based on their height and the position of the sun.

use crate::{
    geometry::{DiscreteHeight, MapGeometry, VoxelPos},
    simulation::time::{InGameTime, TimeOfDay},
};
use bevy::prelude::*;
use hexx::Hex;

use super::{Illuminance, TotalLight};

use std::fmt::Display;

/// The amount of shade on a tile.
#[derive(Component, Clone, Debug, Default)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Shade {
    /// This tile is not shaded.
    #[default]
    FullSun,
    /// This tile is partially shaded.
    PartialSun,
    /// This tile is fully shaded.
    FullShade,
}

impl Shade {
    /// Adds one level of shade to this tile.
    fn add_shade(&mut self) {
        *self = match self {
            Shade::FullSun => Shade::PartialSun,
            Shade::PartialSun => Shade::FullShade,
            Shade::FullShade => Shade::FullShade,
        };
    }

    /// Computes the amount of light recieved by a tile given the shade and total light.
    pub(crate) fn received_light(&self, total_light: &TotalLight) -> Illuminance {
        match (total_light.0, self) {
            (Illuminance::Dark, _) => Illuminance::Dark,
            (Illuminance::DimlyLit, Shade::FullSun) => Illuminance::DimlyLit,
            (Illuminance::DimlyLit, Shade::PartialSun) => Illuminance::Dark,
            (Illuminance::DimlyLit, Shade::FullShade) => Illuminance::Dark,
            (Illuminance::BrightlyLit, Shade::FullSun) => Illuminance::BrightlyLit,
            (Illuminance::BrightlyLit, Shade::PartialSun) => Illuminance::DimlyLit,
            (Illuminance::BrightlyLit, Shade::FullShade) => Illuminance::Dark,
        }
    }
}

impl Display for Shade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shade::FullSun => write!(f, "Full Sun"),
            Shade::PartialSun => write!(f, "Partial Sun"),
            Shade::FullShade => write!(f, "Full Shade"),
        }
    }
}

/// The amount of light currently received by a tile.
#[derive(Component, Clone, Debug, Default)]
pub(crate) struct ReceivedLight(pub(crate) Illuminance);

impl Display for ReceivedLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Computes the amount of shade on each tile.
pub(super) fn compute_shade(
    mut shade_query: Query<&mut Shade>,
    map_geometry: Res<MapGeometry>,
    in_game_time: Res<InGameTime>,
) {
    // PERF: we can be much less aggressive about computing these values
    // They only need to be recomputed when the map geometry changes, or when the time of day changes

    /// The unit vector pointing away from the sun.
    const SHADOW_DIRECTION: Hex = Hex { x: 0, y: 1 };

    // Reset the shade for all tiles
    for mut shade in shade_query.iter_mut() {
        *shade = Shade::FullSun;
    }

    if in_game_time.time_of_day() == TimeOfDay::Night {
        return;
    }

    for (voxel_pos, voxel_data) in map_geometry.all_voxels() {
        if !voxel_data.object_kind.blocks_light() {
            continue;
        }

        let mut i = 0;

        while DiscreteHeight(i) < voxel_pos.height {
            let current_height = voxel_pos.height - DiscreteHeight(i);
            let current_hex = SHADOW_DIRECTION * i as i32;

            let shaded_voxel = VoxelPos {
                hex: current_hex,
                height: current_height,
            };

            if let Some(voxel_data) = map_geometry.get_voxel(shaded_voxel) {
                let entity = voxel_data.entity;
                if let Ok(mut shade) = shade_query.get_mut(entity) {
                    shade.add_shade();
                }
            }
            i += 1;
        }
    }
}

/// Computes the amount of light received by each tile.
pub(super) fn compute_received_light(
    mut terrain_query: Query<(&mut ReceivedLight, &Shade)>,
    total_light: Res<TotalLight>,
) {
    // PERF: this can use change detection to be lazier about updates.
    for (mut received_light, shade) in terrain_query.iter_mut() {
        received_light.0 = shade.received_light(&total_light);
    }
}
