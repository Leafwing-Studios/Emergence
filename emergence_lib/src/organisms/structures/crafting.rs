//! Everything needed to make structures able to craft things.

use bevy::prelude::*;

use crate::items::{Inventory, Recipe};

/// The current state in the crafting progress.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CraftingState {
    /// There are resources missing for the recipe.
    WaitingForInput,

    /// The resource cost has been paid and the recipe is being crafted.
    InProgress,

    /// The recipe has been crafted and the resources need to be claimed.
    Finished,
}

/// The input inventory for a structure.
#[derive(Debug, Component)]
pub struct Input(Inventory);

/// The output inventory for a structure.
#[derive(Debug, Component)]
pub struct Output(Inventory);

/// The recipe that is currently being crafted.
#[derive(Debug, Component)]
pub struct ActiveRecipe(Recipe);

/// The time remaining until the recipe has been crafted.
#[derive(Debug, Component)]
pub struct CraftTimer(Timer);

/// The current state in the crafting progress.
#[derive(Debug, Component)]
pub struct CurCraftState(CraftingState);

/// Make progress of all recipes that are being crafted.
fn progress_crafting(time: Res<Time>, mut query: Query<(&mut CraftTimer, &mut CurCraftState)>) {
    for (mut craft_timer, mut craft_state) in query.iter_mut() {
        if craft_state.0 == CraftingState::InProgress {
            craft_timer.0.tick(time.delta());

            if craft_timer.0.finished() {
                craft_state.0 = CraftingState::Finished;
            }
        }
    }
}

/// Finish the crafting process once the timer ticked down and start the crafting of the next recipe.
fn start_and_finish_crafting(
    mut query: Query<(
        &ActiveRecipe,
        &mut CraftTimer,
        &mut Input,
        &mut Output,
        &mut CurCraftState,
    )>,
) {
    for (active_recipe, mut craft_timer, mut input, mut output, mut craft_state) in query.iter_mut()
    {
        let recipe = &active_recipe.0;

        // Try to finish the crafting by putting the output in the inventory
        if craft_state.0 == CraftingState::Finished
            && output
                .0
                .add_all_or_nothing_many_items(recipe.outputs())
                .is_ok()
        {
            // The next item can be crafted
            craft_state.0 = CraftingState::WaitingForInput;
        }

        // Try to craft the next item by consuming the input and restarting the timer
        if craft_state.0 == CraftingState::WaitingForInput
            && input
                .0
                .remove_all_or_nothing_all_items(recipe.inputs())
                .is_ok()
        {
            // Set the timer to the recipe time
            craft_timer.0.set_duration(*recipe.craft_time());
            craft_timer.0.reset();

            // Start crafting
            craft_state.0 = CraftingState::InProgress;
        }
    }
}

/// Add crafting capabilities to structures.
pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(progress_crafting)
            .add_system(start_and_finish_crafting.after(progress_crafting));
    }
}
