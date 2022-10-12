use crate::cursor::CursorTilePos;
use crate::signals::configs::{SignalColorConfig, SignalConfig};
use crate::signals::emitters::Emitter;
use crate::signals::emitters::StockEmitter::PheromoneAttract;
use crate::signals::{Signal, SignalCreateEvent};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

/// Represents the interface between the player and the hive.
pub struct HiveMindPlugin;

impl Plugin for HiveMindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<HiveMindAction>::default())
            .add_startup_system(initialize_hive_mind)
            .add_system(place_pheromone);
    }
}

#[derive(Component, Clone, Copy)]
pub struct HiveMind;

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HiveMindAction {
    PlacePheromone,
    CyclePheromone,
}

pub const PHEROMONE_ATTRACT_CONFIG: SignalConfig = SignalConfig {
    diffusion_factor: 1e-2,
    decay_probability: 1e-4,
    color_config: SignalColorConfig {
        rgb_color: [0.7, 0.85, 0.9],
        zero_value: 0.2,
        one_value: 0.8,
        is_visible: true,
    },
};

fn initialize_hive_mind(mut commands: Commands) {
    commands
        .spawn()
        .insert(HiveMind)
        .insert_bundle(InputManagerBundle::<HiveMindAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::Space, HiveMindAction::PlacePheromone),
                (KeyCode::P, HiveMindAction::CyclePheromone),
            ]),
        });
}

fn place_pheromone(
    mut signal_create_evw: EventWriter<SignalCreateEvent>,
    cursor_tile_pos: Res<CursorTilePos>,
    hive_mind_query: Query<&ActionState<HiveMindAction>, With<HiveMind>>,
) {
    let hive_mind_state = hive_mind_query.single();

    if hive_mind_state.pressed(HiveMindAction::PlacePheromone) && (*cursor_tile_pos).is_some() {
        signal_create_evw.send(SignalCreateEvent {
            emitter: Emitter::Stock(PheromoneAttract),
            pos: (*cursor_tile_pos).unwrap(),
            initial: Signal::new(0.0),
            config: PHEROMONE_ATTRACT_CONFIG,
        })
    }
}
