use assets::*;
use ball::*;
use bevy::{prelude::*, sprite::collide_aabb::*};
use components::*;
use input::*;
use ui::*;

const WIN_WIDTH: f32 = 800.;
const WIN_HEIGHT: f32 = 800.;
const PADDLE_WIDTH: f32 = 104.;
const PADDLE_HEIGHT: f32 = 24.;
const BRICK_WIDTH: f32 = 64.;
const BRICK_HEIGHT: f32 = 32.;

mod assets;
mod ball;
mod components;
mod input;
mod ui;

#[derive(Resource)]
pub struct PlayerProgress {
    score: f32,
    ball_count: u16,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        PlayerProgress {
            score: 0.,
            ball_count: 3,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    PreGame,
    InGame,
    Change,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Breakout!".to_string(),
                width: WIN_WIDTH,
                height: WIN_HEIGHT,
                resizable: false,
                monitor: MonitorSelection::Index(0),
                ..default()
            },
            ..default()
        }))
        .insert_resource(ClearColor(Color::rgb(0.218, 0.554, 0.777)))
        .add_plugin(BallPlugin)
        .add_plugin(GameInputPlugin)
        .add_plugin(GameAssetsPlugin)
        .add_state(GameState::InGame)
        .add_system_set(
            SystemSet::on_enter(GameState::InGame)
                .with_system(reset_score)
                .with_system(spawn_ball_count)
                .with_system(spawn_bricks)
                .with_system(spawn_paddle)
                .with_system(spawn_score_text)
                .with_system(spawn_ball),
        )
        .add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(increase_ball_speed)
                .with_system(on_ball_loss)
                .with_system(text_update_system),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::InGame)
                .with_system(despawn_balls)
                .with_system(despawn_ball_count)
                .with_system(despawn_paddle)
                .with_system(despawn_text)
                .with_system(despawn_bricks),
        )
        .insert_resource(PlayerProgress::default())
        .add_startup_system(spawn_camera)
        .add_system_to_stage(CoreStage::Last, on_ball_collision)
        .run();
}

fn despawn_paddle(mut commands: Commands, query: Query<(Entity, &Paddle)>) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn increase_ball_speed(mut query: Query<&mut Ball>, time: Res<Time>) {
    for mut ball in query.iter_mut() {
        ball.speed += time.delta_seconds() * 4.;
    }
}

fn reset_score(mut player_progress: ResMut<PlayerProgress>) {
    *player_progress = PlayerProgress::default();
}

fn despawn_bricks(mut commands: Commands, query: Query<Entity, With<Brick>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn despawn_balls(mut commands: Commands, query: Query<Entity, With<Ball>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn_bricks(mut commands: Commands, assets: Res<GameAssets>) {
    for x in 0..10 {
        for y in 0..4 {
            let brick_sprites = [
                &assets.image.brick_blue,
                &assets.image.brick_green,
                &assets.image.brick_yellow,
                &assets.image.brick_red,
            ];

            commands
                .spawn(Brick)
                .insert(SpriteBundle {
                    texture: brick_sprites[y].clone(),
                    transform: Transform::from_xyz(
                        x as f32 * BRICK_WIDTH - 4.5 * BRICK_WIDTH,
                        y as f32 * BRICK_HEIGHT + 200.,
                        0.,
                    ),
                    ..default()
                })
                .insert(Collider {
                    size: Vec2::new(BRICK_WIDTH, BRICK_HEIGHT),
                });
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_paddle(mut commands: Commands, assets: Res<GameAssets>) {
    commands
        .spawn(Paddle { speed: 0. })
        .insert(SpriteBundle {
            transform: Transform::from_xyz(0., -320., 0.),
            texture: assets.image.paddle.clone(),
            ..default()
        })
        .insert(Collider {
            size: Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
        });
}

fn get_wall_collision_direction(position: Vec3) -> Option<Collision> {
    if position.x + BALL_SIZE / 2. > WIN_WIDTH / 2. {
        Some(Collision::Left)
    } else if position.x - BALL_SIZE / 2. < -WIN_WIDTH / 2. {
        Some(Collision::Right)
    } else if position.y + BALL_SIZE / 2. > WIN_HEIGHT / 2. {
        Some(Collision::Bottom)
    } else {
        None
    }
}

fn on_ball_collision(
    mut events: EventReader<BallCollisionEvent>,
    audio: Res<Audio>,
    assets: Res<GameAssets>,
    mut score: ResMut<PlayerProgress>,
) {
    for event in events.iter() {
        match event.0 {
            BallCollisionType::Brick => {
                score.score += 10.0;
                audio.play(assets.audio.drop_004.clone());
            }
            BallCollisionType::Paddle => {
                audio.play(assets.audio.drop_002.clone());
            }
            BallCollisionType::Wall => {
                audio.play(assets.audio.drop_003.clone());
            }
        }
    }
}

fn on_ball_loss(
    mut ball_loss_events: EventReader<BallLossEvent>,
    mut player_progress: ResMut<PlayerProgress>,
) {
    for _ in ball_loss_events.iter() {
        if player_progress.ball_count > 0 {
            player_progress.ball_count -= 1;
        }
    }
}
