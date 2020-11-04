use bevy::prelude::*;

use crate::utils::Position;

pub struct Unit {}
pub struct Ant {}

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(UnitTimer(Timer::from_seconds(2.0, true)))
            .add_system(plan.system())
            .add_system(act.system())
            .add_system(maintain_units.system());
    }
}

struct UnitTimer(Timer);

fn plan(mut commands: Commands) {}

fn act(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<UnitTimer>,
    mut query: Query<(&Unit, &mut Position)>,
) {
    timer.0.tick(time.delta_seconds);
    if timer.0.finished {
        for (_, mut position) in query.iter_mut() {
            position.x = position.x + 1;
        }
    }
}

fn maintain_units(mut commands: Commands) {}
