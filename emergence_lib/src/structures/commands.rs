//! Methods to use [`Commands`] to manipulate structures.

use bevy::{ecs::system::Command, prelude::*};
use leafwing_abilities::prelude::Pool;

use crate::{
    asset_management::manifest::Id,
    construction::ghosts::{GhostHandles, GhostKind, GhostStructureBundle, StructurePreviewBundle},
    crafting::{
        inventories::{InputInventory, OutputInventory, StorageInventory},
        recipe::RecipeManifest,
        CraftingBundle,
    },
    geometry::{Facing, MapGeometry, VoxelPos},
    graphics::InheritedMaterial,
    items::{inventory::Inventory, item_manifest::ItemManifest},
    organisms::{energy::StartingEnergy, OrganismBundle},
    player_interaction::clipboard::ClipboardData,
    signals::Emitter,
};

use super::{
    logistic_buildings::{AbsorbsItems, ReleasesItems},
    structure_assets::StructureHandles,
    structure_manifest::{Structure, StructureKind, StructureManifest},
    Landmark, StructureBundle,
};

/// An extension trait for [`Commands`] for working with structures.
pub(crate) trait StructureCommandsExt {
    /// Spawns a structure defined by `data` at `voxel_pos`.
    ///
    /// Has no effect if the tile position is already occupied by an existing structure.
    fn spawn_structure(
        &mut self,
        voxel_pos: VoxelPos,
        data: ClipboardData,
        starting_energy: StartingEnergy,
    );

    /// Despawns any structure at the provided `voxel_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_structure(&mut self, voxel_pos: VoxelPos);

    /// Spawns a ghost with data defined by `data` at `voxel_pos`.
    ///
    /// Replaces any existing ghost.
    fn spawn_ghost_structure(&mut self, voxel_pos: VoxelPos, data: ClipboardData);

    /// Despawns any ghost at the provided `voxel_pos`.
    ///
    /// Has no effect if the tile position is already empty.
    fn despawn_ghost_structure(&mut self, voxel_pos: VoxelPos);

    /// Spawns a preview with data defined by `item` at `voxel_pos`.
    ///
    /// Replaces any existing preview.
    fn spawn_preview_structure(&mut self, voxel_pos: VoxelPos, data: ClipboardData);
}

impl<'w, 's> StructureCommandsExt for Commands<'w, 's> {
    fn spawn_structure(
        &mut self,
        voxel_pos: VoxelPos,
        data: ClipboardData,
        starting_energy: StartingEnergy,
    ) {
        self.add(SpawnStructureCommand {
            center: voxel_pos,
            data,
            starting_energy,
        });
    }

    fn despawn_structure(&mut self, voxel_pos: VoxelPos) {
        self.add(DespawnStructureCommand { center: voxel_pos });
    }

    fn spawn_ghost_structure(&mut self, voxel_pos: VoxelPos, data: ClipboardData) {
        self.add(SpawnStructureGhostCommand {
            center: voxel_pos,
            data,
        });
    }

    fn despawn_ghost_structure(&mut self, voxel_pos: VoxelPos) {
        self.add(DespawnGhostCommand { voxel_pos });
    }

    fn spawn_preview_structure(&mut self, voxel_pos: VoxelPos, data: ClipboardData) {
        self.add(SpawnStructurePreviewCommand {
            center: voxel_pos,
            data,
        });
    }
}

/// A [`Command`] used to spawn a structure via [`StructureCommandsExt`].
struct SpawnStructureCommand {
    /// The tile position at which to spawn the structure.
    center: VoxelPos,
    /// Data about the structure to spawn.
    data: ClipboardData,
    /// The amount of energy to give the organism.
    starting_energy: StartingEnergy,
}

impl Command for SpawnStructureCommand {
    fn write(self, world: &mut World) {
        let geometry = world.resource::<MapGeometry>();
        // Check that the tile is within the bounds of the map
        if !geometry.is_valid(self.center.hex) {
            return;
        }

        let structure_id = self.data.structure_id;

        let manifest = world.resource::<StructureManifest>();
        let structure_data = manifest.get(structure_id).clone();

        // Check that the tiles needed are appropriate.
        let geometry = world.resource_mut::<MapGeometry>();
        if geometry
            .is_space_available(self.center, &structure_data.footprint, self.data.facing)
            .is_err()
        {
            // Just give up if the terrain is wrong.
            return;
        }

        let map_geometry = world.resource::<MapGeometry>();
        let world_pos = structure_data
            .footprint
            .world_pos(self.data.facing, self.center, map_geometry)
            .unwrap_or_default();

        let facing = self.data.facing;

        let structure_bundle =
            if let Some(structure_handles) = world.get_resource::<StructureHandles>() {
                // TODO: vary this with the footprint of the structure
                let picking_mesh = structure_handles.picking_mesh.clone_weak();
                let scene_handle = structure_handles
                    .scenes
                    .get(&structure_id)
                    .unwrap()
                    .clone_weak();

                StructureBundle::new(
                    self.center,
                    structure_data.footprint.clone(),
                    self.data,
                    picking_mesh,
                    scene_handle,
                    world_pos,
                )
            } else {
                StructureBundle::new(
                    self.center,
                    structure_data.footprint.clone(),
                    self.data,
                    Handle::default(),
                    Handle::default(),
                    world_pos,
                )
            };

        let structure_entity = world.spawn(structure_bundle).id();

        // PERF: these operations could be done in a single archetype move with more branching
        if let Some(organism_details) = &structure_data.organism_variety {
            let mut energy_pool = organism_details.energy_pool.clone();
            match self.starting_energy {
                StartingEnergy::Specific(energy) => {
                    energy_pool.set_current(energy);
                },
                StartingEnergy::Random => {
                    let rng = &mut rand::thread_rng();
                    energy_pool.randomize(rng)
                },
                StartingEnergy::Full => {},
                StartingEnergy::NotAnOrganism => panic!("All organisms must have energy pools, and this variant should never be constructed for organisms."),
            };

            world
                .entity_mut(structure_entity)
                .insert(OrganismBundle::new(
                    energy_pool,
                    organism_details.lifecycle.clone(),
                ));
        };

        match structure_data.kind {
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
                        world.resource_scope(|world, structure_manifest: Mut<StructureManifest>| {
                            let crafting_bundle = CraftingBundle::new(
                                structure_id,
                                starting_recipe,
                                &recipe_manifest,
                                &item_manifest,
                                &structure_manifest,
                            );

                            world.entity_mut(structure_entity).insert(crafting_bundle);
                        })
                    })
                })
            }
            StructureKind::Path => {}
            StructureKind::Landmark => {
                world.entity_mut(structure_entity).insert(Landmark);
            }
            StructureKind::Absorber => {
                world
                    .entity_mut(structure_entity)
                    .insert(AbsorbsItems)
                    .insert(OutputInventory::default())
                    .insert(Emitter::default());
            }
            StructureKind::Releaser => {
                world
                    .entity_mut(structure_entity)
                    .insert(ReleasesItems)
                    .insert(InputInventory::Exact {
                        // TODO: let this be configured by the user using the UI
                        inventory: Inventory::empty_from_item(
                            Id::from_name("crab_egg".to_string()),
                            1,
                        ),
                    })
                    .insert(Emitter::default());
            }
        }

        // TODO: yeet StructureKind and just do this everywhere
        if let Some(vegetative_reproduction) = structure_data.vegetative_reproduction {
            world
                .entity_mut(structure_entity)
                .insert(vegetative_reproduction);
        }

        let mut geometry = world.resource_mut::<MapGeometry>();
        // We've already verified that we can build here, so we can safely unwrap at this point
        geometry
            .add_structure(
                self.center,
                facing,
                &structure_data.footprint,
                structure_data.can_walk_on_roof,
                structure_data.can_walk_through,
                structure_entity,
            )
            .unwrap();
    }
}

/// A [`Command`] used to despawn a structure via [`StructureCommandsExt`].
struct DespawnStructureCommand {
    /// The tile position at which the structure to be despawned is found.
    center: VoxelPos,
}

impl Command for DespawnStructureCommand {
    fn write(self, world: &mut World) {
        let map_geometry = world.resource::<MapGeometry>();
        let Some(structure_entity) = map_geometry.get_structure(self.center) else { return; };

        let facing = *world.entity(structure_entity).get::<Facing>().unwrap();
        let Some(&structure_id) = world
            .entity(structure_entity)
            .get::<Id<Structure>>() else { return; };
        let structure_manifest = world.resource::<StructureManifest>();
        let structure_data = structure_manifest.get(structure_id);
        let footprint = structure_data.footprint.clone();

        let mut geometry = world.resource_mut::<MapGeometry>();
        let maybe_entity = geometry.remove_structure(self.center, &footprint, facing);

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
struct SpawnStructureGhostCommand {
    /// The tile position at which to spawn the structure.
    center: VoxelPos,
    /// Data about the structure to spawn.
    data: ClipboardData,
}

impl Command for SpawnStructureGhostCommand {
    fn write(self, world: &mut World) {
        let structure_id = self.data.structure_id;
        let map_geometry = world.resource::<MapGeometry>();

        // Check that the tile is within the bounds of the map
        if !map_geometry.is_valid(self.center.hex) {
            warn!("Tried to spawn a structure outside of the map bounds.");
            return;
        }

        let manifest = world.resource::<StructureManifest>();
        let footprint = manifest.footprint(structure_id).clone();
        let structure_data = manifest.get(structure_id);
        let facing = self.data.facing;

        let world_pos = structure_data
            .footprint
            .world_pos(self.data.facing, self.center, map_geometry)
            .unwrap_or_default();

        // Check that the tiles needed are appropriate.
        if map_geometry
            .is_space_available(self.center, &footprint, facing)
            .is_err()
        {
            warn!("Tried to spawn a structure in an occupied location.");
            return;
        }

        // Remove any existing ghosts
        let map_geometry = world.resource::<MapGeometry>();

        let mut existing_ghosts: Vec<Entity> = Vec::new();
        for voxel_pos in footprint.normalized(facing, self.center) {
            if let Some(ghost_entity) = map_geometry.get_ghost_structure(voxel_pos) {
                existing_ghosts.push(ghost_entity);
            }
        }

        for ghost_entity in existing_ghosts {
            let facing = *world.entity(ghost_entity).get::<Facing>().unwrap();
            let center = *world.entity(ghost_entity).get::<VoxelPos>().unwrap();
            let structure_id = *world.entity(ghost_entity).get::<Id<Structure>>().unwrap();

            let structure_manifest = world.resource::<StructureManifest>();
            let footprint = structure_manifest.footprint(structure_id).clone();

            world.entity_mut(ghost_entity).despawn_recursive();
            let mut map_geometry = world.resource_mut::<MapGeometry>();
            map_geometry.remove_ghost_structure(center, &footprint, facing);
        }

        let structure_manifest = world.resource::<StructureManifest>();

        // Spawn a ghost
        let ghost_handles = world.resource::<GhostHandles>();
        let structure_handles = world.resource::<StructureHandles>();

        // TODO: vary this with the footprint and height of the structure
        let picking_mesh = structure_handles.picking_mesh.clone_weak();
        let scene_handle = structure_handles
            .scenes
            .get(&structure_id)
            .unwrap()
            .clone_weak();
        let ghostly_handle = ghost_handles.get_material(GhostKind::Ghost);
        let inherited_material = InheritedMaterial(ghostly_handle.clone_weak());

        let facing = self.data.facing;

        let ghost_entity = world
            .spawn(GhostStructureBundle::new(
                self.center,
                self.data,
                structure_manifest,
                picking_mesh,
                scene_handle,
                inherited_material,
                world_pos,
            ))
            .id();

        // Update the index to reflect the new state
        world.resource_scope(|world, mut map_geometry: Mut<MapGeometry>| {
            let structure_manifest = world.resource::<StructureManifest>();
            let structure_variety = structure_manifest.get(structure_id);
            let footprint = &structure_variety.footprint;

            map_geometry
                .add_ghost_structure(facing, self.center, footprint, ghost_entity)
                .unwrap();
        });
    }
}

/// A [`Command`] used to despawn a ghost via [`StructureCommandsExt`].
struct DespawnGhostCommand {
    /// The tile position at which the structure to be despawned is found.
    voxel_pos: VoxelPos,
}

impl Command for DespawnGhostCommand {
    fn write(self, world: &mut World) {
        let map_geometry = world.resource::<MapGeometry>();
        let Some(ghost_entity) = map_geometry.get_ghost_structure(self.voxel_pos) else { return; };

        let facing = *world.entity(ghost_entity).get::<Facing>().unwrap();
        let center = *world.entity(ghost_entity).get::<VoxelPos>().unwrap();
        let structure_id = *world.entity(ghost_entity).get::<Id<Structure>>().unwrap();

        let structure_manifest = world.resource::<StructureManifest>();
        let footprint = structure_manifest.footprint(structure_id).clone();

        let mut map_geometry = world.resource_mut::<MapGeometry>();
        map_geometry.remove_ghost_structure(center, &footprint, facing);
        // Make sure to despawn all children, which represent the meshes stored in the loaded gltf scene.
        world.entity_mut(ghost_entity).despawn_recursive();
    }
}

/// A [`Command`] used to spawn a preview via [`StructureCommandsExt`].
struct SpawnStructurePreviewCommand {
    /// The tile position at which to spawn the structure.
    center: VoxelPos,
    /// Data about the structure to spawn.
    data: ClipboardData,
}

impl Command for SpawnStructurePreviewCommand {
    fn write(self, world: &mut World) {
        let structure_id = self.data.structure_id;
        let map_geometry = world.resource::<MapGeometry>();

        // Check that the tile is within the bounds of the map
        if !map_geometry.is_valid(self.center.hex) {
            warn!("Preview position {:?} not valid.", self.center);
            return;
        }

        let manifest = world.resource::<StructureManifest>();
        let structure_data = manifest.get(structure_id).clone();

        let geometry = world.resource::<MapGeometry>();

        // Compute the world position
        let world_pos = structure_data
            .footprint
            .world_pos(self.data.facing, self.center, map_geometry)
            .unwrap_or_default();

        // Check that the tiles needed are appropriate.
        let forbidden = geometry
            .is_space_available(self.center, &structure_data.footprint, self.data.facing)
            .is_err();

        // Fetch the scene and material to use
        let structure_handles = world.resource::<StructureHandles>();
        let scene_handle = structure_handles
            .scenes
            .get(&self.data.structure_id)
            .unwrap()
            .clone_weak();

        let ghost_kind = match forbidden {
            true => GhostKind::ForbiddenPreview,
            false => GhostKind::Preview,
        };

        let ghost_handles = world.resource::<GhostHandles>();

        let preview_handle = ghost_handles.get_material(ghost_kind);
        let inherited_material = InheritedMaterial(preview_handle.clone_weak());

        // Spawn a preview
        world.spawn(StructurePreviewBundle::new(
            self.data,
            scene_handle,
            inherited_material,
            world_pos,
        ));
    }
}
