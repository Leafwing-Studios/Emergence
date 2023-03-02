//! Generates icons from rendered 3D views

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};

use crate::asset_management::{structures::StructureHandles, AssetState};

/// The render layer used to draw icons.
const ICON_LAYER: RenderLayers = RenderLayers::layer(1);

/// The base width of all icons.
pub(crate) const ICON_SIZE: f32 = 64.0;

pub(super) fn spawn_icon_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                // Set a white background
                clear_color: ClearColorConfig::Custom(Color::PINK),
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
    asset_state: Res<State<AssetState>>,
) {
    // TODO: use a real run condition post stageless
    // It's awful to try and manage this before that because states are bound to a single stage lol
    if *asset_state.current() != AssetState::Ready {
        info!("Assets are not yet loaded");
        return;
    }

    /// The extents of the image to render to
    const ICON_IMAGE_EXTENTS: Extent3d = Extent3d {
        width: ICON_SIZE as u32,
        height: ICON_SIZE as u32,
        // The default value
        depth_or_array_layers: 1,
    };

    // Despawn any old icon scene
    if let Some(scene_root) = *maybe_scene_root {
        commands.entity(scene_root).despawn_recursive();
        *maybe_scene_root = None;
    }

    let mut icon_camera = camera_query.single_mut();

    let mut maybe_icon_structure_id = None;
    let mut maybe_new_icon = None;

    for (structure_id, scene) in structure_handles.scenes.iter() {
        if !structure_handles.icons.contains_key(structure_id) {
            info!("Drawing icon for {structure_id}");

            // Create the image to write to
            let mut image = Image {
                texture_descriptor: TextureDescriptor {
                    label: None,
                    size: ICON_IMAGE_EXTENTS,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Bgra8UnormSrgb,
                    mip_level_count: 1,
                    sample_count: 1,
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT,
                },
                ..default()
            };

            // Fill it with zeros
            image.resize(ICON_IMAGE_EXTENTS);

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
            let image_handle = image_assets.add(image);
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
