use bevy::prelude::*;

#[derive(Component)]
pub struct Rock;

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_rock);
    }
}

fn setup_rock(mut commands: Commands) {
    let color = Color::hsl(5.0, 0.77, 0.42);
    let transform = Transform::from_xyz(200.0, 0.0, 0.0);
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(200.0, 100.0)),
            ..default()
        },
        transform,
        Rock,
    ));
}
