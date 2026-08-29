use crate::GameSet;
use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LinearVelocity, LockedAxes, RigidBody},
    spatial_query::{ShapeCaster, ShapeHits},
};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_ldtk::{LdtkEntity, Worldly, app::LdtkEntityAppExt};

const PLAYER_SPEED: f32 = 300.0;
const PLAYER_JUMP_SPEED: f32 = 270.0;
const PLAYER_JUMP_CUT_SPEED: f32 = 150.0;
const PLAYER_HEIGHT: f32 = 20.0;
const PLAYER_HEIGHT_ANCHOR_OFFSET: f32 = 0.03;
const PLAYER_FOOT_HEIGHT: f32 = 2.0;
const PLAYER_FOOT_ANCHOR: f32 = -(PLAYER_HEIGHT / 2.) + (PLAYER_FOOT_HEIGHT / 2.);
const PLAYER_FOOT_RANGE: f32 = 2.0;

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

// @todo add walking animation and switch between them
#[derive(Resource)]
struct PlayerAnimations {
    idle: PlayerAnimationClip,
}

#[derive(Component)]
struct OnGround;

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
    ground_detector: ShapeCaster,
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
            collider: Collider::rectangle(16., PLAYER_HEIGHT),
            ground_detector: ShapeCaster::new(
                Collider::rectangle(14., PLAYER_FOOT_HEIGHT),
                // Put detector at the player's feet
                Vec2 {
                    x: 0.0,
                    y: PLAYER_FOOT_ANCHOR,
                },
                0.0,
                Dir2::NEG_Y,
            )
            .with_max_distance(PLAYER_FOOT_RANGE),
            axes: LockedAxes::ROTATION_LOCKED,
            // Anchor is down a bit because sprite is not vertically centered
            anchor: Anchor(Vec2::new(0.0, PLAYER_HEIGHT_ANCHOR_OFFSET)),
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
        app.add_systems(Update, ground_detection.before(GameSet::Input));
        app.add_systems(Update, move_player.in_set(GameSet::Input));
        app.add_systems(Update, animate_player);
        app.register_ldtk_entity::<PlayerBundle>("Player");
    }
}

fn ground_detection(mut commands: Commands, player: Single<(Entity, &ShapeHits), With<Player>>) {
    let (player_entity, hits) = *player;
    let is_on_ground = !hits.is_empty();
    if is_on_ground {
        commands.entity(player_entity).insert(OnGround);
    } else {
        commands.entity(player_entity).remove::<OnGround>();
    }
}

fn get_change_for_input(keyboard_input: &ButtonInput<KeyCode>) -> f32 {
    let mut change = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        change += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        change -= 1.0;
    }

    change * PLAYER_SPEED
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut LinearVelocity, Has<OnGround>), With<Player>>,
) {
    let (mut vel, on_ground) = player.into_inner();
    vel.x = get_change_for_input(&keyboard_input);
    if keyboard_input.just_released(KeyCode::ArrowUp) && vel.0.y > 0.0 {
        vel.0.y = vel.0.y.min(PLAYER_JUMP_CUT_SPEED);
    }
    if on_ground && keyboard_input.just_pressed(KeyCode::ArrowUp) {
        vel.y = PLAYER_JUMP_SPEED;
    }
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
