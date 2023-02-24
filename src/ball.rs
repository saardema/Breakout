use bevy::prelude::*;

use crate::*;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(check_collisions.before(ball_movement))
            .add_system(ball_movement);
    }
}

pub struct SpawnBallCommand;

impl Command for SpawnBallCommand {
    fn write(self, world: &mut World) {
        let assets = world.get_resource::<GameAssets>();
        if let Some(assets) = assets {
            world.spawn(BallBundle::new(assets.image.ball.clone()));
        }
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
                        commands.entity(entity).despawn();
                        collision_events.send(BallCollisionEvent(BallCollisionType::Brick));
                    } else if let Some(paddle) = paddle {
                        // Reflection based on paddle hit point
                        let delta = ball_transform.translation.x - other_transform.translation.x;
                        ball.direction.x += delta * 0.008;

                        // Curve balls
                        if paddle.speed.abs() > 12. {
                            ball.curve = paddle.speed.clamp(-50., 50.) / 50.;
                            info!(ball.curve);
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
}

pub fn spawn_ball(commands: &mut Commands, assets: &Res<GameAssets>) {
    commands.spawn(BallBundle::new(assets.image.ball.clone()));
}
