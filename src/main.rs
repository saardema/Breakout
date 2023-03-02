use std::time::Duration;

use assets::*;
use ball::*;
use bevy::{prelude::*, sprite::collide_aabb::*, window::WindowFocused};
use input::*;
use ui::*;

const WIN_WIDTH: f32 = 800.;
const WIN_HEIGHT: f32 = 800.;
const PADDLE_WIDTH: f32 = 104.;
const PADDLE_HEIGHT: f32 = 24.;
const BRICK_WIDTH: f32 = 64.;
const BRICK_HEIGHT: f32 = 32.;
const BG_COLOR: Color = Color::rgba(0.218, 0.554, 0.777, 0.1);

mod assets;
mod ball;
mod input;
mod ui;

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
pub struct AttachedToPaddle;

#[derive(Resource)]
pub struct PlayerProgress {
    score: f32,
    balls_remaining: u16,
    level: u16,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        PlayerProgress {
            score: 0.,
            balls_remaining: 0,
            level: 1,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Start,
    Playing,
    Paused,
    GameOver,
}

pub struct GamePauseEvent {
    pub should_pause: bool,
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
        //
        // Events
        .add_event::<GamePauseEvent>()
        //
        // State independent systems
        .add_startup_system(spawn_camera)
        .add_system(on_window_focus)
        .add_system(on_pause)
        .add_system_to_stage(CoreStage::Last, on_ball_collision)
        // .add_system(print_state)
        //
        // Resources
        .insert_resource(PlayerProgress::default())
        .insert_resource(GameOverTimer(Timer::new(
            Duration::from_secs(2),
            TimerMode::Once,
        )))
        .insert_resource(ClearColor(BG_COLOR))
        //
        // Plugins
        .add_plugin(BallPlugin)
        .add_plugin(GameInputPlugin)
        .add_plugin(GameAssetsPlugin)
        .add_state(GameState::Start)
        //
        // Start state
        .add_system_set(
            SystemSet::on_enter(GameState::Start)
                .with_system(spawn_play_text)
                .with_system(spawn_title_text),
        )
        .add_system_set(SystemSet::on_exit(GameState::Start).with_system(despawn::<Text>))
        //
        // Playing state
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(reset_player_progress.before(spawn_ball_count))
                .with_system(spawn_ball_count)
                .with_system(spawn_bricks)
                .with_system(spawn_paddle)
                .with_system(spawn_level_text.before(spawn_score_text))
                .with_system(spawn_score_text)
                .with_system(spawn_ball),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(increase_ball_speed)
                .with_system(on_ball_loss)
                .with_system(update_ball_count)
                .with_system(update_level_text)
                .with_system(update_score_text),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Playing)
                .with_system(despawn::<Ball>)
                .with_system(despawn::<UiBall>)
                .with_system(despawn::<Paddle>)
                .with_system(despawn::<Text>)
                .with_system(despawn::<Brick>),
        )
        //
        // Paused state
        .add_system_set(SystemSet::on_enter(GameState::Paused).with_system(spawn_play_text))
        .add_system_set(SystemSet::on_exit(GameState::Paused).with_system(despawn::<PlayText>))
        //
        // GameOver state
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(spawn_game_over_text))
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(game_over_timer))
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(despawn::<Text>))
        .run();
}

#[derive(Resource, Default)]
pub struct GameOverTimer(pub Timer);

fn game_over_timer(
    mut timer: ResMut<GameOverTimer>,
    mut state: ResMut<State<GameState>>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        timer.0.reset();
        state.set(GameState::Start).unwrap();
    }
}

fn increase_ball_speed(mut query: Query<&mut Ball>, time: Res<Time>) {
    for mut ball in query.iter_mut() {
        ball.speed += time.delta_seconds() * 4.;
    }
}

fn reset_player_progress(mut player_progress: ResMut<PlayerProgress>) {
    *player_progress = PlayerProgress::default();
}

fn spawn_bricks(mut commands: Commands, assets: Res<GameAssets>) {
    for x in 0..10 {
        for y in 0..6 {
            let brick_sprites = [
                &assets.image.brick_red,
                &assets.image.brick_orange,
                &assets.image.brick_yellow,
                &assets.image.brick_green,
                &assets.image.brick_light_green,
                &assets.image.brick_blue,
            ];

            commands
                .spawn(Brick)
                .insert(SpriteBundle {
                    texture: brick_sprites[y].clone(),
                    transform: Transform::from_xyz(
                        x as f32 * BRICK_WIDTH - 4.5 * BRICK_WIDTH,
                        WIN_HEIGHT / 2. - y as f32 * BRICK_HEIGHT - 100.,
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
            transform: Transform::from_xyz(0., -280., 0.),
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
    mut commands: Commands,
    mut ball_loss_events: EventReader<BallLossEvent>,
    mut player_progress: ResMut<PlayerProgress>,
    mut state: ResMut<State<GameState>>,
) {
    for _ in ball_loss_events.iter() {
        if player_progress.balls_remaining > 0 {
            player_progress.balls_remaining -= 1;
            commands.add(SpawnBallCommand);
        } else {
            state.set(GameState::GameOver).unwrap();
        }
    }
}

fn despawn<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}

fn print_state(mut state: ResMut<State<GameState>>) {
    if state.is_changed() {
        info!("{:?}", state.current());
    }
}

fn on_window_focus(
    mut window_focused: EventReader<WindowFocused>,
    mut pause_event: EventWriter<GamePauseEvent>,
) {
    for window in window_focused.iter() {
        info!("Window focus: {}", window.focused);
        pause_event.send(GamePauseEvent {
            should_pause: !window.focused,
        })
    }
}

fn on_pause(mut pause_event: EventReader<GamePauseEvent>, mut state: ResMut<State<GameState>>) {
    if state.current() != &GameState::GameOver {
        for e in pause_event.iter() {
            print!("{:?} -> ", state.current());
            if e.should_pause && state.current() != &GameState::Paused {
                state.overwrite_push(GameState::Paused).unwrap();
                println!("Pause");
            } else if !e.should_pause && state.current() == &GameState::Paused {
                state.overwrite_pop().unwrap();
                println!("Unpause");
            }
        }
    }
}
