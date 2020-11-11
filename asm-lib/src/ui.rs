use crate::pheromones::Pheromones;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_plugin(FrameTimeDiagnosticsPlugin::default())
			.add_startup_system(ui_setup.system())
			.add_startup_system(fps_setup.system())
			.add_startup_system(pheromones_setup.system())
			.add_system(update_fps.system())
			.add_system(update_pheromones.system());
	}
}

fn ui_setup(commands: &mut Commands) {
	commands
		// 2d camera
		.spawn(UiCameraComponents::default());
}

struct FpsText;

fn fps_setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
	commands
		// texture
		.spawn(TextComponents {
			style: Style {
				align_self: AlignSelf::FlexEnd,
				..Default::default()
			},
			text: Text {
				value: "FPS:".to_string(),
				font: asset_server.load("fonts/FiraSans-Bold.ttf"),
				style: TextStyle {
					font_size: 60.0,
					color: Color::WHITE,
				},
			},
			..Default::default()
		})
		.with(FpsText);
}

fn update_fps(diagnostics: Res<Diagnostics>, mut query: Query<(&mut Text, &FpsText)>) {
	for (mut text, _tag) in query.iter_mut() {
		if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
			if let Some(average) = fps.average() {
				text.value = format!("FPS: {:.2}", average);
			}
		}
	}
}

struct PheromoneText;

fn pheromones_setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
	commands
		// texture
		.spawn(TextComponents {
			style: Style {
				align_self: AlignSelf::Center,
				..Default::default()
			},
			text: Text {
				value: "Pheromones:".to_string(),
				font: asset_server.load("fonts/FiraSans-Bold.ttf"),
				style: TextStyle {
					font_size: 60.0,
					color: Color::WHITE,
				},
			},
			..Default::default()
		})
		.with(PheromoneText);
}

fn update_pheromones(pheromones: Res<Pheromones>, mut query: Query<(&mut Text, &PheromoneText)>) {
	for (mut text, _tag) in query.iter_mut() {
		text.value = format!("Pheromones: {:.2}", pheromones.supply);
	}
}
