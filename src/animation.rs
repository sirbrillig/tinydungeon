use crate::{
    GameSet,
    movement::{FacingDirection, MovementState},
};
use avian2d::dynamics::rigid_body::LinearVelocity;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Clone)]
pub struct AnimationSet {
    pub animation_map: HashMap<AnimationKey, CharacterAnimationClip>,
}

impl AnimationSet {
    pub fn clip_for_key(&self, key: &AnimationKey) -> Option<&CharacterAnimationClip> {
        self.animation_map.get(key)
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug, Default, Hash)]
pub enum AnimationKey {
    #[default]
    Idle,
    Walking,
    Jumping,
    Attacking,
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
            (
                determine_animation_key,
                determine_facing,
                update_facing,
                update_sprites,
                animate_sprites,
            )
                .chain()
                .in_set(GameSet::Animate),
        );
    }
}

fn determine_animation_key(mut query: Query<(&MovementState, &mut AnimationKey)>) {
    for (state, mut key) in query.iter_mut() {
        let next_key = match state {
            MovementState::Jumping => AnimationKey::Jumping,
            MovementState::Walking => AnimationKey::Walking,
            MovementState::Idle => AnimationKey::Idle,
        };
        if *key != next_key {
            *key = next_key;
        }
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
            &AnimationKey,
            &mut Sprite,
            &mut SpriteAnimation,
            &AnimationSet,
        ),
        Changed<AnimationKey>,
    >,
) {
    for (key, mut sprite, mut animation, animation_set) in &mut query {
        let Some(clip) = animation_set.clip_for_key(key) else {
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

fn determine_facing(mut query: Query<(&mut FacingDirection, &LinearVelocity), With<Sprite>>) {
    for (mut facing, vel) in query.iter_mut() {
        let is_walking = vel.x.abs() > 0.1;
        if !is_walking {
            return;
        }
        let next_facing = if vel.x > 0.0 {
            FacingDirection::Right
        } else {
            FacingDirection::Left
        };
        if *facing != next_facing {
            *facing = next_facing;
        }
    }
}

fn update_facing(mut query: Query<(&FacingDirection, &mut Sprite), Changed<FacingDirection>>) {
    for (facing, mut sprite) in &mut query {
        sprite.flip_x = *facing == FacingDirection::Left;
    }
}
