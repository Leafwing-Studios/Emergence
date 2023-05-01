//! Controls how the terrain is generated.

use crate::{
    asset_management::manifest::Id,
    player_interaction::clipboard::ClipboardData,
    simulation::geometry::{Facing, Height, MapGeometry, TilePos},
    structures::{commands::StructureCommandsExt, structure_manifest::StructureManifest},
    terrain::{commands::TerrainCommandsExt, terrain_manifest::Terrain},
    utils::noise::simplex_noise,
    water::WaterTable,
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

        let tile_pos = TilePos { hex };
        // Heights are generated in f32 world coordinates to start
        let hex_height = simplex_noise(tile_pos, &generation_config.low_frequency_noise)
            + simplex_noise(tile_pos, &generation_config.high_frequency_noise);

        // And then discretized to the nearest integer height before being used
        let height = Height::from_world_pos(hex_height);

        commands.spawn_terrain(tile_pos, height, terrain_id);
    }
}

/// Places landmarks according to [`GenerationConfig`].
pub(super) fn generate_landmarks(
    mut commands: Commands,
    generation_config: Res<GenerationConfig>,
    structure_manifest: Res<StructureManifest>,
    mut height_query: Query<&mut Height>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    info!("Generating landmarks...");
    let rng = &mut thread_rng();

    for tile_pos in map_geometry
        .valid_tile_positions()
        .collect::<Vec<TilePos>>()
    {
        for (&structure_id, &chance) in &generation_config.landmark_chances {
            if rng.gen::<f32>() < chance {
                let mut clipboard_data =
                    ClipboardData::generate_from_id(structure_id, &structure_manifest);
                let facing = Facing::random(rng);
                clipboard_data.facing = facing;
                let footprint = &structure_manifest.get(structure_id).footprint;

                // Only try to spawn a structure if the location is valid and there is space
                if map_geometry.is_footprint_valid(tile_pos, footprint, &facing)
                    && map_geometry.is_space_available(tile_pos, footprint, &facing)
                {
                    // Flatten the terrain under the structure before spawning it
                    map_geometry.flatten_height(&mut height_query, tile_pos, footprint, &facing);
                    commands.spawn_structure(
                        tile_pos,
                        ClipboardData::generate_from_id(structure_id, &structure_manifest),
                    );
                }
            }
        }
    }
}

/// Sets the starting water table
pub(super) fn initialize_water_table(
    mut water_table: ResMut<WaterTable>,
    map_geometry: Res<MapGeometry>,
) {
    /// The minimum starting water level for low lying areas
    const LOW_WATER_LINE: Height = Height(1.5);

    for tile_pos in map_geometry.valid_tile_positions() {
        water_table.set(tile_pos, LOW_WATER_LINE);
    }
}
