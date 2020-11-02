use bevy::prelude::*;
use crate::utils::Position;

pub struct Unit{}
pub struct Ant{}
pub fn build_hive(position: Position){
    (Unit{}, Ant{}, position);
}

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_system(plan.system())
        .add_system(act.system())
        .add_system(maintain_units.system());
    }
}

fn plan(mut commands: Commands){}

fn act(mut commands: Commands){}

fn maintain_units(mut commands: Commands){}