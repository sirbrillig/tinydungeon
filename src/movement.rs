use crate::GameSet;
use avian2d::{dynamics::rigid_body::LinearVelocity, spatial_query::ShapeHits};
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (ground_detection, coyote_timer)
                .chain()
                .before(GameSet::Input),
        );
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

#[derive(Component)]
pub struct CoyoteTimer {
    timer: Timer,
}

impl Default for CoyoteTimer {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.15, TimerMode::Once);
        // Start completed so it has to be started explicitly.
        timer.tick(timer.remaining());
        CoyoteTimer { timer }
    }
}

impl CoyoteTimer {
    pub fn end(&mut self) {
        self.timer.tick(self.timer.remaining());
    }

    pub fn can_jump(&self) -> bool {
        !self.timer.is_finished()
    }
}

fn ground_detection(
    mut commands: Commands,
    query: Query<(Entity, &ShapeHits, Has<OnGround>), With<GroundDetection>>,
) {
    for (entity, hits, was_on_ground) in query {
        let is_on_ground = !hits.is_empty();
        if is_on_ground == was_on_ground {
            continue;
        }
        if is_on_ground {
            commands.entity(entity).insert(OnGround);
        } else {
            commands.entity(entity).remove::<OnGround>();
        }
    }
}

fn coyote_timer(
    time: Res<Time>,
    query: Query<(Has<OnGround>, &LinearVelocity, &mut CoyoteTimer), With<GroundDetection>>,
) {
    for (is_on_ground, vel, mut coyote) in query {
        // When on the ground and not moving upward (to catch frames when OnGround has not yet been
        // removed), start the timer.
        if is_on_ground && vel.y <= 0.1 {
            coyote.timer.reset();
            continue;
        }
        coyote.timer.tick(time.delta());
    }
}
