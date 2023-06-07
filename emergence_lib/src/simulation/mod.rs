//! Code that runs the simulation of the game.
//!
//! All plugins in this module should work without rendering.

use crate::asset_management::AssetState;
use crate::construction::ConstructionPlugin;
use crate::crafting::CraftingPlugin;
use crate::geometry::{sync_rotation_to_facing, MapGeometry};
use crate::light::LightPlugin;
use crate::organisms::OrganismPlugin;
use crate::signals::SignalsPlugin;
use crate::simulation::time::TemporalPlugin;
use crate::simulation::weather::WeatherPlugin;
use crate::structures::StructuresPlugin;
use crate::terrain::TerrainPlugin;
use crate::units::UnitsPlugin;
use crate::water::WaterPlugin;
use crate::world_gen::{GenerationConfig, GenerationPlugin};
use bevy::core::FrameCount;
use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::prelude::*;

pub mod time;
pub mod weather;

/// Sets up world geometry
pub struct GeometryPlugin {
    /// Configuration settings for world generation
    pub gen_config: GenerationConfig,
}

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        let map_geometry = MapGeometry::new(&mut app.world, self.gen_config.map_radius);

        app.insert_resource(map_geometry);
    }
}

/// All of the code needed to make the simulation run
pub struct SimulationPlugin {
    /// Configuration settings for world generation
    pub gen_config: GenerationConfig,
}

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        info!("Building simulation plugin...");
        app.add_system(sync_rotation_to_facing)
            .edit_schedule(CoreSchedule::FixedUpdate, |schedule| {
                schedule.configure_set(
                    SimulationSet
                        .run_if(in_state(PauseState::Playing))
                        .run_if(in_state(AssetState::FullyLoaded))
                        .run_if(max_ticks_not_reached),
                );
                schedule.add_system(update_ticks_this_frame.run_if(max_ticks_not_reached));

                schedule.set_build_settings(ScheduleBuildSettings {
                    ambiguity_detection: LogLevel::Warn,
                    ..Default::default()
                });
            })
            .insert_resource(TicksThisFrame { current: 0, max: 3 })
            .add_plugin(GenerationPlugin {
                config: self.gen_config.clone(),
            })
            .add_plugin(CraftingPlugin)
            .add_plugin(ConstructionPlugin)
            .add_plugin(StructuresPlugin)
            .add_plugin(TerrainPlugin)
            .add_plugin(OrganismPlugin)
            .add_plugin(UnitsPlugin)
            .add_plugin(SignalsPlugin)
            .add_plugin(TemporalPlugin)
            .add_plugin(LightPlugin)
            .add_plugin(WaterPlugin)
            .add_plugin(WeatherPlugin);
    }
}

/// Controls whether or not the game is paused.
#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
enum PauseState {
    /// Game logic is running.
    #[default]
    Playing,
    /// Game logic is stopped.
    Paused,
}

/// Simulation systems.
///
/// These:
/// - are run in [`CoreSchedule::FixedUpdate`]
/// - only run in [`PauseState::Playing`]
/// - only run in [`AssetState::FullyLoaded`]
#[derive(SystemSet, PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) struct SimulationSet;

/// Tracks how many ticks have passed this frame.
// BLOCKED: this is a workaround for https://github.com/bevyengine/bevy/issues/8543.
// Once that's fixed and released all this code should be removed.
#[derive(Resource, Debug)]
struct TicksThisFrame {
    /// The number of ticks that have passed this frame.
    current: u8,
    /// The maximum number of ticks that can pass in a frame.
    max: u8,
}

/// Updates [`TicksThisFrame`].
fn update_ticks_this_frame(mut ticks: ResMut<TicksThisFrame>, frame_count: Res<FrameCount>) {
    if frame_count.is_changed() {
        ticks.current = 0;
    }

    ticks.current += 1;
}

/// Stops
fn max_ticks_not_reached(frame_count: Res<FrameCount>, ticks: Res<TicksThisFrame>) -> bool {
    if frame_count.is_changed() {
        return true;
    }

    ticks.current < ticks.max
}
