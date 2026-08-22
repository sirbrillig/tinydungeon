use crate::GameSet;
use bevy::prelude::*;

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_things.in_set(GameSet::Movement));
    }
}

fn move_things(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut thing, velocity) in query.iter_mut() {
        thing.translation += (velocity.0 * time.delta_secs()).extend(0.0);
    }
}
