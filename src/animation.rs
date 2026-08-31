use crate::{GameSet, movement::MovementState};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Clone)]
pub struct AnimationSet {
    pub animation_map: HashMap<MovementState, CharacterAnimationClip>,
}

impl AnimationSet {
    pub fn clip_for_state(&self, state: &MovementState) -> Option<&CharacterAnimationClip> {
        self.animation_map.get(state)
    }
}

#[derive(Component)]
pub struct SpriteAnimation {
    pub frames: usize,
    pub timer: Timer,
}

#[derive(Clone)]
pub struct CharacterAnimationClip {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub frames: usize,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_sprites, animate_sprites)
                .chain()
                .in_set(GameSet::PostInput),
        );
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

fn update_sprites(
    mut query: Query<
        (
            &MovementState,
            &mut Sprite,
            &mut SpriteAnimation,
            &AnimationSet,
        ),
        Changed<MovementState>,
    >,
) {
    for (state, mut sprite, mut animation, animation_set) in &mut query {
        let Some(clip) = animation_set.clip_for_state(state) else {
            continue;
        };
        sprite.image = clip.image.clone();
        sprite.texture_atlas = Some(TextureAtlas {
            layout: clip.layout.clone(),
            index: 0,
        });
        animation.frames = clip.frames;
    }
}
