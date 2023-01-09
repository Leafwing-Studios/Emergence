//! Detailed info about a given organism.

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use std::fmt::Display;

use crate::cursor::CursorTilePos;

use super::{
    structures::{fungi::Fungi, plants::Plant},
    units::Ant,
};

/// The type of the organism, e.g. plant or fungus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrganismType {
    /// A plant.
    Plant,

    /// A fungus.
    Fungus,

    /// An ant.
    Ant,
}

impl Display for OrganismType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrganismType::Plant => "Plant",
                OrganismType::Fungus => "Fungus",
                OrganismType::Ant => "Ant",
            }
        )
    }
}

/// Detailed info about a given entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganismDetails {
    /// The entity ID of the organism that this info is about.
    pub entity: Entity,

    /// The type of the organism, e.g. plant or fungus.
    pub organism_type: OrganismType,
}

/// Detailed info about the organism that is being hovered.
#[derive(Debug, Resource, Default)]
pub struct HoverDetails(pub Option<OrganismDetails>);

/// Display detailed info on hover.
pub struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        // TODO: This should be done after the cursor system
        app.init_resource::<HoverDetails>()
            .add_system(hover_details);
    }
}

/// Get details about the hovered entity.
fn hover_details(
    world: &World,
    cursor_pos: Res<CursorTilePos>,
    mut hover_details: ResMut<HoverDetails>,
    query: Query<(Entity, &TilePos)>,
) {
    if let Some(cursor_pos) = cursor_pos.0 {
        hover_details.0 = None;

        for (entity, tile_pos) in query.iter() {
            if *tile_pos == cursor_pos {
                // Determine the organism type via the marker components
                let organism_type = if world.get::<Plant>(entity).is_some() {
                    Some(OrganismType::Plant)
                } else if world.get::<Fungi>(entity).is_some() {
                    Some(OrganismType::Fungus)
                } else if world.get::<Ant>(entity).is_some() {
                    Some(OrganismType::Ant)
                } else {
                    None
                };

                if let Some(organism_type) = organism_type {
                    hover_details.0 = Some(OrganismDetails {
                        entity,
                        organism_type,
                    });
                }
            }
        }
    } else {
        hover_details.0 = None;
    }
}
