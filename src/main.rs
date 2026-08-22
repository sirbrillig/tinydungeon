mod player;

use bevy::prelude::*;
use player::PlayerPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_plugins(PlayerPlugin);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new().add_plugins((DefaultPlugins, GamePlugin)).run();
}
