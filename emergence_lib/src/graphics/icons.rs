//! Generates icons from rendered 3D views

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{camera::RenderTarget, view::RenderLayers},
};

use crate::asset_management::structures::StructureHandles;

/// The render layer used to draw icons.
const ICON_LAYER: RenderLayers = RenderLayers::layer(1);

pub(super) fn spawn_icon_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                // Set a white background
                clear_color: ClearColorConfig::Custom(Color::WHITE),
                ..default()
            },
            camera: Camera {
                // Don't render to anything unless it's told to
                target: RenderTarget::Image(Handle::default()),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        IconCamera,
        ICON_LAYER,
    ));
}

#[derive(Component)]
pub(super) struct IconCamera;

/// Spawn a scene that does not currently have an icon, and set it up to be rendered to the correct texture
pub(super) fn generate_icons(
    mut structure_handles: ResMut<StructureHandles>,
    mut camera_query: Query<&mut Camera, With<IconCamera>>,
    mut maybe_scene_root: Local<Option<Entity>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    // Despawn any old icon scene
    if let Some(scene_root) = *maybe_scene_root {
        commands.entity(scene_root).despawn();
        *maybe_scene_root = None;
    }

    let mut icon_camera = camera_query.single_mut();

    let mut maybe_icon_structure_id = None;
    let mut maybe_new_icon = None;

    for (structure_id, scene) in structure_handles.scenes.iter() {
        if !structure_handles.icons.contains_key(structure_id) {
            // Spawn the scene to draw
            let scene_root = commands
                .spawn((
                    ICON_LAYER,
                    SceneBundle {
                        scene: scene.clone_weak(),
                        ..default()
                    },
                ))
                .id();

            *maybe_scene_root = Some(scene_root);

            // Set the camera to draw to the correct image
            let image_handle = image_assets.add(Image::default());
            icon_camera.target = RenderTarget::Image(image_handle.clone_weak());
            maybe_icon_structure_id = Some(*structure_id);
            maybe_new_icon = Some(image_handle);

            // Only try and draw one icon per frame
            break;
        }
    }

    if let (Some(icon_structure_id), Some(new_icon)) = (maybe_icon_structure_id, maybe_new_icon) {
        structure_handles.icons.insert(icon_structure_id, new_icon);
    } else {
        // If all of the icons are rendered, draw to nothing
        icon_camera.target = RenderTarget::Image(Handle::default());
    }
}
