//! Everything needed to make structures able to craft things.

use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};

use crate::items::{
    inventory::Inventory,
    recipe::{Recipe, RecipeId, RecipeManifest},
    ItemData, ItemId, ItemManifest,
};

/// The current state in the crafting progress.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub(crate) enum CraftingState {
    /// There are resources missing for the recipe.
    #[default]
    WaitingForInput,

    /// The resource cost has been paid and the recipe is being crafted.
    InProgress,

    /// The recipe has been crafted and the resources need to be claimed.
    Finished,
}

/// The input inventory for a structure.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub(crate) struct InputInventory(Inventory);

impl InputInventory {
    /// The inventory holding the items to be crafted.
    pub(crate) fn inventory(&self) -> &Inventory {
        &self.0
    }
}

/// The output inventory for a structure.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub(crate) struct OutputInventory(Inventory);

impl OutputInventory {
    /// The inventory for the crafting output.
    pub(crate) fn inventory(&self) -> &Inventory {
        &self.0
    }
}

/// The recipe that is currently being crafted, if any.
#[derive(Component, Debug, Default)]
pub(crate) struct ActiveRecipe(Option<RecipeId>);

impl ActiveRecipe {
    /// The ID of the currently active recipe, if one has been selected.
    pub(crate) fn recipe_id(&self) -> &Option<RecipeId> {
        &self.0
    }
}

/// The time remaining until the recipe has been crafted.
#[derive(Component, Debug, Default)]
pub(crate) struct CraftTimer(Timer);

impl CraftTimer {
    /// The timer indicating how much longer the crafting process will take.
    pub(crate) fn timer(&self) -> &Timer {
        &self.0
    }
}

/// All components needed to craft stuff.
#[derive(Debug, Default, Bundle)]
pub(crate) struct CraftingBundle {
    /// The input inventory for the items needed for crafting.
    input_inventory: InputInventory,

    /// The output inventory for the crafted items.
    output_inventory: OutputInventory,

    /// The recipe that is currently being crafted.
    active_recipe: ActiveRecipe,

    /// The "cooldown" for crafting.
    craft_timer: CraftTimer,

    /// The current state for the crafting process.
    craft_state: CraftingState,
}

impl CraftingBundle {
    /// Create a new crafting bundle without an active recipe set.
    pub(crate) fn new(starting_recipe: Option<RecipeId>) -> Self {
        if let Some(recipe_id) = starting_recipe {
            Self {
                // TODO: Don't hard-code these values
                input_inventory: InputInventory(Inventory::new(0)),
                output_inventory: OutputInventory(Inventory::new(1)),
                craft_timer: CraftTimer(Timer::new(Duration::default(), TimerMode::Once)),
                active_recipe: ActiveRecipe(Some(recipe_id)),
                craft_state: CraftingState::WaitingForInput,
            }
        } else {
            Self {
                // TODO: Don't hard-code these values
                input_inventory: InputInventory(Inventory::new(0)),
                output_inventory: OutputInventory(Inventory::new(1)),
                craft_timer: CraftTimer(Timer::new(Duration::ZERO, TimerMode::Once)),
                active_recipe: ActiveRecipe(None),
                craft_state: CraftingState::WaitingForInput,
            }
        }
    }
}

/// Make progress of all recipes that are being crafted.
fn progress_crafting(time: Res<Time>, mut query: Query<(&mut CraftTimer, &mut CraftingState)>) {
    for (mut craft_timer, mut craft_state) in query.iter_mut() {
        if *craft_state == CraftingState::InProgress {
            craft_timer.0.tick(time.delta());

            if craft_timer.0.finished() {
                *craft_state = CraftingState::Finished;
            }
        }
    }
}

/// Finish the crafting process once the timer ticked down and start the crafting of the next recipe.
fn start_and_finish_crafting(
    recipe_manifest: Res<RecipeManifest>,
    item_manifest: Res<ItemManifest>,
    mut query: Query<(
        &ActiveRecipe,
        &mut CraftTimer,
        &mut InputInventory,
        &mut OutputInventory,
        &mut CraftingState,
    )>,
) {
    for (active_recipe, mut craft_timer, mut input, mut output, mut craft_state) in query.iter_mut()
    {
        if let Some(recipe_id) = &active_recipe.0 {
            let recipe = recipe_manifest.get(recipe_id);

            // Try to finish the crafting by putting the output in the inventory
            if *craft_state == CraftingState::Finished
                && output
                    .0
                    .add_items_all_or_nothing(recipe.outputs(), &item_manifest)
                    .is_ok()
            {
                // The next item can be crafted
                *craft_state = CraftingState::WaitingForInput;
            }

            // Try to craft the next item by consuming the input and restarting the timer
            if *craft_state == CraftingState::WaitingForInput
                && input.0.remove_items_all_or_nothing(recipe.inputs()).is_ok()
            {
                // Set the timer to the recipe time
                craft_timer.0.set_duration(*recipe.craft_time());
                craft_timer.0.reset();

                // Start crafting
                *craft_state = CraftingState::InProgress;
            }
        }
    }
}

/// Add crafting capabilities to structures.
pub(crate) struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Load this from an asset file
        let mut item_manifest = HashMap::new();
        item_manifest.insert(ItemId::acacia_leaf(), ItemData::acacia_leaf());

        // TODO: Load this from an asset file
        let mut recipe_manifest = HashMap::new();
        recipe_manifest.insert(
            RecipeId::acacia_leaf_production(),
            Recipe::acacia_leaf_production(),
        );

        app.insert_resource(ItemManifest::new(item_manifest))
            .insert_resource(RecipeManifest::new(recipe_manifest))
            .add_system(progress_crafting)
            .add_system(start_and_finish_crafting.after(progress_crafting));
    }
}
