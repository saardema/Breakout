use bevy::prelude::*;

use crate::*;

pub const BALL_SIZE: f32 = 22.;

#[derive(Component, Default)]
pub struct Ball {
    pub direction: Vec2,
    pub speed: f32,
    pub curve: f32,
}

pub struct BallCollisionEvent(pub BallCollisionType);

pub enum BallCollisionType {
    Wall,
    Paddle,
    Brick,
}

pub struct BallLossEvent;
pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(check_collisions.before(ball_movement))
                .with_system(ball_movement)
                .with_system(ball_loss),
        )
        .add_event::<BallCollisionEvent>()
        .add_event::<BallLossEvent>();
    }
}

#[derive(Bundle)]
pub struct BallBundle {
    pub ball: Ball,
    pub sprite_bundle: SpriteBundle,
    pub collider: Collider,
}

impl BallBundle {
    fn new(texture: Handle<Image>) -> Self {
        Self {
            ball: Ball {
                direction: Vec2::new(0., 1.),
                speed: 400.,
                curve: 0.,
            },
            sprite_bundle: SpriteBundle {
                texture,
                transform: Transform::from_xyz(0., 50., 0.),
                ..default()
            },
            collider: Collider {
                size: Vec2::splat(BALL_SIZE),
            },
        }
    }
}

fn ball_movement(time: Res<Time>, mut query: Query<(&mut Ball, &mut Transform)>) {
    for (mut ball, mut ball_transform) in query.iter_mut() {
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

fn ball_loss(
    mut commands: Commands,
    mut ball_loss_event: EventWriter<BallLossEvent>,
    mut ball_query: Query<(Entity, &mut Transform), With<Ball>>,
) {
    for (entity, ball_transform) in ball_query.iter_mut() {
        if ball_transform.translation.y < -WIN_HEIGHT / 2. {
            commands.entity(entity).despawn();
            ball_loss_event.send(BallLossEvent);
        }
    }
}

fn check_collisions(
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
    let mut bricks_to_despawn = Vec::new();

    for (mut ball, ball_collider, ball_transform) in ball_query.iter_mut() {
        let mut collision = get_wall_collision_direction(ball_transform.translation);

        if collision.is_some() {
            collision_events.send(BallCollisionEvent(BallCollisionType::Wall));
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
                    if brick.is_some() {
                        if !bricks_to_despawn.contains(&entity) {
                            bricks_to_despawn.push(entity);
                        }
                        collision_events.send(BallCollisionEvent(BallCollisionType::Brick));
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

                    break;
                }
            }
        }

        match collision {
            Some(Collision::Left) => ball.direction.x = -ball.direction.x.abs(),
            Some(Collision::Right) => ball.direction.x = ball.direction.x.abs(),
            Some(Collision::Top) => ball.direction.y = ball.direction.y.abs(),
            Some(Collision::Bottom) => ball.direction.y = -ball.direction.y.abs(),
            _ => {}
        }
    }

    for entity in bricks_to_despawn.iter() {
        commands.entity(*entity).despawn();
    }
}

pub fn spawn_ball(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn(BallBundle::new(assets.image.ball.clone()));
}
