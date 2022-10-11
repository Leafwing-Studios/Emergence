use crate::curves::linear_combination;
use crate::signals::configs::SignalConfigs;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TileColor;

pub struct MapOverlayPlugin;

impl Plugin for MapOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(color_tiles);
    }
}

// Color::WHITE cannot be used, as it has the RGB variant, not the RGBA variant
const RGBA_WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

/// Computes a [`TileColor`] from the given colors, by applying each color in order
/// [`over`](AlphaCompose::over) the baseline tile color [`RGBA_WHITE`].
fn compute_tile_color(colors: &[Color]) -> TileColor {
    let mut total_color = RGBA_WHITE;
    for color in colors {
        total_color = color.over(&total_color)
    }
    TileColor(total_color)
}

/// Color tiles based on the signals present.
fn color_tiles(signal_configs: Res<SignalConfigs>) {}

pub trait AlphaCompose {
    fn over(&self, other: &Self) -> Self;
}

impl AlphaCompose for Color {
    /// Porter and Duff ["over" operation](https://en.wikipedia.org/wiki/Alpha_compositing) for
    /// blending two colours.
    ///
    /// `self` is blended over `other`.
    fn over(&self, other: &Color) -> Color {
        match (*self, *other) {
            (
                Color::Rgba {
                    red: self_red,
                    green: self_green,
                    blue: self_blue,
                    alpha: self_alpha,
                },
                Color::Rgba {
                    red: other_red,
                    green: other_green,
                    blue: other_blue,
                    alpha: other_alpha,
                },
            ) => {
                let alpha = linear_combination(1.0, other_alpha, self_alpha);
                Color::Rgba {
                    red: linear_combination(self_red, other_red * other_alpha, self_alpha) / alpha,
                    green: linear_combination(self_green, other_green * other_alpha, self_alpha)
                        / alpha,
                    blue: linear_combination(self_blue, other_blue * other_alpha, self_alpha)
                        / alpha,
                    alpha,
                }
            }
            _ => unimplemented!(),
        }
    }
}
