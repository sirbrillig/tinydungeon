use avian2d::collision::collider::{CollidingEntities, collider_hierarchy::ColliderOf};
use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    GameSet,
    animation::{AnimationKey, AnimationSet, CharacterAnimationClip, SpriteAnimation},
    attack::{HitBoxBundle, HurtBox},
    enemies::EnemyHurtBox,
    movement::{GameLayers, Knockback},
    player::Player,
};

pub fn powers_plugin(app: &mut App) {
    app.add_systems(Startup, setup_powers);
    app.add_systems(
        Update,
        (activate_bell, process_bell)
            .chain()
            .in_set(GameSet::Powers),
    );
    app.add_systems(
        Update,
        (detect_hit, handle_got_hit)
            .chain()
            .in_set(GameSet::Reactions),
    );
}

// @todo support activating different bells
#[derive(Component)]
pub struct ActivateBell;

#[derive(Component)]
pub struct BellActive {
    timer: Timer,
}

#[derive(Component)]
pub struct RepulsionBell;

#[derive(Bundle)]
struct BellBundle {
    animation_key: AnimationKey,
    sprite_sheet: Sprite,
    animation: SpriteAnimation,
    timer: BellActive,
}

#[derive(Resource)]
pub struct PowerAnimations(AnimationSet);

fn setup_powers(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let repulsion = CharacterAnimationClip {
        image: asset_server.load("explosions.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(64),
            12,
            1,
            None,
            Some(UVec2::new(0, 64 * 5)),
        )),
        frames: 10,
    };
    commands.insert_resource(PowerAnimations(AnimationSet {
        animation_map: HashMap::from([(AnimationKey::Repulsion, repulsion)]),
    }));
}

fn activate_bell(
    query: Query<Entity, Added<ActivateBell>>,
    animations: Res<PowerAnimations>,
    mut commands: Commands,
) {
    for player in query.iter() {
        commands.entity(player).remove::<ActivateBell>();
        let clip = animations.0.clone();
        let frames = 10;
        let power_time = 0.2;
        commands.spawn((
            RepulsionBell,
            ChildOf(player),
            clip,
            HitBoxBundle::new(
                GameLayers::PlayerPowerBox,
                GameLayers::EnemyHurtBox,
                40.,
                40.,
            ),
            BellBundle {
                // The timer for the power itself
                timer: BellActive {
                    timer: Timer::from_seconds(power_time, TimerMode::Once),
                },
                animation_key: AnimationKey::Repulsion,
                sprite_sheet: Sprite::default(),
                // The timer for the animation
                animation: SpriteAnimation {
                    frames,
                    timer: Timer::from_seconds(power_time / (frames as f32), TimerMode::Repeating),
                },
            },
        ));
    }
}

fn process_bell(
    mut query: Query<(Entity, &mut BellActive)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (bell, mut active) in query.iter_mut() {
        active.timer.tick(time.delta());
        if active.timer.is_finished() {
            commands.entity(bell).despawn();
        }
    }
}

#[derive(Component)]
pub struct HitByBell;

fn detect_hit(
    query: Query<(&CollidingEntities, &ColliderOf), (With<HurtBox>, With<EnemyHurtBox>)>,
    bells: Query<(), With<RepulsionBell>>,
    mut commands: Commands,
) {
    for (hurtbox, owner) in query.iter() {
        let enemy = owner.body;
        for hitbox in hurtbox.iter() {
            if bells.contains(*hitbox) {
                commands.entity(enemy).insert(HitByBell);
            }
        }
    }
}

fn handle_got_hit(
    query: Query<Entity, Added<HitByBell>>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    for enemy in query.iter() {
        commands.entity(enemy).remove::<HitByBell>();
        commands.entity(enemy).insert(Knockback {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
            collided_with: *player,
        });
    }
}
