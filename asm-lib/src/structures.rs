use bevy::prelude::*;

pub struct Structure {}
pub struct Plant {}
pub struct Fungi {}

pub struct StructuresPlugin;
impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app;
    }
}
