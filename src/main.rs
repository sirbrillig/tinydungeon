mod physics;
mod player;
mod rock;

use bevy::prelude::*;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use rock::RockPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_plugins((PlayerPlugin, PhysicsPlugin));
         app.add_plugins((PlayerPlugin, RockPlugin, PhysicsPlugin));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new().add_plugins((DefaultPlugins, GamePlugin)).run();
}
