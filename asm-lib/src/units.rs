use bevy::prelude::*;
use crate::utils::Position;

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

fn generate_units(mut commands: Commands){}

fn plan(mut commands: Commands){}

fn act(mut commands: Commands){}

fn maintain_units(mut commands: Commands){}