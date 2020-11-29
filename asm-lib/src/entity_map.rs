use bevy::prelude::*;
use multimap::MultiMap;

use crate::position::Position;

pub struct EntityMapPlugin;
impl Plugin for EntityMapPlugin {
    fn build(&self, app: &mut AppBuilder){
		app.add_stage_after(stage::UPDATE, "BOOKKEEPING")
			.add_resource(EntityMap::new())
			.add_system_to_stage("BOOKKEEPING", update_entity_map);
	}
}

#[derive(Debug)]
pub struct EntityMap {
	pub mm: MultiMap<Position, Entity>
}

impl EntityMap {

	pub fn new() -> EntityMap{
		EntityMap{
			mm: MultiMap::new()
		}
	}
}

fn update_entity_map(mut entity_map: ResMut<EntityMap>, query: Query<(&Position, Entity), Changed<Position>>){
	let moved_entities: Vec<Entity> = query.iter().map(|q| q.1).collect();

	// Filter out any entities that moved, since they're no longer in the recorded position
	entity_map.mm.retain(|&_k, &v| !moved_entities.contains(&v));

	// Add entries back in to reflect new positions
	for (p, e) in query.iter() {
		entity_map.mm.insert(*p, e);
	}
}