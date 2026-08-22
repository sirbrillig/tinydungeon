mod debug;
mod player;
mod rock;

use avian2d::{
    PhysicsPlugins,
    debug_render::{PhysicsDebugPlugin, PhysicsGizmos},
    dynamics::integrator::Gravity,
};
use bevy::prelude::*;
use player::PlayerPlugin;
use rock::RockPlugin;
use debug::DebugPlugin;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameSet {
    Input,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_plugins((
            PlayerPlugin,
            RockPlugin,
            PhysicsPlugins::default().with_length_unit(50.0),
            DebugPlugin,
        ));
        app.insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: false,
                ..default()
            },
        );
        app.insert_resource(Gravity::ZERO);
        app.configure_sets(Update, (GameSet::Input).chain());
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GamePlugin, PhysicsDebugPlugin))
        .run();
}
