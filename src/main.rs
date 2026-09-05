mod ai;
mod animation;
mod attack;
mod debug;
mod enemies;
mod movement;
mod player;
mod wall;

use animation::AnimationPlugin;
use avian2d::{
    PhysicsPlugins,
    debug_render::{PhysicsDebugPlugin, PhysicsGizmos},
    dynamics::integrator::Gravity,
};
use bevy::prelude::*;
use bevy_behave::prelude::BehavePlugin;
use bevy_ecs_ldtk::{LdtkPlugin, LdtkWorldBundle, LevelSelection};
use debug::DebugPlugin;
use enemies::EnemyPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;
use wall::WallPlugin;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GameSet {
    Input,
    PostInput,
    Animate,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_world, setup_camera));
        app.add_plugins((
            LdtkPlugin,
            BehavePlugin::default(),
            MovementPlugin,
            AnimationPlugin,
            PlayerPlugin,
            EnemyPlugin,
            WallPlugin,
            PhysicsPlugins::default().with_length_unit(50.0),
            DebugPlugin,
            ai::plugin,
        ));
        app.insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: false,
                ..default()
            },
        );
        app.insert_resource(Gravity(Vec2::NEG_Y * 1000.0));
        app.insert_resource(LevelSelection::index(0));
        app.configure_sets(
            Update,
            (GameSet::Input, GameSet::PostInput, GameSet::Animate).chain(),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(304.0, 232.0, 0.0),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::AutoMin {
                min_width: 608.0,
                min_height: 464.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("test_map.ldtk").into(),
        ..default()
    });
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            GamePlugin,
            PhysicsDebugPlugin,
        ))
        .run();
}
