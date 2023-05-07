use bevy::{
    app::AppExit,
    asset::LoadState,
    audio::AudioPlugin,
    gltf::GltfPlugin,
    prelude::*,
    utils::{Duration, HashMap, Instant},
};

use std::fmt::{Display, Formatter};

/// The path to the asset folder, from the root of the repository.
const ROOT_ASSET_FOLDER: &str = "emergence_game/assets";

/// The path to the asset folder from the perspective of the [`AssetPlugin`] is specified relative to the executable.
///
/// As a result, we need to go up two levels to translate.
const PATH_ADAPTOR: &str = "../../";

pub(super) fn verify_assets_load() {
    App::new()
        .init_resource::<AssetHandles>()
        .insert_resource(TimeOut {
            start: Instant::now(),
            max: Duration::from_secs(10),
        })
        .add_plugins(MinimalPlugins)
        // This must come before the asset format plugins for AssetServer to exist
        .add_plugin(AssetPlugin {
            asset_folder: format!("{}{}", PATH_ADAPTOR, ROOT_ASSET_FOLDER),
            watch_for_changes: false,
        })
        // These plugins are required for the asset loaders to be detected.
        // Without this, AssetServer::load_folder will return an empty list
        // as file types without an associated loader registered are silently skipped.
        .add_plugin(ImagePlugin::default())
        .add_plugin(GltfPlugin)
        .add_plugin(AudioPlugin)
        .add_startup_system(load_assets)
        .add_system(check_if_assets_loaded)
        .run()
}

#[derive(Default, Resource, Debug, Clone, PartialEq)]
struct AssetHandles {
    handles: HashMap<String, HandleStatus>,
}

impl Display for AssetHandles {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();

        for (name, handle) in self.handles.iter() {
            string += &format!("    {} - {:?}\n", name, handle.load_state);
        }

        write!(f, "{string}")
    }
}

impl AssetHandles {
    fn all_loaded(&self) -> bool {
        self.handles
            .values()
            .all(|handle| handle.load_state == LoadState::Loaded)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct HandleStatus {
    handle: HandleUntyped,
    load_state: LoadState,
}

#[derive(Resource, Debug)]
struct TimeOut {
    start: Instant,
    max: Duration,
}

impl TimeOut {
    fn timed_out(&self) -> bool {
        self.start.elapsed() > self.max
    }
}

fn load_assets(asset_server: Res<AssetServer>, mut asset_handles: ResMut<AssetHandles>) {
    // Try to load all assets
    let all_handles = asset_server.load_folder(".").unwrap();
    assert!(all_handles.len() > 0);
    for handle in all_handles {
        let asset_path = asset_server.get_handle_path(&handle).unwrap();
        asset_handles.handles.insert(
            asset_path.path().to_str().unwrap().to_string(),
            HandleStatus {
                handle,
                load_state: LoadState::NotLoaded,
            },
        );
    }
}

fn check_if_assets_loaded(
    mut asset_handles: ResMut<AssetHandles>,
    asset_server: Res<AssetServer>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    time_out: Res<TimeOut>,
    mut previous_asset_handles: Local<AssetHandles>,
) {
    *previous_asset_handles = asset_handles.clone();

    for mut handle_status in asset_handles.handles.values_mut() {
        if handle_status.load_state == LoadState::NotLoaded {
            handle_status.load_state =
                asset_server.get_load_state(handle_status.handle.clone_weak());
        }
    }

    if asset_handles.all_loaded() {
        println!("{}", *asset_handles);
        println!("All assets loaded successfully, exiting.");
        app_exit_events.send(AppExit);
    } else {
        if *asset_handles != *previous_asset_handles {
            println!("{}", *asset_handles);
        }

        if time_out.timed_out() {
            panic!("Timed out waiting for assets to load.");
        }
    }
}
