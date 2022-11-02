//! Units are organisms that can move freely.

use crate::curves::{BottomClampedLine, Mapping, Sigmoid};
use crate::organisms::pathfinding::get_weighted_random_passable_neighbor;
use crate::organisms::{OrganismBundle, OrganismType};
use crate::signals::emitters::{Emitter, StockEmitter};
use crate::signals::tile_signals::TileSignals;
use crate::terrain::generation::GenerationConfig;
use crate::terrain::ImpassableTerrain;
use crate::tiles::organisms::{OrganismStorage, OrganismStorageItem};
use crate::tiles::terrain::{TerrainStorage, TerrainStorageItem};
use crate::tiles::IntoTileBundle;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapId, TilemapSize};
use bevy_ecs_tilemap::prelude::TileBundle;
use bevy_ecs_tilemap::tiles::TilePos;

/// Marker component for [`UnitBundle`]
#[derive(Component, Clone, Default)]
pub struct Unit;

/// An organism that can move around freely.
#[derive(Bundle, Default)]
pub struct UnitBundle {
    /// Marker component.
    unit: Unit,
    /// A unit is an organism.
    #[bundle]
    organism_bundle: OrganismBundle,
}

/// Marker component for worker ants
#[derive(Component, Clone, Default)]
pub struct Ant;

/// A worker ant
#[derive(Bundle, Default)]
pub struct AntBundle {
    /// Marker struct.
    ant: Ant,
    /// Ants are units.
    #[bundle]
    unit_bundle: UnitBundle,
    /// Data needed to visualize the ant.
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
            .insert_resource(PheromoneTransducer::<BottomClampedLine>::default())
            .add_system(act);
    }
}
/// Global timer that controls when units should act
struct UnitTimer(Timer);

/// System modelling ant behaviour.
#[allow(clippy::too_many_arguments)]
fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    generation_config: Res<GenerationConfig>,
    mut query: Query<(&Unit, &mut TilePos)>,
    impassable_query: Query<&ImpassableTerrain>,
    terrain_storage_query: Query<TerrainStorage>,
    organism_storage_query: Query<OrganismStorage>,
    tile_signals_query: Query<&TileSignals>,
    pheromone_sensor: Res<PheromoneTransducer<BottomClampedLine>>,
) {
    let terrain_tile_storage = terrain_storage_query.single();
    let organism_tile_storage = organism_storage_query.single();
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in query.iter_mut() {
            *position = wander(
                &position,
                &terrain_tile_storage,
                &organism_tile_storage,
                &impassable_query,
                &tile_signals_query,
                &pheromone_sensor,
                &generation_config.map_size,
            );
        }
    }
}

/// Transduces a pheromone signal into a weight used to make decisions.
///
/// The transduction is modelled by mapping the signal to a weight using a curve.
pub struct PheromoneTransducer<C: Mapping> {
    /// Curve used to model transduction.
    curve: C,
}

impl PheromoneTransducer<Sigmoid> {
    /// Creates a [`Sigmoid`]-based transducer.
    pub fn new(
        min: f32,
        max: f32,
        first_percentile: f32,
        last_percentile: f32,
    ) -> PheromoneTransducer<Sigmoid> {
        PheromoneTransducer {
            curve: Sigmoid::new(min, max, first_percentile, last_percentile),
        }
    }

    /// Transduce a signal into a weight used for decision making.
    pub fn signal_to_weight(&self, attraction: f32, repulsion: f32) -> f32 {
        1.0 + self.curve.map(attraction) - self.curve.map(repulsion)
    }
}

impl Default for PheromoneTransducer<Sigmoid> {
    fn default() -> Self {
        PheromoneTransducer {
            curve: Sigmoid::new(0.0, 0.1, 0.01, 0.09),
        }
    }
}

impl PheromoneTransducer<BottomClampedLine> {
    /// Creates a [`BottomClampedLine`]-based transducer.
    pub fn new(p0: Vec2, p1: Vec2) -> PheromoneTransducer<BottomClampedLine> {
        PheromoneTransducer {
            curve: BottomClampedLine::new_from_points(p0, p1),
        }
    }

    /// Transduce a signal into a weight used for decision making.
    pub fn signal_to_weight(&self, attraction: f32, repulsion: f32) -> f32 {
        info!("attraction: {attraction:?}");
        1.0 + self.curve.map(attraction) - self.curve.map(repulsion)
    }
}

impl Default for PheromoneTransducer<BottomClampedLine> {
    fn default() -> Self {
        PheromoneTransducer {
            curve: BottomClampedLine::new_from_points(Vec2::new(0.0, 0.0), Vec2::new(0.01, 1.0)),
        }
    }
}

/// Pathfinding for ants.
fn wander(
    position: &TilePos,
    terrain_tile_storage: &TerrainStorageItem,
    organism_tile_storage: &OrganismStorageItem,
    impassable_query: &Query<&ImpassableTerrain>,
    tile_signals_query: &Query<&TileSignals>,
    pheromone_sensor: &PheromoneTransducer<BottomClampedLine>,
    map_size: &TilemapSize,
) -> TilePos {
    let signals_to_weight = |tile_signals: &TileSignals| {
        let weight = pheromone_sensor.signal_to_weight(
            tile_signals.get(&Emitter::Stock(StockEmitter::PheromoneAttract)),
            0.0,
        );
        info!("calculated weight: {weight:?}");
        weight
    };
    let target = get_weighted_random_passable_neighbor(
        position,
        terrain_tile_storage,
        organism_tile_storage,
        impassable_query,
        tile_signals_query,
        signals_to_weight,
        map_size,
    );

    target.unwrap_or(*position)
}
