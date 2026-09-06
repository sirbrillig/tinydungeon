use std::time::Duration;

use avian2d::collision::collider::{Collider, CollisionLayers};
use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::{ai::AiSet, animation::AnimationProgress, attack::{Attacking, HitBox}, enemies::Enemy, movement::{FacingDirection, GameLayers}};

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

fn action(query: Query<(&Attack, &BehaveCtx), Added<Attack>>, facings: Query<&FacingDirection, With<Enemy>>, mut commands: Commands) {
    for (attack, ctx) in query.iter() {
        let Ok(facing) = facings.get(ctx.target_entity()) else {
            continue;
        };
        let offset_x = match facing {
            FacingDirection::Left => -12.0,
            FacingDirection::Right => 12.0,
        };

        // @todo only spawn (or make active) hitbox in attack frames of animation
        let hitbox = commands.spawn((
                HitBox,
                CollisionLayers::new(GameLayers::EnemyHitBox, [GameLayers::PlayerHurtBox]),
                // @todo let attack collider size and position be configurable by attack
                Collider::rectangle(16., 14.0),
                Transform::from_xyz(offset_x, 0.0, 0.0),
                ChildOf(ctx.target_entity()),
        )).id();

        commands.entity(ctx.target_entity()).insert(Attacking {
            timer: Timer::new(
                Duration::from_secs_f32(attack.duration_secs),
                TimerMode::Once,
            ),
            hitbox,
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
    time: Res<Time>,
) {
    for ctx in query.iter() {
        let Ok((mut attacking, mut progress)) = attackers.get_mut(ctx.target_entity()) else {
            continue;
        };
        attacking.timer.tick(time.delta());
        progress.0 = attacking.timer.fraction();
        if attacking.timer.is_finished() {
            commands.entity(ctx.target_entity()).remove::<Attacking>();
            commands
                .entity(ctx.target_entity())
                .remove::<AnimationProgress>();
            commands.entity(attacking.hitbox).despawn();
            commands.trigger(ctx.success());
        }
    }
}
