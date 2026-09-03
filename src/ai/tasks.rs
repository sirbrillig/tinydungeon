use bevy::app::App;

pub mod move_toward_entity;
pub mod wait_until_player_is_near;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        wait_until_player_is_near::plugin,
        move_toward_entity::plugin,
    ));
}
