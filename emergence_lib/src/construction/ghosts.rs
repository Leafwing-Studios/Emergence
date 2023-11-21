//! Ghosts are translucent phantom structures, used to show structures that could be or are planned to be built.
//!
//! There is an important distinction between "ghosts" and "previews", even though they appear similarly to players.
//! Ghosts are buildings that are genuinely planned to be built.
//! Previews are simply hovered, and used as a visual aid to show placement.

use crate::crafting::item_tags::ItemKind;
use crate::crafting::recipe::ActiveRecipe;
use crate::crafting::workers::WorkersPresent;
use crate::enum_iter::IterableEnum;
use crate::geometry::MapGeometry;
use crate::organisms::energy::StartingEnergy;
use crate::player_interaction::picking::PickableVoxel;
use crate::simulation::SimulationSet;
use crate::structures::commands::StructureCommandsExt;
use crate::structures::structure_manifest::{Structure, StructureManifest};
use crate::terrain::terrain_manifest::TerrainManifest;
use crate::{self as emergence_lib, graphics::InheritedMaterial};
use bevy::prelude::*;
use bevy::utils::{Duration, HashMap};
use bevy_mod_raycast::RaycastMesh;
use emergence_macros::IterableEnum;

use crate::{
    asset_management::manifest::Id,
    crafting::inventories::{CraftingState, InputInventory},
    geometry::{Facing, VoxelPos},
    player_interaction::clipboard::ClipboardData,
    signals::{Emitter, SignalStrength, SignalType},
};

use super::terraform::TerraformingAction;
use super::ConstructionStrategy;

/// Systems and resources for working with ghosts.
pub struct GhostPlugin;

impl Plugin for GhostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GhostHandles>().add_systems(
            FixedUpdate,
            (
                validate_ghost_structures,
                ghost_structure_signals.after(validate_ghost_structures),
                ghost_structure_lifecycle.after(validate_ghost_structures),
            )
                .in_set(SimulationSet),
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
#[derive(Reflect, Component, Clone, Copy, Debug)]
pub(crate) struct Ghost;

/// A marker component indicating that this structure should be rendered in a transparent style.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct Ghostly;

/// The components needed to create a functioning ghost of any kind.
#[derive(Bundle)]
pub(super) struct GhostBundle {
    /// Marker component
    ghost: Ghost,
    /// The location of the ghost
    voxel_pos: VoxelPos,
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
    pub(super) fn new(
        voxel_pos: VoxelPos,
        construction_materials: InputInventory,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        GhostBundle {
            ghost: Ghost,
            voxel_pos,
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
    raycast_mesh: RaycastMesh<PickableVoxel>,
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
        voxel_pos: VoxelPos,
        clipboard_data: ClipboardData,
        structure_manifest: &StructureManifest,
        picking_mesh: Handle<Mesh>,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        let structure_id = clipboard_data.structure_id;
        let construction_strategy = structure_manifest.construction_data(structure_id);
        let construction_materials = construction_strategy.unwrap().materials.clone();

        GhostStructureBundle {
            ghost_bundle: GhostBundle::new(
                voxel_pos,
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
///
/// Unlike [`GhostStructureBundle`], these are not pickable.
/// Instead, they are selected by the player by selecting the associated terrain.
#[derive(Bundle)]
pub(crate) struct GhostTerraformBundle {
    /// The type of terraforming action being performed
    terraforming_action: TerraformingAction,
    /// Marks this entity as a ghost
    ghost: Ghost,
    /// The material to be used by all children in the scene
    pub(super) inherited_material: InheritedMaterial,
    /// The child scene that contains the gltF model used
    pub(super) scene_bundle: SceneBundle,
    /// The number of workers that are present / allowed to build this structure.
    workers_present: WorkersPresent,
    /// Tracks work that needs to be done on this building
    crafting_state: CraftingState,
}

impl GhostTerraformBundle {
    /// Creates a new [`GhostTerraformBundle`].
    pub(crate) fn new(
        terraforming_action: TerraformingAction,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        GhostTerraformBundle {
            terraforming_action,
            ghost: Ghost,
            inherited_material,
            scene_bundle: SceneBundle {
                scene: scene_handle.clone_weak(),
                transform: Transform::from_translation(world_pos),
                ..default()
            },
            workers_present: WorkersPresent::new(6),
            crafting_state: CraftingState::NeedsInput,
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
    pub(super) preview: Preview,
    /// The material to be used by all children in the scene
    pub(super) inherited_material: InheritedMaterial,
    /// The child scene that contains the gltF model used
    pub(super) scene_bundle: SceneBundle,
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
        data: ClipboardData,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        StructurePreviewBundle {
            preview_bundle: PreviewBundle {
                preview: Preview,
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

/// The set of components needed to spawn a ghost of a [`TerraformingAction`].
///
/// Unlike [`GhostStructureBundle`], these are not pickable.
/// Instead, they are selected by the player by selecting the associated terrain.
#[derive(Bundle)]
pub(crate) struct TerraformPreviewBundle {
    /// The type of terraforming action being performed
    terraforming_action: TerraformingAction,
    /// Shared components for all previews
    preview_bundle: PreviewBundle,
}

impl TerraformPreviewBundle {
    /// Creates a new [`TerraformPreviewBundle`].
    pub(crate) fn new(
        terraforming_action: TerraformingAction,
        scene_handle: Handle<Scene>,
        inherited_material: InheritedMaterial,
        world_pos: Vec3,
    ) -> Self {
        TerraformPreviewBundle {
            preview_bundle: PreviewBundle {
                preview: Preview,
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
    /// Controls how strong the signals that are emitted by ghosts are
    const GHOST_SIGNAL_STRENGTH: f32 = 100.;

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
                                let signal_strength = SignalStrength::new(GHOST_SIGNAL_STRENGTH);
                                emitter.signals.push((signal_type, signal_strength))
                            }
                        }
                        InputInventory::Tagged { tag, .. } => {
                            // Emit signals to cause workers to bring the correct item to this ghost
                            let signal_type = SignalType::Pull(ItemKind::Tag(*tag));
                            let signal_strength = SignalStrength::new(GHOST_SIGNAL_STRENGTH);
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
                        let signal_strength = SignalStrength::new(GHOST_SIGNAL_STRENGTH);
                        emitter.signals.push((signal_type, signal_strength))
                    }
                }
                _ => (),
            }
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
            &VoxelPos,
            &Id<Structure>,
            &Facing,
            &ActiveRecipe,
            &WorkersPresent,
        ),
        With<Ghost>,
    >,
    structure_manifest: Res<StructureManifest>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (
        mut crafting_state,
        input_inventory,
        &center,
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
                        required: construction_data.unwrap().work.unwrap_or_default(),
                    },
                    false => CraftingState::NeedsInput,
                };
            }
            CraftingState::InProgress { progress, required } => {
                let mut updated_progress = progress;

                // Scale construction speed linearly with the number of workers present (and vigor)
                updated_progress += workers_present.effective_workers() as u32 * time.delta();

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
                let structure_data = structure_manifest.get(structure_id);

                for &voxel_pos in structure_data.footprint.normalized(facing, center).iter() {
                    commands.despawn_ghost_structure(voxel_pos);
                }

                // Spawn the seedling form of a structure if any
                if let ConstructionStrategy::Seedling(seedling) =
                    structure_manifest.get(structure_id).construction_strategy
                {
                    commands.spawn_structure(
                        center,
                        ClipboardData {
                            structure_id: seedling,
                            facing,
                            active_recipe: active_recipe.clone(),
                        },
                        StartingEnergy::Full,
                    );
                } else {
                    commands.spawn_structure(
                        center,
                        ClipboardData {
                            structure_id,
                            facing,
                            active_recipe: active_recipe.clone(),
                        },
                        StartingEnergy::NotAnOrganism,
                    );
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Ensures that all ghosts can be built.
pub(super) fn validate_ghost_structures(
    map_geometry: Res<MapGeometry>,
    ghost_query: Query<(&VoxelPos, &Id<Structure>, &Facing), With<Ghost>>,
    structure_manifest: Res<StructureManifest>,
    mut commands: Commands,
) {
    // We only need to validate this when the map geometry changes.
    if !map_geometry.is_changed() {
        return;
    }

    for (&voxel_pos, &structure_id, &facing) in ghost_query.iter() {
        let structure_details = structure_manifest.get(structure_id);

        if map_geometry
            .is_space_available(voxel_pos, &structure_details.footprint, facing)
            .is_ok()
        {
            commands.despawn_ghost_structure(voxel_pos);
        }
    }
}
