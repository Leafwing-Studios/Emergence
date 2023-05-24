//! Vegetative reproduction is the spread of organisms (typically plants) via roots and shoots.
//!
//! In Emergence, this allows organisms to spread to nearby tiles without seeds.
use bevy::prelude::*;
use leafwing_abilities::prelude::Pool;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{
    asset_management::manifest::Id,
    player_interaction::clipboard::ClipboardData,
    simulation::geometry::{Facing, MapGeometry, TilePos},
    structures::{
        commands::StructureCommandsExt,
        structure_manifest::{Structure, StructureManifest},
    },
};

use super::energy::{Energy, EnergyPool};

/// A component that allows an organism to spread to nearby tiles.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct VegetativeReproduction {
    /// The minimum time remaining until this organism can spread again.
    timer: Timer,
    /// The energy cost to reproduce.
    energy_cost: Energy,
}

/// The unprocessed equivalent of [`VegetativeReproduction`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawVegetativeReproduction {
    /// The minimum time between each spread, measured in seconds.
    pub period: f32,
    /// The energy cost to reproduce.
    pub energy_cost: f32,
}

impl From<RawVegetativeReproduction> for VegetativeReproduction {
    fn from(raw: RawVegetativeReproduction) -> Self {
        VegetativeReproduction {
            timer: Timer::from_seconds(raw.period, TimerMode::Once),
            energy_cost: Energy(raw.energy_cost),
        }
    }
}

/// Spreads organisms to nearby tiles.
pub(super) fn vegetative_spread(
    mut query: Query<(
        &TilePos,
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

    for (tile_pos, &structure_id, mut vegetative_reproduction, mut energy_pool) in query.iter_mut()
    {
        vegetative_reproduction.timer.tick(delta_time);
        if !vegetative_reproduction.timer.finished() {
            continue;
        }

        let current_energy = energy_pool.current();
        if current_energy < vegetative_reproduction.energy_cost {
            continue;
        }

        // PERF: we should just be returning a Vec<TilePos> or an [Option<TilePos; 6] here and allocating once
        let empty_neighbors = tile_pos.empty_neighbors(&map_geometry);
        let Some(&tile_to_spawn_in) = empty_neighbors
            .into_iter()
            .collect::<Vec<TilePos>>()
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

        commands.spawn_structure(tile_to_spawn_in, clipboard_data);

        // Reset the timer once we've successfully spawned a new organism
        vegetative_reproduction.timer.reset();

        // Pay the energy cost
        energy_pool.set_current(current_energy - vegetative_reproduction.energy_cost);
    }
}
