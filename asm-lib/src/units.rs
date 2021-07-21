use bevy::prelude::*;

use crate::entity_map::EntityMap;
use crate::graphics::sprite_bundle_from_position;
use crate::id::ID;
use crate::organisms::{Impassable, OrganismBundle};
use crate::position::Position;

#[derive(Clone, Default)]
pub struct Unit;

#[derive(Bundle, Default)]
pub struct UnitBundle {
    unit: Unit,
    #[bundle]
    organism_bundle: OrganismBundle,
}
#[derive(Clone, Default)]
pub struct Ant;

#[derive(Bundle, Default)]
pub struct AntBundle {
    ant: Ant,
    #[bundle]
    unit_bundle: UnitBundle,
}

impl AntBundle {
    pub fn new(position: Position, material: Handle<ColorMaterial>) -> Self {
        Self {
            unit_bundle: UnitBundle {
                organism_bundle: OrganismBundle {
                    sprite_bundle: sprite_bundle_from_position(position, material),
                    id: ID::Ant,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub struct UnitsPlugin;
impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(UnitTimer(Timer::from_seconds(0.5, true)))
            .add_system(act.system());
    }
}
/// Global timer that controls when units should act
struct UnitTimer(Timer);

fn act(
    time: Res<Time>,
    entity_map: Res<EntityMap>,
    mut timer: ResMut<UnitTimer>,
    mut query: Query<(&Unit, &mut Position)>,
    passable_query: Query<&Impassable>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (_, mut position) in query.iter_mut() {
            *position = wander(*position, &entity_map, &passable_query);
        }
    }
}

fn wander(
    position: Position,
    entity_map: &EntityMap,
    passable_query: &Query<&Impassable>,
) -> Position {
    let target = position.random_passable_neighbor(entity_map, passable_query);

    match target {
        Some(p) => p,
        None => position,
    }
}
