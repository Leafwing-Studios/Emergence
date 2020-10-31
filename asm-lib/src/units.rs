use bevy::prelude::*;
pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(generate_units.system())
        .add_system(plan.system())
        .add_system(act.system())
        .add_system(maintain_units.system());
    }
}

fn generate_units(){}

fn plan(){}

fn act(){}

fn maintain_units(){}