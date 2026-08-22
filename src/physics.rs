use crate::GameSet;
use bevy::math::bounding::IntersectsVolume;
use bevy::{math::bounding::Aabb2d, prelude::*};

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Collider {
    pub half_size: Vec2,
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_things.in_set(GameSet::Movement));
        app.add_systems(Update, handle_collisions.in_set(GameSet::Collision));
    }
}

fn move_things(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut thing, velocity) in query.iter_mut() {
        thing.translation += (velocity.0 * time.delta_secs()).extend(0.0);
    }
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}

impl Axis {
    fn index(self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
        }
    }
}

fn handle_collisions_dir(
    thing: &Transform,
    velocity: &mut Velocity,
    collider: &Collider,
    statics: &Query<(&Transform, &Collider), Without<Velocity>>,
    delta: f32,
    axis: Axis,
) {
    let index = axis.index();
    let next = thing.translation[index] + velocity.0[index] * delta;
    let mut pos = thing.translation.truncate();
    pos[index] = next;
    let bound = Aabb2d::new(pos, collider.half_size);
    for (target, target_collider) in statics.iter() {
        let target_bound = Aabb2d::new(target.translation.truncate(), target_collider.half_size);
        if bound.intersects(&target_bound) {
            velocity.0[index] = 0.0;
        }
    }
}

fn handle_collisions(
    mut movers: Query<(&Transform, &mut Velocity, &Collider)>,
    statics: Query<(&Transform, &Collider), Without<Velocity>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (thing, mut velocity, collider) in movers.iter_mut() {
        handle_collisions_dir(
            thing,
            &mut velocity,
            collider,
            &statics,
            delta,
            Axis::X,
        );
        handle_collisions_dir(
            thing,
            &mut velocity,
            collider,
            &statics,
            delta,
            Axis::Y,
        );
    }
}
