use bevy::{ecs::component::Component, time::Timer};

#[derive(Component)]
pub struct Attacking {
    pub timer: Timer,
}
