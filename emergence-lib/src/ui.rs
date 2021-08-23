use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(build_ui.system());
    }
}

fn build_ui(mut commands: Commands) {
    commands
        // 2d camera
        .spawn_bundle(UiCameraBundle::default());
}
