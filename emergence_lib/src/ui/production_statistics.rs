//! Displays information about population counts and production over time.

use bevy::{prelude::*, utils::HashMap};

use crate::{
    asset_management::manifest::Id,
    crafting::inventories::{InputInventory, OutputInventory, StorageInventory},
    geometry::Volume,
    items::item_manifest::{Item, ItemManifest},
    light::TotalLight,
    litter::Litter,
    simulation::{time::InGameTime, weather::CurrentWeather},
    units::{item_interaction::UnitInventory, unit_manifest::Unit},
    water::WaterVolume,
    world_gen::WorldGenState,
};

use super::{FiraSansFontFamily, LeftPanel};

use std::fmt::Display;

/// Resources and systems for production statistics
pub(super) struct ProductionStatisticsPlugin;

impl Plugin for ProductionStatisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Census>()
            .init_resource::<ItemCount>()
            .add_systems(
                Update,
                (census, update_item_count).distributive_run_if(in_state(WorldGenState::Complete)),
            )
            .add_startup_system(spawn_production_statistics_menu)
            .add_systems(
                Update,
                update_production_statistics.run_if(in_state(WorldGenState::Complete)),
            );
    }
}

/// Marker component for production stat UI
#[derive(Component)]
struct ProductionStats;

/// Initializes the production statistics menu
fn spawn_production_statistics_menu(
    mut commands: Commands,
    left_panel_query: Query<Entity, With<LeftPanel>>,
    fonts: Res<FiraSansFontFamily>,
) {
    let style = TextStyle {
        font: fonts.regular.clone_weak(),
        font_size: 24.,
        color: Color::WHITE,
    };

    let text = Text::from_sections([
        TextSection::new("TIME", style.clone()),
        TextSection::new("WEATHER", style.clone()),
        TextSection::new("LIGHT", style.clone()),
        TextSection::new("TOTAL_WATER", style.clone()),
        TextSection::new("CENSUS", style.clone()),
        TextSection::new("ITEM_COUNT", style),
    ]);

    let production_stats_entity = commands
        .spawn(TextBundle {
            text,
            ..Default::default()
        })
        .insert(ProductionStats)
        .id();

    let left_panel_entity = left_panel_query.single();
    commands
        .entity(left_panel_entity)
        .add_child(production_stats_entity);
}

/// Updates information about the production statistics to be displayed
fn update_production_statistics(
    mut query: Query<&mut Text, With<ProductionStats>>,
    in_game_time: Res<InGameTime>,
    current_weather: Res<CurrentWeather>,
    total_light: Res<TotalLight>,
    water_volume_query: Query<&WaterVolume>,
    census: Res<Census>,
    item_count: Res<ItemCount>,
    item_manifest: Res<ItemManifest>,
) {
    let mut text = query.single_mut();
    let mut total_water_volume = Volume::ZERO;
    for water_volume in water_volume_query.iter() {
        total_water_volume += water_volume.volume();
    }

    let average_water_volume = total_water_volume / water_volume_query.iter().len() as f32;

    text.sections[0].value = format!("{}\n", *in_game_time);
    text.sections[1].value = format!("Weather: {}\n", current_weather.get());
    text.sections[2].value = format!("Light: {}\n", *total_light);
    text.sections[3].value = format!("{average_water_volume} average volume of water per tile \n",);
    text.sections[4].value = format!("{}\n", *census);
    text.sections[5].value = format!("{}\n", item_count.display(&item_manifest));
}

/// Tracks the population of organisms
#[derive(Debug, Resource, Default)]
pub(crate) struct Census {
    /// The total number of units of any kind
    total_units: usize,
}

impl Display for Census {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Population: {}", self.total_units)
    }
}

/// Counts the number of organisms
fn census(mut census: ResMut<Census>, unit_query: Query<(), With<Id<Unit>>>) {
    census.total_units = unit_query.iter().len();
}

/// Counts the total number of items across all inventories of each type.
#[derive(Debug, Resource, Default)]
struct ItemCount {
    /// The number of items of each type
    map: HashMap<Id<Item>, u32>,
}

impl ItemCount {
    /// Returns a human-readable string representation of the item count
    fn display(&self, item_manifest: &ItemManifest) -> String {
        let mut string = String::new();

        for (item_id, count) in self.map.iter() {
            let name = item_manifest.name(*item_id);
            string.push_str(&format!("{name}: {count}\n"));
        }

        string
    }
}

/// Count the total number of items across all inventories of each type.
fn update_item_count(
    mut item_count: ResMut<ItemCount>,
    input_inventory_query: Query<&InputInventory>,
    output_inventory_query: Query<&OutputInventory>,
    storage_inventory_query: Query<&StorageInventory>,
    unit_inventory_query: Query<&UnitInventory>,
    litter_query: Query<&Litter>,
) {
    // Reset the item count
    item_count.map.clear();

    for inventory in input_inventory_query.iter() {
        for item_slot in inventory.iter() {
            *item_count.map.entry(item_slot.item_id()).or_default() += item_slot.count();
        }
    }

    for inventory in output_inventory_query.iter() {
        for item_slot in inventory.iter() {
            *item_count.map.entry(item_slot.item_id()).or_default() += item_slot.count();
        }
    }

    for inventory in storage_inventory_query.iter() {
        for item_slot in inventory.iter() {
            *item_count.map.entry(item_slot.item_id()).or_default() += item_slot.count();
        }
    }

    for inventory in unit_inventory_query.iter() {
        for item_id in inventory.iter() {
            *item_count.map.entry(*item_id).or_default() += 1;
        }
    }

    for litter in litter_query.iter() {
        for item_slot in litter.contents.iter() {
            *item_count.map.entry(item_slot.item_id()).or_default() += item_slot.count();
        }
    }
}
