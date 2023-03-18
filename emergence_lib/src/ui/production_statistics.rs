//! Displays information about population counts and production over time.

use bevy::prelude::*;

use crate::infovis::Census;

use super::{FiraSansFontFamily, LeftPanel};

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

/// Intializes the production statistics menu
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

    let text = Text::from_section("CENSUS", style);

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
    census: Res<Census>,
) {
    let mut census_text = query.single_mut();
    census_text.sections[0].value = format!("{}", *census);
}
