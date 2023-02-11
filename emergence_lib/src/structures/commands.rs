//! Methods to use [`Commands`] to manipulate structures.

use bevy::{
    ecs::system::Command,
    prelude::{Commands, DespawnRecursiveExt, Mut, World},
};

use crate::{
    organisms::OrganismBundle,
    player_interaction::clipboard::StructureData,
    simulation::geometry::{MapGeometry, TilePos},
};

use super::{crafting::CraftingBundle, ghost::GhostBundle, StructureBundle, StructureManifest};

/// An extension trait for [`Commands`] for working with structures.
pub(crate) trait StructureCommandsExt {
    /// Spawns a structure with data defined by `item` at `tile_pos`.
    ///
    /// Has no effect if the tile position is already occupied by an existing structure.
    fn spawn_structure(&mut self, tile_pos: TilePos, data: StructureData);

    /// Despawns any structure at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_structure(&mut self, tile_pos: TilePos);

    /// Spawns a ghost with data defined by `item` at `tile_pos`.
    ///
    /// Replaces any existing ghost.
    fn spawn_ghost(&mut self, tile_pos: TilePos, data: StructureData);

    /// Despawns any ghost at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_ghost(&mut self, tile_pos: TilePos);
}

impl<'w, 's> StructureCommandsExt for Commands<'w, 's> {
    fn spawn_structure(&mut self, tile_pos: TilePos, data: StructureData) {
        self.add(SpawnStructureCommand { tile_pos, data });
    }

    fn despawn_structure(&mut self, tile_pos: TilePos) {
        self.add(DespawnStructureCommand { tile_pos });
    }

    fn spawn_ghost(&mut self, tile_pos: TilePos, data: StructureData) {
        self.add(SpawnGhostCommand { tile_pos, data });
    }

    fn despawn_ghost(&mut self, tile_pos: TilePos) {
        self.add(DespawnGhostCommand { tile_pos });
    }
}

/// A [`Command`] used to spawn a structure via [`StructureCommandsExt`].
struct SpawnStructureCommand {
    /// The tile position at which to spawn the structure.
    tile_pos: TilePos,
    /// Data about the structure to spawn.
    data: StructureData,
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

        let structure_entity =
            world.resource_scope(|world, structure_manifest: Mut<StructureManifest>| {
                let structure_details = structure_manifest.get(&self.data.id);

                let structure_entity = world
                    .spawn(StructureBundle::new(self.tile_pos, self.data))
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
    /// The tile position at which the structure to be despawned is found.
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

/// A [`Command`] used to spawn a ghost via [`StructureCommandsExt`].
struct SpawnGhostCommand {
    /// The tile position at which to spawn the structure.
    tile_pos: TilePos,
    /// Data about the structure to spawn.
    data: StructureData,
}

impl Command for SpawnGhostCommand {
    fn write(self, world: &mut World) {
        let mut geometry = world.resource_mut::<MapGeometry>();

        // Check that the tile is within the bounds of the map
        if !geometry.is_valid(self.tile_pos) {
            return;
        }

        // Remove any existing ghosts
        let maybe_existing_ghost = geometry.ghost_index.remove(&self.tile_pos);

        if let Some(existing_ghost) = maybe_existing_ghost {
            world.entity_mut(existing_ghost).despawn_recursive();
        }

        // Spawn a ghost
        let ghost_entity = world.spawn(GhostBundle::new(self.tile_pos, self.data)).id();

        let mut geometry = world.resource_mut::<MapGeometry>();
        geometry.ghost_index.insert(self.tile_pos, ghost_entity);
    }
}

/// A [`Command`] used to despawn a ghost via [`StructureCommandsExt`].
struct DespawnGhostCommand {
    /// The tile position at which the structure to be despawned is found.
    tile_pos: TilePos,
}

impl Command for DespawnGhostCommand {
    fn write(self, world: &mut World) {
        let mut geometry = world.resource_mut::<MapGeometry>();
        let maybe_entity = geometry.ghost_index.remove(&self.tile_pos);

        // Check that there's something there to despawn
        if maybe_entity.is_none() {
            return;
        }

        let ghost_entity = maybe_entity.unwrap();
        // Make sure to despawn all children, which represent the meshes stored in the loaded gltf scene.
        world.entity_mut(ghost_entity).despawn_recursive();
    }
}
