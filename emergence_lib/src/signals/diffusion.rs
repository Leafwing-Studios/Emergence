use crate::tiles::MAP_CENTER;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::curves::{ergonomic_sigmoid, linear_combination};

pub struct DiffusionPlugin;

impl Plugin for DiffusionPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, initialize_signal)
            .add_startup_system_to_stage(StartupStage::PostStartup, initialize_deltas)
            .add_system(propagate_signal)
            .add_system(update_signal.after(propagate_signal));
    }
}

#[derive(Component, Copy, Clone, Default, Debug)]
struct Signal(f32);

#[derive(Component, Copy, Clone, Default, Debug)]
struct IncomingSignal(f32);

impl IncomingSignal {
    fn add(&mut self, v: f32) {
        self.0 += v;
    }
}

#[derive(Component, Copy, Clone, Default, Debug)]
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

impl From<Signal> for Color {
    fn from(signal: Signal) -> Self {
        // What are the possible values for a signal? [0, \infty)
        // What are we mapping to? [0, 1]
        // Use a shifted sigmoid to represent this
        Color::Rgba {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: ergonomic_sigmoid(signal.0, 0.0, 1.0, 0.0, 0.005),
        }
    }
}

impl From<&Signal> for Color {
    fn from(signal: &Signal) -> Self {
        (*signal).into()
    }
}

// Color::WHITE cannot be used, as it has the RGB variant, not the RGBA variant
const RGBA_WHITE: Color = Color::rgba(1.0, 1.0, 1.0, 1.0);

impl From<Signal> for TileColor {
    fn from(signal: Signal) -> Self {
        let signal_color: Color = signal.into();
        TileColor(signal_color.over(&RGBA_WHITE))
    }
}

impl From<&Signal> for TileColor {
    fn from(signal: &Signal) -> Self {
        let signal_color: Color = signal.into();
        TileColor(signal_color.over(&RGBA_WHITE))
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
                    // Initialize signal at the center tile for testing purposes
                    let signal = if tile_pos.x == MAP_CENTER.x && tile_pos.y == MAP_CENTER.y {
                        Signal(1.0)
                    } else {
                        Signal(0.0)
                    };
                    let tile_color: TileColor = signal.into();
                    commands.entity(tile_id).insert(signal).insert(tile_color);
                }
            }
        }
    }
}

fn initialize_deltas(mut commands: Commands, tilemap_storage_q: Query<&TileStorage>) {
    for tilemap_storage in tilemap_storage_q.iter() {
        for &tile_id in tilemap_storage.iter() {
            if let Some(tile_id) = tile_id {
                commands
                    .entity(tile_id)
                    .insert(IncomingSignal::default())
                    .insert(OutgoingSignal::default());
            }
        }
    }
}

pub const OUTGOING_RATE: f32 = 0.005;

fn propagate_signal(
    tilemap_q: Query<(&TilemapType, &TileStorage)>,
    mut tile_outgoing_q: Query<(&TilePos, &Signal, &mut OutgoingSignal)>,
    mut neighbor_incoming_q: Query<&mut IncomingSignal>,
) {
    for (map_type, tile_storage) in tilemap_q.iter() {
        for (tile_pos, this_signal, mut this_outgoing) in tile_outgoing_q.iter_mut() {
            let neighbors = get_tile_neighbors(tile_pos, tile_storage, map_type);
            let mut total_outgoing = 0.0;
            for neighbor_id in neighbors.into_iter() {
                if let Ok(mut neighbor_incoming) = neighbor_incoming_q.get_mut(neighbor_id) {
                    let outgoing = OUTGOING_RATE * this_signal.0;
                    neighbor_incoming.add(outgoing);
                    total_outgoing += outgoing;
                }
            }
            this_outgoing.add(total_outgoing);
        }
    }
}

fn update_signal(
    mut commands: Commands,
    mut tile_q: Query<(Entity, &mut Signal, &IncomingSignal, &OutgoingSignal)>,
) {
    for (tile_entity, mut signal, incoming, outgoing) in tile_q.iter_mut() {
        signal.apply(incoming, outgoing);
        let tile_color: TileColor = (*signal).into();
        commands
            .entity(tile_entity)
            .insert(tile_color)
            .insert(IncomingSignal::default())
            .insert(OutgoingSignal::default());
    }
}
