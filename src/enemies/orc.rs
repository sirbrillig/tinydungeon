use crate::ai::tasks::move_toward_entity::MoveTowardEntity;
use crate::ai::tasks::target_in_range::TargetInRange;
use crate::ai::tasks::wait_until_player_is_near::{DetectionDistance, WaitUntilPlayerIsNear};
use crate::animation::{AnimationSet, CharacterAnimationClip};
use crate::enemies::{EnemyCoreBundle, EnemySettings};
use crate::movement::*;
use bevy::prelude::*;
use bevy_behave::behave;
use bevy_behave::prelude::*;
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
    #[sprite_sheet("Orc-Idle.png", 100, 100, 6, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
    core: EnemyCoreBundle,
    detection_distance: DetectionDistance,
}

impl Default for OrcBundle {
    fn default() -> Self {
        Self {
            orc: Orc,
            sprite_sheet: Sprite::default(),
            detection_distance: DetectionDistance(3500.0),
            core: EnemyCoreBundle::with_settings(EnemySettings {
                sprite_height: ENEMY_HEIGHT,
                sprite_height_offset: ENEMY_HEIGHT_ANCHOR_OFFSET,
                speed: 25.0,
                ground_detector_height: ENEMY_FOOT_HEIGHT,
                ground_detector_anchor: ENEMY_FOOT_ANCHOR,
                ground_detector_range: ENEMY_FOOT_RANGE,
                animation_default_frames: 6,
            }),
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
            Behave::Fallback => {
                Behave::Sequence => {
                   Behave::spawn_named("Is player in attack range", TargetInRange {range: 600.0 }),
                   Behave::Wait(0.8), // @todo make an attack
                },
                Behave::Sequence => {
                    Behave::spawn_named("Is player in chase range", TargetInRange {range: 3500.0}),
                    Behave::spawn_named("Move toward player", MoveTowardEntity {near_distance: 600.0, far_distance: 3500.0}),
                },
                Behave::spawn_named("Is player in at least chase range", WaitUntilPlayerIsNear),
            }
        }
    };
    commands.spawn((
        Name::new("Orc"),
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
