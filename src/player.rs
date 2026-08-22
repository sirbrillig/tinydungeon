use crate::{GameSet, physics::Velocity};
use bevy::prelude::*;

const PLAYER_SIZE: f32 = 10.0;
const PLAYER_SPEED: f32 = 300.0;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
        app.add_systems(Update, move_player.in_set(GameSet::Input));
    }
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = meshes.add(Circle::new(PLAYER_SIZE));
    let color = Color::hsl(230.0, 0.95, 0.7);
    commands.spawn((
        Mesh2d(shape),
        MeshMaterial2d(materials.add(color)),
        Player,
        Velocity::default(),
    ));
}

fn get_change_for_input(keyboard_input: Res<ButtonInput<KeyCode>>) -> Vec2 {
    let mut change = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        change.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        change.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        change.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        change.y -= 1.0;
    }

    change.normalize_or_zero() * PLAYER_SPEED
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vel: Single<&mut Velocity, With<Player>>,
) {
    vel.0 = get_change_for_input(keyboard_input);
}
