use bevy::app::App;

pub mod attack;
pub mod move_toward_entity;
pub mod stop_moving;
pub mod target_in_range;
pub mod wait_until_player_is_near;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        wait_until_player_is_near::plugin,
        move_toward_entity::plugin,
        stop_moving::plugin,
        target_in_range::plugin,
        attack::plugin,
    ));
}
