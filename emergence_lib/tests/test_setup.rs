// use common::{bevy_app, interaction_app, minimal_app, simulation_app};

use emergence_lib::simulation::generation::GenerationConfig;
use emergence_lib::testing::{interaction_app, minimal_app, simulation_app};

#[test]
fn minimal_app_can_update() {
    let mut app = minimal_app();

    app.update()
}

#[test]
#[ignore = "Cannot end-to-end test game without a GPU."]
fn simulation_app_can_update() {
    let mut app = simulation_app(GenerationConfig::default());

    app.update()
}

#[test]
#[ignore = "Cannot test interaction without a virtual window."]
// Blocked on https://github.com/bevyengine/bevy/pull/6256
fn interaction_app_can_update() {
    let mut app = interaction_app(GenerationConfig::default());

    app.update()
}
