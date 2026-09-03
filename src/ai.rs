use bevy::app::App;

pub mod tasks;

pub fn plugin(app: &mut App) {
    app.add_plugins(tasks::plugin);
}
