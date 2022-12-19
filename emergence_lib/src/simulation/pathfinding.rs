//! Various odds and ends useful for pathfinding
use crate::simulation::map::filters::MapFilter;
use crate::simulation::map::hex_patch::HexPatch;
use crate::simulation::map::resources::MapData;
use crate::simulation::map::MapPositions;
use bevy::prelude::{Changed, Component, Query, Resource, With, Without};
use bevy::utils::HashSet;
use bevy_ecs_tilemap::tiles::TilePos;

/// Marker struct specifying that an entity is impassable for pathfinding
#[derive(Component, Clone, Copy, Default)]
pub struct Impassable;

/// Caches:
/// * `bool` indicating whether a given position is passable
/// * [`HexPatch<bool>`](HexPatch) indicating whether positions
/// in hex patch are passable for each position
#[derive(Resource)]
pub struct PassabilityCache {
    /// The [`MapFilter`] inner type which caches whether a given position is passable, and its
    /// corresponding hex patch
    inner: MapFilter,
    /// Tile positions that were marked as impassable in the last
    /// [`update_from_impassable_query`](PassabilityCache::update_from_impassable_query) operation
    previously_impassable: HashSet<TilePos>,
}

impl PassabilityCache {
    /// Creates new [`PassabilityCache`]
    pub fn new(template: &MapPositions) -> PassabilityCache {
        PassabilityCache {
            inner: MapFilter::new_with_default(true, template, [].into_iter()),
            previously_impassable: HashSet::new(),
        }
    }

    /// Get neighbors associated with a given tile position, if it exists in the cache
    pub fn get_patch(&self, tile_pos: &TilePos) -> Option<&HexPatch<MapData<bool>>> {
        self.inner.get_patch(tile_pos)
    }

    /// Update from an [`Impassable`] query
    pub fn update_from_impassable_query(&mut self, impassable: &Query<&TilePos, With<Impassable>>) {
        let currently_impassable = HashSet::from_iter(impassable.iter().copied());

        let update_to_passable = self.previously_impassable.difference(&currently_impassable);
        let update_to_impassable = currently_impassable.difference(&self.previously_impassable);

        self.inner
            .update(update_to_passable.map(|position| (*position, true)));
        self.inner
            .update(update_to_impassable.map(|position| (*position, false)));

        self.previously_impassable = currently_impassable;
    }

    /// Update from newly impassable and newly passable queries
    pub fn update_from_changed_passable_queries(
        &mut self,
        newly_impassable: &Query<&TilePos, (With<Impassable>, Changed<Impassable>)>,
        newly_passable: &Query<&TilePos, (Without<Impassable>, Changed<Impassable>)>,
    ) {
        let newly_impassable = HashSet::from_iter(newly_impassable.iter().copied());
        let newly_passable = HashSet::from_iter(newly_passable.iter().copied());

        self.previously_impassable = self
            .previously_impassable
            .iter()
            .filter(|position| !newly_passable.contains(position))
            .chain(newly_impassable.iter())
            .copied()
            .collect::<HashSet<TilePos>>();
    }
}
