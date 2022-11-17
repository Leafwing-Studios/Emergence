//! The `IntoSprite` trait allows an entity to specify how it should be visualized using
//! `bevy_ecs_tilemap`.
//!
//! This specification requires three pieces of information from the user:
//! * the [`Tilemap`] that the entity's sprite belongs to ([`ROOT_PATH`](IntoSprite::ROOT_PATH))
//! * the path of the assets used as sprites (given in the implementation of [`leaf_path`](IntoSprite::leaf_path))

use crate::enum_iter::IterableEnum;
use crate::graphics::{Tilemap, TilemapRegister};
use bevy::asset::{AssetPath, AssetServer};
use bevy::prelude::Res;
use bevy_ecs_tilemap::map::TilemapTexture;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileTextureIndex};
use std::path::PathBuf;

pub trait SpriteIndex: IterableEnum {
    /// Path to the folder containing sprite assets indexed by this implementor.
    const ROOT_PATH: &'static str;

    /// Path of a particular entity variant.
    fn leaf_path(&self) -> &'static str;

    /// Returns `ROOT_PATH + leaf_path()`.
    fn full_path(&self) -> AssetPath<'static> {
        let path = PathBuf::from(Self::ROOT_PATH).join(self.leaf_path());

        AssetPath::new(path, None)
    }

    /// Returns all the sprite paths in `ROOT_PATH`
    fn all_paths() -> Vec<AssetPath<'static>> {
        Self::variants()
            .map(|variant| variant.full_path())
            .collect()
    }

    /// Loads associated sprites into a [`TilemapTexture::Vector`](TilemapTexture::Vector).
    fn load(asset_server: &Res<AssetServer>) -> TilemapTexture {
        TilemapTexture::Vector(
            Self::all_paths()
                .into_iter()
                .map(|p| asset_server.load(p))
                .collect(),
        )
    }
}

/// Enumerates the sprite assets needed for a particular [`Tilemap`] variant.
#[bevy_trait_query::queryable]
pub trait IntoSprite {
    /// Return the tilemap this sprite belongs to
    fn tilemap(&self) -> Tilemap;

    /// Return the sprite's index, which is the index marking it against the other sprites in the
    /// same tilemap.
    ///
    /// See the [`SpriteIndex`] trait and the structs that implement it for easy ways to refer to
    /// a particular sprite's index.
    fn index(&self) -> u32;

    /// Returns the sprite's index as a [`TileTextureIndex`].
    fn texture_index(&self) -> TileTextureIndex {
        TileTextureIndex(self.index())
    }

    /// Creates a [`TileBundle`] for an entity of this type, which can be used to initialize it in [`bevy_ecs_tilemap`].
    fn tile_bundle(
        &self,
        position: TilePos,
        tilemap_register: &Res<TilemapRegister>,
    ) -> TileBundle {
        let tilemap = self.tilemap();
        TileBundle {
            position,
            texture_index: self.texture_index(),
            tilemap_id: *tilemap_register
                .register
                .get(tilemap.index())
                .unwrap_or_else(|| panic!("Layer {:?} not registered", tilemap)),
            ..Default::default()
        }
    }
}
