//! Initializes organisms in the world.

use crate::geometry::{Facing, MapGeometry};
use crate::organisms::energy::StartingEnergy;
use crate::player_interaction::clipboard::ClipboardData;
use crate::simulation::rng::GlobalRng;
use crate::structures::commands::StructureCommandsExt;
use crate::structures::structure_manifest::StructureManifest;

use bevy::prelude::*;
use rand::Rng;

use super::GenerationConfig;

/// Create starting structures according to [`GenerationConfig`], and randomly place them on
/// top of the terrain.
pub(super) fn generate_structures(
    mut commands: Commands,
    config: Res<GenerationConfig>,
    structure_manifest: Res<StructureManifest>,
    map_geometry: Res<MapGeometry>,
    mut rng: ResMut<GlobalRng>,
) {
    info!("Generating structures...");

    // Collect out so we can mutate the height map to flatten the terrain while in the loop
    for voxel_pos in map_geometry.walkable_voxels() {
        for (&structure_id, &chance) in &config.structure_chances {
            if rng.gen::<f32>() < chance {
                let mut clipboard_data =
                    ClipboardData::generate_from_id(structure_id, &structure_manifest);
                let facing = Facing::random(rng.get_mut());
                clipboard_data.facing = facing;
                let footprint = &structure_manifest.get(structure_id).footprint;

                // Only try to spawn a structure if the location is valid and there is space
                if map_geometry.is_footprint_valid(voxel_pos, footprint, facing)
                    && map_geometry
                        .is_space_available(voxel_pos, footprint, facing)
                        .is_ok()
                {
                    commands.spawn_structure(
                        voxel_pos,
                        ClipboardData::generate_from_id(structure_id, &structure_manifest),
                        StartingEnergy::Random,
                    );
                }
            }
        }
    }
}
