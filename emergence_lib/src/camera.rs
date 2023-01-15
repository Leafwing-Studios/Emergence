//! Camera controls and movement.
//!
//! This RTS-style camera can zoom and pan.
//!
//! Adapted from [`bevy_pancam`](https://github.com/johanhelsing/bevy_pancam/blob/f4da39912a8982baa63b4bc5502e5f83f4338cc5/src/lib.rs)
//! (thank you, Johann Helsing and co.) We additionally provide integration with
//! [Leafwing Input Manager](https://crates.io/crates/leafwing-input-manager).

use bevy::{prelude::*, render::camera::OrthographicProjection};
#[cfg(feature = "debug_tools")]
use debug_tools::bevy_egui;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::axislike::SingleAxis;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::VirtualDPad;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputManagerBundle;

/// Camera logic
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PanCam>();

        app.add_plugin(InputManagerPlugin::<CameraZoom>::default())
            .add_plugin(InputManagerPlugin::<CameraAction>::default())
            .add_startup_system_to_stage(StartupStage::Startup, setup)
            .add_system(camera_movement.label(PanCamSystemLabel))
            .add_system(camera_zoom.label(PanCamSystemLabel));
    }
}

/// Spawns a [`Camera2dBundle`] and sets up the [`InputManagerBundle`]s that handle camera motion
fn setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default())
        .insert(InputManagerBundle::<CameraZoom> {
            input_map: InputMap::default()
                .insert(SingleAxis::mouse_wheel_y(), CameraZoom::Zoom)
                .build(),
            ..default()
        })
        .insert(InputManagerBundle::<CameraAction> {
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), CameraAction::Pan)
                .insert(VirtualDPad::arrow_keys(), CameraAction::Pan)
                .build(),
            ..default()
        });
}

/// A component that adds panning camera controls to an orthographic camera
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PanCam {
    /// How sensitive the camera is to zoom actions (mouse wheel scroll or touchpad pinch)
    pub zoom_sensitivity: f32,
    /// Camera button pan sensitivity
    pub button_pan_sensitivity: f32,
    /// When true, zooming the camera will center on the mouse cursor
    ///
    /// When false, the camera will stay in place, zooming towards the
    /// middle of the screen
    pub zoom_to_cursor: bool,
    /// The minimum scale for the camera
    ///
    /// The orthographic projection's scale will be clamped at this value when zooming in
    pub min_scale: f32,
    /// The maximum scale for the camera
    ///
    /// If present, the orthographic projection's scale will be clamped at
    /// this value when zooming out.
    pub max_scale: Option<f32>,
    /// The minimum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_x: Option<f32>,
    /// The maximum x position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_x: Option<f32>,
    /// The minimum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub min_y: Option<f32>,
    /// The maximum y position of the camera window
    ///
    /// If present, the orthographic projection will be clamped to this boundary both
    /// when dragging the window, and zooming out.
    pub max_y: Option<f32>,
}

impl Default for PanCam {
    fn default() -> Self {
        Self {
            zoom_sensitivity: 100.,
            button_pan_sensitivity: 8.,
            zoom_to_cursor: true,
            min_scale: 0.00001,
            max_scale: None,
            min_x: None,
            max_x: None,
            min_y: None,
            max_y: None,
        }
    }
}

/// Enumerates actions that are managed by `leafwing_input_manager` for camera zoom
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
enum CameraZoom {
    /// Camera zoom
    Zoom,
}

/// Actions that manipulate the camera
#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
enum CameraAction {
    /// Move the camera from side to side
    Pan,
}

/// Plugin that adds the necessary systems for `PanCam` components to work
#[derive(Default)]
pub struct PanCamPlugin;

/// Label to allow ordering of `PanCamPlugin`
#[derive(SystemLabel)]
pub struct PanCamSystemLabel;

impl Plugin for PanCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_movement.label(PanCamSystemLabel))
            .add_system(camera_zoom.label(PanCamSystemLabel));

        app.register_type::<PanCam>();
    }
}

/// Handles camera zoom
fn camera_zoom(
    mut query: Query<(
        &PanCam,
        &mut OrthographicProjection,
        &mut Transform,
        &ActionState<CameraZoom>,
    )>,
    windows: Res<Windows>,
    #[cfg(feature = "debug_tools")] egui_ctx: Option<ResMut<bevy_egui::EguiContext>>,
) {
    #[cfg(feature = "debug_tools")]
    if let Some(mut egui_ctx) = egui_ctx {
        if egui_ctx.ctx_mut().wants_pointer_input() || egui_ctx.ctx_mut().wants_keyboard_input() {
            return;
        }
    }

    let (cam, mut proj, mut pos, action_state) = query.single_mut();

    let scroll = cam.zoom_sensitivity * action_state.value(CameraZoom::Zoom);

    if scroll == 0. {
        return;
    }
    info!("scroll: {scroll}");

    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width(), window.height());
    let mouse_normalized_screen_pos = window
        .cursor_position()
        .map(|cursor_pos| (cursor_pos / window_size) * 2. - Vec2::ONE);

    let old_scale = proj.scale;
    proj.scale = (proj.scale * (1. + -scroll * 0.001)).max(cam.min_scale);

    // Apply max scale constraint
    if let Some(max_scale) = cam.max_scale {
        proj.scale = proj.scale.min(max_scale);
    }

    // If there is both a min and max x boundary, that limits how far we can zoom. Make sure we don't exceed that
    if let (Some(min_x_bound), Some(max_x_bound)) = (cam.min_x, cam.max_x) {
        let max_safe_scale = max_scale_within_x_bounds(min_x_bound, max_x_bound, &proj);
        proj.scale = proj.scale.min(max_safe_scale);
    }
    // If there is both a min and max y boundary, that limits how far we can zoom. Make sure we don't exceed that
    if let (Some(min_y_bound), Some(max_y_bound)) = (cam.min_y, cam.max_y) {
        let max_safe_scale = max_scale_within_y_bounds(min_y_bound, max_y_bound, &proj);
        proj.scale = proj.scale.min(max_safe_scale);
    }

    // Move the camera position to normalize the projection window
    if let (Some(mouse_normalized_screen_pos), true) =
        (mouse_normalized_screen_pos, cam.zoom_to_cursor)
    {
        let proj_size = Vec2::new(proj.right, proj.top);
        let mouse_world_pos =
            pos.translation.truncate() + mouse_normalized_screen_pos * proj_size * old_scale;
        pos.translation = (mouse_world_pos - mouse_normalized_screen_pos * proj_size * proj.scale)
            .extend(pos.translation.z);

        // As we zoom out, we don't want the viewport to move beyond the provided boundary. If the most recent
        // change to the camera zoom would move cause parts of the window beyond the boundary to be shown, we
        // need to change the camera position to keep the viewport within bounds. The four if statements below
        // provide this behavior for the min and max x and y boundaries.
        let proj_size = Vec2::new(proj.right - proj.left, proj.top - proj.bottom) * proj.scale;

        let half_of_viewport = proj_size / 2.;

        if let Some(min_x_bound) = cam.min_x {
            let min_safe_cam_x = min_x_bound + half_of_viewport.x;
            pos.translation.x = pos.translation.x.max(min_safe_cam_x);
        }
        if let Some(max_x_bound) = cam.max_x {
            let max_safe_cam_x = max_x_bound - half_of_viewport.x;
            pos.translation.x = pos.translation.x.min(max_safe_cam_x);
        }
        if let Some(min_y_bound) = cam.min_y {
            let min_safe_cam_y = min_y_bound + half_of_viewport.y;
            pos.translation.y = pos.translation.y.max(min_safe_cam_y);
        }
        if let Some(max_y_bound) = cam.max_y {
            let max_safe_cam_y = max_y_bound - half_of_viewport.y;
            pos.translation.y = pos.translation.y.min(max_safe_cam_y);
        }
    }
}

/// Used to find the maximum safe zoom out/projection scale when
/// we have been provided with minimum and maximum x boundaries for the camera.
fn max_scale_within_x_bounds(
    min_x_bound: f32,
    max_x_bound: f32,
    proj: &OrthographicProjection,
) -> f32 {
    let bounds_width = max_x_bound - min_x_bound;

    // projection width in world space:
    // let proj_width = (proj.right - proj.left) * proj.scale;

    // we're at the boundary when proj_width == bounds_width
    // that means (proj.right - proj.left) * scale == bounds_width

    // if we solve for scale, we get:
    bounds_width / (proj.right - proj.left)
}

/// Used to find the maximum safe zoom out/projection scale when
/// we have been provided with minimum and maximum y boundaries for the camera. It behaves
/// identically to [`max_scale_within_x_bounds`] but uses the height of the window and projection
/// instead of their width.
fn max_scale_within_y_bounds(
    min_y_bound: f32,
    max_y_bound: f32,
    proj: &OrthographicProjection,
) -> f32 {
    let bounds_height = max_y_bound - min_y_bound;

    // projection height in world space:
    // let proj_height = (proj.top - proj.bottom) * proj.scale;

    // we're at the boundary when proj_height == bounds_height
    // that means (proj.top - proj.bottom) * scale == bounds_height

    // if we solve for scale, we get:
    bounds_height / (proj.top - proj.bottom)
}

/// Handles camera movement
fn camera_movement(
    windows: Res<Windows>,
    mut camera_query: Query<(
        &PanCam,
        &mut Transform,
        &OrthographicProjection,
        &ActionState<CameraAction>,
    )>,
    #[cfg(feature = "debug_tools")] egui_ctx: Option<ResMut<bevy_egui::EguiContext>>,
) {
    #[cfg(feature = "debug_tools")]
    if let Some(mut egui_ctx) = egui_ctx {
        if egui_ctx.ctx_mut().wants_pointer_input() || egui_ctx.ctx_mut().wants_keyboard_input() {
            return;
        }
    }

    if let Ok((cam, mut transform, projection, action_state)) = camera_query.get_single_mut() {
        if let Some(window) = windows.get_primary() {
            let window_size = Vec2::new(window.width(), window.height());

            if let Some(camera_pan_data) = action_state.clamped_axis_pair(CameraAction::Pan) {
                let camera_pan_vector = camera_pan_data.xy() * cam.button_pan_sensitivity;

                let proj_size = Vec2::new(
                    projection.right - projection.left,
                    projection.top - projection.bottom,
                ) * projection.scale;

                let world_units_per_device_pixel = proj_size / window_size;

                // The proposed new camera position
                let delta_world = camera_pan_vector * world_units_per_device_pixel;
                let mut proposed_cam_transform = transform.translation + delta_world.extend(0.);

                // Check whether the proposed camera movement would be within the provided boundaries, override it if we
                // need to do so to stay within bounds.
                if let Some(min_x_boundary) = cam.min_x {
                    let min_safe_cam_x = min_x_boundary + proj_size.x / 2.;
                    proposed_cam_transform.x = proposed_cam_transform.x.max(min_safe_cam_x);
                }
                if let Some(max_x_boundary) = cam.max_x {
                    let max_safe_cam_x = max_x_boundary - proj_size.x / 2.;
                    proposed_cam_transform.x = proposed_cam_transform.x.min(max_safe_cam_x);
                }
                if let Some(min_y_boundary) = cam.min_y {
                    let min_safe_cam_y = min_y_boundary + proj_size.y / 2.;
                    proposed_cam_transform.y = proposed_cam_transform.y.max(min_safe_cam_y);
                }
                if let Some(max_y_boundary) = cam.max_y {
                    let max_safe_cam_y = max_y_boundary - proj_size.y / 2.;
                    proposed_cam_transform.y = proposed_cam_transform.y.min(max_safe_cam_y);
                }

                transform.translation = proposed_cam_transform;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::OrthographicProjection;

    use super::*;

    // Simple mock function to construct a square projection window and run it, plus some square boundaries, through
    // the provided scale func
    fn mock_scale_func(
        proj_size: f32,
        bound_width: f32,
        scale_func: &dyn Fn(f32, f32, &OrthographicProjection) -> f32,
    ) -> f32 {
        let proj = OrthographicProjection {
            left: -(proj_size / 2.),
            bottom: -(proj_size / 2.),
            right: (proj_size / 2.),
            top: (proj_size / 2.),
            ..default()
        };
        let min_bound = -(bound_width / 2.);
        let max_bound = bound_width / 2.;

        scale_func(min_bound, max_bound, &proj)
    }

    // projection and bounds are equal-width, both have symmetric edges. Expect max scale of 1.0
    #[test]
    fn test_max_scale_x_01() {
        assert_eq!(mock_scale_func(100., 100., &max_scale_within_x_bounds), 1.);
    }

    // boundaries are 1/2 the size of the projection window, expects max scale of 0.5
    #[test]
    fn test_max_scale_x_02() {
        assert_eq!(mock_scale_func(100., 50., &max_scale_within_x_bounds), 0.5);
    }

    // boundaries are 2x the size of the projection window, expects max scale of 2.0
    #[test]
    fn test_max_scale_x_03() {
        assert_eq!(mock_scale_func(100., 200., &max_scale_within_x_bounds), 2.);
    }

    // projection and bounds are equal-height, expects max scale of 1.0
    #[test]
    fn test_max_scale_y_01() {
        assert_eq!(mock_scale_func(100., 100., &max_scale_within_y_bounds), 1.);
    }

    // boundaries are 1/2 the size of the projection window, expects max scale of 0.5
    #[test]
    fn test_max_scale_y_02() {
        assert_eq!(mock_scale_func(100., 50., &max_scale_within_y_bounds), 0.5);
    }

    // boundaries are 2x the size of the projection window, expects max scale of 2.0
    #[test]
    fn test_max_scale_y_03() {
        assert_eq!(mock_scale_func(100., 200., &max_scale_within_y_bounds), 2.);
    }
}
