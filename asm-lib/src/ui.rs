use crate::pheromones::Pheromones;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_plugin(FrameTimeDiagnosticsPlugin::default())
			.add_startup_system(build_ui)
			.add_system(update_fps)
			.add_system(update_pheromones);
	}
}

struct FpsText;
struct PheromoneText;

fn build_ui(
	commands: &mut Commands,
	asset_server: Res<AssetServer>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	commands
		// 2d camera
		.spawn(UiCameraBundle::default())
		// Root node
		.spawn(NodeBundle {
			style: Style {
				size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
				justify_content: JustifyContent::SpaceBetween,
				..Default::default()
			},
			material: materials.add(Color::NONE.into()),
			..Default::default()
		})
		.with_children(|parent| {
			parent
				// FPS
				.spawn(TextBundle {
					style: Style {
						align_self: AlignSelf::FlexEnd,
						..Default::default()
					},
					text: Text {
						value: "FPS:".to_string(),
						font: asset_server.load("fonts/FiraSans-Bold.ttf"),
						style: TextStyle {
							font_size: 60.0,
							alignment: TextAlignment::default(),
							color: Color::WHITE,
						},
					},
					..Default::default()
				})
				.with(FpsText)
				// Pheromones
				.spawn(TextBundle {
					style: Style {
						align_self: AlignSelf::FlexEnd,
						..Default::default()
					},
					text: Text {
						value: "Pheromones:".to_string(),
						font: asset_server.load("fonts/FiraSans-Bold.ttf"),
						style: TextStyle {
							font_size: 60.0,
							alignment: TextAlignment::default(),
							color: Color::WHITE,
						},
					},
					..Default::default()
				})
				.with(PheromoneText);
		});
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

fn update_pheromones(pheromones: Res<Pheromones>, mut query: Query<(&mut Text, &PheromoneText)>) {
	for (mut text, _tag) in query.iter_mut() {
		text.value = format!("Pheromones: {:.2}", pheromones.supply);
	}
}
