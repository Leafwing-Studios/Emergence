use crate::organisms::{Composition, OrganismBundle, OrganismType};
use crate::signals::SignalId;
use crate::texture::IntoTile;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos};

#[derive(Bundle, Default)]
pub struct StructureBundle {
    structure: Structure,
    #[bundle]
    organism_bundle: OrganismBundle,
}

// TODO: replace with better defaults
#[derive(Component, Clone, Default)]
pub struct Structure {
    upkeep_rate: f32,
    starting_mass: f32,
    despawn_mass: f32,
}

#[derive(Component, Clone, Default)]
pub struct Plant {
    photosynthesis_rate: f32,
}

#[derive(Bundle, Default)]
pub struct PlantBundle {
    plant: Plant,
    #[bundle]
    structure_bundle: StructureBundle,
    #[bundle]
    tile_bundle: TileBundle,
}

impl PlantBundle {
    pub fn new(tilemap_id: TilemapId, position: TilePos) -> Self {
        Self {
            structure_bundle: StructureBundle {
                structure: Default::default(),
                organism_bundle: OrganismBundle {
                    signal_id: SignalId::Plant,
                    ..Default::default()
                },
            },
            tile_bundle: (OrganismType::Plant).into_tile(tilemap_id, position),
            ..Default::default()
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct Fungi;

#[derive(Bundle, Default)]
pub struct FungiBundle {
    fungi: Fungi,
    #[bundle]
    structure_bundle: StructureBundle,
    #[bundle]
    tile_bundle: TileBundle,
}

impl FungiBundle {
    pub fn new(tilemap_id: TilemapId, position: TilePos) -> Self {
        Self {
            structure_bundle: StructureBundle {
                organism_bundle: OrganismBundle {
                    signal_id: SignalId::Fungus,
                    ..Default::default()
                },
                ..Default::default()
            },
            tile_bundle: (OrganismType::Fungus).into_tile(tilemap_id, position),
            ..Default::default()
        }
    }
}

pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(photosynthesize)
            .add_system(upkeep)
            .add_system(cleanup);
    }
}

fn photosynthesize(time: Res<Time>, mut query: Query<(&Plant, &mut Composition)>) {
    for (plant, mut comp) in query.iter_mut() {
        comp.mass += plant.photosynthesis_rate * time.delta_seconds() * comp.mass.powf(2.0 / 3.0);
    }
}

fn upkeep(time: Res<Time>, mut query: Query<(&Structure, &mut Composition)>) {
    for (structure, mut comp) in query.iter_mut() {
        comp.mass -= structure.upkeep_rate * time.delta_seconds() * comp.mass;
    }
}

fn cleanup(mut commands: Commands, query: Query<(&Structure, Entity, &Composition)>) {
    for (structure, ent, comp) in query.iter() {
        if comp.mass <= structure.despawn_mass {
            commands.entity(ent).despawn();
        }
    }
}
