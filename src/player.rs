use crate::animation::{AnimationKey, AnimationSet, CharacterAnimationClip};
use crate::attack::HurtBox;
use crate::movement::*;
use crate::{GameSet, animation::SpriteAnimation};
use avian2d::collision::collider::CollisionLayers;
use avian2d::collision::collision_events::CollisionStart;
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
pub const PLAYER_HEIGHT: f32 = 20.0;
const PLAYER_HEIGHT_ANCHOR_OFFSET: f32 = 0.03;
const PLAYER_FOOT_HEIGHT: f32 = 2.0;
const PLAYER_FOOT_ANCHOR: f32 = -(PLAYER_HEIGHT / 2.) + (PLAYER_FOOT_HEIGHT / 2.);
const PLAYER_FOOT_RANGE: f32 = 2.0;

#[derive(Resource)]
pub struct PlayerAnimations(AnimationSet);

#[derive(Component, Default)]
pub struct Player;

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    state: MovementState,
    animation_key: AnimationKey,
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
            animation_key: AnimationKey::Idle,
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
            (handle_got_hit, handle_hurt, handle_invincible)
                .chain()
                .in_set(GameSet::Reactions),
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

    // Add hurt box in a child (which we cannot do during init because ldtk plugin does not support it)
    commands.entity(event.entity).with_children(|parent| {
        parent
            .spawn((
                HurtBox,
                CollisionLayers::new(GameLayers::PlayerHurtBox, [GameLayers::EnemyHitBox]),
                Collider::rectangle(16., PLAYER_HEIGHT),
            ))
            .observe(on_hit);
    });
}

#[derive(Component)]
pub struct GotHit {
    hit_by: Entity,
}

#[derive(Component)]
pub struct HurtState {
    pub timer: Timer,
}

#[derive(Component)]
pub struct Invincible {
    pub timer: Timer,
}

fn on_hit(event: On<CollisionStart>, mut commands: Commands) {
    let Some(player) = event.body1 else {
        return;
    };
    let Some(enemy) = event.body2 else {
        return;
    };
    commands.entity(player).insert(GotHit { hit_by: enemy });
}

fn handle_got_hit(
    query: Query<(Entity, &GotHit, Has<Invincible>), Added<GotHit>>,
    mut commands: Commands,
) {
    for (player, hit, invincible) in query.iter() {
        commands.entity(player).remove::<GotHit>();
        if invincible {
            continue;
        }
        // Add HurtState to visually show the player get hurt
        commands.entity(player).insert(HurtState {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        });
        // Add Knockback to knock the player back
        commands.entity(player).insert(Knockback {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            collided_with: hit.hit_by,
        });
        // Make player invincible briefly
        commands.entity(player).insert(Invincible {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
        });
    }
}

fn handle_hurt(
    mut query: Query<(&mut Sprite, &mut HurtState, Entity)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut _sprite, mut hurt, entity) in query.iter_mut() {
        if hurt.timer.is_finished() {
            commands.entity(entity).remove::<HurtState>();
            continue;
        }
        hurt.timer.tick(time.delta());
        // @todo play a hurt animation (time it to the hurt timer using AnimationProgress)
    }
}

fn handle_invincible(
    mut query: Query<(&mut Sprite, &mut Invincible, Entity)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut sprite, mut invincible, entity) in query.iter_mut() {
        if invincible.timer.is_finished() {
            sprite.color.set_alpha(1.0);
            commands.entity(entity).remove::<Invincible>();
            continue;
        }
        invincible.timer.tick(time.delta());
        let elapsed = invincible.timer.elapsed().as_secs();
        let flicker_hz = 15;
        let alpha = if (elapsed * flicker_hz) % 2 == 0 {
            0.3
        } else {
            1.0
        };
        sprite.color.set_alpha(alpha);
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
    player: Single<
        (&mut LinearVelocity, &MovementSpeed, &mut CoyoteTimer),
        (With<Player>, Without<CannotMove>),
    >,
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
            (AnimationKey::Idle, idle),
            (AnimationKey::Walking, walk),
            (AnimationKey::Jumping, jump),
        ]),
    }));
}
