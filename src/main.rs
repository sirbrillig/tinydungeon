mod physics;
mod player;
mod rock;

use bevy::{ecs::schedule::{LogLevel, ScheduleBuildSettings}, prelude::*};
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use rock::RockPlugin;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameSet {
    Input,
    Movement,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        });
        app.add_systems(Startup, setup_camera);
        app.add_plugins((PlayerPlugin, RockPlugin, PhysicsPlugin));
        app.configure_sets(Update, (GameSet::Input, GameSet::Movement).chain());
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new().add_plugins((DefaultPlugins, GamePlugin)).run();
}
