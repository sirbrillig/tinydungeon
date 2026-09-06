use avian2d::physics_transform::Position;
use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::{
    ai::{
        AiSet,
        tasks::{TaskReported, move_toward_entity::ChaseTarget},
    },
    movement::FacingDirection,
};

#[derive(Component, Clone)]
pub struct FaceTarget;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, action.in_set(AiSet::Behavior));
}

fn action(
    query: Query<(Entity, &BehaveCtx), (With<FaceTarget>, Without<TaskReported>)>,
    mut commands: Commands,
    entities: Query<&Position>,
    mut mover_props: Query<(&ChaseTarget, &Position, &mut FacingDirection)>,
) {
    for (task, ctx) in query.iter() {
        let Ok((target, mover_pos, mut facing)) = mover_props.get_mut(ctx.target_entity()) else {
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
        if *facing != player_side {
            *facing = player_side;
        }
        commands.trigger(ctx.success());
        commands.entity(task).insert(TaskReported);
    }
}
