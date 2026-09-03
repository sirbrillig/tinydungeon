use bevy::prelude::*;

use crate::GameSet;

pub mod tasks;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum AiSet {
    Behavior,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(tasks::plugin);
    app.configure_sets(Update, AiSet::Behavior.in_set(GameSet::Input));
}
