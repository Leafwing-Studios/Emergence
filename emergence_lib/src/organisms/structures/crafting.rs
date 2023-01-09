//! Everything needed to make structures able to craft things.

use bevy::prelude::*;

use crate::items::{Inventory, Recipe};

/// The current state in the crafting progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CraftingState {
    /// There are resources missing for the recipe.
    WaitingForInput,

    /// The resource cost has been paid and the recipe is being crafted.
    InProgress,

    /// The recipe has been crafted and the resources need to be claimed.
    Finished,
}

/// The input inventory for a structure.
#[derive(Debug, Component)]
pub struct InputInventory(pub Inventory);

/// The output inventory for a structure.
#[derive(Debug, Component)]
pub struct OutputInventory(pub Inventory);

/// The recipe that is currently being crafted.
#[derive(Debug, Component)]
pub struct ActiveRecipe(pub Recipe);

/// The time remaining until the recipe has been crafted.
#[derive(Debug, Component)]
pub struct CraftTimer(pub Timer);

/// The current state in the crafting progress.
#[derive(Debug, Component)]
pub struct CurCraftState(pub CraftingState);

/// All components needed to craft stuff.
#[derive(Debug, Bundle)]
pub struct CraftingBundle {
    /// The input inventory for the items needed for crafting.
    input_inventory: InputInventory,

    /// The output inventory for the crafted items.
    output_inventory: OutputInventory,

    /// The recipe that is currently being crafted.
    active_recipe: ActiveRecipe,

    /// The "cooldown" for crafting.
    craft_timer: CraftTimer,

    /// The current state for the crafting process.
    craft_state: CurCraftState,
}

impl CraftingBundle {
    /// Create a new crafting bundle for the given recipe.
    pub fn new(recipe: Recipe) -> Self {
        Self {
            // TODO: Don't hard-code these values
            input_inventory: InputInventory(Inventory::new(0, 0)),
            output_inventory: OutputInventory(Inventory::new(1, 10)),
            craft_timer: CraftTimer(Timer::new(*recipe.craft_time(), TimerMode::Once)),
            active_recipe: ActiveRecipe(recipe),
            craft_state: CurCraftState(CraftingState::WaitingForInput),
        }
    }
}

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
        &mut InputInventory,
        &mut OutputInventory,
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
            info!("Crafted items: {:?}", recipe.outputs());
            // The next item can be crafted
            craft_state.0 = CraftingState::WaitingForInput;
        }

        // Try to craft the next item by consuming the input and restarting the timer
        if craft_state.0 == CraftingState::WaitingForInput
            && input
                .0
                .remove_all_or_nothing_many_items(recipe.inputs())
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
