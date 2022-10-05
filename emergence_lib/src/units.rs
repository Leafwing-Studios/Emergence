use crate::generation::{OrganismTilemap, TerrainTilemap};
use crate::organisms::{OrganismBundle, OrganismType};
use crate::pathfinding::get_random_passable_neighbor;
use crate::signals::SignalId;
use crate::terrain::ImpassableTerrain;
use crate::tiles::IntoTile;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::TileBundle;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

#[derive(Component, Clone, Default)]
pub struct Unit;

#[derive(Bundle, Default)]
pub struct UnitBundle {
    unit: Unit,
    #[bundle]
    organism_bundle: OrganismBundle,
}

#[derive(Component, Clone, Default)]
pub struct Ant;

#[derive(Bundle, Default)]
pub struct AntBundle {
    ant: Ant,
    #[bundle]
    unit_bundle: UnitBundle,
    #[bundle]
    tile_bundle: TileBundle,
}

impl AntBundle {
    pub fn new(tilemap_id: TilemapId, position: TilePos) -> Self {
        Self {
            unit_bundle: UnitBundle {
                organism_bundle: OrganismBundle {
                    signal_id: SignalId::Ant,
                    ..Default::default()
                },
                ..Default::default()
            },
            tile_bundle: OrganismType::Ant.as_tile_bundle(tilemap_id, position),
            ..Default::default()
        }
    }
}

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UnitTimer(Timer::from_seconds(0.5, true)))
            .add_system(act);
    }
}
/// Global timer that controls when units should act
struct UnitTimer(Timer);

fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    mut query: Query<(&Unit, &mut TilePos)>,
    impassable_query: Query<&ImpassableTerrain>,
    terrain_tilemap_query: Query<&TileStorage, With<TerrainTilemap>>,
    organism_tilemap_query: Query<&TileStorage, With<OrganismTilemap>>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in query.iter_mut() {
            *position = wander(
                &position,
                &impassable_query,
                &terrain_tilemap_query,
                &organism_tilemap_query,
            );
        }
    }
}

fn wander(
    position: &TilePos,
    impassable_query: &Query<&ImpassableTerrain>,
    terrain_tilemap_query: &Query<&TileStorage, With<TerrainTilemap>>,
    organism_tilemap_query: &Query<&TileStorage, With<OrganismTilemap>>,
) -> TilePos {
    let target = get_random_passable_neighbor(
        position,
        impassable_query,
        terrain_tilemap_query,
        organism_tilemap_query,
    );

    target.unwrap_or(*position)
}
