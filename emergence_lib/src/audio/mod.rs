//! Controls audio loading and hadles
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// Label for audio plugin
pub struct GameAudioPlugin;

/// Adds all logic to play audio for the game.
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_startup_system(play_background_music);
    }
}

/// Controls playing the background music for the game
// TODO: Add more sounds to play as ambience and background
fn play_background_music(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("audio/music/game/first_landing_demo.mp3"))
        .with_volume(0.5)
        .looped();
}
