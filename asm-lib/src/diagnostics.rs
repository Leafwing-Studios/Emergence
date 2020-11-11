use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct DiagnosticsPlugin;
impl Plugin for DiagnosticsPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_plugin(FrameTimeDiagnosticsPlugin::default())
			.add_startup_system(diagnostics_setup.system())
			.add_system(fps_update_system.system());
	}
}

struct FpsText;

fn fps_update_system(diagnostics: Res<Diagnostics>, mut query: Query<(&mut Text, &FpsText)>) {
	for (mut text, _tag) in query.iter_mut() {
		if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
			if let Some(average) = fps.average() {
				text.value = format!("FPS: {:.2}", average);
			}
		}
	}
}

fn diagnostics_setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
	commands
		// 2d camera
		.spawn(UiCameraComponents::default())
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
