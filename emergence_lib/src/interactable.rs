//! Exhaustively enumerates all of the types of objects that can be interacted with.
//!
//! Used for signalling, unit behaviors and more.

use crate::tiles::{IntoTileBundle, LayerType};

use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::tiles::TileTextureIndex;

use bevy::ecs::component::Component;
use bevy::utils::HashMap;

/// An object that can be interacted with by units
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Interactable {
    /// Plant
    Plant,
    /// Fungus
    Fungus,
    /// Ant
    Ant,
}

impl Interactable {
    /// Get the tilemap this Interactable's tile is in
    pub const fn tilemap_type(&self) -> LayerType {
        use Interactable::*;

        match self {
            Plant | Fungus | Ant => LayerType::Organism,
        }
    }

    /// Get the tilemap id of the tilemap this Interactable's tile is in
    pub fn tilemap_id(&self, tilemap_ids: &HashMap<LayerType, TilemapId>) -> TilemapId {
        *tilemap_ids.get(&self.tilemap_type()).unwrap()
    }
}

impl IntoTileBundle for Interactable {
    fn tile_texture(
        &self,
        tilemap_ids: &HashMap<LayerType, TilemapId>,
    ) -> (TilemapId, TileTextureIndex) {
        todo!()
    }

    fn tile_texture_path(&self) -> &'static str {
        use Interactable::*;

        match self {
            Plant => "tile-plant.png",
            Fungus => "tile-fungus.png",
            Ant => "tile-ant.png",
        }
    }
}
