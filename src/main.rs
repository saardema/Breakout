use assets::*;
use ball::{spawn_ball, BallPlugin};
use bevy::{ecs::system::Command, prelude::*, sprite::collide_aabb::*};
use input::GameInputPlugin;

const WIN_WIDTH: f32 = 800.;
const WIN_HEIGHT: f32 = 800.;
const PADDLE_WIDTH: f32 = 104.;
const PADDLE_HEIGHT: f32 = 24.;
const BALL_SIZE: f32 = 22.;
const BRICK_WIDTH: f32 = 64.;
const BRICK_HEIGHT: f32 = 32.;

mod assets;
mod ball;
mod input;

#[derive(Component)]
pub struct Brick;

#[derive(Component, Default)]
pub struct Ball {
    pub direction: Vec2,
    pub speed: f32,
    pub curve: f32,
}

#[derive(Component)]
pub struct Paddle {
    speed: f32,
}

#[derive(Component)]
pub struct Collider {
    size: Vec2,
}

pub enum BallCollisionType {
    Wall,
    Paddle,
    Brick,
}

pub struct BallCollisionEvent(BallCollisionType);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Breakout!".to_string(),
                width: WIN_WIDTH,
                height: WIN_HEIGHT,
                resizable: false,
                ..default()
            },
            ..default()
        }))
        .add_startup_system(init)
        .add_plugin(BallPlugin)
        .add_plugin(GameInputPlugin)
        .add_plugin(GameAssetsPlugin)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_paddle)
        .add_system_to_stage(CoreStage::Last, collision_sounds)
        .add_event::<BallCollisionEvent>()
        .insert_resource(ClearColor(Color::rgb(0.218, 0.554, 0.777)))
        .run();
}

fn init(
    mut commands: Commands,
    bricks: Query<Entity, With<Brick>>,
    assets: Res<GameAssets>,
    balls: Query<Entity, With<Ball>>,
) {
    reset_bricks(&mut commands, &bricks);
    spawn_bricks(&mut commands, &assets);
    clear_balls(&mut commands, &balls);
    spawn_ball(&mut commands, &assets);
}

fn reset_bricks(commands: &mut Commands, query: &Query<Entity, With<Brick>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn clear_balls(commands: &mut Commands, query: &Query<Entity, With<Ball>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn spawn_bricks(commands: &mut Commands, assets: &Res<GameAssets>) {
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
    } else if position.y - BALL_SIZE / 2. < -WIN_HEIGHT / 2. {
        Some(Collision::Top)
    } else {
        None
    }
}

fn collision_sounds(
    mut events: EventReader<BallCollisionEvent>,
    audio: Res<Audio>,
    assets: Res<GameAssets>,
) {
    for event in events.iter() {
        match event.0 {
            BallCollisionType::Brick => {
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
