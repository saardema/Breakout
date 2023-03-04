use bevy::{ecs::system::Command, prelude::*};

use crate::*;

pub const BALL_SIZE: f32 = 22.;

#[derive(Component)]
pub struct Ball {
    pub direction: Vec2,
    pub speed: f32,
    pub curve: f32,
    pub ball_type: BallType,
}

#[derive(PartialEq)]
pub enum BallType {
    Regular,
    FireBall,
}

#[derive(Component)]
pub struct FireBall {
    pub age: f32,
}

pub struct BallCollisionEvent(pub BallCollisionType);

#[derive(PartialEq)]
pub enum BallCollisionType {
    Wall,
    Paddle,
}

pub struct AllBallsLostEvent;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(check_collisions.before(ball_movement))
                .with_system(ball_movement)
                .with_system(ball_loss),
        )
        .add_event::<BallCollisionEvent>()
        .add_event::<AllBallsLostEvent>();
    }
}

fn ball_movement(
    time: Res<Time>,
    mut balls_query: Query<(&mut Ball, &mut Transform, Option<&AttachedToPaddle>)>,
    paddle_query: Query<&Transform, (With<Paddle>, Without<Ball>)>,
) {
    for (mut ball, mut ball_transform, attached) in balls_query.iter_mut() {
        if attached.is_some() {
            if let Ok(paddle_transform) = paddle_query.get_single() {
                ball_transform.translation.x = paddle_transform.translation.x;
                ball_transform.translation.y =
                    paddle_transform.translation.y + PADDLE_HEIGHT / 2. + BALL_SIZE / 2.;
            }
        } else {
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
        }
    }
}

fn ball_loss(
    mut commands: Commands,
    mut ball_loss_event: EventWriter<AllBallsLostEvent>,
    mut ball_query: Query<(Entity, &mut Transform), With<Ball>>,
) {
    for (entity, ball_transform) in ball_query.iter_mut() {
        if ball_transform.translation.y < -WIN_HEIGHT / 2. {
            commands.entity(entity).despawn();
        }
    }

    if ball_query.is_empty() {
        ball_loss_event.send(AllBallsLostEvent);
    }
}

fn check_collisions(
    mut commands: Commands,
    mut brick_events: EventWriter<BrickDesctructionEvent>,
    mut collision_events: EventWriter<BallCollisionEvent>,
    mut ball_query: Query<(&mut Ball, &Collider, &mut Transform), Without<AttachedToPaddle>>,
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
    let mut bricks_to_despawn = Vec::new();

    for (mut ball, ball_collider, ball_transform) in ball_query.iter_mut() {
        let mut collision = get_wall_collision_direction(ball_transform.translation);

        if collision.is_some() {
            collision_events.send(BallCollisionEvent(BallCollisionType::Wall));

            match collision {
                Some(Collision::Left) => ball.direction.x = -ball.direction.x.abs(),
                Some(Collision::Right) => ball.direction.x = ball.direction.x.abs(),
                Some(Collision::Top) => ball.direction.y = ball.direction.y.abs(),
                Some(Collision::Bottom) => ball.direction.y = -ball.direction.y.abs(),
                _ => {}
            }
        } else {
            for (entity, other_collider, other_transform, brick, paddle) in
                collider_query.iter_mut()
            {
                collision = collide(
                    ball_transform.translation,
                    ball_collider.size,
                    other_transform.translation,
                    other_collider.size,
                );

                if collision.is_some() {
                    if let Some(brick) = brick {
                        if !bricks_to_despawn.contains(&entity) {
                            bricks_to_despawn.push(entity);
                        }

                        brick_events.send(BrickDesctructionEvent {
                            position: other_transform.translation,
                            brick_type: brick.brick_type.clone(),
                        });
                    } else if let Some(paddle) = paddle {
                        // Reflection based on paddle hit point
                        let delta = ball_transform.translation.x - other_transform.translation.x;
                        ball.direction.x += delta * 0.008;

                        // Curve balls
                        if paddle.speed.abs() > 12. {
                            ball.curve = paddle.speed.clamp(-50., 50.) / 50.;
                        }

                        // Bounce up
                        ball.direction.y = ball.direction.y.abs();
                        collision_events.send(BallCollisionEvent(BallCollisionType::Paddle));
                    }

                    if ball.ball_type != BallType::FireBall {
                        match collision {
                            Some(Collision::Left) => ball.direction.x = -ball.direction.x.abs(),
                            Some(Collision::Right) => ball.direction.x = ball.direction.x.abs(),
                            Some(Collision::Top) => ball.direction.y = ball.direction.y.abs(),
                            Some(Collision::Bottom) => ball.direction.y = -ball.direction.y.abs(),
                            _ => {}
                        }
                    }

                    break;
                }
            }
        }
    }

    for entity in bricks_to_despawn.iter() {
        commands.entity(*entity).despawn_recursive();
    }
}

pub fn spawn_ball(mut commands: Commands) {
    commands.add(SpawnBallCommand);
}

pub struct SpawnBallCommand;

impl Command for SpawnBallCommand {
    fn write(self, world: &mut World) {
        let assets = world.get_resource::<GameAssets>();
        let progress = world.get_resource::<PlayerProgress>();
        if let Some(assets) = assets {
            world.spawn((
                Ball {
                    direction: Vec2::new(rand::random::<f32>() * 2. - 1., 1.),
                    speed: 300. + progress.unwrap().level as f32 * 50.,
                    curve: 0.,
                    ball_type: BallType::Regular,
                },
                SpriteBundle {
                    texture: assets.image.ball.clone(),
                    transform: Transform::from_xyz(0., 0., 10.),
                    ..default()
                },
                Collider {
                    size: Vec2::splat(BALL_SIZE),
                },
                AttachedToPaddle,
            ));
        }
    }
}
