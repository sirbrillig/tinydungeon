use avian2d::physics_transform::Position;
use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::{
    ai::{AiSet, tasks::move_toward_entity::ChaseTarget},
    movement::FacingDirection,
};

#[derive(Component, Clone)]
pub struct IsFacingTarget;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, action.in_set(AiSet::Behavior));
}

fn action(
    query: Query<&BehaveCtx, With<IsFacingTarget>>,
    mut commands: Commands,
    entities: Query<&Position>,
    mover_props: Query<(&ChaseTarget, &Position, &FacingDirection)>,
) {
    for ctx in query.iter() {
        let Ok((target, mover_pos, facing)) = mover_props.get(ctx.target_entity()) else {
            continue;
        };
        let Ok(target_pos) = entities.get(target.0) else {
            continue;
        };

        let player_side = if target_pos.x < mover_pos.x {
            FacingDirection::Left
        } else {
            FacingDirection::Right
        };

        if player_side == *facing {
            commands.trigger(ctx.success());
        } else {
            commands.trigger(ctx.failure());
        }
    }
}
