use bevy::app::CoreStage::PreUpdate;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct DiffusionPlugin;

const PROPAGATE: &'static str = "propagate_signal";
const UPDATE: &'static str = "update_signal";

impl Plugin for DiffusionPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(PreUpdate, PROPAGATE, SystemStage::parallel())
            .add_stage_after(PROPAGATE, UPDATE, SystemStage::parallel())
            .add_startup_system_to_stage(StartupStage::PostStartup, initialize_signal)
            .add_system_to_stage(PROPAGATE, propagate_signal)
            .add_system_to_stage(UPDATE, update_signal);
    }
}

#[derive(Component, Copy, Clone, Default)]
struct Signal(f32);

#[derive(Component, Copy, Clone, Default)]
struct IncomingSignal(f32);

impl IncomingSignal {
    fn add(&mut self, v: f32) {
        self.0 += v;
    }
}

#[derive(Component, Copy, Clone, Default)]
struct OutgoingSignal(f32);

impl OutgoingSignal {
    fn add(&mut self, v: f32) {
        self.0 += v;
    }
}

impl Signal {
    fn apply(&mut self, incoming: &IncomingSignal, outgoing: &OutgoingSignal) {
        self.0 += incoming.0 - outgoing.0;
    }
}

trait AlphaCompose {
    fn over(&self, other: &Self) -> Self;
}

impl AlphaCompose for Color {
    /// Porter and Duff ["over" operation](https://en.wikipedia.org/wiki/Alpha_compositing) for blending two colours.
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
                let alpha = lin_comb(1.0, other_alpha, self_alpha);
                Color::Rgba {
                    red: lin_comb(self_red, other_red * other_alpha, self_alpha) / alpha,
                    green: lin_comb(self_green, other_green * other_alpha, self_alpha) / alpha,
                    blue: lin_comb(self_blue, other_blue * other_alpha, self_alpha) / alpha,
                    alpha,
                }
            }
            _ => unimplemented!(),
        }
    }
}

/// Take a linear combination of `x` and `y`, by the factor `c`.
///
/// Formally: `lin_comb(x, y, c) = x * c + y * (1.0 - c)`.
fn lin_comb(x: f32, y: f32, c: f32) -> f32 {
    x * c + y * (1.0 - c)
}

impl From<Signal> for Color {
    fn from(signal: Signal) -> Self {
        Color::Rgba {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: (signal.0 * 3.0).clamp(0.0, 1.0),
        }
    }
}

impl From<&Signal> for Color {
    fn from(signal: &Signal) -> Self {
        (*signal).into()
    }
}

const RGBA_WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

impl From<Signal> for TileColor {
    fn from(signal: Signal) -> Self {
        let signal_color: Color = signal.into();
        TileColor(signal_color.over(&RGBA_WHITE).into())
    }
}

impl From<&Signal> for TileColor {
    fn from(signal: &Signal) -> Self {
        let signal_color: Color = signal.into();
        TileColor(signal_color.over(&RGBA_WHITE).into())
    }
}

fn initialize_signal(
    mut commands: Commands,
    tilemap_storage_q: Query<&TileStorage>,
    tile_pos_q: Query<&TilePos>,
) {
    for tilemap_storage in tilemap_storage_q.iter() {
        for &tile_id in tilemap_storage.iter() {
            if let Some(tile_id) = tile_id {
                if let Ok(tile_pos) = tile_pos_q.get(tile_id) {
                    let signal = if tile_pos.x == 0 && tile_pos.y == 0 {
                        Signal(1.0)
                    } else {
                        Signal(0.0)
                    };
                    let tile_color: TileColor = signal.into();
                    commands
                        .entity(tile_id)
                        .insert(signal)
                        .insert(tile_color)
                        .insert(IncomingSignal::default())
                        .insert(OutgoingSignal::default());
                }
            }
        }
    }
}

fn propagate_signal(
    tilemap_q: Query<(&TileStorage, &TilemapType)>,
    mut tile_outgoing_q: Query<(&TilePos, &Signal, &mut OutgoingSignal)>,
    mut neighbor_incoming_q: Query<&mut IncomingSignal>,
) {
    for (tile_storage, map_type) in tilemap_q.iter() {
        for &tile_id in tile_storage.iter() {
            if let Some(tile_id) = tile_id {
                if let Ok((tile_pos, this_signal, mut this_outgoing)) =
                    tile_outgoing_q.get_mut(tile_id)
                {
                    let neighbors = get_tile_neighbors(tile_pos, tile_storage, map_type);
                    let mut total_outgoing = 0.0;
                    for neighbor_id in neighbors.into_iter() {
                        if let Ok(mut neighbor_incoming) = neighbor_incoming_q.get_mut(neighbor_id)
                        {
                            let outgoing = 0.001 * this_signal.0;
                            neighbor_incoming.add(outgoing);
                            total_outgoing += outgoing;
                        }
                    }
                    this_outgoing.add(total_outgoing);
                }
            }
        }
    }
}

fn update_signal(
    mut commands: Commands,
    tilemap_q: Query<&TileStorage>,
    mut tile_q: Query<(&mut Signal, &IncomingSignal, &OutgoingSignal)>,
) {
    for tile_storage in tilemap_q.iter() {
        for &tile_id in tile_storage.iter() {
            if let Some(tile_id) = tile_id {
                if let Ok((mut signal, incoming, outgoing)) = tile_q.get_mut(tile_id) {
                    signal.apply(incoming, outgoing);
                    let tile_color: TileColor = (*signal).into();
                    commands
                        .entity(tile_id)
                        .insert(tile_color)
                        .insert(IncomingSignal::default())
                        .insert(OutgoingSignal::default());
                }
            }
        }
    }
}
