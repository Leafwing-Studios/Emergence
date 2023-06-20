//! Tools to alter the terrain type and height.

use bevy::{ecs::system::Command, prelude::*};
use hexx::Hex;

use crate::{
    asset_management::manifest::Id,
    crafting::{
        inventories::{InputInventory, OutputInventory},
        item_tags::ItemKind,
    },
    geometry::VoxelPos,
    items::{inventory::Inventory, item_manifest::Item},
    signals::{Emitter, SignalStrength, SignalType},
    terrain::{
        commands::TerrainCommandsExt,
        terrain_manifest::{Terrain, TerrainManifest},
    },
};

/// An option presented to players for how to terraform the world.
///
/// These are generally higher level than the actual [`TerraformingAction`]s,
/// which represent the actual changes to the terrain that can be performed by units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TerraformingTool {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Replace the existing soil with the provided [`Id<Terrain>`].
    Change(Id<Terrain>),
}

/// When `Zoning` is set, this is added  as a component added to terrain ghosts, causing them to be manipulated by units.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TerraformingAction {
    /// Raise the height of this tile once
    Raise,
    /// Lower the height of this tile once
    Lower,
    /// Set the desired terrain material of this tile
    Change(Id<Terrain>),
}

impl TerraformingAction {
    /// The number of items needed to perform each action.
    const N_ITEMS: u32 = 3;

    /// The items needed to perform this action.
    pub(crate) fn input_inventory(&self) -> InputInventory {
        // TODO: vary these inventories based on the terrain type
        let soil_id = Id::<Item>::from_name("soil".to_string());

        match self {
            Self::Raise => InputInventory::Exact {
                inventory: Inventory::new_from_item(soil_id, Self::N_ITEMS),
            },
            Self::Lower => InputInventory::NULL,
            Self::Change(_terrain) => InputInventory::Exact {
                inventory: Inventory::new_from_item(soil_id, Self::N_ITEMS),
            },
        }
    }

    /// The items that must be taken away to perform this action.
    pub(crate) fn output_inventory(&self) -> OutputInventory {
        // TODO: vary these inventories based on the terrain type
        let soil_id = Id::<Item>::from_name("soil".to_string());

        match self {
            Self::Raise => OutputInventory::NULL,
            Self::Lower => OutputInventory {
                inventory: Inventory::full_from_item(soil_id, Self::N_ITEMS),
            },
            Self::Change(_terrain) => OutputInventory {
                inventory: Inventory::full_from_item(soil_id, Self::N_ITEMS),
            },
        }
    }

    /// The pretty formatted name of this action.
    pub(crate) fn display(&self, terrain_manifest: &TerrainManifest) -> String {
        match self {
            Self::Raise => "Raise".to_string(),
            Self::Lower => "Lower".to_string(),
            Self::Change(terrain_id) => terrain_manifest.name(*terrain_id).to_string(),
        }
    }
}

impl From<TerraformingTool> for TerraformingAction {
    fn from(choice: TerraformingTool) -> Self {
        match choice {
            TerraformingTool::Raise => Self::Raise,
            TerraformingTool::Lower => Self::Lower,
            TerraformingTool::Change(terrain) => Self::Change(terrain),
        }
    }
}

impl From<TerraformingAction> for TerraformingTool {
    fn from(action: TerraformingAction) -> Self {
        match action {
            TerraformingAction::Raise => Self::Raise,
            TerraformingAction::Lower => Self::Lower,
            TerraformingAction::Change(terrain) => Self::Change(terrain),
        }
    }
}

/// Manages the progression of terraforming actions, cleaning them up when they are complete.
pub(super) fn terraforming_lifecycle(
    mut terrain_query: Query<(
        Entity,
        &InputInventory,
        &OutputInventory,
        &VoxelPos,
        &TerraformingAction,
    )>,
    mut commands: Commands,
) {
    for (terrain_entity, input_inventory, output_inventory, &voxel_pos, &terraforming_action) in
        terrain_query.iter_mut()
    {
        if input_inventory.inventory().is_full() && output_inventory.is_empty() {
            commands
                .entity(terrain_entity)
                .remove::<(TerraformingAction, InputInventory, OutputInventory)>();

            // TODO: remove any visual effects
            // commands.despawn_ghost_terrain(voxel_pos);
            commands.apply_terraforming_action(voxel_pos, terraforming_action);
        }
    }
}

/// Computes the correct signals for terraformed terrain to send throughout their lifecycle
pub(super) fn terraforming_signals(
    mut query: Query<(&InputInventory, &OutputInventory, &mut Emitter), With<TerraformingAction>>,
) {
    /// The signal strength for terraforming signals
    const TERRAFORMING_SIGNAL_STRENGTH: f32 = 20.;

    for (input_inventory, output_inventory, mut emitter) in query.iter_mut() {
        // Reset all emitters
        emitter.signals.clear();

        // If the input inventory is not full, emit a pull signal for the item
        match input_inventory {
            InputInventory::Exact { inventory } => {
                // Emit signals to cause workers to bring the correct item to this ghost
                for item_slot in inventory.iter() {
                    let signal_type = SignalType::Pull(ItemKind::Single(item_slot.item_id()));
                    let signal_strength = SignalStrength::new(TERRAFORMING_SIGNAL_STRENGTH);
                    emitter.signals.push((signal_type, signal_strength))
                }
            }
            InputInventory::Tagged { tag, .. } => {
                // Emit signals to cause workers to bring the correct item to this ghost
                let signal_type = SignalType::Pull(ItemKind::Tag(*tag));
                let signal_strength = SignalStrength::new(TERRAFORMING_SIGNAL_STRENGTH);
                emitter.signals.push((signal_type, signal_strength))
            }
        }

        // If the output inventory is not empty, emit a push signal for the item
        for item_slot in output_inventory.iter() {
            let signal_type = SignalType::Push(ItemKind::Single(item_slot.item_id()));
            let signal_strength = SignalStrength::new(TERRAFORMING_SIGNAL_STRENGTH);
            emitter.signals.push((signal_type, signal_strength))
        }
    }
}

/// An extension trait for working with terraforming actions for [`Commands`].
pub trait TerraformingCommandsExt {
    /// Starts a [`TerraformingAction`] at the given hex.
    fn start_terraform(&mut self, hex: Hex, action: TerraformingAction);

    /// Previews a [`TerraformingAction`] at the given hex.
    fn preview_terraform(&mut self, hex: Hex, action: TerraformingAction);

    /// Removes any [`TerraformingAction`] at the given hex.
    fn remove_terraform(&mut self, hex: Hex);

    /// Completes the provided [`TerraformingAction`] at the given hex.
    fn complete_terraform(&mut self, hex: Hex, action: TerraformingAction);
}

impl TerraformingCommandsExt for Commands<'_, '_> {
    fn start_terraform(&mut self, hex: Hex, action: TerraformingAction) {
        todo!()
    }

    fn preview_terraform(&mut self, hex: Hex, action: TerraformingAction) {
        todo!()
    }

    fn remove_terraform(&mut self, hex: Hex) {
        todo!()
    }

    fn complete_terraform(&mut self, hex: Hex, action: TerraformingAction) {
        todo!()
    }
}

struct TerraformCommand {
    hex: Hex,
    action: TerraformingAction,
    preview: bool,
}

impl Command for TerraformCommand {
    fn write(self, world: &mut World) {
        todo!()
    }
}

struct DespawnTerraformCommand {
    hex: Hex,
}

impl Command for DespawnTerraformCommand {
    fn write(self, world: &mut World) {
        todo!()
    }
}
