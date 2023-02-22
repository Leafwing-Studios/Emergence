//! Everything needed to make structures able to craft things.

use std::{fmt::Display, time::Duration};

use bevy::{prelude::*, utils::HashMap};

use crate::{
    items::{
        inventory::Inventory,
        recipe::{Recipe, RecipeId, RecipeManifest},
        ItemData, ItemId, ItemManifest,
    },
    signals::{Emitter, SignalStrength, SignalType},
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

impl Display for CraftingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CraftingState::WaitingForInput => "Waiting for input",
            CraftingState::InProgress => "In progress",
            CraftingState::Finished => "Finished",
        };

        write!(f, "{str}")
    }
}

/// The input inventory for a structure.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub(crate) struct InputInventory {
    /// Inner storage
    pub(crate) inventory: Inventory,
}

/// The output inventory for a structure.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub(crate) struct OutputInventory {
    /// Inner storage
    pub(crate) inventory: Inventory,
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
#[derive(Debug, Bundle)]
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

    /// Emits signals, drawing units towards this structure to ensure crafting flows smoothly
    emitter: Emitter,
}

impl CraftingBundle {
    /// Create a new crafting bundle without an active recipe set.
    pub(crate) fn new(
        starting_recipe: Option<RecipeId>,
        recipe_manifest: &RecipeManifest,
        item_manifest: &ItemManifest,
    ) -> Self {
        if let Some(recipe_id) = starting_recipe {
            let recipe = recipe_manifest.get(recipe_id);

            Self {
                input_inventory: InputInventory {
                    inventory: recipe.input_inventory(item_manifest),
                },
                output_inventory: OutputInventory {
                    inventory: recipe.output_inventory(item_manifest),
                },
                craft_timer: CraftTimer(Timer::new(Duration::default(), TimerMode::Once)),
                active_recipe: ActiveRecipe(Some(recipe_id)),
                craft_state: CraftingState::WaitingForInput,
                emitter: Emitter::default(),
            }
        } else {
            Self {
                input_inventory: InputInventory {
                    inventory: Inventory::new(0),
                },
                output_inventory: OutputInventory {
                    inventory: Inventory::new(1),
                },
                craft_timer: CraftTimer(Timer::new(Duration::ZERO, TimerMode::Once)),
                active_recipe: ActiveRecipe(None),
                craft_state: CraftingState::WaitingForInput,
                emitter: Emitter::default(),
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
            let recipe = recipe_manifest.get(*recipe_id);

            // Try to finish the crafting by putting the output in the inventory
            if *craft_state == CraftingState::Finished
                && output
                    .add_items_all_or_nothing(recipe.outputs(), &item_manifest)
                    .is_ok()
            {
                // The next item can be crafted
                *craft_state = CraftingState::WaitingForInput;
            }

            // Try to craft the next item by consuming the input and restarting the timer
            if *craft_state == CraftingState::WaitingForInput
                && input.remove_items_all_or_nothing(recipe.inputs()).is_ok()
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

/// Causes crafting structures to emit signals based on the items they have and need.
fn set_emitter(mut crafting_query: Query<(&mut Emitter, &InputInventory, &OutputInventory)>) {
    for (mut emitter, input_inventory, output_inventory) in crafting_query.iter_mut() {
        // Reset and recompute all signals
        // TODO: may eventually want to just reset crafting signals
        emitter.signals.clear();

        for item_slot in input_inventory.iter() {
            if !item_slot.is_full() {
                let signal_type = SignalType::Pull(item_slot.item_id());
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
            }
        }

        for item_slot in output_inventory.iter() {
            if item_slot.is_full() {
                let signal_type = SignalType::Push(item_slot.item_id());
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
            } else if !item_slot.is_empty() {
                let signal_type = SignalType::Contains(item_slot.item_id());
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength));
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
        item_manifest.insert(ItemId::leuco_chunk(), ItemData::leuco_chunk());

        // TODO: Load this from an asset file
        let mut recipe_manifest = HashMap::new();
        recipe_manifest.insert(
            RecipeId::acacia_leaf_production(),
            Recipe::acacia_leaf_production(),
        );
        recipe_manifest.insert(
            RecipeId::leuco_chunk_production(),
            Recipe::leuco_chunk_production(),
        );

        app.insert_resource(ItemManifest::new(item_manifest))
            .insert_resource(RecipeManifest::new(recipe_manifest))
            .add_system(progress_crafting)
            .add_system(start_and_finish_crafting.after(progress_crafting))
            .add_system(set_emitter.after(start_and_finish_crafting));
    }
}
