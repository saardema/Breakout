use bevy::prelude::*;

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct Paddle {
    pub speed: f32,
}

#[derive(Component)]
pub struct Collider {
    pub size: Vec2,
}

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct GameOverText;

#[derive(Component)]
pub struct AttachedToPaddle;
