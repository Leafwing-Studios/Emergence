//! Displays information about population counts and production over time.

use bevy::prelude::*;

use crate::{
    asset_management::manifest::Id,
    light::TotalLight,
    simulation::{geometry::MapGeometry, time::InGameTime, weather::CurrentWeather},
    units::unit_manifest::Unit,
    water::WaterTable,
    world_gen::WorldGenState,
};

use super::{FiraSansFontFamily, LeftPanel};

use std::fmt::Display;

/// Resources and systems for production statistics
pub(super) struct ProductionStatisticsPlugin;

impl Plugin for ProductionStatisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Census>()
            .add_system(census)
            .add_startup_system(spawn_production_statistics_menu)
            .add_system(update_production_statistics.run_if(in_state(WorldGenState::Complete)));
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
        TextSection::new("CENSUS", style),
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
    water_table: Res<WaterTable>,
    map_geometry: Res<MapGeometry>,
    census: Res<Census>,
) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("{}\n", *in_game_time);
    text.sections[1].value = format!("Weather: {}\n", current_weather.get());
    text.sections[2].value = format!("Light: {}\n", *total_light);
    text.sections[3].value = format!(
        "{} average volume of water per tile \n",
        water_table.average_volume(&map_geometry)
    );
    text.sections[4].value = format!("{}", *census);
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
