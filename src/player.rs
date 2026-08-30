use crate::GameSet;
use avian2d::{
    collision::collider::Collider,
    dynamics::rigid_body::{Friction, LinearVelocity, LockedAxes, RigidBody},
    spatial_query::{ShapeCaster, ShapeHits},
};
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_ldtk::{LdtkEntity, Worldly, app::LdtkEntityAppExt};

const PLAYER_SPEED: f32 = 130.0;
const PLAYER_JUMP_SPEED: f32 = 270.0;
const PLAYER_JUMP_CUT_SPEED: f32 = 150.0;
const PLAYER_HEIGHT: f32 = 20.0;
const PLAYER_HEIGHT_ANCHOR_OFFSET: f32 = 0.03;
const PLAYER_FOOT_HEIGHT: f32 = 2.0;
const PLAYER_FOOT_ANCHOR: f32 = -(PLAYER_HEIGHT / 2.) + (PLAYER_FOOT_HEIGHT / 2.);
const PLAYER_FOOT_RANGE: f32 = 2.0;

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug, Default)]
enum PlayerState {
    #[default]
    Idle,
    Walking,
    Jumping,
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Debug, Default)]
enum FacingDirection {
    #[default]
    Right,
    Left,
}

#[derive(Component)]
struct SpriteAnimation {
    frames: usize,
    timer: Timer,
}

struct PlayerAnimationClip {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    frames: usize,
}

#[derive(Resource)]
struct PlayerAnimations {
    idle: PlayerAnimationClip,
    walk: PlayerAnimationClip,
    jump: PlayerAnimationClip,
}

impl PlayerAnimations {
    pub fn clip_for_state(&self, state: &PlayerState) -> &PlayerAnimationClip {
        match state {
            PlayerState::Idle => &self.idle,
            PlayerState::Walking => &self.walk,
            PlayerState::Jumping => &self.jump,
        }
    }
}

#[derive(Component)]
struct OnGround;

#[derive(Component, Default)]
pub struct Player;

#[derive(Bundle, LdtkEntity)]
struct PlayerBundle {
    player: Player,
    state: PlayerState,
    #[sprite_sheet("Priest-Idle.png", 100, 100, 6, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
    #[worldly]
    worldly: Worldly,
    body: RigidBody,
    friction: Friction,
    collider: Collider,
    ground_detector: ShapeCaster,
    axes: LockedAxes,
    anchor: Anchor,
    animation: SpriteAnimation,
    facing: FacingDirection,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            state: PlayerState::Idle,
            sprite_sheet: Sprite::default(),
            worldly: Worldly::default(),
            body: RigidBody::Dynamic,
            friction: Friction::ZERO
                .with_combine_rule(avian2d::dynamics::rigid_body::CoefficientCombine::Min),
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
            facing: FacingDirection::Right,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player);
        app.add_systems(Update, ground_detection.before(GameSet::Input));
        app.add_systems(Update, move_player.in_set(GameSet::Input));
        app.add_systems(
            Update,
            (
                determine_state,
                determine_facing,
                update_sprite,
                update_facing,
                animate_player,
            )
                .chain()
                .after(GameSet::Input),
        );
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

fn determine_state(
    player: Single<(&mut PlayerState, &LinearVelocity, Has<OnGround>), With<Player>>,
) {
    let (mut state, vel, on_ground) = player.into_inner();
    let is_walking = vel.x.abs() > 0.1;
    let next_state = match (on_ground, is_walking) {
        (false, _) => PlayerState::Jumping,
        (true, true) => PlayerState::Walking,
        (true, false) => PlayerState::Idle,
    };
    if *state != next_state {
        *state = next_state;
    }
}

fn determine_facing(player: Single<(&mut FacingDirection, &LinearVelocity), With<Player>>) {
    let (mut facing, vel) = player.into_inner();
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

fn update_sprite(
    mut query: Query<(&PlayerState, &mut Sprite, &mut SpriteAnimation), Changed<PlayerState>>,
    animations: Res<PlayerAnimations>,
) {
    for (state, mut sprite, mut animation) in &mut query {
        let clip = animations.clip_for_state(state);
        sprite.image = clip.image.clone();
        sprite.texture_atlas = Some(TextureAtlas {
            layout: clip.layout.clone(),
            index: 0,
        });
        animation.frames = clip.frames;
    }
}

fn update_facing(mut query: Query<(&FacingDirection, &mut Sprite), Changed<FacingDirection>>) {
    for (facing, mut sprite) in &mut query {
        sprite.flip_x = *facing == FacingDirection::Left;
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
    let idle = PlayerAnimationClip {
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
    let walk = PlayerAnimationClip {
        image: asset_server.load("Priest-Walk.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            8,
            1,
            None,
            None,
        )),
        frames: 8,
    };
    let jump = PlayerAnimationClip {
        image: asset_server.load("Priest-Walk.png"),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(100),
            8,
            1,
            None,
            None,
        )),
        frames: 1,
    };
    commands.insert_resource(PlayerAnimations { idle, walk, jump });
}

fn animate_player(time: Res<Time>, mut query: Query<(&mut SpriteAnimation, &mut Sprite)>) {
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
