//! Code for managing boolean (true/false) data that is deeply tied to the map

use crate::simulation::map::resources::MapResource;
use crate::simulation::map::MapPositions;
use bevy_ecs_tilemap::tiles::TilePos;

/// Boolean valued [`MapResource`]s are useful as filters.
pub type MapFilter = MapResource<bool>;

impl MapFilter {
    /// Create new from an underlying [`MapPositions`] template.
    ///
    /// This allocates capacity and initializes patches based on the [`MapPositions`] template
    /// provided, and the specified default value for data.
    pub fn new_with_default(
        default: bool,
        template: &MapPositions,
        data: impl Iterator<Item = (TilePos, bool)>,
    ) -> MapFilter {
        let mut storage = MapResource::generate_storage(
            template,
            template
                .iter_positions()
                .map(|position| (*position, default)),
        );

        let patches = MapResource::generate_patches(&storage, template);

        for (position, value) in data {
            if let Some(tile_value) = storage.get_mut(&position) {
                *(tile_value.get_mut()) = value;
            }
        }

        MapResource { storage, patches }
    }
}
