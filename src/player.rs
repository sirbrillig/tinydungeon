use crate::GameSet;
use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LinearVelocity, LockedAxes, RigidBody},
};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_ldtk::{LdtkEntity, Worldly, app::LdtkEntityAppExt};

const PLAYER_SPEED: f32 = 300.0;

#[derive(Component)]
struct SpriteAnimation {
    frames: usize,
    timer: Timer, // Timer::from_seconds(0.1, TimerMode::Repeating)
}

struct PlayerAnimationClip {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frames: usize,
}

#[derive(Resource)]
struct PlayerAnimations {
    idle: PlayerAnimationClip,
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    #[sprite_sheet("Priest-Idle.png", 100, 100, 6, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
    #[worldly]
    worldly: Worldly,
    body: RigidBody,
    collider: Collider,
    axes: LockedAxes,
    anchor: Anchor,
    animation: SpriteAnimation,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            sprite_sheet: Sprite::default(),
            worldly: Worldly::default(),
            body: RigidBody::Dynamic,
            collider: Collider::rectangle(16., 20.),
            axes: LockedAxes::ROTATION_LOCKED,
            anchor: Anchor(Vec2::new(0.0, 0.03)),
            animation: SpriteAnimation {
                frames: 6,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
        app.add_systems(Update, move_player.in_set(GameSet::Input));
        app.add_systems(Update, animate_player);
        app.register_ldtk_entity::<PlayerBundle>("Player");
    }
}

fn get_change_for_input(keyboard_input: Res<ButtonInput<KeyCode>>) -> Vec2 {
    let mut change = Vec2::ZERO;

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        change.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        change.x -= 1.0;
    }
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        change.y += 10.0;
    }

    change.normalize_or_zero() * PLAYER_SPEED
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut vel: Single<&mut LinearVelocity, With<Player>>,
) {
    let change = get_change_for_input(keyboard_input);
    vel.0.x = change.x;
    vel.0.y += change.y;
}

fn setup_player(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let clip = PlayerAnimationClip {
        image: asset_server.load("Priest-Idle.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            6,
            1,
            None,
            None,
        )),
        frames: 6,
    };
    commands.insert_resource(PlayerAnimations { idle: clip });
}

fn animate_player(time: Res<Time>, mut query: Query<(&mut SpriteAnimation, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        // We track how long the current sprite has been displayed for
        config.timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = (atlas.index + 1) % config.frames;
        }
    }
}
