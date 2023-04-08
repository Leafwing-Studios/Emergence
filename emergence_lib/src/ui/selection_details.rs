//! Displays information about the currently selected object(s).

use bevy::{ecs::query::QueryEntityError, prelude::*};

use crate::{
    asset_management::AssetState,
    crafting::recipe::RecipeManifest,
    items::item_manifest::ItemManifest,
    player_interaction::{
        camera::{CameraMode, CameraSettings},
        selection::CurrentSelection,
        InteractionSystem,
    },
    signals::Signals,
    simulation::geometry::MapGeometry,
    structures::structure_manifest::StructureManifest,
    terrain::terrain_manifest::TerrainManifest,
    units::unit_manifest::UnitManifest,
};

use self::{
    ghost_structure_details::{GhostStructureDetails, GhostStructureDetailsQuery},
    organism_details::{OrganismDetails, OrganismDetailsQuery},
    structure_details::{CraftingDetails, StructureDetails, StructureDetailsQuery},
    terrain_details::{
        GhostTerrainDetailsQuery, TerraformingDetails, TerrainDetails, TerrainDetailsQuery,
    },
    unit_details::{UnitDetails, UnitDetailsQuery},
};

use super::{FiraSansFontFamily, RightPanel};

/// Initializes and updates the selection details panel.
pub(super) struct SelectionDetailsPlugin;

impl Plugin for SelectionDetailsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionDetails>()
            .add_startup_system(populate_selection_panel)
            .add_system(
                get_details
                    .pipe(clear_details_on_error)
                    .after(InteractionSystem::SelectTiles)
                    .before(update_selection_details)
                    .run_if(in_state(AssetState::Ready)),
            )
            .add_system(change_camera_mode.after(update_selection_details))
            .add_system(update_selection_details.run_if(in_state(AssetState::Ready)));
    }
}

/// The root node for the selection panel.
#[derive(Component)]
struct SelectionPanel;

/// The UI node that stores all ghost structure details.
#[derive(Component, Default)]
struct GhostStructureDetailsMarker;

/// The UI node that stores all structure details.
#[derive(Component, Default)]
struct StructureDetailsMarker;

/// The UI node that stores all terrain details.
#[derive(Component, Default)]
struct TerrainDetailsMarker;

/// The UI node that stores all unit details.
#[derive(Component, Default)]
struct UnitDetailsMarker;

/// Estabilishes UI elements for selection details panel.
fn populate_selection_panel(
    mut commands: Commands,
    font_family: Res<FiraSansFontFamily>,
    parent_query: Query<Entity, With<RightPanel>>,
) {
    let key_text_style = TextStyle {
        color: Color::rgb(0.9, 0.9, 0.9),
        font: font_family.regular.clone_weak(),
        font_size: 20.,
    };

    let right_panel = parent_query.single();

    let selection = commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Px(500.)),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                background_color: Color::rgba(0., 0., 0., 0.9).into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            SelectionPanel,
        ))
        .id();

    let ghost_structure_details =
        populate_details::<GhostStructureDetailsMarker>(&mut commands, &key_text_style);
    let structure_details =
        populate_details::<StructureDetailsMarker>(&mut commands, &key_text_style);
    let terrain_details = populate_details::<TerrainDetailsMarker>(&mut commands, &key_text_style);
    let unit_details = populate_details::<UnitDetailsMarker>(&mut commands, &key_text_style);

    commands.entity(right_panel).add_child(selection);
    commands
        .entity(selection)
        .add_child(ghost_structure_details)
        .add_child(structure_details)
        .add_child(terrain_details)
        .add_child(unit_details);
}

/// Changes the camera mode when the "follow unit" button is pressed.
fn change_camera_mode(
    mut camera_query: Query<&mut CameraSettings>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // FIXME: This should be a button press, not a key press.
    if keyboard_input.just_pressed(KeyCode::C) {
        let mut camera_settings = camera_query.single_mut();
        camera_settings.camera_mode = match camera_settings.camera_mode {
            CameraMode::FollowUnit => CameraMode::Free,
            CameraMode::Free => CameraMode::FollowUnit,
        };
    }
}

/// Updates UI elements for selection details panel based on new information.
fn update_selection_details(
    selection_details: Res<SelectionDetails>,
    mut selection_panel_query: Query<&mut Visibility, With<SelectionPanel>>,
    mut ghost_structures_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<GhostStructureDetailsMarker>,
            Without<StructureDetailsMarker>,
            Without<TerrainDetailsMarker>,
            Without<UnitDetailsMarker>,
        ),
    >,
    mut structure_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<StructureDetailsMarker>,
            Without<GhostStructureDetailsMarker>,
            Without<TerrainDetailsMarker>,
            Without<UnitDetailsMarker>,
        ),
    >,
    mut unit_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<UnitDetailsMarker>,
            Without<GhostStructureDetailsMarker>,
            Without<StructureDetailsMarker>,
            Without<TerrainDetailsMarker>,
        ),
    >,
    mut terrain_details_query: Query<
        (&mut Style, &mut Text),
        (
            With<TerrainDetailsMarker>,
            Without<GhostStructureDetailsMarker>,
            Without<StructureDetailsMarker>,
            Without<UnitDetailsMarker>,
        ),
    >,
    structure_manifest: Res<StructureManifest>,
    unit_manifest: Res<UnitManifest>,
    terrain_manifest: Res<TerrainManifest>,
    recipe_manifest: Res<RecipeManifest>,
    item_manifest: Res<ItemManifest>,
) {
    let mut parent_visibility = selection_panel_query.single_mut();
    let (mut ghost_structure_style, mut ghost_structure_text) =
        ghost_structures_details_query.single_mut();
    let (mut structure_style, mut structure_text) = structure_details_query.single_mut();
    let (mut unit_style, mut unit_text) = unit_details_query.single_mut();
    let (mut terrain_style, mut terrain_text) = terrain_details_query.single_mut();

    match *selection_details {
        SelectionDetails::GhostStructure(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_structure_style.display = Display::Flex;
            structure_style.display = Display::None;
            terrain_style.display = Display::None;
            unit_style.display = Display::None;
        }
        SelectionDetails::Structure(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_structure_style.display = Display::None;
            structure_style.display = Display::Flex;
            terrain_style.display = Display::None;
            unit_style.display = Display::None;
        }
        SelectionDetails::Terrain(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_structure_style.display = Display::None;
            structure_style.display = Display::None;
            terrain_style.display = Display::Flex;
            unit_style.display = Display::None;
        }
        SelectionDetails::Unit(_) => {
            *parent_visibility = Visibility::Visible;
            ghost_structure_style.display = Display::None;
            structure_style.display = Display::None;
            terrain_style.display = Display::None;
            unit_style.display = Display::Flex;
        }
        SelectionDetails::None => {
            // Don't bother messing with Display here to avoid triggering a pointless relayout
            *parent_visibility = Visibility::Hidden;
        }
    }

    match &*selection_details {
        SelectionDetails::GhostStructure(details) => {
            ghost_structure_text.sections[0].value =
                details.display(&item_manifest, &structure_manifest, &recipe_manifest);
        }
        SelectionDetails::Structure(details) => {
            structure_text.sections[0].value =
                details.display(&structure_manifest, &unit_manifest, &item_manifest);
        }
        SelectionDetails::Terrain(details) => {
            terrain_text.sections[0].value = details.display(
                &terrain_manifest,
                &structure_manifest,
                &item_manifest,
                &unit_manifest,
            );
        }
        SelectionDetails::Unit(details) => {
            unit_text.sections[0].value = details.display(
                &unit_manifest,
                &item_manifest,
                &structure_manifest,
                &terrain_manifest,
            );
        }
        SelectionDetails::None => (),
    };
}

/// Generates the details node with the marker component `T` and its children.
///
/// The returned [`Entity`] is for the root node.
fn populate_details<T: Component + Default>(
    commands: &mut Commands,
    key_text_style: &TextStyle,
) -> Entity {
    commands
        .spawn((
            TextBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                text: Text::from_section("", key_text_style.clone()),
                ..default()
            },
            T::default(),
        ))
        .id()
}

/// Detailed info about the selected organism.
#[derive(Debug, Resource, Default)]
pub(crate) enum SelectionDetails {
    /// A ghost of a structure is selected
    GhostStructure(GhostStructureDetails),
    /// A structure is selected
    Structure(StructureDetails),
    /// A tile is selected.
    Terrain(TerrainDetails),
    /// A unit is selected
    Unit(UnitDetails),
    /// Nothing is selected
    #[default]
    None,
}

/// Get details about the selected object(s).
fn get_details(
    selection_type: Res<CurrentSelection>,
    mut selection_details: ResMut<SelectionDetails>,
    ghost_structure_query: Query<GhostStructureDetailsQuery>,
    ghost_terrain_query: Query<GhostTerrainDetailsQuery>,
    organism_query: Query<OrganismDetailsQuery>,
    structure_query: Query<StructureDetailsQuery>,
    terrain_query: Query<TerrainDetailsQuery>,
    unit_query: Query<UnitDetailsQuery>,
    map_geometry: Res<MapGeometry>,
    structure_manifest: Res<StructureManifest>,
    unit_manifest: Res<UnitManifest>,
    recipe_manifest: Res<RecipeManifest>,
    signals: Res<Signals>,
) -> Result<(), QueryEntityError> {
    *selection_details = match &*selection_type {
        CurrentSelection::GhostStructure(ghost_structure_entity) => {
            let ghost_query_item = ghost_structure_query.get(*ghost_structure_entity)?;
            SelectionDetails::GhostStructure(GhostStructureDetails {
                entity: *ghost_structure_entity,
                tile_pos: *ghost_query_item.tile_pos,
                structure_id: *ghost_query_item.structure_id,
                input_inventory: ghost_query_item.input_inventory.clone(),
                crafting_state: ghost_query_item.crafting_state.clone(),
                active_recipe: ghost_query_item.active_recipe.clone(),
            })
        }
        CurrentSelection::Structure(structure_entity) => {
            let structure_query_item = structure_query.get(*structure_entity)?;

            let crafting_details =
                if let Some((input, output, active_recipe, workers_present, state)) =
                    structure_query_item.crafting
                {
                    let maybe_recipe_id = *active_recipe.recipe_id();
                    let recipe =
                        maybe_recipe_id.map(|recipe_id| recipe_manifest.get(recipe_id).clone());

                    Some(CraftingDetails {
                        input_inventory: input.inventory().clone(),
                        output_inventory: output.inventory.clone(),
                        recipe,
                        workers_present: workers_present.clone(),
                        state: state.clone(),
                    })
                } else {
                    None
                };

            // Not all structures are organisms
            let maybe_organism_details =
                organism_query
                    .get(*structure_entity)
                    .ok()
                    .map(|query_item| OrganismDetails {
                        prototypical_form: structure_manifest
                            .get(*structure_query_item.structure_id)
                            .organism_variety.as_ref()
                            .expect("All structures with organism components must be registered in the manifest as organisms")
                            .prototypical_form,
                        lifecycle: query_item.lifecycle.clone(),
                        energy_pool: query_item.energy_pool.clone(),
                    });

            SelectionDetails::Structure(StructureDetails {
                entity: structure_query_item.entity,
                tile_pos: *structure_query_item.tile_pos,
                structure_id: *structure_query_item.structure_id,
                crafting_details,
                maybe_organism_details,
                storage_inventory: structure_query_item.storage_inventory.cloned(),
                marked_for_removal: structure_query_item.marked_for_removal.is_some(),
            })
        }
        CurrentSelection::Terrain(selected_tiles) => {
            // FIXME: display info about multiple tiles correctly
            if let Some(tile_pos) = selected_tiles.selection().iter().next() {
                let terrain_entity = map_geometry.get_terrain(*tile_pos).unwrap();
                let terrain_query_item = terrain_query.get(terrain_entity)?;

                let maybe_terraforming_details = if let Some(ghost_terrain_entity) =
                    map_geometry.get_ghost_terrain(*tile_pos)
                {
                    let ghost_terrain_query_item = ghost_terrain_query.get(ghost_terrain_entity)?;
                    Some(TerraformingDetails {
                        terraforming_action: *ghost_terrain_query_item.terraforming_action,
                        input_inventory: ghost_terrain_query_item.input_inventory.clone(),
                        output_inventory: ghost_terrain_query_item.output_inventory.clone(),
                    })
                } else {
                    None
                };

                SelectionDetails::Terrain(TerrainDetails {
                    entity: terrain_entity,
                    terrain_id: *terrain_query_item.terrain_id,
                    tile_pos: *tile_pos,
                    height: *terrain_query_item.height,
                    signals: signals.all_signals_at_position(*tile_pos),
                    zoning: terrain_query_item.zoning.clone(),
                    maybe_terraforming_details,
                })
            } else {
                SelectionDetails::None
            }
        }
        CurrentSelection::Unit(unit_entity) => {
            let unit_query_item = unit_query.get(*unit_entity)?;
            // All units are organisms
            let organism_query_item = organism_query.get(*unit_entity)?;
            let organism_details = OrganismDetails {
                prototypical_form: unit_manifest
                    .get(*unit_query_item.unit_id)
                    .organism_variety
                    .prototypical_form,
                lifecycle: organism_query_item.lifecycle.clone(),
                energy_pool: organism_query_item.energy_pool.clone(),
            };

            let unit_data = unit_manifest.get(*unit_query_item.unit_id);

            SelectionDetails::Unit(UnitDetails {
                entity: unit_query_item.entity,
                unit_id: *unit_query_item.unit_id,
                diet: unit_data.diet.clone(),
                tile_pos: *unit_query_item.tile_pos,
                held_item: unit_query_item.held_item.clone(),
                goal: unit_query_item.goal.clone(),
                action: unit_query_item.action.clone(),
                impatience_pool: unit_query_item.impatience_pool.clone(),
                organism_details,
            })
        }
        CurrentSelection::None => SelectionDetails::None,
    };

    Ok(())
}

/// If something went wrong in [`get_details`], clear the selection.
pub(crate) fn clear_details_on_error(
    In(result): In<Result<(), QueryEntityError>>,
    mut current_selection: ResMut<CurrentSelection>,
    mut selection_details: ResMut<SelectionDetails>,
) {
    if result.is_err() {
        *current_selection = CurrentSelection::None;
        *selection_details = SelectionDetails::None;
    }
}

/// Details for ghost structures
mod ghost_structure_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use crate::{
        asset_management::manifest::Id,
        crafting::{
            components::{ActiveRecipe, CraftingState, InputInventory},
            recipe::RecipeManifest,
        },
        items::item_manifest::ItemManifest,
        signals::Emitter,
        simulation::geometry::TilePos,
        structures::structure_manifest::{Structure, StructureManifest},
    };

    /// Data needed to populate [`GhostStructureDetails`].
    #[derive(WorldQuery)]
    pub(super) struct GhostStructureDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of structure
        pub(super) structure_id: &'static Id<Structure>,
        /// The tile position of this ghost
        pub(crate) tile_pos: &'static TilePos,
        /// The inputs that must be added to construct this ghost
        pub(super) input_inventory: &'static InputInventory,
        /// The ghost's progress through construction
        pub(crate) crafting_state: &'static CraftingState,
        /// The signal emitter
        pub(super) emitter: &'static Emitter,
        /// The recipe that will be crafted when the structure is first built
        pub(super) active_recipe: &'static ActiveRecipe,
    }

    /// Detailed info about a given ghost.
    #[derive(Debug)]
    pub(crate) struct GhostStructureDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The tile position of this structure
        pub(crate) tile_pos: TilePos,
        /// The type of structure, e.g. plant or fungus.
        pub(crate) structure_id: Id<Structure>,
        /// The inputs that must be added to construct this ghost
        pub(super) input_inventory: InputInventory,
        /// The ghost's progress through construction
        pub(super) crafting_state: CraftingState,
        /// The recipe that will be crafted when the structure is first built
        pub(super) active_recipe: ActiveRecipe,
    }

    impl GhostStructureDetails {
        /// The pretty formatting for this type
        pub(crate) fn display(
            &self,
            item_manifest: &ItemManifest,
            structure_manifest: &StructureManifest,
            recipe_manifest: &RecipeManifest,
        ) -> String {
            let entity = self.entity;
            let structure_id = structure_manifest.name(self.structure_id);
            let tile_pos = &self.tile_pos;
            let crafting_state = &self.crafting_state;
            let recipe = self.active_recipe.display(recipe_manifest);
            let construction_materials = self.input_inventory.display(item_manifest);

            format!(
                "Entity: {entity:?}
Tile: {tile_pos}
Ghost structure type: {structure_id}
Recipe: {recipe}
Construction materials: {construction_materials}
{crafting_state}"
            )
        }
    }
}

/// Details for organisms
mod organism_details {
    use bevy::ecs::query::WorldQuery;

    use crate::{
        organisms::{energy::EnergyPool, lifecycle::Lifecycle, OrganismId},
        structures::structure_manifest::StructureManifest,
        units::unit_manifest::UnitManifest,
    };

    /// Data needed to populate [`OrganismDetails`].
    #[derive(WorldQuery)]
    pub(super) struct OrganismDetailsQuery {
        /// The organism's current progress through its lifecycle
        pub(super) lifecycle: &'static Lifecycle,
        /// The current and max energy
        pub(super) energy_pool: &'static EnergyPool,
    }

    /// Detailed info about a given organism.
    #[derive(Debug)]
    pub(crate) struct OrganismDetails {
        /// The prototypical "base" bersion of this orgnaism
        pub(super) prototypical_form: OrganismId,
        /// The organism's current progress through its lifecycle
        pub(super) lifecycle: Lifecycle,
        /// The current and max energy
        pub(super) energy_pool: EnergyPool,
    }

    impl OrganismDetails {
        /// Pretty formatting for this type
        pub(crate) fn display(
            &self,
            structure_manifest: &StructureManifest,
            unit_manifest: &UnitManifest,
        ) -> String {
            let prototypical_form = self
                .prototypical_form
                .display(structure_manifest, unit_manifest);
            let lifecycle = self.lifecycle.display(structure_manifest, unit_manifest);

            let energy_pool = &self.energy_pool;

            format!(
                "Prototypical form: {prototypical_form}
Lifecycle: {lifecycle}
Energy: {energy_pool}"
            )
        }
    }
}

/// Details for structures
mod structure_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use super::organism_details::OrganismDetails;
    use crate::{
        asset_management::manifest::Id,
        construction::demolition::MarkedForDemolition,
        crafting::{
            components::{
                ActiveRecipe, CraftingState, InputInventory, OutputInventory, StorageInventory,
                WorkersPresent,
            },
            recipe::RecipeData,
        },
        items::{inventory::Inventory, item_manifest::ItemManifest},
        simulation::geometry::TilePos,
        structures::structure_manifest::{Structure, StructureManifest},
        units::unit_manifest::UnitManifest,
    };

    /// Data needed to populate [`StructureDetails`].
    #[derive(WorldQuery)]
    pub(super) struct StructureDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of structure
        pub(super) structure_id: &'static Id<Structure>,
        /// The tile position of this structure
        pub(crate) tile_pos: &'static TilePos,
        /// Crafting-related components
        pub(super) crafting: Option<(
            &'static InputInventory,
            &'static OutputInventory,
            &'static ActiveRecipe,
            &'static WorkersPresent,
            &'static CraftingState,
        )>,
        /// If this structure stores things, its inventory.
        pub(super) storage_inventory: Option<&'static StorageInventory>,
        /// Is this structure marked for removal?
        pub(super) marked_for_removal: Option<&'static MarkedForDemolition>,
    }

    /// Detailed info about a given structure.
    #[derive(Debug)]
    pub(crate) struct StructureDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The tile position of this structure
        pub(crate) tile_pos: TilePos,
        /// The type of structure, e.g. plant or fungus.
        pub(crate) structure_id: Id<Structure>,
        /// If this structure is crafting something, the details about that.
        pub(crate) crafting_details: Option<CraftingDetails>,
        /// If this structure stores things, its inventory.
        pub(crate) storage_inventory: Option<StorageInventory>,
        /// Details about this organism, if it is one.
        pub(crate) maybe_organism_details: Option<OrganismDetails>,
        /// Is this structure slated for removal?
        pub(crate) marked_for_removal: bool,
    }

    impl StructureDetails {
        /// The pretty foramtting for this type
        pub(crate) fn display(
            &self,
            structure_manifest: &StructureManifest,
            unit_manifest: &UnitManifest,
            item_manifest: &ItemManifest,
        ) -> String {
            let entity = self.entity;
            let structure_id = structure_manifest.name(self.structure_id);
            let tile_pos = &self.tile_pos;

            let mut string = format!(
                "Entity: {entity:?}
Structure type: {structure_id}
Tile: {tile_pos}"
            );

            if self.marked_for_removal {
                string += "\nMarked for removal!";
            }

            if let Some(crafting) = &self.crafting_details {
                string += &format!("\n{}", crafting.display(item_manifest));
            }

            if let Some(storage) = &self.storage_inventory {
                string += &format!("\nStoring: {}", storage.display(item_manifest));
            }

            if let Some(organism) = &self.maybe_organism_details {
                string += &format!("\n{}", organism.display(structure_manifest, unit_manifest));
            };

            string
        }
    }

    /// The details about crafting processes.
    #[derive(Debug, Clone)]
    pub(crate) struct CraftingDetails {
        /// The inventory for the input items.
        pub(crate) input_inventory: Inventory,

        /// The inventory for the output items.
        pub(crate) output_inventory: Inventory,

        /// The recipe used, if any.
        pub(crate) recipe: Option<RecipeData>,

        /// The state of the ongoing crafting process.
        pub(crate) state: CraftingState,

        /// The number of workers that are presently working on this.
        pub(crate) workers_present: WorkersPresent,
    }

    impl CraftingDetails {
        /// The pretty formatting for this type.
        pub(crate) fn display(&self, item_manifest: &ItemManifest) -> String {
            let input_inventory = self.input_inventory.display(item_manifest);
            let output_inventory = self.output_inventory.display(item_manifest);
            let crafting_state = &self.state;

            let recipe_string = match &self.recipe {
                Some(recipe) => recipe.display(item_manifest),
                None => "None".to_string(),
            };

            let workers_present = &self.workers_present;

            format!(
                "Recipe: {recipe_string}
Input: {input_inventory}
{crafting_state}
Workers present: {workers_present}
Output: {output_inventory}"
            )
        }
    }
}

/// Details for terrain
mod terrain_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use crate::{
        asset_management::manifest::Id,
        construction::{terraform::TerraformingAction, zoning::Zoning},
        crafting::components::{InputInventory, OutputInventory},
        items::item_manifest::ItemManifest,
        signals::LocalSignals,
        simulation::geometry::{Height, TilePos},
        structures::structure_manifest::StructureManifest,
        terrain::terrain_manifest::{Terrain, TerrainManifest},
        units::unit_manifest::UnitManifest,
    };

    /// Data needed to populate [`TerrainDetails`].
    #[derive(WorldQuery)]
    pub(super) struct TerrainDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The height of the tile
        pub(super) height: &'static Height,
        /// The type of terrain
        pub(super) terrain_id: &'static Id<Terrain>,
        /// The zoning applied to this terrain
        pub(super) zoning: &'static Zoning,
    }

    /// Data needed to populate [`TerraformingDetails`].
    #[derive(WorldQuery)]
    pub(super) struct GhostTerrainDetailsQuery {
        /// The terraforming action being performed
        pub(super) terraforming_action: &'static TerraformingAction,
        /// The inputs that must be added to complete this terraforming action
        pub(super) input_inventory: &'static InputInventory,
        /// The outputs that must be removed to complete this terraforming action
        pub(super) output_inventory: &'static OutputInventory,
    }

    /// Detailed info about a given terraforming ghost.
    #[derive(Debug)]
    pub(crate) struct TerraformingDetails {
        /// The terraforming action being performed
        pub(super) terraforming_action: TerraformingAction,
        /// The inputs that must be added to complete this terraforming action
        pub(super) input_inventory: InputInventory,
        /// The outputs that must be removed to complete this terraforming action
        pub(super) output_inventory: OutputInventory,
    }

    impl TerraformingDetails {
        /// The pretty formatting for this type
        pub(crate) fn display(
            &self,
            item_manifest: &ItemManifest,
            terrain_manifest: &TerrainManifest,
        ) -> String {
            let terraforming_action = self.terraforming_action.display(terrain_manifest);
            let input = self.input_inventory.display(item_manifest);
            let output = self.output_inventory.display(item_manifest);

            format!(
                "Terraforming: {terraforming_action}
Input: {input}
Output: {output}"
            )
        }
    }

    /// Detailed info about a given piece of terrain.
    #[derive(Debug)]
    pub(crate) struct TerrainDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of terrain
        pub(super) terrain_id: Id<Terrain>,
        /// The location of the tile
        pub(super) tile_pos: TilePos,
        /// The height of the tile
        pub(super) height: Height,
        /// The signals on this tile
        pub(super) signals: LocalSignals,
        /// The zoning of this tile
        pub(super) zoning: Zoning,
        /// The details about the terraforming process, if any
        pub(super) maybe_terraforming_details: Option<TerraformingDetails>,
    }

    impl TerrainDetails {
        /// The pretty formatting for this type
        pub(crate) fn display(
            &self,
            terrain_manifest: &TerrainManifest,
            structure_manifest: &StructureManifest,
            item_manifest: &ItemManifest,
            unit_manifest: &UnitManifest,
        ) -> String {
            let entity = self.entity;
            let terrain_type = terrain_manifest.name(self.terrain_id);
            let tile_pos = &self.tile_pos;
            let height = &self.height;
            let signals = self.signals.display(
                item_manifest,
                structure_manifest,
                terrain_manifest,
                unit_manifest,
            );
            let zoning = self.zoning.display(structure_manifest, terrain_manifest);

            let base_string = format!(
                "Entity: {entity:?}
Terrain type: {terrain_type}
Tile: {tile_pos}
Height: {height}
Zoning: {zoning}"
            );

            if let Some(terraforming_details) = &self.maybe_terraforming_details {
                let terraforming_details =
                    terraforming_details.display(item_manifest, terrain_manifest);
                format!("{base_string}\n\n{terraforming_details}\n\nSignals:\n{signals}")
            } else {
                format!("{base_string}\n\nSignals:\n{signals}")
            }
        }
    }
}

/// Details for units
mod unit_details {
    use bevy::ecs::{prelude::*, query::WorldQuery};

    use crate::{
        asset_management::manifest::Id,
        items::item_manifest::ItemManifest,
        simulation::geometry::TilePos,
        structures::structure_manifest::StructureManifest,
        terrain::terrain_manifest::TerrainManifest,
        units::{
            actions::CurrentAction,
            goals::Goal,
            hunger::Diet,
            impatience::ImpatiencePool,
            item_interaction::UnitInventory,
            unit_manifest::{Unit, UnitManifest},
        },
    };

    use super::organism_details::OrganismDetails;

    /// Data needed to populate [`UnitDetails`].
    #[derive(WorldQuery)]
    pub(super) struct UnitDetailsQuery {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of unit
        pub(super) unit_id: &'static Id<Unit>,
        /// The current location
        pub(super) tile_pos: &'static TilePos,
        /// What's being carried
        pub(super) held_item: &'static UnitInventory,
        /// What this unit is trying to achieve
        pub(super) goal: &'static Goal,
        /// What is currently being done
        pub(super) action: &'static CurrentAction,
        /// How frustrated the unit is
        pub(super) impatience_pool: &'static ImpatiencePool,
    }

    /// Detailed info about a given unit.
    #[derive(Debug)]
    pub(crate) struct UnitDetails {
        /// The root entity
        pub(super) entity: Entity,
        /// The type of unit
        pub(super) unit_id: Id<Unit>,
        /// What does this unit eat?
        pub(super) diet: Diet,
        /// The current location
        pub(super) tile_pos: TilePos,
        /// What's being carried
        pub(super) held_item: UnitInventory,
        /// What this unit is trying to achieve
        pub(super) goal: Goal,
        /// What is currently being done
        pub(super) action: CurrentAction,
        /// Details about this organism, if it is one.
        pub(crate) organism_details: OrganismDetails,
        /// How frustrated the unit is
        pub(super) impatience_pool: ImpatiencePool,
    }

    impl UnitDetails {
        /// The pretty formatting for this type.
        pub(crate) fn display(
            &self,
            unit_manifest: &UnitManifest,
            item_manifest: &ItemManifest,
            structure_manifest: &StructureManifest,
            terrain_manifest: &TerrainManifest,
        ) -> String {
            let entity = self.entity;
            let unit_name = unit_manifest.name(self.unit_id);
            let diet = self.diet.display(item_manifest);
            let tile_pos = &self.tile_pos;
            let held_item = self.held_item.display(item_manifest);
            let goal = self
                .goal
                .display(item_manifest, structure_manifest, terrain_manifest);
            let action = &self.action.display(item_manifest);
            let impatience_pool = &self.impatience_pool;
            let organism_details = self
                .organism_details
                .display(structure_manifest, unit_manifest);

            format!(
                "Entity: {entity:?}
Unit type: {unit_name}
Tile: {tile_pos}
Diet: {diet}
Holding: {held_item}
Goal: {goal}
Action: {action}
Impatience: {impatience_pool}
{organism_details}"
            )
        }
    }
}
