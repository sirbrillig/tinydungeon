use crate::animation::SpriteAnimation;
use crate::animation::{AnimationSet, CharacterAnimationClip};
use crate::movement::*;
use crate::player::Player;
use avian2d::dynamics::rigid_body::LinearVelocity;
use avian2d::physics_transform::Position;
use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{Friction, LockedAxes, RigidBody},
    spatial_query::ShapeCaster,
};
use bevy::{prelude::*, sprite::Anchor};
use bevy_behave::behave;
use bevy_behave::prelude::*;
use bevy_ecs_ldtk::{LdtkEntity, Worldly, app::LdtkEntityAppExt};
use std::collections::HashMap;

const ENEMY_HEIGHT: f32 = 16.0;
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
    speed: MovementSpeed,
    ground_detection: GroundDetection,
    ground_detector: ShapeCaster,
    axes: LockedAxes,
    anchor: Anchor,
    animation: SpriteAnimation,
    facing: FacingDirection,
    // @todo only add this when needed for a enemy
    detection_distance: DetectionDistance,
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
            speed: MovementSpeed(25.0),
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
            detection_distance: DetectionDistance(3500.0),
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_enemy);
        app.add_systems(Update, (wait_for_player, move_toward_entity).chain());
        app.register_ldtk_entity::<EnemyBundle>("Enemy");
        app.add_observer(on_spawned);
        app.add_observer(on_test_action);
    }
}

fn on_spawned(
    event: On<Add, Enemy>,
    mut commands: Commands,
    animations: Res<EnemyAnimations>,
    player: Single<Entity, With<Player>>,
) {
    // Add animation map (must do in a System so we can access World things like commands)
    commands.entity(event.entity).insert(animations.0.clone());

    let tree = behave! {
        Behave::Forever => {
            Behave::Sequence => {
                // @todo stop if player is not near
                Behave::spawn_named("Wait until player is near", WaitUntilPlayerIsNear { player: *player }),
                Behave::trigger(TestAction),
                Behave::spawn_named("Move toward player", MoveTowardEntity { target: *player }),
            }
        }
    };
    commands.spawn((
        Name::new("Behave tree"),
        BehaveTree::new(tree).with_logging(true),
        ChildOf(event.entity),
    ));
}

#[derive(Component, Clone)]
struct WaitUntilPlayerIsNear {
    player: Entity,
}

#[derive(Component, Clone, Copy)]
struct DetectionDistance(f32);

fn wait_for_player(
    query: Query<(&WaitUntilPlayerIsNear, &BehaveCtx)>,
    mut commands: Commands,
    entities: Query<&Position>,
    mover_props: Query<&DetectionDistance>,
) {
    for (task, ctx) in query.iter() {
        let Ok(player_pos) = entities.get(task.player) else {
            continue;
        };
        let Ok(enemy_pos) = entities.get(ctx.target_entity()) else {
            continue;
        };
        let Ok(near_distance) = mover_props.get(ctx.target_entity()) else {
            continue;
        };
        let distance_to_player = player_pos.distance_squared(enemy_pos.0);
        if distance_to_player <= near_distance.0 {
            commands.trigger(ctx.success());
        }
    }
}

#[derive(Component, Clone)]
struct MoveTowardEntity {
    target: Entity,
}

fn move_toward_entity(
    query: Query<(&MoveTowardEntity, &BehaveCtx)>,
    mut commands: Commands,
    entities: Query<&Position>,
    mut mover_props: Query<(&mut LinearVelocity, &MovementSpeed)>,
) {
    for (task, ctx) in query.iter() {
        let Ok(target_pos) = entities.get(task.target) else {
            continue;
        };
        let Ok(mover_pos) = entities.get(ctx.target_entity()) else {
            continue;
        };
        let direction = if target_pos.x > mover_pos.x {
            FacingDirection::Right
        } else {
            FacingDirection::Left
        };
        let Ok((mut vel, speed)) = mover_props.get_mut(ctx.target_entity()) else {
            continue;
        };
        vel.x = match direction {
            FacingDirection::Left => -speed.0,
            FacingDirection::Right => speed.0,
        };
        commands.trigger(ctx.success());
    }
}

#[derive(Component, Default, Clone)]
struct TestAction;

fn on_test_action(trigger: On<BehaveTrigger<TestAction>>, mut commands: Commands) {
    println!("testing!");
    commands.trigger(trigger.ctx().success());
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
