use std::collections::HashMap;
use crate::animation::{AnimationSet, CharacterAnimationClip};
use crate::movement::*;
use crate::{animation::SpriteAnimation};
use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{Friction, LinearVelocity, LockedAxes, RigidBody},
    spatial_query::ShapeCaster,
};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_ldtk::{LdtkEntity, Worldly, app::LdtkEntityAppExt};

const ENEMY_SPEED: f32 = 90.0;
const ENEMY_JUMP_SPEED: f32 = 255.0;
const ENEMY_JUMP_CUT_SPEED: f32 = 190.0;
const ENEMY_HEIGHT: f32 = 20.0;
const ENEMY_HEIGHT_ANCHOR_OFFSET: f32 = 0.03;
const ENEMY_FOOT_HEIGHT: f32 = 2.0;
const ENEMY_FOOT_ANCHOR: f32 = -(ENEMY_HEIGHT / 2.) + (ENEMY_FOOT_HEIGHT / 2.);
const ENEMY_FOOT_RANGE: f32 = 2.0;

#[derive(Resource)]
struct EnemyAnimations(AnimationSet);

#[derive(Component, Default)]
pub struct Enemy;

#[derive(Bundle, LdtkEntity)]
struct EnemyBundle {
    enemy: Enemy,
    state: MovementState,
    #[sprite_sheet("Orc-Idle.png", 100, 100, 6, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
    #[worldly]
    worldly: Worldly,
    body: RigidBody,
    friction: Friction,
    collider: Collider,
    ground_detection: GroundDetection,
    ground_detector: ShapeCaster,
    axes: LockedAxes,
    anchor: Anchor,
    animation: SpriteAnimation,
    facing: FacingDirection,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy,
            state: MovementState::Idle,
            sprite_sheet: Sprite::default(),
            worldly: Worldly::default(),
            body: RigidBody::Dynamic,
            friction: Friction::ZERO
                .with_combine_rule(avian2d::dynamics::rigid_body::CoefficientCombine::Min),
            collider: Collider::rectangle(16., ENEMY_HEIGHT),
            ground_detection: GroundDetection,
            ground_detector: ShapeCaster::new(
                Collider::rectangle(14., ENEMY_FOOT_HEIGHT),
                // Put detector at the feet
                Vec2 {
                    x: 0.0,
                    y: ENEMY_FOOT_ANCHOR,
                },
                0.0,
                Dir2::NEG_Y,
            )
            .with_max_distance(ENEMY_FOOT_RANGE),
            axes: LockedAxes::ROTATION_LOCKED,
            // Anchor is down a bit because sprite is not vertically centered
            anchor: Anchor(Vec2::new(0.0, ENEMY_HEIGHT_ANCHOR_OFFSET)),
            animation: SpriteAnimation {
                frames: 6,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            facing: FacingDirection::Right,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_enemy);
        app.register_ldtk_entity::<EnemyBundle>("Enemy");
        app.add_observer(on_spawned);
    }
}

fn on_spawned(
    event: On<Add, Enemy>,
    mut commands: Commands,
    animations: Res<EnemyAnimations>,
) {
    // Add animation map (must do in a System so we can access World things like commands)
    commands.entity(event.entity).insert(animations.0.clone());
}

fn determine_movement_state(
    enemy: Single<(&mut MovementState, &LinearVelocity, Has<OnGround>), With<Enemy>>,
) {
    let (mut state, vel, on_ground) = enemy.into_inner();
    let is_walking = vel.x.abs() > 0.1;
    let next_state = match (on_ground, is_walking) {
        (false, _) => MovementState::Jumping,
        (true, true) => MovementState::Walking,
        (true, false) => MovementState::Idle,
    };
    if *state != next_state {
        *state = next_state;
    }
}

fn determine_facing(enemy: Single<(&mut FacingDirection, &LinearVelocity), With<Enemy>>) {
    let (mut facing, vel) = enemy.into_inner();
    let is_walking = vel.x.abs() > 0.1;
    if !is_walking {
        return;
    }
    let next_facing = if vel.x > 0.0 {
        FacingDirection::Right
    } else {
        FacingDirection::Left
    };
    if *facing != next_facing {
        *facing = next_facing;
    }
}

fn update_facing(mut query: Query<(&FacingDirection, &mut Sprite), Changed<FacingDirection>>) {
    for (facing, mut sprite) in &mut query {
        sprite.flip_x = *facing == FacingDirection::Left;
    }
}

fn get_change_for_input(keyboard_input: &ButtonInput<KeyCode>) -> f32 {
    let change = if keyboard_input.pressed(KeyCode::ArrowRight) {
        1.0
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
        -1.0
    } else {
        0.0
    };
    change * ENEMY_SPEED
}

fn move_enemy(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    enemy: Single<(&mut LinearVelocity, Has<OnGround>), With<Enemy>>,
) {
    // @todo this should not respond to keyboard
    let (mut vel, on_ground) = enemy.into_inner();
    vel.x = get_change_for_input(&keyboard_input);
    if keyboard_input.just_released(KeyCode::ArrowUp) && vel.0.y > 0.0 {
        vel.0.y = vel.0.y.min(ENEMY_JUMP_CUT_SPEED);
    }
    if on_ground && keyboard_input.just_pressed(KeyCode::ArrowUp) {
        vel.y = ENEMY_JUMP_SPEED;
    }
}

fn setup_enemy(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let idle = CharacterAnimationClip {
        image: asset_server.load("Orc-Idle.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            6,
            1,
            None,
            None,
        )),
        frames: 6,
    };
    let walk = CharacterAnimationClip {
        image: asset_server.load("Orc-Walk.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            8,
            1,
            None,
            None,
        )),
        frames: 8,
    };
    let jump = CharacterAnimationClip {
        image: asset_server.load("Orc-Walk.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            8,
            1,
            None,
            None,
        )),
        frames: 1,
    };
    commands.insert_resource(EnemyAnimations(AnimationSet {
        animation_map: HashMap::from([
            (MovementState::Idle, idle),
            (MovementState::Walking, walk),
            (MovementState::Jumping, jump),
        ]),
    }));
}
