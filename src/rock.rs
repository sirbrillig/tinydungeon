use bevy::prelude::*;

#[derive(Component)]
pub struct Rock;

pub struct RockPlugin;

impl Plugin for RockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_rock);
    }
}

fn setup_rock(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = meshes.add(Circle::new(50.0));
    let color = Color::hsl(5.0, 0.77, 0.42);
    commands.spawn((
        Mesh2d(shape),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(200.0, 0.0, 0.0),
        Rock,
    ));
}
