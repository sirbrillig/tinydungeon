use crate::ai::tasks::move_toward_entity::MoveTowardEntity;
use crate::ai::tasks::wait_until_player_is_near::{DetectionDistance, WaitUntilPlayerIsNear};
use crate::animation::{AnimationSet, CharacterAnimationClip, SpriteAnimation};
use crate::enemies::EnemyCoreBundle;
use crate::movement::*;
use avian2d::collision::collider::Collider;
use avian2d::spatial_query::ShapeCaster;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_behave::behave;
use bevy_behave::prelude::*;
use bevy_ecs_ldtk::Worldly;
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use std::collections::HashMap;

const ENEMY_HEIGHT: f32 = 16.0;
const ENEMY_HEIGHT_ANCHOR_OFFSET: f32 = 0.01;
const ENEMY_FOOT_HEIGHT: f32 = 2.0;
const ENEMY_FOOT_ANCHOR: f32 = -(ENEMY_HEIGHT / 2.) + (ENEMY_FOOT_HEIGHT / 2.);
const ENEMY_FOOT_RANGE: f32 = 2.0;

#[derive(Component, Default)]
pub struct Orc;

#[derive(Bundle, LdtkEntity)]
struct OrcBundle {
    orc: Orc,
    #[worldly]
    worldly: Worldly,
    #[sprite_sheet("Orc-Idle.png", 100, 100, 6, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
    core: EnemyCoreBundle,
    detection_distance: DetectionDistance,
}

impl Default for OrcBundle {
    fn default() -> Self {
        Self {
            orc: Orc,
            worldly: Worldly::default(),
            sprite_sheet: Sprite::default(),
            detection_distance: DetectionDistance(3500.0),
            core: EnemyCoreBundle {
                speed: MovementSpeed(22.0),
                collider: Collider::rectangle(16., ENEMY_HEIGHT),
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
                // Anchor is down a bit because sprite is not vertically centered
                anchor: Anchor(Vec2::new(0.0, ENEMY_HEIGHT_ANCHOR_OFFSET)),
                animation: SpriteAnimation {
                    frames: 6,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                ..Default::default()
            },
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_enemy);
    app.register_ldtk_entity::<OrcBundle>("Orc");
    app.add_observer(on_spawned);
}

#[derive(Resource)]
struct OrcAnimations(AnimationSet);

fn on_spawned(event: On<Add, Orc>, mut commands: Commands, animations: Res<OrcAnimations>) {
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
    commands.insert_resource(OrcAnimations(AnimationSet {
        animation_map: HashMap::from([
            (MovementState::Idle, idle),
            (MovementState::Walking, walk),
            (MovementState::Jumping, jump),
        ]),
    }));
}
