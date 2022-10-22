//! Units are organisms that can move freely.

use crate::curves::Sigmoid;
use crate::organisms::pathfinding::get_weighted_random_passable_neighbor;
use crate::organisms::{OrganismBundle, OrganismType};
use crate::signals::emitters::{Emitter, StockEmitter};
use crate::signals::tile_signals::TileSignals;
use crate::terrain::generation::{GenerationConfig, OrganismTilemap, TerrainTilemap};
use crate::terrain::ImpassableTerrain;
use crate::tiles::IntoTile;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapId, TilemapSize};
use bevy_ecs_tilemap::prelude::TileBundle;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

/// Marker component for [`UnitBundle`]
#[derive(Component, Clone, Default)]
pub struct Unit;

/// An organism that can move around freely.
#[derive(Bundle, Default)]
pub struct UnitBundle {
    unit: Unit,
    #[bundle]
    organism_bundle: OrganismBundle,
}

/// Marker component for worker ants
#[derive(Component, Clone, Default)]
pub struct Ant;

/// A worker ant
#[derive(Bundle, Default)]
pub struct AntBundle {
    ant: Ant,
    #[bundle]
    unit_bundle: UnitBundle,
    #[bundle]
    tile_bundle: TileBundle,
}

impl AntBundle {
    /// Creates a new [`AntBundle`]
    pub fn new(tilemap_id: TilemapId, position: TilePos) -> Self {
        Self {
            unit_bundle: UnitBundle {
                organism_bundle: OrganismBundle {
                    ..Default::default()
                },
                ..Default::default()
            },
            tile_bundle: OrganismType::Ant.as_tile_bundle(tilemap_id, position),
            ..Default::default()
        }
    }
}

/// Contains unit behavior
pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UnitTimer(Timer::from_seconds(0.5, true)))
            .insert_resource(PheromoneSensor::new())
            .add_system(act);
    }
}
/// Global timer that controls when units should act
struct UnitTimer(Timer);

#[allow(clippy::too_many_arguments)]
fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    generation_config: Res<GenerationConfig>,
    mut query: Query<(&Unit, &mut TilePos)>,
    impassable_query: Query<&ImpassableTerrain>,
    terrain_tilemap_query: Query<&TileStorage, With<TerrainTilemap>>,
    organism_tilemap_query: Query<&TileStorage, With<OrganismTilemap>>,
    tile_signals_query: Query<&TileSignals>,
    pheromone_sensor: Res<PheromoneSensor>,
) {
    let terrain_tile_storage = terrain_tilemap_query.single();
    let organism_tile_storage = organism_tilemap_query.single();
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in query.iter_mut() {
            *position = wander(
                &position,
                terrain_tile_storage,
                organism_tile_storage,
                &impassable_query,
                &tile_signals_query,
                &pheromone_sensor,
                &generation_config.map_size,
            );
        }
    }
}

pub struct PheromoneSensor {
    sigmoid: Sigmoid,
}

impl PheromoneSensor {
    pub fn new() -> PheromoneSensor {
        PheromoneSensor {
            sigmoid: Sigmoid::new(0.0, 1.0, 0.2, 0.99),
        }
    }

    pub fn signal_to_weight(&self, attraction: f32, repulsion: f32) -> f32 {
        self.sigmoid.map(attraction) - self.sigmoid.map(repulsion)
    }
}

fn wander(
    position: &TilePos,
    terrain_tile_storage: &TileStorage,
    organism_tile_storage: &TileStorage,
    impassable_query: &Query<&ImpassableTerrain>,
    tile_signals_query: &Query<&TileSignals>,
    pheromone_sensor: &PheromoneSensor,
    map_size: &TilemapSize,
) -> TilePos {
    // let target = get_random_passable_neighbor(
    //     position,
    //     organism_tile_storage,
    //     terrain_tile_storage,
    //     impassable_query,
    //     map_size,
    // );

    let signals_to_weight = |tile_signals: &TileSignals| {
        1.0 + pheromone_sensor.signal_to_weight(
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneAttract)),
            0.0,
        )
    };
    let target = get_weighted_random_passable_neighbor(
        position,
        organism_tile_storage,
        terrain_tile_storage,
        impassable_query,
        tile_signals_query,
        signals_to_weight,
        map_size,
    );

    target.unwrap_or(*position)
}
