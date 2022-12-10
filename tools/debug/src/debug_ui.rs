//! Debugging user interface for development
use crate::*;

/// Generate debug labels for tile positions
pub fn generate_debug_labels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tilemap_q: Query<(&Transform, &TilemapType, &TilemapGridSize)>,
    tile_q: Query<&TilePos>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font,
        font_size: 15.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::CENTER;

    for q in &tilemap_q {
        let (tilemap_transform, map_type, grid_size) = q;

        let label_bundles: Vec<Text2dBundle> = tile_q
            .iter()
            .map(|tile_pos| {
                let tile_pos_transform = Transform::from_translation(
                    tile_pos.center_in_world(grid_size, map_type).extend(1.0),
                );
                let transform = *tilemap_transform * tile_pos_transform;
                Text2dBundle {
                    text: Text::from_section(
                        format!("{}, {}", tile_pos.x, tile_pos.y),
                        text_style.clone(),
                    )
                    .with_alignment(text_alignment),
                    transform,
                    ..Default::default()
                }
            })
            .collect();
        commands.spawn_batch(label_bundles);
    }
}

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
                color: Color::LIME_GREEN,
            },
        )])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(15.0),
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
    // key: Res<Input<KeyCode>>, // add this later to toggle the fps display
    mut fpstext_query: Query<&mut Text, With<FpsText>>,
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

        text.sections[0].value = format!(" {:.1} fps, {:.3} ms/frame", fps, frame_time,);
    }
}
