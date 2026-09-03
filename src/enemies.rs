use crate::ai::tasks::move_toward_entity::{ChaseTarget, MoveTowardEntity};
use crate::ai::tasks::wait_until_player_is_near::{DetectionDistance, WaitUntilPlayerIsNear};
use crate::animation::SpriteAnimation;
use crate::animation::{AnimationSet, CharacterAnimationClip};
use crate::movement::*;
use crate::player::Player;
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
        app.add_systems(Update, set_chase_target);
        app.register_ldtk_entity::<EnemyBundle>("Enemy");
        app.add_observer(on_spawned);
    }
}

fn set_chase_target(
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    query: Query<Entity, (With<Enemy>, Without<ChaseTarget>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(ChaseTarget(*player));
    }
}

fn on_spawned(event: On<Add, Enemy>, mut commands: Commands, animations: Res<EnemyAnimations>) {
    // Add animation map (must do in a System so we can access World things like commands)
    commands.entity(event.entity).insert(animations.0.clone());

    let tree = behave! {
        Behave::Forever => {
            Behave::Sequence => {
                // @todo stop if player is not near
                Behave::spawn_named("Wait until player is near", WaitUntilPlayerIsNear),
                Behave::spawn_named("Move toward player", MoveTowardEntity),
            }
        }
    };
    commands.spawn((
        Name::new("Behave tree"),
        BehaveTree::new(tree).with_logging(true),
        ChildOf(event.entity),
    ));
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
