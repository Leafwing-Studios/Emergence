//! Ghosts are translucent phantom structures, used to show structures that could be or are planned to be built.
//!
//! There is an important distinction between "ghosts" and "previews", even though they appear similarly to players.
//! Ghosts are buildings that are genuinely planned to be built.
//! Previews are simply hovered, and used as a visual aid to show placement.

use crate::crafting::components::{OutputInventory, WorkersPresent};
use crate::crafting::item_tags::ItemKind;
use crate::enum_iter::IterableEnum;
use crate::simulation::geometry::MapGeometry;
use crate::simulation::SimulationSet;
use crate::structures::commands::StructureCommandsExt;
use crate::structures::structure_manifest::{Structure, StructureManifest};
use crate::terrain::commands::TerrainCommandsExt;
use crate::terrain::terrain_manifest::TerrainManifest;
use crate::{self as emergence_lib, graphics::InheritedMaterial};
use bevy::prelude::*;
use bevy::utils::{Duration, HashMap};
use bevy_mod_raycast::RaycastMesh;
use emergence_macros::IterableEnum;

use crate::{
    asset_management::manifest::Id,
    crafting::components::{ActiveRecipe, CraftingState, InputInventory},
    player_interaction::clipboard::ClipboardData,
    signals::{Emitter, SignalStrength, SignalType},
    simulation::geometry::{Facing, TilePos},
};

use super::terraform::TerraformingAction;
use super::ConstructionStrategy;

/// Systems and resources for working with ghosts.
pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GhostHandles>().add_systems(
            (
                validate_ghost_structures,
                ghost_structure_signals.after(validate_ghost_structures),
                ghost_structure_lifecycle.after(validate_ghost_structures),
                ghost_terrain_signals,
                terraforming_lifecycle,
            )
                .in_set(SimulationSet)
                .in_schedule(CoreSchedule::FixedUpdate),
        );
    }
}

/// Stores the assets needed to render ghosts.
#[derive(Debug, Resource)]
pub(crate) struct GhostHandles {
    /// The materials used to render ghosts.
    materials: HashMap<GhostKind, Handle<StandardMaterial>>,
}

impl GhostHandles {
    /// Gets the material handle for the given [`GhostKind`].
    pub fn get_material(&self, kind: GhostKind) -> Handle<StandardMaterial> {
        self.materials[&kind].clone()
    }
}

impl FromWorld for GhostHandles {
    fn from_world(world: &mut World) -> Self {
        let mut map = HashMap::new();
        let mut assets = world.resource_mut::<Assets<StandardMaterial>>();
        for kind in GhostKind::variants() {
            let material = assets.add(kind.material());
            map.insert(kind, material);
        }
        GhostHandles { materials: map }
    }
}

/// A marker component that indicates that a structure or terrain element is planned to be built, rather than actually existing.
#[derive(Reflect, FromReflect, Component, Clone, Copy, Debug)]
pub(crate) struct Ghost;

/// A marker component indicating that this structure should be rendered in a transparent style.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghostly;

/// The components needed to create a functioning ghost of any kind.
#[derive(Bundle)]
struct GhostBundle {
    /// Marker component
    ghost: Ghost,
    /// The location of the ghost
    tile_pos: TilePos,
    /// The items required to actually seed this item
    construction_materials: InputInventory,
    /// The material to be used by all children in the scene
    inherited_material: InheritedMaterial,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
    /// Emits signals, drawing units towards this ghost to build it
    emitter: Emitter,
}

impl GhostBundle {
    /// Creates a new [`GhostBundle`].
    fn new(
        tile_pos: TilePos,
        construction_materials: InputInventory,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        GhostBundle {
            ghost: Ghost,
            tile_pos,
            construction_materials,
            inherited_material,
            scene_bundle: SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(world_pos),
                ..default()
            },
            emitter: Emitter::default(),
        }
    }
}

/// The set of components needed to spawn a ghost of a [`Structure`].
#[derive(Bundle)]
pub(crate) struct GhostStructureBundle {
    /// Shared components across all ghosts
    ghost_bundle: GhostBundle,
    /// The variety of structure
    structure_id: Id<Structure>,
    /// What should the structure craft when it is first built?
    active_recipe: ActiveRecipe,
    /// The direction the ghost is facing
    facing: Facing,
    /// Makes ghost structures pickable
    raycast_mesh: RaycastMesh<(Ghost, Structure)>,
    /// The mesh used for raycasting
    picking_mesh: Handle<Mesh>,
    /// The number of workers that are present / allowed to build this structure.
    workers_present: WorkersPresent,
    /// Tracks work that needs to be done on this building
    crafting_state: CraftingState,
}

impl GhostStructureBundle {
    /// Creates a new [`GhostStructureBundle`].
    pub(crate) fn new(
        tile_pos: TilePos,
        clipboard_data: ClipboardData,
        structure_manifest: &StructureManifest,
        picking_mesh: Handle<Mesh>,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        let structure_id = clipboard_data.structure_id;
        let construction_strategy = structure_manifest.construction_data(structure_id);
        let construction_materials = construction_strategy.materials.clone();

        GhostStructureBundle {
            ghost_bundle: GhostBundle::new(
                tile_pos,
                construction_materials,
                scene_handle,
                inherited_material,
                world_pos,
            ),
            facing: clipboard_data.facing,
            structure_id,
            active_recipe: clipboard_data.active_recipe,
            raycast_mesh: RaycastMesh::default(),
            picking_mesh,
            workers_present: WorkersPresent::new(6),
            crafting_state: CraftingState::NeedsInput,
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
        tile_pos: TilePos,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
        input_inventory: InputInventory,
        output_inventory: OutputInventory,
    ) -> Self {
        GhostTerrainBundle {
            ghost_bundle: GhostBundle::new(
                tile_pos,
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

/// The variety of ghost: this controls how it is rendered.
#[derive(IterableEnum, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum GhostKind {
    /// A structure that is going to be built.
    Ghost,
    /// A ghost, but currently selected
    SelectedGhost,
    /// A structure that players are holding in their clipboard and planning to place
    Preview,
    /// A preview that cannot be built in its current location
    ForbiddenPreview,
}

impl GhostKind {
    /// The material associated with each variety of ghost.
    pub(crate) fn material(&self) -> StandardMaterial {
        use crate::graphics::palette::infovis::{
            FORBIDDEN_PREVIEW_COLOR, GHOST_COLOR, PREVIEW_COLOR, SELECTED_GHOST_COLOR,
        };

        let base_color = match self {
            GhostKind::Ghost => GHOST_COLOR,
            GhostKind::SelectedGhost => SELECTED_GHOST_COLOR,
            GhostKind::Preview => PREVIEW_COLOR,
            GhostKind::ForbiddenPreview => FORBIDDEN_PREVIEW_COLOR,
        };

        StandardMaterial {
            base_color,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        }
    }
}

/// A marker component that indicates that this structure or terrain modification is planned to be built, rather than actually existing.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Preview;

/// The set of components needed to spawn a structure or terraforming preview.
#[derive(Bundle)]
pub(crate) struct PreviewBundle {
    /// Marker component
    preview: Preview,
    /// The location of the preview
    tile_pos: TilePos,
    /// The material to be used by all children in the scene
    inherited_material: InheritedMaterial,
    /// The child scene that contains the gltF model used
    scene_bundle: SceneBundle,
}

/// The components needed to create a preview of a [`Structure`].
#[derive(Bundle)]
pub(crate) struct StructurePreviewBundle {
    /// Shared components for all previews
    preview_bundle: PreviewBundle,
    /// The variety of structure
    structure_id: Id<Structure>,
    /// The direction the preview is facing
    facing: Facing,
}

impl StructurePreviewBundle {
    /// Creates a new [`StructurePreviewBundle`].
    pub(crate) fn new(
        tile_pos: TilePos,
        data: ClipboardData,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        StructurePreviewBundle {
            preview_bundle: PreviewBundle {
                preview: Preview,
                tile_pos,
                inherited_material,
                scene_bundle: SceneBundle {
                    scene: scene_handle.clone_weak(),
                    transform: Transform::from_translation(world_pos),
                    ..default()
                },
            },
            structure_id: data.structure_id,
            facing: data.facing,
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
        tile_pos: TilePos,
        terraforming_action: TerraformingAction,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        TerrainPreviewBundle {
            preview_bundle: PreviewBundle {
                preview: Preview,
                tile_pos,
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

/// Computes the correct signals for ghost structures to send throughout their lifecycle
pub(super) fn ghost_structure_signals(
    mut ghost_query: Query<
        (
            &Id<Structure>,
            &mut Emitter,
            Ref<CraftingState>,
            &InputInventory,
            &WorkersPresent,
        ),
        With<Ghost>,
    >,
) {
    for (&structure_id, mut emitter, crafting_state, input_inventory, workers_present) in
        ghost_query.iter_mut()
    {
        if crafting_state.is_changed() {
            // Reset any signals.
            emitter.signals.clear();

            match *crafting_state {
                CraftingState::NeedsInput => {
                    match input_inventory {
                        InputInventory::Exact { inventory } => {
                            // Emit signals to cause workers to bring the correct item to this ghost
                            for item_slot in inventory.iter() {
                                let signal_type =
                                    SignalType::Pull(ItemKind::Single(item_slot.item_id()));
                                let signal_strength = SignalStrength::new(10.);
                                emitter.signals.push((signal_type, signal_strength))
                            }
                        }
                        InputInventory::Tagged { tag, .. } => {
                            // Emit signals to cause workers to bring the correct item to this ghost
                            let signal_type = SignalType::Pull(ItemKind::Tag(*tag));
                            let signal_strength = SignalStrength::new(10.);
                            emitter.signals.push((signal_type, signal_strength))
                        }
                    }
                }
                CraftingState::InProgress {
                    progress: _,
                    required: _,
                } => {
                    if workers_present.needs_more() {
                        let workplace_id = WorkplaceId::structure(structure_id);

                        let signal_type = SignalType::Work(workplace_id);
                        let signal_strength = SignalStrength::new(10.);
                        emitter.signals.push((signal_type, signal_strength))
                    }
                }
                _ => (),
            }
        }
    }
}

/// Computes the correct signals for ghost terrain to send throughout their lifecycle
fn ghost_terrain_signals(
    mut query: Query<
        (&InputInventory, &OutputInventory, &mut Emitter),
        (With<Ghost>, With<TerraformingAction>),
    >,
) {
    for (input_inventory, output_inventory, mut emitter) in query.iter_mut() {
        // Reset all emitters
        emitter.signals.clear();

        // If the input inventory is not full, emit a pull signal for the item
        match input_inventory {
            InputInventory::Exact { inventory } => {
                // Emit signals to cause workers to bring the correct item to this ghost
                for item_slot in inventory.iter() {
                    let signal_type = SignalType::Pull(ItemKind::Single(item_slot.item_id()));
                    let signal_strength = SignalStrength::new(10.);
                    emitter.signals.push((signal_type, signal_strength))
                }
            }
            InputInventory::Tagged { tag, .. } => {
                // Emit signals to cause workers to bring the correct item to this ghost
                let signal_type = SignalType::Pull(ItemKind::Tag(*tag));
                let signal_strength = SignalStrength::new(10.);
                emitter.signals.push((signal_type, signal_strength))
            }
        }

        // If the output inventory is not empty, emit a push signal for the item
        for item_slot in output_inventory.iter() {
            let signal_type = SignalType::Push(ItemKind::Single(item_slot.item_id()));
            let signal_strength = SignalStrength::new(10.);
            emitter.signals.push((signal_type, signal_strength))
        }
    }
}

/// An identifier for a workplace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorkplaceId {
    /// This workplace is a structure
    Structure(Id<Structure>),
    /// This workplace is a terrain modification
    Terrain(TerraformingAction),
}

impl WorkplaceId {
    /// Creates a new [`WorkplaceId`].
    ///
    /// This is typically constructed via an [`AnyOf`] query.
    ///
    /// # Panics
    /// Panics if both `structure_id` and `terrain_id` are `Some`.
    pub(crate) fn new(ids: (Option<&Id<Structure>>, Option<&TerraformingAction>)) -> Self {
        match ids {
            (Some(structure_id), None) => WorkplaceId::Structure(*structure_id),
            (None, Some(terraforming_choice)) => WorkplaceId::Terrain(*terraforming_choice),
            _ => panic!("Workplace must be either a terrain XOR a structure"),
        }
    }

    /// Creates a new [`WorkplaceId`] from a [`Id<Structure>`].
    pub(crate) fn structure(structure_id: Id<Structure>) -> Self {
        WorkplaceId::Structure(structure_id)
    }

    /// Creates a new [`WorkplaceId`] from a [`TerraformingAction`].
    #[allow(dead_code)]
    pub(crate) fn terrain(terraforming_choice: TerraformingAction) -> Self {
        WorkplaceId::Terrain(terraforming_choice)
    }

    /// Returns the pretty name of this workplace.
    pub(crate) fn name(
        &self,
        structure_manifest: &StructureManifest,
        terrain_manifest: &TerrainManifest,
    ) -> String {
        match self {
            WorkplaceId::Structure(structure_id) => {
                structure_manifest.name(*structure_id).to_string()
            }
            WorkplaceId::Terrain(terraforming_action) => {
                terraforming_action.display(terrain_manifest)
            }
        }
    }
}

/// Manages the progression of structure ghosts from input needed -> work needed -> built.
///
/// Transforms ghosts into structures once all of their construction materials have been supplied and enough work has been performed.
pub(super) fn ghost_structure_lifecycle(
    mut ghost_query: Query<
        (
            &mut CraftingState,
            &InputInventory,
            &TilePos,
            &Id<Structure>,
            &Facing,
            &ActiveRecipe,
            &WorkersPresent,
        ),
        With<Ghost>,
    >,
    structure_manifest: Res<StructureManifest>,
    time: Res<FixedTime>,
    mut commands: Commands,
) {
    for (
        mut crafting_state,
        input_inventory,
        &tile_pos,
        &structure_id,
        &facing,
        active_recipe,
        workers_present,
    ) in ghost_query.iter_mut()
    {
        let construction_data = structure_manifest.construction_data(structure_id);

        match *crafting_state {
            CraftingState::NeedsInput => {
                *crafting_state = match input_inventory.inventory().is_full() {
                    true => CraftingState::InProgress {
                        progress: Duration::ZERO,
                        required: construction_data.work.unwrap_or_default(),
                    },
                    false => CraftingState::NeedsInput,
                };
            }
            CraftingState::InProgress { progress, required } => {
                let mut updated_progress = progress;

                // Scale construction speed linearly with the number of workers present (and vigor)
                updated_progress += workers_present.effective_workers() as u32 * time.period;

                *crafting_state = if updated_progress >= required {
                    CraftingState::RecipeComplete
                } else {
                    CraftingState::InProgress {
                        progress: updated_progress,
                        required,
                    }
                }
            }
            CraftingState::RecipeComplete => {
                commands.despawn_ghost_structure(tile_pos);

                // Spawn the seedling form of a structure if any
                if let ConstructionStrategy::Seedling(seedling) =
                    structure_manifest.get(structure_id).construction_strategy
                {
                    commands.spawn_structure(
                        tile_pos,
                        ClipboardData {
                            structure_id: seedling,
                            facing,
                            active_recipe: active_recipe.clone(),
                        },
                    );
                } else {
                    commands.spawn_structure(
                        tile_pos,
                        ClipboardData {
                            structure_id,
                            facing,
                            active_recipe: active_recipe.clone(),
                        },
                    );
                }
            }
            _ => unreachable!(),
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
            &TilePos,
            &TerraformingAction,
        ),
        With<Ghost>,
    >,
    mut commands: Commands,
) {
    for (input_inventory, output_inventory, &tile_pos, &terraforming_action) in
        terraforming_ghost_query.iter_mut()
    {
        if input_inventory.inventory().is_full() && output_inventory.is_empty() {
            commands.despawn_ghost_terrain(tile_pos);
            commands.apply_terraforming_action(tile_pos, terraforming_action);
        }
    }
}

/// Ensures that all ghosts can be built.
pub(super) fn validate_ghost_structures(
    map_geometry: Res<MapGeometry>,
    ghost_query: Query<(&TilePos, &Id<Structure>, &Facing), With<Ghost>>,
    structure_manifest: Res<StructureManifest>,
    mut commands: Commands,
) {
    // We only need to validate this when the map geometry changes.
    if !map_geometry.is_changed() {
        return;
    }

    for (&tile_pos, &structure_id, &facing) in ghost_query.iter() {
        let structure_details = structure_manifest.get(structure_id);
        let footprint = structure_details.footprint.rotated(facing);

        if !map_geometry.can_build(tile_pos, &footprint) {
            commands.despawn_ghost_structure(tile_pos);
        }
    }
}
