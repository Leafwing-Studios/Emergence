//! Mathematical noise functions that produce steadily changing pseudo-random values.

use noisy_bevy::fbm_simplex_2d_seeded;

use crate::simulation::geometry::{Height, TilePos};
use bevy::math::Vec2;

/// A settings struct for [`simplex_noise`].
#[derive(Debug, Clone)]
pub struct SimplexSettings {
    /// Controls the size of the features in the noise function.
    ///
    /// Higher values mean smaller features.
    pub frequency: f32,
    /// Controls the vertical scale of the noise function.
    ///
    /// Higher values mean deeper valleys and higher mountains.
    pub amplitude: f32,
    /// How many times will the fbm be sampled?
    pub octaves: usize,
    /// Controls the smoothness of the noise.
    ///
    /// Lower values are smoother.
    pub lacunarity: f32,
    /// Scale the output of the fbm function
    pub gain: f32,
    /// Arbitary seed that determines the noise function output
    pub seed: f32,
}

/// Computes the value of the noise function at a given position.
///
/// This can then be used to determine the height of a tile.
pub fn simplex_noise(tile_pos: TilePos, settings: &SimplexSettings) -> f32 {
    let SimplexSettings {
        frequency,
        amplitude,
        octaves,
        lacunarity,
        gain,
        seed,
    } = *settings;

    let pos = Vec2::new(tile_pos.hex.x as f32, tile_pos.hex.y as f32);

    Height::MIN.into_world_pos()
        + (fbm_simplex_2d_seeded(pos * frequency, octaves, lacunarity, gain, seed) * amplitude)
            .abs()
}
