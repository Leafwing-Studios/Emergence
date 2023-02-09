//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::{ecs::system::Command, prelude::*, utils::HashMap};

use crate::{
    items::recipe::RecipeId,
    organisms::OrganismBundle,
    simulation::geometry::{Facing, MapGeometry, TilePos},
};

use self::crafting::{CraftingBundle, CraftingPlugin};

pub mod crafting;

/// A central lookup for how each variety the structure works.
#[derive(Resource, Debug, Deref, DerefMut)]
struct StructureInfo {
    map: HashMap<StructureId, StructureData>,
}

/// Information about a single [`StructureId`] variety of structure.
#[derive(Debug, Clone)]
struct StructureData {
    organism: bool,
    crafts: bool,
    starting_recipe: Option<RecipeId>,
}

impl Default for StructureInfo {
    fn default() -> Self {
        let mut map = HashMap::default();

        // TODO: read these from files
        map.insert(
            StructureId::new("leuco"),
            StructureData {
                organism: true,
                crafts: true,
                starting_recipe: None,
            },
        );

        map.insert(
            StructureId::new("acacia"),
            StructureData {
                organism: true,
                crafts: true,
                starting_recipe: Some(RecipeId::acacia_leaf_production()),
            },
        );

        StructureInfo { map }
    }
}

/// The data needed to build a structure
#[derive(Bundle)]
struct StructureBundle {
    /// Data characterizing structures
    structure: StructureId,
    /// The direction this structure is facing
    facing: Facing,
    /// The location of this structure
    tile_pos: TilePos,
}

impl StructureBundle {
    /// Creates a new structure
    pub fn new(id: StructureId, tile_pos: TilePos) -> Self {
        StructureBundle {
            structure: id,
            facing: Facing::default(),
            tile_pos,
        }
    }
}

/// Structures are static buildings that take up one or more tile
#[derive(Component, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StructureId {
    /// The unique identifier for this variety of structure.
    pub(crate) id: String,
}

impl StructureId {
    /// The size of a single structure
    pub const SIZE: f32 = 1.0;
    /// The offset required to have a structure sit on top of the tile correctly
    pub const OFFSET: f32 = Self::SIZE / 2.0;

    /// Initialize a structure ID via a string.
    pub(crate) fn new(id: &'static str) -> Self {
        StructureId { id: id.to_string() }
    }
}

/// The systems that make structures tick.
pub struct StructuresPlugin;

impl Plugin for StructuresPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CraftingPlugin)
            .init_resource::<StructureInfo>();
    }
}

/// An extension trait for [`Commands`] for working with structures.
pub trait StructureCommandsExt {
    /// Spawns a structure of type `id` at `tile_pos`.
    ///
    /// Has no effect if the tile position is already occupied by an existing structure.
    fn spawn_structure(&mut self, tile_pos: TilePos, id: StructureId);

    /// Despawns any structure at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_structure(&mut self, tile_pos: TilePos);
}

impl<'w, 's> StructureCommandsExt for Commands<'w, 's> {
    fn spawn_structure(&mut self, tile_pos: TilePos, id: StructureId) {
        self.add(SpawnStructureCommand { tile_pos, id });
    }

    fn despawn_structure(&mut self, tile_pos: TilePos) {
        self.add(DespawnStructureCommand { tile_pos });
    }
}

/// A [`Command`] used to spawn a structure via [`StructureCommandsExt`].
struct SpawnStructureCommand {
    tile_pos: TilePos,
    id: StructureId,
}

impl Command for SpawnStructureCommand {
    fn write(self, world: &mut World) {
        let geometry = world.resource::<MapGeometry>();

        // Check that the tile is empty.
        if geometry.structure_index.contains_key(&self.tile_pos) {
            return;
        }

        // Check that the tile is within the bounds of the map
        if !geometry.is_valid(self.tile_pos) {
            return;
        }

        let structure_entity = world.resource_scope(|world, structure_info: Mut<StructureInfo>| {
            let structure_details = structure_info.get(&self.id).unwrap();

            let structure_entity = world
                .spawn(StructureBundle::new(self.id, self.tile_pos))
                .id();

            // PERF: this could be done in a single archetype move with more branching
            if structure_details.organism {
                world
                    .entity_mut(structure_entity)
                    .insert(OrganismBundle::default());
            };

            if structure_details.crafts {
                world
                    .entity_mut(structure_entity)
                    .insert(CraftingBundle::new(
                        structure_details.starting_recipe.clone(),
                    ));
            };

            structure_entity
        });

        let mut geometry = world.resource_mut::<MapGeometry>();
        geometry
            .structure_index
            .insert(self.tile_pos, structure_entity);
    }
}

/// A [`Command`] used to despawn a structure via [`StructureCommandsExt`].
struct DespawnStructureCommand {
    tile_pos: TilePos,
}

impl Command for DespawnStructureCommand {
    fn write(self, world: &mut World) {
        let mut geometry = world.resource_mut::<MapGeometry>();
        let maybe_entity = geometry.structure_index.remove(&self.tile_pos);

        // Check that there's something there to despawn
        if maybe_entity.is_none() {
            return;
        }

        let structure_entity = maybe_entity.unwrap();
        // Make sure to despawn all children, which represent the meshes stored in the loaded gltf scene.
        world.entity_mut(structure_entity).despawn_recursive();
    }
}
