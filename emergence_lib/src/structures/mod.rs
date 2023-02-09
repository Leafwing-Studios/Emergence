//! Structures (or buildings) are plants and fungi that serve a role in the bustling organic factory.
//!
//! Typically, these will produce and transform resources (much like machines in other factory builders),
//! but they can also be used for defense, research, reproduction, storage and more exotic effects.

use bevy::{ecs::system::Command, prelude::*};

use crate::simulation::geometry::{Facing, MapGeometry, TilePos};

use self::crafting::CraftingPlugin;

pub mod crafting;
mod sessile;

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
        app.add_plugin(CraftingPlugin);
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

        let structure_entity = world
            .spawn(StructureBundle::new(self.id, self.tile_pos))
            .id();

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
