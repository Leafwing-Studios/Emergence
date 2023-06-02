//! Vegetative reproduction is the spread of organisms (typically plants) via roots and shoots.
//!
//! In Emergence, this allows organisms to spread to nearby tiles without seeds.
use bevy::prelude::*;
use leafwing_abilities::prelude::Pool;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

use crate::{
    asset_management::manifest::Id,
    geometry::{Facing, MapGeometry, VoxelPos},
    player_interaction::clipboard::ClipboardData,
    structures::{
        commands::StructureCommandsExt,
        structure_manifest::{Structure, StructureManifest},
    },
};

use super::energy::{Energy, EnergyPool, StartingEnergy};

/// A component that allows an organism to spread to nearby tiles.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct VegetativeReproduction {
    /// The minimum time remaining until this organism can spread again.
    timer: Timer,
    /// The minimum energy required to reproduce.
    ///
    /// Energy is split between the parent and child organisms.
    energy_threshold: Energy,
}

impl Display for VegetativeReproduction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.1}/{:.1} s ({} energy)",
            self.timer.elapsed().as_secs_f32(),
            self.timer.duration().as_secs_f32(),
            self.energy_threshold.0,
        )
    }
}

/// The unprocessed equivalent of [`VegetativeReproduction`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawVegetativeReproduction {
    /// The minimum time between each spread, measured in seconds.
    pub period: f32,
    /// The minimum energy required to reproduce.
    ///
    /// Energy is split between the parent and child organisms.
    pub energy_threshold: f32,
}

impl From<RawVegetativeReproduction> for VegetativeReproduction {
    fn from(raw: RawVegetativeReproduction) -> Self {
        VegetativeReproduction {
            timer: Timer::from_seconds(raw.period, TimerMode::Once),
            energy_threshold: Energy(raw.energy_threshold),
        }
    }
}

/// Spreads organisms to nearby tiles.
pub(super) fn vegetative_spread(
    mut query: Query<(
        &VoxelPos,
        &Id<Structure>,
        &mut VegetativeReproduction,
        &mut EnergyPool,
    )>,
    map_geometry: Res<MapGeometry>,
    structure_manifest: Res<StructureManifest>,
    fixed_time: Res<FixedTime>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    let delta_time = fixed_time.period;

    for (&voxel_pos, &structure_id, mut vegetative_reproduction, mut energy_pool) in
        query.iter_mut()
    {
        vegetative_reproduction.timer.tick(delta_time);
        if !vegetative_reproduction.timer.finished() {
            continue;
        }

        let current_energy = energy_pool.current();
        if current_energy < vegetative_reproduction.energy_threshold {
            continue;
        }

        // PERF: we should just be returning a Vec<VoxelPos> or an [Option<VoxelPos; 6] here and allocating once
        let empty_neighbors = map_geometry
            .valid_neighbors(voxel_pos)
            .iter()
            .filter(|maybe_pos| match maybe_pos {
                Some(pos) => map_geometry.get_voxel_object(*pos).is_none(),
                None => false,
            });
        let Some(&tile_to_spawn_in) = empty_neighbors
            .flatten()
			// Just skip this organism if there are no empty neighbors
            .choose(&mut rng) else { continue };

        let clipboard_data = ClipboardData {
            structure_id,
            facing: Facing::random(&mut rng),
            active_recipe: structure_manifest
                .get(structure_id)
                .starting_recipe()
                .clone(),
        };

        // Split the energy between the parent and child organisms
        let half_current = current_energy / 2.;
        energy_pool.set_current(half_current);

        commands.spawn_structure(
            tile_to_spawn_in,
            clipboard_data,
            StartingEnergy::Specific(half_current),
        );

        // Reset the timer once we've successfully spawned a new organism
        vegetative_reproduction.timer.reset();

        // Pay the energy cost
    }
}
