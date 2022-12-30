//! Debugging user interface for development

use crate::*;
use bevy::prelude::*;

// Modified text_debug example from the Bevy UI examples (https://github.com/bevyengine/bevy/blob/main/examples/ui/text_debug.rs)
/// Tag for the changing fps text component.
#[derive(Component)]
pub struct FpsText;

/// A marker entity for the developer controls
#[derive(Component)]
pub struct DevControlMarker;

/// Generate text elements on the screen
pub fn initialize_infotext(mut commands: Commands, asset_server: Res<AssetServer>) {
    init_dev_action(&mut commands);
    init_fps_text(&mut commands, asset_server);
}

fn init_dev_action(commands: &mut Commands) {
    let dev_controls = DevControls::default();
    commands
        .spawn(DevControlMarker)
        .insert(InputManagerBundle::<DevAction> {
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (dev_controls.toggle_dev_mode, DevAction::ToggleDevMode),
                (dev_controls.toggle_tile_labels, DevAction::ToggleTileLabels),
                (dev_controls.toggle_fps, DevAction::ToggleInfoText),
                (dev_controls.toggle_inspector, DevAction::ToggleInspector),
            ]),
        });
}

fn init_fps_text(commands: &mut Commands, asset_server: Res<AssetServer>) {
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
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
    debug_info: Res<DebugInfo>,
) {
    let mut fps_text = fps_text_query.single_mut();

    // Clear out any fps text if it shouldn't be showing
    if !debug_info.dev_mode || !debug_info.show_fps_info {
        if !fps_text.sections[0].value.is_empty() {
            fps_text.sections[0].value = String::new();
        }
        return;
    }

    if debug_info.show_fps_info {
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

        fps_text.sections[0].value = format!(" {fps:.1} fps, {frame_time:.3} ms/frame");
    }
}

/// Toggle showing debug info
pub fn show_debug_info(
    dev: Query<&ActionState<DevAction>, With<DevControlMarker>>,
    mut debug_info: ResMut<DebugInfo>,
) {
    let dev = dev.single();
    let dev_mode = dev.just_pressed(DevAction::ToggleDevMode);

    // Toggle the dev mode so that what happens is intuitive to the user
    if dev_mode {
        if debug_info.dev_mode {
            debug_info.disable();
            info!("Debug Info disabled");
        } else {
            debug_info.enable();
            info!("Debug Info enabled");
        }
    }

    if !debug_info.dev_mode {
        // There is no other work to be done on this callback
        // The DebugInfo shouldn't be changed after it gets disabled, or is already disabled
        return;
    }

    let tile_labels = dev.just_pressed(DevAction::ToggleTileLabels);
    let fps_info = dev.just_pressed(DevAction::ToggleInfoText);
    let inspector = dev.just_pressed(DevAction::ToggleInspector);

    toggle_debug_var(tile_labels, &mut debug_info.show_tile_labels, "Tile labels");
    toggle_debug_var(fps_info, &mut debug_info.show_fps_info, "FPS info");
    toggle_debug_var(inspector, &mut debug_info.show_inspector, "Egui inspector");
}

/// Toggle a debug variable only if the given action is active.
/// Also logs out which variable is being toggled.
fn toggle_debug_var(is_action_active: bool, var: &mut bool, log_str: &'static str) {
    if !is_action_active {
        return;
    }

    if *var {
        *var = false;
        info!("{} off", log_str);
    } else {
        *var = true;
        info!("{} on", log_str);
    }
}
