mod debug;
mod player;
mod rock;

use avian2d::{
    PhysicsPlugins,
    debug_render::{PhysicsDebugPlugin, PhysicsGizmos},
    dynamics::integrator::Gravity,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkPlugin, LdtkWorldBundle, LevelSelection};
use debug::DebugPlugin;
use player::PlayerPlugin;
use rock::RockPlugin;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameSet {
    Input,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_world, setup_camera));
        app.add_plugins((
            LdtkPlugin,
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
        app.insert_resource(LevelSelection::index(0));
        app.configure_sets(Update, (GameSet::Input).chain());
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("test_map.ldtk").into(),
        ..default()
    });
}

fn main() {
    DefaultPlugins.set(ImagePlugin::default_nearest());
    App::new()
        .add_plugins((DefaultPlugins, GamePlugin, PhysicsDebugPlugin))
        .run();
}
