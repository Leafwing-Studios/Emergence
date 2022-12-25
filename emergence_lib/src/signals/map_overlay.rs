//! Plugin for displaying signals as coloured overlays on the game map.

use crate::curves::linear_combination;
use crate::signals::configs::SignalConfigs;
use crate::signals::tile_signals::TileSignals;
use crate::simulation::map::resources::MapResource;
use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TileColor, TilePos};

/// Colours tiles based on the signals present. Signal colours are defined in their
/// [`SignalConfig`](crate::signals::configs::SignalConfig).
pub struct MapOverlayPlugin;

impl Plugin for MapOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, color_tiles);
    }
}

/// We cannot directly use [`Color::WHITE`] cannot be used, as it has the RGB variant, not the
/// RGBA variant
pub const RGBA_WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

/// Color tiles based on the signals present.
fn color_tiles(
    mut commands: Commands,
    terrain_tile_query: Query<(Entity, &TilePos)>,
    map_signals: Res<MapResource<TileSignals>>,
    signal_configs: Res<SignalConfigs>,
) {
    let tile_colors: Vec<(Entity, TileColor)> = terrain_tile_query
        .iter()
        .map(|(entity, position)| {
            let tile_signals = map_signals.get(position).unwrap();
            let tile_color = TileColor(tile_signals.read().compute_combined_color(&signal_configs));
            (entity, tile_color)
        })
        .collect();
    commands.insert_or_spawn_batch(tile_colors);
}

/// A colour that can be [alpha composed](https://en.wikipedia.org/wiki/Alpha_compositing).
///
/// Currently we only implement the `over` operation.
pub trait AlphaCompose {
    /// Porter and Duff ["over" operation](https://en.wikipedia.org/wiki/Alpha_compositing) for
    /// blending two colours.
    ///
    /// `self` is blended over `other`.
    fn over(&self, other: &Self) -> Self;
}

impl AlphaCompose for Color {
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
