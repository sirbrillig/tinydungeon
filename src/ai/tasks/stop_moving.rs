use avian2d::dynamics::rigid_body::LinearVelocity;
use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::ai::AiSet;

#[derive(Component, Clone)]
pub struct StopMoving;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, stop_moving.in_set(AiSet::Behavior));
}

fn stop_moving(
    query: Query<&BehaveCtx, With<StopMoving>>,
    mut commands: Commands,
    mut mover_props: Query<&mut LinearVelocity>,
) {
    for ctx in query.iter() {
        let Ok(mut vel) = mover_props.get_mut(ctx.target_entity()) else {
            continue;
        };
        vel.x = 0.0;
        commands.trigger(ctx.success());
    }
}
