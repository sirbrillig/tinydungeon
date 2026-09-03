use avian2d::{dynamics::rigid_body::LinearVelocity, physics_transform::Position};
use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::movement::{FacingDirection, MovementSpeed};

#[derive(Component, Clone)]
pub struct MoveTowardEntity {
    pub target: Entity,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, move_toward_entity);
}

fn move_toward_entity(
    query: Query<(&MoveTowardEntity, &BehaveCtx)>,
    mut commands: Commands,
    entities: Query<&Position>,
    mut mover_props: Query<(&mut LinearVelocity, &MovementSpeed)>,
) {
    for (task, ctx) in query.iter() {
        let Ok(target_pos) = entities.get(task.target) else {
            continue;
        };
        let Ok(mover_pos) = entities.get(ctx.target_entity()) else {
            continue;
        };
        let direction = if target_pos.x > mover_pos.x {
            FacingDirection::Right
        } else {
            FacingDirection::Left
        };
        let Ok((mut vel, speed)) = mover_props.get_mut(ctx.target_entity()) else {
            continue;
        };
        vel.x = match direction {
            FacingDirection::Left => -speed.0,
            FacingDirection::Right => speed.0,
        };
        commands.trigger(ctx.success());
    }
}
