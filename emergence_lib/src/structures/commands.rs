//! Methods to use [`Commands`] to manipulate structures.

use bevy::{
    ecs::system::Command,
    prelude::{warn, Commands, DespawnRecursiveExt, Mut, World},
};
use hexx::Direction;
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng};

use crate::{
    asset_management::{
        manifest::{structure::StructureKind, ItemManifest, RecipeManifest},
        structures::StructureHandles,
    },
    graphics::InheritedMaterial,
    organisms::OrganismBundle,
    player_interaction::clipboard::ClipboardData,
    signals::Emitter,
    simulation::geometry::{Facing, MapGeometry, TilePos},
};

use super::{
    construction::{GhostBundle, GhostKind, PreviewBundle},
    crafting::{CraftingBundle, StorageInventory},
    StructureBundle, StructureManifest,
};

/// An extension trait for [`Commands`] for working with structures.
pub(crate) trait StructureCommandsExt {
    /// Spawns a structure defined by `data` at `tile_pos`.
    ///
    /// Has no effect if the tile position is already occupied by an existing structure.
    fn spawn_structure(&mut self, tile_pos: TilePos, data: ClipboardData);

    /// Spawns a structure with randomized `data` at `tile_pos`.
    ///
    /// Some fields of data will be randomized.
    /// This is intended to be used for world generation.
    fn spawn_randomized_structure(
        &mut self,
        tile_pos: TilePos,
        data: ClipboardData,
        rng: &mut ThreadRng,
    );

    /// Despawns any structure at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_structure(&mut self, tile_pos: TilePos);

    /// Spawns a ghost with data defined by `data` at `tile_pos`.
    ///
    /// Replaces any existing ghost.
    fn spawn_ghost(&mut self, tile_pos: TilePos, data: ClipboardData);

    /// Despawns any ghost at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_ghost(&mut self, tile_pos: TilePos);

    /// Spawns a preview with data defined by `item` at `tile_pos`.
    ///
    /// Replaces any existing preview.
    fn spawn_preview(&mut self, tile_pos: TilePos, data: ClipboardData, forbidden: bool);

    /// Despawns any preview at the provided `tile_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_preview(&mut self, tile_pos: TilePos);
}

impl<'w, 's> StructureCommandsExt for Commands<'w, 's> {
    fn spawn_structure(&mut self, tile_pos: TilePos, data: ClipboardData) {
        self.add(SpawnStructureCommand {
            tile_pos,
            data,
            randomized: false,
        });
    }

    fn spawn_randomized_structure(
        &mut self,
        tile_pos: TilePos,
        mut data: ClipboardData,
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

    fn spawn_ghost(&mut self, tile_pos: TilePos, data: ClipboardData) {
        self.add(SpawnGhostCommand { tile_pos, data });
    }

    fn despawn_ghost(&mut self, tile_pos: TilePos) {
        self.add(DespawnGhostCommand { tile_pos });
    }

    fn spawn_preview(&mut self, tile_pos: TilePos, data: ClipboardData, forbidden: bool) {
        self.add(SpawnPreviewCommand {
            tile_pos,
            data,
            forbidden,
        });
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
    data: ClipboardData,
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

        let structure_variety = world
            .resource::<StructureManifest>()
            .get(self.data.structure_id)
            .clone();

        let structure_handles = world.resource::<StructureHandles>();

        let picking_mesh = structure_handles.picking_mesh.clone_weak();
        let scene_handle = structure_handles
            .scenes
            .get(&self.data.structure_id)
            .unwrap()
            .clone_weak();
        let world_pos = self.tile_pos.top_of_tile(world.resource::<MapGeometry>());

        let structure_entity = world
            .spawn(StructureBundle::new(
                self.tile_pos,
                self.data,
                picking_mesh,
                scene_handle,
                world_pos,
            ))
            .id();

        // PERF: these operations could be done in a single archetype move with more branching
        if let Some(organism_details) = &structure_variety.organism {
            world
                .entity_mut(structure_entity)
                .insert(OrganismBundle::new(organism_details.energy_pool.clone()));
        };

        match structure_variety.kind {
            StructureKind::Storage {
                max_slot_count,
                reserved_for,
            } => {
                world
                    .entity_mut(structure_entity)
                    .insert(StorageInventory::new(max_slot_count, reserved_for))
                    .insert(Emitter::default());
            }
            StructureKind::Crafting { starting_recipe } => {
                world.resource_scope(|world, recipe_manifest: Mut<RecipeManifest>| {
                    world.resource_scope(|world, item_manifest: Mut<ItemManifest>| {
                        let crafting_bundle = match self.randomized {
                            false => CraftingBundle::new(
                                starting_recipe,
                                &recipe_manifest,
                                &item_manifest,
                            ),
                            true => {
                                let rng = &mut thread_rng();
                                CraftingBundle::randomized(
                                    starting_recipe,
                                    &recipe_manifest,
                                    &item_manifest,
                                    rng,
                                )
                            }
                        };

                        world.entity_mut(structure_entity).insert(crafting_bundle);
                    })
                })
            }
        }

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
    data: ClipboardData,
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

        // Spawn a ghost
        let structure_handles = world.resource::<StructureHandles>();

        let picking_mesh = structure_handles.picking_mesh.clone_weak();
        let scene_handle = structure_handles
            .scenes
            .get(&self.data.structure_id)
            .unwrap()
            .clone_weak();
        let ghostly_handle = structure_handles
            .ghost_materials
            .get(&GhostKind::Ghost)
            .unwrap();
        let inherited_material = InheritedMaterial(ghostly_handle.clone_weak());

        let world_pos = self.tile_pos.top_of_tile(world.resource::<MapGeometry>());

        let ghost_entity = world
            .spawn(GhostBundle::new(
                self.tile_pos,
                self.data,
                structure_manifest,
                picking_mesh,
                scene_handle,
                inherited_material,
                world_pos,
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
    data: ClipboardData,
    /// Is this structure allowed to be built here?
    forbidden: bool,
}

impl Command for SpawnPreviewCommand {
    fn write(self, world: &mut World) {
        let mut map_geometry = world.resource_mut::<MapGeometry>();

        // Check that the tile is within the bounds of the map
        if !map_geometry.is_valid(self.tile_pos) {
            warn!("Preview position {:?} not valid.", self.tile_pos);
            return;
        }

        // Compute the world position
        let world_pos = self.tile_pos.top_of_tile(&map_geometry);

        // Remove any existing previews at this location
        let maybe_existing_preview = map_geometry.preview_index.remove(&self.tile_pos);
        if let Some(existing_preview) = maybe_existing_preview {
            world.entity_mut(existing_preview).despawn_recursive();
        }

        // Fetch the scene and material to use
        let structure_handles = world.resource::<StructureHandles>();
        let scene_handle = structure_handles
            .scenes
            .get(&self.data.structure_id)
            .unwrap()
            .clone_weak();

        let ghost_kind = match self.forbidden {
            true => GhostKind::ForbiddenPreview,
            false => GhostKind::Preview,
        };

        let preview_handle = structure_handles.ghost_materials.get(&ghost_kind).unwrap();
        let inherited_material = InheritedMaterial(preview_handle.clone_weak());

        // Spawn a preview
        let preview_entity = world
            .spawn(PreviewBundle::new(
                self.tile_pos,
                self.data,
                scene_handle,
                inherited_material,
                world_pos,
            ))
            .id();

        // Update the index to reflect the new state
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
