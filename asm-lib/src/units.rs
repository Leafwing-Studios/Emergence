use bevy::prelude::*;

use crate::graphics::make_sprite_components;
use crate::id::ID;
use crate::position::Position;
use crate::entity_map::EntityMap;

pub struct Unit {}
pub struct Ant {}

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(UnitTimer(Timer::from_seconds(0.5, true)))
            .add_system(act.system());
    }
}

struct UnitTimer(Timer);

fn act(
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    mut query: Query<(&Unit, &mut Position)>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        for (_, mut position) in query.iter_mut() {
            *position = wander(*position);
        }
    }
}

fn wander(position: Position) -> Position {
    let target = position.random_neighbor();

    match target {
        Some(p) => p,
        None => position,
    }
}

pub fn build_ant(commands: &mut Commands, handle: Handle<ColorMaterial>, position: Position) {
    commands
        .spawn(make_sprite_components(&position, handle))
        .with(Unit {})
        .with(Ant {})
        .with(ID::Ant)
        .with(position);
}
