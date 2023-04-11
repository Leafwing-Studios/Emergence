//! Generating and representing terrain as game objects.

use bevy::prelude::*;
use bevy_mod_raycast::RaycastMesh;

use crate::asset_management::manifest::plugin::ManifestPlugin;
use crate::asset_management::manifest::Id;
use crate::asset_management::AssetCollectionExt;
use crate::construction::zoning::Zoning;
use crate::crafting::components::StorageInventory;
use crate::crafting::item_tags::ItemKind;
use crate::organisms::energy::VigorModifier;
use crate::player_interaction::selection::ObjectInteraction;
use crate::signals::{Emitter, SignalModifier, SignalStrength, SignalType};
use crate::simulation::geometry::{Height, MapGeometry, TilePos};
use crate::simulation::SimulationSet;

use self::terrain_assets::TerrainHandles;
use self::terrain_manifest::{RawTerrainManifest, Terrain};

pub(crate) mod commands;
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
                    set_terrain_emitters.in_set(TerrainEmitters),
                    update_litter_index,
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
    /// The location of this terrain hex
    tile_pos: TilePos,
    /// The height of this terrain hex
    height: Height,
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
    /// Modifies the intensity of emitters on this tile.
    signal_modifer: SignalModifier,
    /// Modifies the rate of work and the energy costs on this tile.
    vigor_modifier: VigorModifier,
    /// Controls the signals produced by this terrain tile.
    emitter: Emitter,
    /// Stores littered items
    storage_inventory: StorageInventory,
}

impl TerrainBundle {
    /// Creates a new Terrain entity.
    fn new(
        terrain_id: Id<Terrain>,
        tile_pos: TilePos,
        scene: Handle<Scene>,
        mesh: Handle<Mesh>,
        map_geometry: &MapGeometry,
    ) -> Self {
        let world_pos = tile_pos.into_world_pos(map_geometry);
        let scene_bundle = SceneBundle {
            scene,
            transform: Transform::from_translation(world_pos),
            ..Default::default()
        };

        let height = map_geometry.get_height(tile_pos).unwrap();

        TerrainBundle {
            terrain_id,
            tile_pos,
            height,
            raycast_mesh: RaycastMesh::<Terrain>::default(),
            mesh,
            object_interaction: ObjectInteraction::None,
            zoning: Zoning::None,
            scene_bundle,
            signal_modifer: SignalModifier::None,
            vigor_modifier: VigorModifier::None,
            emitter: Emitter::default(),
            storage_inventory: StorageInventory::new(1, None),
        }
    }
}

/// Updates the game state appropriately whenever the height of a tile is changed.
fn respond_to_height_changes(
    mut terrain_query: Query<(Ref<Height>, &TilePos, &mut Transform, &Children)>,
    mut column_query: Query<&mut Transform, (With<Parent>, Without<Height>)>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (height, &tile_pos, mut transform, children) in terrain_query.iter_mut() {
        if height.is_changed() {
            map_geometry.update_height(tile_pos, *height);
            transform.translation.y = height.into_world_pos();
            // During terrain initialization we ensure that the column is always the 0th child
            let column_child = children[0];
            let mut column_transform = column_query.get_mut(column_child).unwrap();
            *column_transform = height.column_transform();
        }
    }
}

/// Updates the signals produced by terrain tiles.
fn set_terrain_emitters(
    mut query: Query<(&mut Emitter, Ref<StorageInventory>), With<Id<Terrain>>>,
) {
    for (mut emitter, storage_inventory) in query.iter_mut() {
        if storage_inventory.is_changed() {
            emitter.signals.clear();
            for item_slot in storage_inventory.iter() {
                let item_kind = ItemKind::Single(item_slot.item_id());

                let signal_type = match storage_inventory.is_full() {
                    true => SignalType::Push(item_kind),
                    false => SignalType::Contains(item_kind),
                };
                let signal_strength = SignalStrength::new(10.);

                emitter.signals.push((signal_type, signal_strength));
            }
        }
    }
}

#[derive(SystemSet, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TerrainEmitters;

/// Tracks how much litter is on the ground on each tile.
fn update_litter_index(
    query: Query<(&TilePos, &StorageInventory), (With<Id<Terrain>>, Changed<StorageInventory>)>,
    mut map_geometry: ResMut<MapGeometry>,
) {
    for (&tile_pos, litter) in query.iter() {
        map_geometry.set_litter_state(tile_pos, litter.state());
    }
}
