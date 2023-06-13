//! Controls how the terrain is generated.

use crate::{
    asset_management::manifest::Id,
    geometry::{Facing, Height, MapGeometry, Volume},
    organisms::energy::StartingEnergy,
    player_interaction::clipboard::ClipboardData,
    structures::{commands::StructureCommandsExt, structure_manifest::StructureManifest},
    terrain::{commands::TerrainCommandsExt, terrain_manifest::Terrain},
    utils::noise::simplex_noise,
    water::WaterVolume,
};
use bevy::prelude::*;
use hexx::{shapes::hexagon, Hex};
use rand::{seq::SliceRandom, thread_rng, Rng};

use super::GenerationConfig;

/// Creates the world according to [`GenerationConfig`].
pub(crate) fn generate_terrain(
    mut commands: Commands,
    generation_config: Res<GenerationConfig>,
    map_geometry: Res<MapGeometry>,
) {
    info!("Generating terrain...");
    let mut rng = thread_rng();

    let terrain_weights = &generation_config.terrain_weights;
    let terrain_variants: Vec<Id<Terrain>> = terrain_weights.keys().copied().collect();

    for hex in hexagon(Hex::ZERO, map_geometry.radius) {
        // FIXME: can we not just sample from our terrain_weights directly?
        let &terrain_id = terrain_variants
            .choose_weighted(&mut rng, |terrain_type| {
                terrain_weights.get(terrain_type).unwrap()
            })
            .unwrap();

        // Heights are generated in f32 world coordinates to start
        let hex_height = simplex_noise(hex, &generation_config.low_frequency_noise)
            + simplex_noise(hex, &generation_config.high_frequency_noise);

        // And then discretized to the nearest integer height before being used
        let height = Height::from_world_pos(hex_height);
        let entity = map_geometry.get_terrain(hex).unwrap();

        commands.hydrate_terrain(entity, height, terrain_id);
    }
}

/// Places landmarks according to [`GenerationConfig`].
pub(super) fn generate_landmarks(
    mut commands: Commands,
    generation_config: Res<GenerationConfig>,
    structure_manifest: Res<StructureManifest>,
    map_geometry: Res<MapGeometry>,
) {
    info!("Generating landmarks...");
    let rng = &mut thread_rng();

    for voxel_pos in map_geometry.walkable_voxels() {
        for (&structure_id, &chance) in &generation_config.landmark_chances {
            if rng.gen::<f32>() < chance {
                let mut clipboard_data =
                    ClipboardData::generate_from_id(structure_id, &structure_manifest);
                let facing = Facing::random(rng);
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
                        StartingEnergy::NotAnOrganism,
                    );
                }
            }
        }
    }
}

/// Sets the starting water table
pub(super) fn initialize_water_table(mut water_query: Query<&mut WaterVolume>) {
    let starting_volume = WaterVolume::new(Volume(1.5));

    for mut water_volume in water_query.iter_mut() {
        *water_volume = starting_volume;
    }
}
