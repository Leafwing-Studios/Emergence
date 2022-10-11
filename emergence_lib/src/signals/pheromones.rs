use crate::signals::configs::SignalConfig;
use crate::signals::emitters::{Emitter, StockEmitter};
use crate::signals::SignalCreateEvent;
use bevy::prelude::*;

pub struct PheromonesPlugin;

impl Plugin for PheromonesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(generate_signal_create_event)
    }
}

pub struct PheromoneAttractConfig {
    emitter: Emitter,
    signal_config: SignalConfig,
}

impl Default for PheromoneAttractConfig {
    fn default() -> Self {
        todo!()
    }
}

fn generate_signal_create_event(
    signal_create_evw: EventWriter<SignalCreateEvent>,
    cursor_tile_pos: Res<CursorTilePos>,
) {
}
