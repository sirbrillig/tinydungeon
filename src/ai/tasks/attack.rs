use std::time::Duration;

use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::{ai::AiSet, animation::AnimationProgress, attack::Attacking};

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
            timer: Timer::new(
                Duration::from_secs_f32(attack.duration_secs),
                TimerMode::Once,
            ),
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
            commands.trigger(ctx.success());
        }
    }
}
