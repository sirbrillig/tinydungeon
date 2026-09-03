use avian2d::{dynamics::rigid_body::LinearVelocity, physics_transform::Position};
use bevy::prelude::*;
use bevy_behave::prelude::*;

use crate::movement::{FacingDirection, MovementSpeed};

#[derive(Component, Clone)]
pub struct MoveTowardEntity;

#[derive(Component)]
pub struct ChaseTarget(pub Entity);

pub fn plugin(app: &mut App) {
    app.add_systems(Update, move_toward_entity);
}

fn move_toward_entity(
    query: Query<&BehaveCtx, With<MoveTowardEntity>>,
    mut commands: Commands,
    entities: Query<&Position>,
    mut mover_props: Query<(&mut LinearVelocity, &ChaseTarget, &MovementSpeed, &Position)>,
) {
    for ctx in query.iter() {
        let Ok((mut vel, target, speed, mover_pos)) = mover_props.get_mut(ctx.target_entity())
        else {
            continue;
        };
        let Ok(target_pos) = entities.get(target.0) else {
            continue;
        };
        let direction = if target_pos.x > mover_pos.x {
            FacingDirection::Right
        } else {
            FacingDirection::Left
        };
        vel.x = match direction {
            FacingDirection::Left => -speed.0,
            FacingDirection::Right => speed.0,
        };
        commands.trigger(ctx.success());
    }
}
