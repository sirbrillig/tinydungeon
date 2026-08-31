use crate::GameSet;
use avian2d::spatial_query::ShapeHits;
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ground_detection.before(GameSet::Input));
    }
}

#[derive(Component)]
pub struct GroundDetection;

#[derive(Component)]
pub struct OnGround;

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug, Default, Hash)]
pub enum MovementState {
    #[default]
    Idle,
    Walking,
    Jumping,
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FacingDirection {
    #[default]
    Right,
    Left,
}

fn ground_detection(
    mut commands: Commands,
    query: Query<(Entity, &ShapeHits, Has<OnGround>), With<GroundDetection>>,
) {
    for (player_entity, hits, was_on_ground) in query {
        let is_on_ground = !hits.is_empty();
        if is_on_ground == was_on_ground {
            return;
        }
        if is_on_ground {
            commands.entity(player_entity).insert(OnGround);
        } else {
            commands.entity(player_entity).remove::<OnGround>();
        }
    }
}
