use bevy::prelude::*;

const PLAYER_SIZE: f32 = 10.0;
const PLAYER_SPEED: f32 = 300.0;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
        app.add_systems(Update, move_player);
    }
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = meshes.add(Circle::new(PLAYER_SIZE));
    let color = Color::hsl(230.0, 0.95, 0.7);
    commands.spawn((Mesh2d(shape), MeshMaterial2d(materials.add(color)), Player));
}

fn get_change_for_input(keyboard_input: Res<ButtonInput<KeyCode>>, time: Res<Time>) -> Vec2 {
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

    change.normalize_or_zero() * PLAYER_SPEED * time.delta_secs()
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_transform: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let change = get_change_for_input(keyboard_input, time);
    player_transform.translation += change.extend(0.0);
}
