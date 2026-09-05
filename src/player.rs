use crate::animation::{AnimationSet, CharacterAnimationClip};
use crate::movement::*;
use crate::{GameSet, animation::SpriteAnimation};
use avian2d::collision::collider::CollisionLayers;
use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{Friction, LinearVelocity, LockedAxes, RigidBody},
    spatial_query::ShapeCaster,
};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_ldtk::{LdtkEntity, Worldly, app::LdtkEntityAppExt};
use std::collections::HashMap;

const PLAYER_JUMP_SPEED: f32 = 255.0;
const PLAYER_JUMP_CUT_SPEED: f32 = 190.0;
const PLAYER_HEIGHT: f32 = 20.0;
const PLAYER_HEIGHT_ANCHOR_OFFSET: f32 = 0.03;
const PLAYER_FOOT_HEIGHT: f32 = 2.0;
const PLAYER_FOOT_ANCHOR: f32 = -(PLAYER_HEIGHT / 2.) + (PLAYER_FOOT_HEIGHT / 2.);
const PLAYER_FOOT_RANGE: f32 = 2.0;

#[derive(Resource)]
struct PlayerAnimations(AnimationSet);

#[derive(Component, Default)]
pub struct Player;

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    state: MovementState,
    #[sprite_sheet("Priest-Idle.png", 100, 100, 6, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
    #[worldly]
    worldly: Worldly,
    body: RigidBody,
    friction: Friction,
    layers: CollisionLayers,
    collider: Collider,
    speed: MovementSpeed,
    ground_detection: GroundDetection,
    coyote_time: CoyoteTimer,
    ground_detector: ShapeCaster,
    axes: LockedAxes,
    anchor: Anchor,
    animation: SpriteAnimation,
    facing: FacingDirection,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            state: MovementState::Idle,
            sprite_sheet: Sprite::default(),
            worldly: Worldly::default(),
            body: RigidBody::Dynamic,
            friction: Friction::ZERO
                .with_combine_rule(avian2d::dynamics::rigid_body::CoefficientCombine::Min),
            layers: CollisionLayers::new(GameLayers::Player, [GameLayers::Environment]),
            collider: Collider::rectangle(16., PLAYER_HEIGHT),
            speed: MovementSpeed(90.0),
            ground_detection: GroundDetection,
            coyote_time: CoyoteTimer::default(),
            ground_detector: ShapeCaster::new(
                Collider::rectangle(14., PLAYER_FOOT_HEIGHT),
                // Put detector at the player's feet
                Vec2 {
                    x: 0.0,
                    y: PLAYER_FOOT_ANCHOR,
                },
                0.0,
                Dir2::NEG_Y,
            )
            .with_max_distance(PLAYER_FOOT_RANGE),
            axes: LockedAxes::ROTATION_LOCKED,
            // Anchor is down a bit because sprite is not vertically centered
            anchor: Anchor(Vec2::new(0.0, PLAYER_HEIGHT_ANCHOR_OFFSET)),
            animation: SpriteAnimation {
                frames: 6,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            facing: FacingDirection::Right,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
        app.add_systems(Update, move_player.in_set(GameSet::Input));
        app.add_systems(
            Update,
            (determine_movement_state).chain().after(GameSet::Input),
        );
        app.register_ldtk_entity::<PlayerBundle>("Player");
        app.add_observer(on_player_spawned);
    }
}

fn on_player_spawned(
    event: On<Add, Player>,
    mut commands: Commands,
    animations: Res<PlayerAnimations>,
) {
    // Add player animation map (must do in a System so we can access World things like commands)
    commands.entity(event.entity).insert(animations.0.clone());
}

fn determine_movement_state(
    player: Single<(&mut MovementState, &LinearVelocity, Has<OnGround>), With<Player>>,
) {
    let (mut state, vel, on_ground) = player.into_inner();
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

fn get_change_for_input(keyboard_input: &ButtonInput<KeyCode>) -> f32 {
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        1.0
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) {
        -1.0
    } else {
        0.0
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut LinearVelocity, &MovementSpeed, &mut CoyoteTimer), With<Player>>,
) {
    let (mut vel, speed, mut coyote) = player.into_inner();
    vel.x = get_change_for_input(&keyboard_input) * speed.0;
    if keyboard_input.just_released(KeyCode::ArrowUp) && vel.0.y > 0.0 {
        vel.0.y = vel.0.y.min(PLAYER_JUMP_CUT_SPEED);
    }
    if coyote.can_jump() && keyboard_input.just_pressed(KeyCode::ArrowUp) {
        vel.y = PLAYER_JUMP_SPEED;
        // End the timer when actually jumping.
        coyote.end();
    }
}

fn setup_player(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let idle = CharacterAnimationClip {
        image: asset_server.load("Priest-Idle.png"),
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
        image: asset_server.load("Priest-Walk.png"),
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
        image: asset_server.load("Priest-Walk.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            8,
            1,
            None,
            None,
        )),
        frames: 1,
    };
    commands.insert_resource(PlayerAnimations(AnimationSet {
        animation_map: HashMap::from([
            (MovementState::Idle, idle),
            (MovementState::Walking, walk),
            (MovementState::Jumping, jump),
        ]),
    }));
}
