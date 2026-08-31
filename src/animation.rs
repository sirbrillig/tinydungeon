use crate::GameSet;
use bevy::prelude::*;

#[derive(Component)]
pub struct SpriteAnimation {
    pub frames: usize,
    pub timer: Timer,
}

pub struct CharacterAnimationClip {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub frames: usize,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (animate_sprites,).in_set(GameSet::PostInput));
    }
}

fn animate_sprites(time: Res<Time>, mut query: Query<(&mut SpriteAnimation, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        // We track how long the current sprite has been displayed for
        config.timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = (atlas.index + 1) % config.frames.max(1);
        }
    }
}
