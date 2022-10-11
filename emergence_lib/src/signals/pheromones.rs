use crate::signals::configs::SignalConfig;
use crate::signals::emitters::{Emitter, StockEmitter};
use bevy::prelude::*;

pub struct PheromonesPlugin;

impl Plugin for PheromonesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system()
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

fn generate_create_event(cursor_pos: Res<CursorTilePos>) {}
