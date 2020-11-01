use bevy::prelude::*;

pub struct SignalsPlugin;
impl Plugin for SignalsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
           .add_system(generate_signals.system())
           .add_system(propagate_signals.system())
           .add_system(decay_signals.system());
    }
}

fn generate_signals(mut commands: Commands){}

fn propagate_signals(mut commands: Commands){}

fn decay_signals(mut commands: Commands){}
