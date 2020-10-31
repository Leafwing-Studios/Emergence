use bevy::prelude::*;
pub struct StructuresPlugin;
impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
           .add_startup_system(generate_structures.system())
           .add_system(grow_structures.system())
           .add_system(propagate_structures.system());
    }
}

fn generate_structures(){}

fn grow_structures(){}

fn propagate_structures(){}