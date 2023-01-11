//! Detailed info about a given organism.

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use std::fmt::Display;

use crate::{
    cursor::CursorTilePos,
    items::{Inventory, Recipe},
};

use super::{
    structures::{
        crafting::{
            ActiveRecipe, CraftTimer, CraftingState, CurCraftState, InputInventory, OutputInventory,
        },
        fungi::Fungi,
        plants::Plant,
    },
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

/// The details about crafting processes.
#[derive(Debug, Clone)]
pub struct CraftingDetails {
    /// The inventory for the input items.
    pub input_inventory: Inventory,

    /// The inventory for the output items.
    pub output_inventory: Inventory,

    /// The recipe that's currently being crafted.
    pub active_recipe: Recipe,

    /// The state of the ongoing crafting process.
    pub state: CraftingState,

    /// The time remaining to finish crafting.
    pub timer: Timer,
}

/// Detailed info about a given entity.
#[derive(Debug, Clone)]
pub struct OrganismDetails {
    /// The entity ID of the organism that this info is about.
    pub entity: Entity,

    /// The type of the organism, e.g. plant or fungus.
    pub organism_type: OrganismType,

    /// If this organism is crafting something, the details about that.
    pub crafting_details: Option<CraftingDetails>,
}

/// Detailed info about the organism that is being hovered.
#[derive(Debug, Resource, Default, Deref)]
pub struct HoverDetails(Option<OrganismDetails>);

/// Display detailed info on hover.
pub struct DetailsPlugin;

impl Plugin for DetailsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building DetailsPlugin...");

        app.init_resource::<HoverDetails>()
            // TODO: This should be done after the cursor system
            .add_system(hover_details);
    }
}

/// Get details about the hovered entity.
fn hover_details(
    cursor_pos: Res<CursorTilePos>,
    mut hover_details: ResMut<HoverDetails>,
    query: Query<(
        Entity,
        &TilePos,
        Option<&Plant>,
        Option<&Fungi>,
        Option<&Ant>,
        Option<(
            &InputInventory,
            &OutputInventory,
            &ActiveRecipe,
            &CurCraftState,
            &CraftTimer,
        )>,
    )>,
) {
    if let Some(cursor_pos) = **cursor_pos {
        hover_details.0 = None;

        for (entity, tile_pos, plant, fungi, ant, crafting_stuff) in query.iter() {
            if *tile_pos == cursor_pos {
                // Determine the organism type via the marker components
                let organism_type = if plant.is_some() {
                    Some(OrganismType::Plant)
                } else if fungi.is_some() {
                    Some(OrganismType::Fungus)
                } else if ant.is_some() {
                    Some(OrganismType::Ant)
                } else {
                    None
                };

                let crafting_details =
                    if let Some((input, output, recipe, state, timer)) = crafting_stuff {
                        Some(CraftingDetails {
                            input_inventory: input.0.clone(),
                            output_inventory: output.0.clone(),
                            active_recipe: recipe.0.clone(),
                            state: state.0.clone(),
                            timer: timer.0.clone(),
                        })
                    } else {
                        None
                    };

                if let Some(organism_type) = organism_type {
                    hover_details.0 = Some(OrganismDetails {
                        entity,
                        organism_type,
                        crafting_details,
                    });
                }
            }
        }
    } else {
        hover_details.0 = None;
    }
}
