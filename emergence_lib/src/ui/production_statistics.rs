//! Displays information about population counts and production over time.

use bevy::prelude::*;

use crate::{
    infovis::Census,
    simulation::{
        geometry::MapGeometry, light::TotalLight, time::InGameTime, weather::CurrentWeather,
    },
    water::WaterTable,
};

use super::{FiraSansFontFamily, LeftPanel};

/// Resources and systems for production statistics
pub(super) struct ProductionStatisticsPlugin;

impl Plugin for ProductionStatisticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_production_statistics_menu)
            .add_system(update_production_statistics);
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
    text.sections[2].value = format!("{}\n", *total_light);
    text.sections[3].value = format!(
        "{} average volume of water per tile \n",
        water_table.average_volume(&map_geometry)
    );
    text.sections[4].value = format!("{}", *census);
}
