//! Generating and representing terrain as game objects.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate::asset_management::manifest::plugin::ManifestPlugin;
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetCollectionExt;
use crate::construction::zoning::Zoning;
use crate::geometry::{MapGeometry, VoxelPos};
use crate::light::shade::{ReceivedLight, Shade};
use crate::player_interaction::selection::ObjectInteraction;
use crate::signals::Emitter;
use crate::simulation::SimulationSet;
use crate::water::{WaterBundle, WaterSet};

use self::litter::{
    carry_floating_litter_with_current, clear_empty_litter, make_litter_float,
    set_terrain_emitters, update_litter_index, Litter, LitterDrift, TerrainEmitters,
};
use self::terrain_assets::TerrainHandles;
use self::terrain_manifest::{RawTerrainManifest, Terrain, TerrainManifest};

pub(crate) mod commands;
pub(crate) mod litter;
pub(crate) mod terrain_assets;
pub mod terrain_manifest;

/// All logic and initialization needed for terrain.
pub(crate) struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ManifestPlugin::<RawTerrainManifest>::new())
            .add_asset_collection::<TerrainHandles>()
            .add_systems(
                (
                    respond_to_height_changes,
                    make_litter_float.after(respond_to_height_changes),
                    carry_floating_litter_with_current
                        .after(make_litter_float)
                        .after(WaterSet::HorizontalWaterMovement),
                    // We need two copies of this system
                    // because we care about cleaning up litter inventories before we try and drift
                    // but we also want to clean up after because we may have condensed litter inventories by drifting
                    clear_empty_litter.before(carry_floating_litter_with_current),
                    clear_empty_litter.after(carry_floating_litter_with_current),
                    set_terrain_emitters
                        .after(carry_floating_litter_with_current)
                        .in_set(TerrainEmitters),
                    update_litter_index.after(carry_floating_litter_with_current),
                )
                    .in_set(SimulationSet)
                    .in_schedule(CoreSchedule::FixedUpdate),
            );
    }
}

/// All of the components needed to define a piece of terrain.
#[derive(Bundle)]
struct TerrainBundle {
    /// The type of terrain
    terrain_id: Id<Terrain>,
    /// The location and height of this terrain hex
    voxel_pos: VoxelPos,
    /// Makes the tiles pickable
    raycast_mesh: RaycastMesh<Terrain>,
    /// The mesh used for raycasting
    mesh: Handle<Mesh>,
    /// How is the terrain being interacted with?
    object_interaction: ObjectInteraction,
    /// The structure that should be built here.
    zoning: Zoning,
    /// The scene used to construct the terrain tile.
    scene_bundle: SceneBundle,
    /// Controls the signals produced by this terrain tile.
    emitter: Emitter,
    /// Stores littered items
    litter: Litter,
    /// Tracks how long until litter drifts away
    litter_drift: LitterDrift,
    /// The amount of shade cast on this tile.
    shade: Shade,
    /// The amount of light currently being received by this tile.
    received_light: ReceivedLight,
    /// The components used to track the water table at this tile.
    water_bundle: WaterBundle,
}

impl TerrainBundle {
    /// Creates a new Terrain entity.
    fn new(
        terrain_id: Id<Terrain>,
        voxel_pos: VoxelPos,
        scene: Handle<Scene>,
        mesh: Handle<Mesh>,
        terrain_manifest: &TerrainManifest,
        map_geometry: &MapGeometry,
    ) -> Self {
        let world_pos = voxel_pos.into_world_pos(map_geometry);
        let scene_bundle = SceneBundle {
            scene,
            transform: Transform::from_translation(world_pos),
            ..Default::default()
        };

        let height = map_geometry.get_height(voxel_pos.hex).unwrap();
        let terrain_data = terrain_manifest.get(terrain_id);

        TerrainBundle {
            terrain_id,
            voxel_pos,
            raycast_mesh: RaycastMesh::<Terrain>::default(),
            mesh,
            object_interaction: ObjectInteraction::None,
            zoning: Zoning::None,
            scene_bundle,
            emitter: Emitter::default(),
            litter: Litter::default(),
            litter_drift: LitterDrift::default(),
            shade: Shade::default(),
            received_light: ReceivedLight::default(),
            water_bundle: WaterBundle {
                soil_water_capacity: terrain_data.soil_water_capacity,
                soil_water_evaporation_rate: terrain_data.soil_water_evaporation_rate,
                soil_water_flow_rate: terrain_data.soil_water_flow_rate,
                ..Default::default()
            },
        }
    }
}

/// Updates the game state appropriately whenever the height of a tile is changed.
fn respond_to_height_changes(
    mut terrain_query: Query<(Ref<VoxelPos>, &mut Transform, &Children), With<Id<Terrain>>>,
    mut column_query: Query<&mut Transform, (With<Parent>, Without<VoxelPos>)>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (voxel_pos, mut transform, children) in terrain_query.iter_mut() {
        if voxel_pos.is_changed() {
            let height = voxel_pos.height();
            map_geometry.update_height(voxel_pos.hex, height);
            transform.translation.y = height.into_world_pos();
            // During terrain initialization we ensure that the column is always the 0th child
            let column_child = children[0];
            let mut column_transform = column_query.get_mut(column_child).unwrap();
            *column_transform = height.column_transform();
        }
    }
}
