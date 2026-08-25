use crate::GameSet;
use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LinearVelocity, LockedAxes, RigidBody},
};
use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkEntity, Worldly, app::LdtkEntityAppExt};

const PLAYER_SPEED: f32 = 300.0;

#[derive(Component, Default)]
pub struct Player;

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet("Walk.png", 64, 64, 6, 5, 0, 0, 0)]
    sprite_sheet: Sprite,
    #[worldly]
    worldly: Worldly,
    body: RigidBody,
    collider: Collider,
    axes: LockedAxes,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            sprite_sheet: Sprite::default(),
            worldly: Worldly::default(),
            body: RigidBody::Dynamic,
            collider: Collider::rectangle(12., 28.),
            axes: LockedAxes::ROTATION_LOCKED,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_player.in_set(GameSet::Input));
        app.register_ldtk_entity::<PlayerBundle>("Player");
    }
}

fn get_change_for_input(keyboard_input: Res<ButtonInput<KeyCode>>) -> Vec2 {
    let mut change = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        change.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        change.x -= 1.0;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        change.y += 10.0;
    }

    change.normalize_or_zero() * PLAYER_SPEED
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vel: Single<&mut LinearVelocity, With<Player>>,
) {
    let change = get_change_for_input(keyboard_input);
    vel.0.x = change.x;
    vel.0.y += change.y;
}
