//! Methods to use [`Commands`] to manipulate structures.

use bevy::{
    ecs::system::Command,
    prelude::{Commands, DespawnRecursiveExt, Mut, World},
};
use hexx::Direction;
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng};

use crate::{
    items::{recipe::RecipeManifest, ItemManifest},
    organisms::OrganismBundle,
    player_interaction::clipboard::StructureData,
    simulation::geometry::{Facing, MapGeometry, TilePos},
};

use super::{
    crafting::CraftingBundle,
    ghost::{GhostBundle, PreviewBundle},
    StructureBundle, StructureManifest,
};

/// An extension trait for [`Commands`] for working with structures.
pub(crate) trait StructureCommandsExt {
    /// Spawns a structure defined by `data` at `tile_pos`.
    ///
    /// Has no effect if the tile position is already occupied by an existing structure.
    fn spawn_structure(&mut self, tile_pos: TilePos, data: StructureData);

    /// Spawns a structure with randomized `data` at `tile_pos`.
    ///
    /// Some fields of data will be randomized.
    /// This is intended to be used for world generation.
    fn spawn_randomized_structure(
        &mut self,
        tile_pos: TilePos,
        data: StructureData,
        rng: &mut ThreadRng,
    );

    /// Despawns any structure at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_structure(&mut self, tile_pos: TilePos);

    /// Spawns a ghost with data defined by `data` at `tile_pos`.
    ///
    /// Replaces any existing ghost.
    fn spawn_ghost(&mut self, tile_pos: TilePos, data: StructureData);

    /// Despawns any ghost at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_ghost(&mut self, tile_pos: TilePos);

    /// Spawns a preview with data defined by `item` at `tile_pos`.
    ///
    /// Replaces any existing preview.
    fn spawn_preview(&mut self, tile_pos: TilePos, data: StructureData);

    /// Despawns any preview at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_preview(&mut self, tile_pos: TilePos);
}

impl<'w, 's> StructureCommandsExt for Commands<'w, 's> {
    fn spawn_structure(&mut self, tile_pos: TilePos, data: StructureData) {
        self.add(SpawnStructureCommand {
            tile_pos,
            data,
            randomized: false,
        });
    }

    fn spawn_randomized_structure(
        &mut self,
        tile_pos: TilePos,
        mut data: StructureData,
        rng: &mut ThreadRng,
    ) {
        let direction = *Direction::ALL_DIRECTIONS.choose(rng).unwrap();
        data.facing = Facing { direction };

        self.add(SpawnStructureCommand {
            tile_pos,
            data,
            randomized: true,
        });
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

    fn spawn_preview(&mut self, tile_pos: TilePos, data: StructureData) {
        self.add(SpawnPreviewCommand { tile_pos, data });
    }

    fn despawn_preview(&mut self, tile_pos: TilePos) {
        self.add(DespawnPreviewCommand { tile_pos });
    }
}

/// A [`Command`] used to spawn a structure via [`StructureCommandsExt`].
struct SpawnStructureCommand {
    /// The tile position at which to spawn the structure.
    tile_pos: TilePos,
    /// Data about the structure to spawn.
    data: StructureData,
    /// Should the generated structure be randomized
    randomized: bool,
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
                let structure_details = structure_manifest.get(self.data.structure_id);

                let structure_entity = world
                    .spawn(StructureBundle::new(self.tile_pos, self.data))
                    .id();

                // PERF: this could be done in a single archetype move with more branching
                if let Some(organism_details) = &structure_details.organism {
                    world
                        .entity_mut(structure_entity)
                        .insert(OrganismBundle::new(organism_details.energy_pool.clone()));
                };

                if structure_details.crafts {
                    world.resource_scope(|world, recipe_manifest: Mut<RecipeManifest>| {
                        world.resource_scope(|world, item_manifest: Mut<ItemManifest>| {
                            let crafting_bundle = match self.randomized {
                                false => CraftingBundle::new(
                                    structure_details.starting_recipe,
                                    &recipe_manifest,
                                    &item_manifest,
                                ),
                                true => {
                                    let rng = &mut thread_rng();
                                    CraftingBundle::randomized(
                                        structure_details.starting_recipe,
                                        &recipe_manifest,
                                        &item_manifest,
                                        rng,
                                    )
                                }
                            };

                            world.entity_mut(structure_entity).insert(crafting_bundle);
                        })
                    })
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

        let structure_manifest = world.resource::<StructureManifest>();
        let variety_data = structure_manifest.get(self.data.structure_id);
        let construction_materials = variety_data.construction_materials.clone();

        // Spawn a ghost
        let ghost_entity = world
            .spawn(GhostBundle::new(
                self.tile_pos,
                self.data,
                construction_materials,
            ))
            .id();

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

/// A [`Command`] used to spawn a preview via [`StructureCommandsExt`].
struct SpawnPreviewCommand {
    /// The tile position at which to spawn the structure.
    tile_pos: TilePos,
    /// Data about the structure to spawn.
    data: StructureData,
}

impl Command for SpawnPreviewCommand {
    fn write(self, world: &mut World) {
        let mut geometry = world.resource_mut::<MapGeometry>();

        // Check that the tile is within the bounds of the map
        if !geometry.is_valid(self.tile_pos) {
            return;
        }

        // Remove any existing previews
        let maybe_existing_preview = geometry.preview_index.remove(&self.tile_pos);

        if let Some(existing_preview) = maybe_existing_preview {
            world.entity_mut(existing_preview).despawn_recursive();
        }

        // Spawn a preview
        let preview_entity = world
            .spawn(PreviewBundle::new(self.tile_pos, self.data))
            .id();

        let mut geometry = world.resource_mut::<MapGeometry>();
        geometry.preview_index.insert(self.tile_pos, preview_entity);
    }
}

/// A [`Command`] used to despawn a preview via [`StructureCommandsExt`].
struct DespawnPreviewCommand {
    /// The tile position at which the structure to be despawned is found.
    tile_pos: TilePos,
}

impl Command for DespawnPreviewCommand {
    fn write(self, world: &mut World) {
        let mut geometry = world.resource_mut::<MapGeometry>();
        let maybe_entity = geometry.preview_index.remove(&self.tile_pos);

        // Check that there's something there to despawn
        if maybe_entity.is_none() {
            return;
        }

        let preview_entity = maybe_entity.unwrap();
        // Make sure to despawn all children, which represent the meshes stored in the loaded gltf scene.
        world.entity_mut(preview_entity).despawn_recursive();
    }
}
