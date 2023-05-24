//! Vegetative reproduction is the spread of organisms (typically plants) via roots and shoots.
//!
//! In Emergence, this allows organisms to spread to nearby tiles without seeds.
use bevy::prelude::*;
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

/// A component that allows an organism to spread to nearby tiles.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct VegetativeReproduction {
    /// The time remaining until this organism can spread again.
    timer: Timer,
}

/// The unprocessed equivalent of [`VegetativeReproduction`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawVegetativeReproduction {
    /// The time between each spread, measured in seconds.
    pub period: f32,
}

impl From<RawVegetativeReproduction> for VegetativeReproduction {
    fn from(raw: RawVegetativeReproduction) -> Self {
        VegetativeReproduction {
            timer: Timer::from_seconds(raw.period, TimerMode::Repeating),
        }
    }
}

/// Spreads organisms to nearby tiles.
pub(super) fn vegetative_spread(
    mut query: Query<(&TilePos, &Id<Structure>, &mut VegetativeReproduction)>,
    map_geometry: Res<MapGeometry>,
    structure_manifest: Res<StructureManifest>,
    fixed_time: Res<FixedTime>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    let delta_time = fixed_time.period;

    for (tile_pos, &structure_id, mut vegetative_reproduction) in query.iter_mut() {
        vegetative_reproduction.timer.tick(delta_time);
        if !vegetative_reproduction.timer.finished() {
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
    }
}
