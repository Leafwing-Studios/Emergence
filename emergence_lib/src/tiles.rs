use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileTexture};

pub trait IntoTile {
    fn tile_texture(&self) -> TileTexture;

    fn tile_texture_path(&self) -> &'static str;

    fn as_tile_bundle(&self, tilemap_id: TilemapId, position: TilePos) -> TileBundle {
        TileBundle {
            position,
            tilemap_id,
            texture: self.tile_texture(),
            ..Default::default()
        }
    }
}
