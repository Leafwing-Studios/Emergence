use bevy::prelude::*;
use rand::distributions::Standard;
use rand::Rng;

use crate::utils::{HexDirection, Position};

pub struct Unit {}
pub struct Ant {}

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(UnitTimer(Timer::from_seconds(0.5, true)))
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
            let mut rng = &mut rand::thread_rng();
            let direction: HexDirection = rng.sample(Standard);

            let offset = direction.offset();

            position.alpha += offset.alpha;
            position.beta += offset.beta;
        }
    }
}

// NE <-> SE
// NW <-> SW

fn maintain_units(mut commands: Commands) {}
