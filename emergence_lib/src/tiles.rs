use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileTexture};

/// A type that can be transformed into a tile that is compatible with [`bevy_ecs_tilemap`].
pub trait IntoTile {
    /// The corresponding [`TileTexture`].
    fn tile_texture(&self) -> TileTexture;

    /// The asset path to the [`TileTexture`].
    fn tile_texture_path(&self) -> &'static str;

    /// Uses the data stored in `self` to create a new, matching [`TileBundle`].
    fn as_tile_bundle(&self, tilemap_id: TilemapId, position: TilePos) -> TileBundle {
        TileBundle {
            position,
            tilemap_id,
            texture: self.tile_texture(),
            ..Default::default()
        }
    }
}
