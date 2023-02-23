use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

use input::GameInputPlugin;

const WIN_WIDTH: f32 = 800.;
const WIN_HEIGHT: f32 = 800.;
const PADDLE_WIDTH: f32 = 104.;
const PADDLE_HEIGHT: f32 = 24.;
const BALL_SIZE: f32 = 22.;
const BRICK_WIDTH: f32 = 64.;
const BRICK_HEIGHT: f32 = 32.;

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
pub struct Wall;

#[derive(Component)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
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
        .add_plugin(GameInputPlugin)
        // .add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_ball)
        .add_startup_system(spawn_paddle)
        .add_startup_system(spawn_walls)
        .add_startup_system(spawn_bricks)
        .add_system(ball_movement)
        .add_system(handle_ball_collision)
        .add_event::<BallCollisionEvent>()
        .insert_resource(ClearColor(Color::rgb(0.218, 0.554, 0.777)))
        .run();
}

pub struct AudioHandles {
    pub drop_002: Handle<AudioSource>,
    pub drop_003: Handle<AudioSource>,
    pub drop_004: Handle<AudioSource>,
}

pub struct ImageHandles {
    pub ball: Handle<Image>,
    pub paddle: Handle<Image>,
    pub brick_blue: Handle<Image>,
    pub brick_green: Handle<Image>,
    pub brick_red: Handle<Image>,
    pub brick_yellow: Handle<Image>,
}

#[derive(Resource)]
pub struct AssetHandles {
    pub audio: AudioHandles,
    pub image: ImageHandles,
}

fn load_assets(asset_server: Res<AssetServer>, mut cmd: Commands) {
    cmd.insert_resource(AssetHandles {
        audio: AudioHandles {
            drop_002: asset_server.load("/sound/drop_002.ogg"),
            drop_003: asset_server.load("/sound/drop_003.ogg"),
            drop_004: asset_server.load("/sound/drop_004.ogg"),
        },
        image: ImageHandles {
            ball: asset_server.load("sprites/ball.png"),
            paddle: asset_server.load("sprites/paddle.png"),
            brick_blue: asset_server.load("sprites/brick_blue.png"),
            brick_green: asset_server.load("sprites/brick_green.png"),
            brick_red: asset_server.load("sprites/brick_red.png"),
            brick_yellow: asset_server.load("sprites/brick_yellow.png"),
        },
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_ball(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Ball {
            direction: Vec2::new(0., -1.),
            speed: 400.,
            curve: 0.,
        })
        .insert(SpriteBundle {
            transform: Transform::from_xyz(0., 50., 0.),
            texture: asset_server.load("sprites/ball.png"),
            ..default()
        })
        .insert(Collider {
            width: BALL_SIZE,
            height: BALL_SIZE,
        });
}

fn spawn_paddle(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Paddle { speed: 0. })
        .insert(SpriteBundle {
            transform: Transform::from_xyz(0., -320., 0.),
            texture: asset_server.load("sprites/paddle.png"),
            ..default()
        })
        .insert(Collider {
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        });
}

fn spawn_walls(mut commands: Commands) {
    // Left wall
    commands
        .spawn(Wall)
        .insert(Collider {
            width: 50.,
            height: WIN_HEIGHT,
        })
        .insert(Transform::from_xyz(-WIN_WIDTH / 2. - 25., 0., 0.));

    // Right wall
    commands
        .spawn(Wall)
        .insert(Collider {
            width: 50.,
            height: WIN_HEIGHT,
        })
        .insert(Transform::from_xyz(WIN_WIDTH / 2. + 25., 0., 0.));

    // Top wall
    commands
        .spawn(Wall)
        .insert(Collider {
            width: WIN_WIDTH,
            height: 50.,
        })
        .insert(Transform::from_xyz(0., WIN_HEIGHT / 2. + 25., 0.));

    // Bottom wall
    commands
        .spawn(Wall)
        .insert(Collider {
            width: WIN_WIDTH,
            height: 50.,
        })
        .insert(Transform::from_xyz(0., -WIN_HEIGHT / 2. - 25., 0.));
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

            commands
                .spawn(Brick)
                .insert(SpriteBundle {
                    texture: asset_server.load(brick_sprites[y]),
                    transform: Transform::from_xyz(
                        x as f32 * BRICK_WIDTH - 4.5 * BRICK_WIDTH,
                        y as f32 * BRICK_HEIGHT + 200.,
                        0.,
                    ),
                    ..default()
                })
                .insert(Collider {
                    width: BRICK_WIDTH,
                    height: BRICK_HEIGHT,
                });
        }
    }
}

fn ball_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut collision_events: EventWriter<BallCollisionEvent>,
    mut ball_query: Query<(&mut Ball, &Collider, &mut Transform)>,
    mut collider_query: Query<
        (
            Entity,
            &Collider,
            &Transform,
            Option<&Brick>,
            Option<&Paddle>,
        ),
        Without<Ball>,
    >,
) {
    let mut reflect_x = false;
    let mut reflect_y = false;

    let (mut ball, ball_collider, mut ball_transform) = ball_query.single_mut();

    for (entity, other_collider, other_transform, brick, paddle) in collider_query.iter_mut() {
        let collision = collide(
            ball_transform.translation,
            Vec2::new(ball_collider.width, ball_collider.height),
            other_transform.translation,
            Vec2::new(other_collider.width, other_collider.height),
        );

        if let Some(collision) = collision {
            if brick.is_some() {
                commands.entity(entity).despawn();
                collision_events.send(BallCollisionEvent(BallCollisionType::Brick));
            } else if paddle.is_some() {
                // Reflection based on paddle hit point
                let delta = ball_transform.translation.x - other_transform.translation.x;
                ball.direction.x += delta * 0.008;

                let paddle_speed = paddle.unwrap().speed;

                if paddle_speed > 12. || paddle_speed < -12. {
                    ball.curve = paddle_speed.clamp(-50., 50.) / 50.;
                }

                collision_events.send(BallCollisionEvent(BallCollisionType::Paddle));
            } else {
                collision_events.send(BallCollisionEvent(BallCollisionType::Wall));
            }

            match collision {
                Collision::Left => reflect_x = ball.direction.x > 0.0,
                Collision::Right => reflect_x = ball.direction.x < 0.0,
                Collision::Top => reflect_y = ball.direction.y < 0.0,
                Collision::Bottom => reflect_y = ball.direction.y > 0.0,
                _ => {}
            }
        }
    }

    if reflect_x {
        ball.direction.x *= -1.;
    }

    if reflect_y {
        ball.direction.y *= -1.;
    }

    // Curveball
    if ball.curve.abs() > 0.09 {
        ball.curve *= 0.95;
        ball.direction = Vec2::from_angle(ball.curve * 0.06).rotate(ball.direction);
    } else {
        ball.curve = 0.;
    }

    // Translate
    ball_transform.translation +=
        ball.direction.extend(0.).normalize() * ball.speed * time.delta_seconds().min(1.);
    ball_transform.translation.x = ball_transform
        .translation
        .x
        .clamp(-WIN_WIDTH / 2., WIN_WIDTH / 2.);
}

fn handle_ball_collision(
    mut events: EventReader<BallCollisionEvent>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    for event in events.iter() {
        match event.0 {
            BallCollisionType::Brick => {
                audio.play(asset_server.load("sound/drop_004.ogg"));
            }
            BallCollisionType::Paddle => {
                audio.play(asset_server.load("sound/drop_002.ogg"));
            }
            BallCollisionType::Wall => {
                audio.play(asset_server.load("sound/drop_003.ogg"));
            }
        }
    }
}
