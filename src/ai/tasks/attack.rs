use avian2d::collision::collider::{Collider, CollisionLayers};
use bevy::prelude::*;
use bevy_behave::prelude::*;
use std::range::Range;

use crate::{
    ai::AiSet,
    animation::{AnimationProgress, SpriteAnimation},
    attack::{Attacking, HitBox},
    enemies::Enemy,
    movement::{FacingDirection, GameLayers},
};

#[derive(Component, Clone)]
pub struct HitBoxConfig {
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Component, Clone)]
pub struct Attack {
    pub duration_secs: f32,
    pub active_frames: Range<usize>,
    pub hitbox: HitBoxConfig,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (action, attack_timer).chain().in_set(AiSet::Behavior),
    );
}

fn action(query: Query<(&Attack, &BehaveCtx), Added<Attack>>, mut commands: Commands) {
    for (attack, ctx) in query.iter() {
        commands.entity(ctx.target_entity()).insert(Attacking {
            timer: Timer::from_seconds(attack.duration_secs, TimerMode::Once),
            hitbox: None,
        });

        commands
            .entity(ctx.target_entity())
            .insert(AnimationProgress(0.0));
    }
}

fn attack_timer(
    query: Query<(&Attack, &BehaveCtx)>,
    mut commands: Commands,
    mut attackers: Query<(&mut Attacking, &mut AnimationProgress, &SpriteAnimation)>,
    facings: Query<&FacingDirection, With<Enemy>>,
    time: Res<Time>,
) {
    for (attack, ctx) in query.iter() {
        let Ok((mut attacking, mut progress, animation)) = attackers.get_mut(ctx.target_entity())
        else {
            continue;
        };
        attacking.timer.tick(time.delta());
        progress.0 = attacking.timer.fraction();

        let frame = (progress.0 * animation.frames as f32) as usize;
        if attack.active_frames.contains(&frame)
            && attacking.hitbox.is_none()
            && let Ok(facing) = facings.get(ctx.target_entity())
        {
            add_hitbox(
                ctx.target_entity(),
                attack,
                &mut attacking,
                facing,
                &mut commands,
            );
        }

        if attacking.timer.is_finished() {
            commands.entity(ctx.target_entity()).remove::<Attacking>();
            commands
                .entity(ctx.target_entity())
                .remove::<AnimationProgress>();
            remove_hitbox(&attacking, &mut commands);
            commands.trigger(ctx.success());
        }
    }
}

fn add_hitbox(
    enemy: Entity,
    attack: &Attack,
    attacking: &mut Attacking,
    facing: &FacingDirection,
    commands: &mut Commands,
) {
    let offset_x = match facing {
        FacingDirection::Left => -attack.hitbox.offset_x,
        FacingDirection::Right => attack.hitbox.offset_x,
    };
    let hitbox = commands
        .spawn((
            HitBox,
            CollisionLayers::new(GameLayers::EnemyHitBox, [GameLayers::PlayerHurtBox]),
            Collider::rectangle(attack.hitbox.width, attack.hitbox.height),
            Transform::from_xyz(offset_x, attack.hitbox.offset_y, 0.0),
            ChildOf(enemy),
        ))
        .id();
    attacking.hitbox = Some(hitbox);
}

fn remove_hitbox(attacking: &Attacking, commands: &mut Commands) {
    if let Some(hitbox) = attacking.hitbox {
        commands.entity(hitbox).despawn();
    }
}
