use avian2d::collision::collider::{Collider, CollisionLayers};
use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::{
    ai::AiSet,
    animation::AnimationProgress,
    attack::{Attacking, HitBox},
    enemies::Enemy,
    movement::{FacingDirection, GameLayers},
};

#[derive(Component, Clone)]
pub struct Attack {
    pub duration_secs: f32,
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
    query: Query<&BehaveCtx, With<Attack>>,
    mut commands: Commands,
    mut attackers: Query<(&mut Attacking, &mut AnimationProgress)>,
    facings: Query<&FacingDirection, With<Enemy>>,
    time: Res<Time>,
) {
    for ctx in query.iter() {
        let Ok((mut attacking, mut progress)) = attackers.get_mut(ctx.target_entity()) else {
            continue;
        };
        attacking.timer.tick(time.delta());
        progress.0 = attacking.timer.fraction();

        if progress.0 > 0.5
            && attacking.hitbox.is_none()
            && let Ok(facing) = facings.get(ctx.target_entity())
        {
            add_hitbox(ctx.target_entity(), &mut attacking, facing, &mut commands);
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
    attacking: &mut Attacking,
    facing: &FacingDirection,
    commands: &mut Commands,
) {
    let offset_x = match facing {
        FacingDirection::Left => -18.0,
        FacingDirection::Right => 18.0,
    };
    let hitbox = commands
        .spawn((
            HitBox,
            CollisionLayers::new(GameLayers::EnemyHitBox, [GameLayers::PlayerHurtBox]),
            // @todo let attack collider size and position be configurable by attack
            Collider::rectangle(16.0, 22.0),
            Transform::from_xyz(offset_x, 0.0, 0.0),
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
