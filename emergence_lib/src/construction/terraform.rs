//! Tools to alter the terrain type and height.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    crafting::{
        inventories::{InputInventory, OutputInventory},
        item_tags::ItemKind,
    },
    geometry::VoxelPos,
    graphics::InheritedMaterial,
    items::{inventory::Inventory, item_manifest::Item},
    signals::{Emitter, SignalStrength, SignalType},
    terrain::{
        commands::TerrainCommandsExt,
        terrain_manifest::{Terrain, TerrainManifest},
    },
};

use super::ghosts::{Ghost, GhostBundle, Preview, PreviewBundle};

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

/// The set of components needed to spawn a ghost of a [`TerraformingAction`].
#[derive(Bundle)]
pub(crate) struct GhostTerrainBundle {
    /// Shared components across all ghosts
    ghost_bundle: GhostBundle,
    /// The action that will be performed when this terrain is built
    terraforming_action: TerraformingAction,
    /// The inventory that holds any material that needs to be taken away
    output_inventory: OutputInventory,
}

impl GhostTerrainBundle {
    /// Creates a new [`GhostTerrainBundle`].
    pub(crate) fn new(
        terraforming_action: TerraformingAction,
        voxel_pos: VoxelPos,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
        input_inventory: InputInventory,
        output_inventory: OutputInventory,
    ) -> Self {
        GhostTerrainBundle {
            ghost_bundle: GhostBundle::new(
                voxel_pos,
                input_inventory,
                scene_handle,
                inherited_material,
                world_pos,
            ),
            terraforming_action,
            output_inventory,
        }
    }
}

/// The components needed to create a preview of a [`TerraformingAction`].
#[derive(Bundle)]
pub(crate) struct TerrainPreviewBundle {
    /// Shared components for all previews
    preview_bundle: PreviewBundle,
    /// The action that will be performed when this terrain is built
    terraforming_action: TerraformingAction,
}

impl TerrainPreviewBundle {
    /// Creates a new [`TerrainPreviewBundle`].
    pub(crate) fn new(
        voxel_pos: VoxelPos,
        terraforming_action: TerraformingAction,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        TerrainPreviewBundle {
            preview_bundle: PreviewBundle {
                preview: Preview,
                voxel_pos,
                inherited_material,
                scene_bundle: SceneBundle {
                    scene: scene_handle.clone_weak(),
                    transform: Transform::from_translation(world_pos),
                    ..default()
                },
            },
            terraforming_action,
        }
    }
}

/// Manages the progression of terraforming ghosts.
///
/// Transforms ghosts into terrain once all of their inputs and outputs have been met.
pub(super) fn terraforming_lifecycle(
    mut terraforming_ghost_query: Query<
        (
            &InputInventory,
            &OutputInventory,
            &VoxelPos,
            &TerraformingAction,
        ),
        With<Ghost>,
    >,
    mut commands: Commands,
) {
    for (input_inventory, output_inventory, &voxel_pos, &terraforming_action) in
        terraforming_ghost_query.iter_mut()
    {
        if input_inventory.inventory().is_full() && output_inventory.is_empty() {
            commands.despawn_ghost_terrain(voxel_pos);
            commands.apply_terraforming_action(voxel_pos, terraforming_action);
        }
    }
}

/// Computes the correct signals for ghost terrain to send throughout their lifecycle
pub(super) fn ghost_terrain_signals(
    mut query: Query<
        (&InputInventory, &OutputInventory, &mut Emitter),
        (With<Ghost>, With<TerraformingAction>),
    >,
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
