use crate::config::MAP_SIZE;
use crate::terrain::ImpassableTerrain;
use bevy::prelude::*;
use bevy_ecs_tilemap::helpers::hex_grid::axial::AxialPos;
use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexDirection;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use rand::distributions::Distribution;
use rand::seq::SliceRandom;
use rand::Rng;

/// Generates a random hexagonal direction using the `rng` and `distribution` provided.
#[allow(unused)]
fn random_direction<R: Rng + ?Sized, D: Distribution<usize>>(
    mut rng: &mut R,
    distribution: D,
) -> HexDirection {
    let choice = distribution.sample(&mut rng);
    HexDirection::from(choice)
}

pub struct HexNeighborPositions {
    north_west: Option<TilePos>,
    west: Option<TilePos>,
    south_west: Option<TilePos>,
    south_east: Option<TilePos>,
    east: Option<TilePos>,
    north_east: Option<TilePos>,
}

impl HexNeighborPositions {
    pub fn get_passable_neighbors(
        tile_pos: &TilePos,
        terrain_tile_storage: &TileStorage,
        organism_tile_storage: &TileStorage,
        impassable_query: &Query<&ImpassableTerrain>,
    ) -> HexNeighborPositions {
        use bevy_ecs_tilemap::helpers::hex_grid::neighbors::HexRowDirection::*;
        let axial_pos = AxialPos::from(tile_pos);
        let predicate = |pos| {
            if let Some(terrain_entity) = terrain_tile_storage.get(&pos) {
                if impassable_query.get(terrain_entity).is_err() {
                    if let Some(organism_entity) = organism_tile_storage.get(&pos) {
                        if impassable_query.get(organism_entity).is_err() {
                            Some(pos)
                        } else {
                            None
                        }
                    } else {
                        Some(pos)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        HexNeighborPositions {
            north_west: axial_pos
                .offset_compass_row(NorthWest)
                .as_tile_pos_given_map_size(&MAP_SIZE)
                .and_then(predicate),
            west: axial_pos
                .offset_compass_row(West)
                .as_tile_pos_given_map_size(&MAP_SIZE)
                .and_then(predicate),
            south_west: axial_pos
                .offset_compass_row(SouthWest)
                .as_tile_pos_given_map_size(&MAP_SIZE)
                .and_then(predicate),
            south_east: axial_pos
                .offset_compass_row(SouthEast)
                .as_tile_pos_given_map_size(&MAP_SIZE)
                .and_then(predicate),
            east: axial_pos
                .offset_compass_row(East)
                .as_tile_pos_given_map_size(&MAP_SIZE)
                .and_then(predicate),
            north_east: axial_pos
                .offset_compass_row(NorthEast)
                .as_tile_pos_given_map_size(&MAP_SIZE)
                .and_then(predicate),
        }
    }

    pub fn choose_random<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<TilePos> {
        let possible_choices = [
            self.north_west,
            self.west,
            self.south_west,
            self.south_east,
            self.east,
            self.north_east,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<TilePos>>();

        possible_choices.choose(rng).cloned()
    }
}
