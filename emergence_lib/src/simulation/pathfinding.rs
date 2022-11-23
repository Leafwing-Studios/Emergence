//! Various odds and ends useful for pathfinding

use crate::simulation::map::filters::MapFilter;
use crate::simulation::map::MapPositions;
use bevy::prelude::{Component, Deref, DerefMut, Query, Resource, With};
use bevy_ecs_tilemap::tiles::TilePos;

/// Marker struct specifying that an entity is impassable for pathfinding
#[derive(Component, Clone, Copy, Default)]
pub struct Impassable;

/// Caches:
/// * data for whether a given position is passable
/// * filters (of type [`HexNeighbors<bool>`](crate::simulation::map::hex_patch::HexPatch)) for each position
#[derive(Resource, Deref, DerefMut)]
pub struct PassableFilters {
    /// The [`MapFilter`] inner type
    pub inner: MapFilter,
}

impl PassableFilters {
    /// Create from a [`Impassable`] query
    pub fn from_impassable_query(
        query: &Query<&TilePos, With<Impassable>>,
        template: &MapPositions,
    ) -> PassableFilters {
        PassableFilters {
            inner: MapFilter::new_with_default(
                true,
                template,
                query.iter().map(|pos| (*pos, false)),
            ),
        }
    }
}
