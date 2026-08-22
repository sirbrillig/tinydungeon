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

fn handle_collisions(
    mut movers: Query<(&Transform, &mut Velocity, &Collider)>,
    statics: Query<(&Transform, &Collider), Without<Velocity>>,
    time: Res<Time>,
) {
    for (thing, mut velocity, collider) in movers.iter_mut() {
        let next = thing.translation.truncate() + velocity.0 * time.delta_secs();
        let bound = Aabb2d::new(next, collider.half_size);
        for (target, target_collider) in statics {
            let target_bound =
                Aabb2d::new(target.translation.truncate(), target_collider.half_size);
            if bound.intersects(&target_bound) {
                velocity.0 = Vec2::ZERO;
            }
        }
    }
}
