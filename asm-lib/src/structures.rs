use bevy::prelude::*;

use crate::config::{
    STRUCTURE_DESPAWN_MASS, STRUCTURE_GROWTH_RATE, STRUCTURE_STARTING_MASS, STRUCTURE_UPKEEP_RATE,
};
use crate::graphics::make_sprite_components;
use crate::organisms::Mass;
use crate::utils::Position;

pub struct Structure {}
pub struct Plant {}
pub struct Fungi {}

pub struct StructureConfig {
    growth_rate: f32,
    upkeep_rate: f32,
    starting_mass: f32,
    despawn_mass: f32,
}

pub fn build_plant(
    commands: &mut Commands,
    handle: Handle<ColorMaterial>,
    position: Position,
    config: &Res<StructureConfig>,
) {
    commands
        .spawn(make_sprite_components(&position, handle))
        .with(Structure {})
        .with(Plant {})
        .with(position)
        .with(Mass {
            mass: config.starting_mass,
        });
}

pub fn build_fungi(
    commands: &mut Commands,
    handle: Handle<ColorMaterial>,
    position: Position,
    config: &Res<StructureConfig>,
) {
    commands
        .spawn(make_sprite_components(&position, handle))
        .with(Structure {})
        .with(Fungi {})
        .with(position)
        .with(Mass {
            mass: config.starting_mass,
        });
}

pub struct StructuresPlugin;
impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(StructureConfig {
            growth_rate: STRUCTURE_GROWTH_RATE,
            upkeep_rate: STRUCTURE_UPKEEP_RATE,
            starting_mass: STRUCTURE_STARTING_MASS,
            despawn_mass: STRUCTURE_DESPAWN_MASS,
        })
        .add_system(photosynthesize.system())
        .add_system(upkeep.system())
        .add_system(cleanup.system())
        .add_system(debug_mass.system());
    }
}

fn photosynthesize(
    config: Res<StructureConfig>,
    time: Res<Time>,
    mut query: Query<(&Plant, &mut Mass)>,
) {
    for (_, mut i) in query.iter_mut() {
        i.mass += config.growth_rate * time.delta_seconds * i.mass.powf(2.0 / 3.0);
    }
}

fn upkeep(
    config: Res<StructureConfig>,
    time: Res<Time>,
    mut query: Query<(&Structure, &mut Mass)>,
) {
    for (_, mut i) in query.iter_mut() {
        i.mass -= config.upkeep_rate * time.delta_seconds * i.mass;
    }
}

fn cleanup(
    commands: &mut Commands,
    config: Res<StructureConfig>,
    query: Query<(&Structure, Entity, &Mass)>,
) {
    for (_, ent, i) in query.iter() {
        if i.mass <= config.despawn_mass {
            commands.despawn(ent);
        }
    }
}

fn debug_mass(query: Query<&Mass>) {
    let mut total_mass: f32 = 0.0;

    for i in query.iter() {
        total_mass += i.mass;
    }

    dbg!(total_mass);
}
