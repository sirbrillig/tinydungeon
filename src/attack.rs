use avian2d::collision::{collider::Sensor, collision_events::CollisionEventsEnabled};
use bevy::{ecs::component::Component, time::Timer};

#[derive(Component)]
pub struct Attacking {
    pub timer: Timer,
}

#[derive(Component)]
#[require(Sensor, CollisionEventsEnabled)]
pub struct HurtBox;

#[derive(Component)]
#[require(Sensor, CollisionEventsEnabled)]
pub struct HitBox;
