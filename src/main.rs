use bevy::prelude::*;
use input::GameInputPlugin;

const WIN_WIDTH: f32 = 800.;
const WIN_HEIGHT: f32 = 800.;
const PADDLE_WIDTH: f32 = 104.;
const BALL_SIZE: f32 = 22.;

mod input;

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
        .add_plugin(GameInputPlugin)
        .add_startup_system(setup)
        .add_startup_system(spawn_bricks)
        .add_system(move_ball)
        .insert_resource(ClearColor(Color::rgb(0.218, 0.554, 0.777)))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(Paddle).insert(SpriteBundle {
        transform: Transform::from_xyz(0., -350., 0.),
        texture: asset_server.load("sprites/paddle.png"),
        ..default()
    });

    commands
        .spawn(Ball {
            direction: Vec2::new(rand::random(), rand::random()),
            speed: 3.,
        })
        .insert(SpriteBundle {
            transform: Transform::from_xyz(0., -50., 0.),
            texture: asset_server.load("sprites/ball.png"),
            ..default()
        });
}

fn spawn_bricks(mut commands: Commands, asset_server: Res<AssetServer>) {
    for x in 0..10 {
        for y in 0..4 {
            let brick_sprites = [
                "sprites/brick_blue.png",
                "sprites/brick_green.png",
                "sprites/brick_yellow.png",
                "sprites/brick_red.png",
            ];

            commands.spawn(Brick).insert(SpriteBundle {
                texture: asset_server.load(brick_sprites[y]),
                transform: Transform::from_xyz(
                    x as f32 * 64. - 4.5 * 64.,
                    (y * 32 + 200) as f32,
                    0.,
                ),
                ..default()
            });
        }
    }
}

fn move_ball(mut q: Query<(&mut Ball, &mut Transform)>) {
    let (mut ball, mut transform) = q.single_mut();
    transform.translation += ball.direction.extend(0.).normalize() * ball.speed;
    let bound_x = WIN_WIDTH / 2. - BALL_SIZE / 2.;
    let bound_y = WIN_HEIGHT / 2. - BALL_SIZE / 2.;

    if transform.translation.x < -bound_x || transform.translation.x > bound_x {
        ball.direction.x *= -1.;
    }

    if transform.translation.y < -bound_y || transform.translation.y > bound_y {
        ball.direction.y *= -1.;
    }
}

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct Ball {
    pub direction: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct Paddle;
