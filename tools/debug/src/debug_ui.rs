//! Debugging user interface for development

use crate::*;

// Modified text_debug example from the Bevy UI examples (https://github.com/bevyengine/bevy/blob/main/examples/ui/text_debug.rs)
/// Tag for the changing fps text component.
#[derive(Component)]
pub struct FpsText;
/// Geneal tag for prototyping changing text ui elements
#[derive(Component)]
pub struct ChangingText;

/// Generate text elements on the screen
pub fn initialize_infotext(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    // Create a text section in the top left corner to draw the fps readout
    commands.spawn((
        TextBundle::from_sections([TextSection::new(
            "",
            TextStyle {
                font,
                font_size: 30.0,
                color: Color::YELLOW_GREEN,
            },
        )])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(15.0),
                left: Val::Px(160.0),
                ..default()
            },
            ..default()
        }),
        FpsText,
    ));
}

/// Change text being displayed on the screen.
pub fn change_infotext(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut fpstext_query: Query<&mut Text, With<FpsText>>,
    bools: Res<DebugInfo>,
) {
    for mut text in &mut fpstext_query {
        let mut fps = 0.0;
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                fps = fps_smoothed;
            }
        }

        let mut frame_time = time.delta_seconds_f64();
        if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        {
            if let Some(frame_time_smoothed) = frame_time_diagnostic.smoothed() {
                frame_time = frame_time_smoothed;
            }
        }

        if bools.dev_mode {
            if bools.show_fps_info {
                // info!("displaying fps info");
                text.sections[0].value = format!(" {fps:.1} fps, {frame_time:.3} ms/frame");
            } else if !bools.show_fps_info {
                text.sections[0].value = format!("");
            }
        }
    }
}
