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
const BG_COLOR: Color = Color::rgb(0.218, 0.554, 0.777);
const BASE_BRICK_SCORE: f32 = 10.;
const SCORE_MULTIPLIER_TIMEOUT: f32 = 1.;
const SCORE_MULTIPLIER: f32 = 50.;
const SCORE_ANIM_MAX_DURATION: f32 = 0.6;
const EXTRA_BALL_COUNT: u8 = 3;
const BALLS_SPEED_TIME_INCREMENT: f32 = 2.;
const BRICK_COLUMNS: usize = 10;
const BRICK_ROWS: usize = 6;
const FIREBALL_CHANCE: f32 = 0.04;
const MAX_FIREBALL_AGE: f32 = 4.;

mod assets;
mod ball;
mod input;
mod ui;

pub struct ScoreIncrementEvent(f32);

#[derive(Component)]
pub struct Brick {
    brick_type: BrickType,
}

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

#[derive(PartialEq, Clone)]
pub enum BrickType {
    Regular,
    Fireball,
}

pub struct BrickDesctructionEvent {
    position: Vec3,
    brick_type: BrickType,
}

#[derive(Resource)]
pub struct PlayerProgress {
    score: f32,
    extra_balls_remaining: u8,
    level: u16,
    bonus_score: f32,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        PlayerProgress {
            score: 0.,
            extra_balls_remaining: EXTRA_BALL_COUNT,
            level: 1,
            bonus_score: 0.,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum GameState {
    Start,
    Playing,
    LevelCompleted,
    Paused,
    GameOver,
}

pub struct GamePauseEvent {
    pub should_pause: bool,
}

#[derive(Resource)]
struct ScoreIncrementTimer(pub Timer);

#[derive(Resource)]
pub struct StateTransitionTimer(pub Timer);

fn main() {
    let mut app = App::new();

    // Plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            title: "Breakout!".to_string(),
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
            resizable: false,
            monitor: if cfg!(debug_assertions) {
                MonitorSelection::Index(0)
            } else {
                MonitorSelection::Current
            },
            ..default()
        },
        ..default()
    }));

    // Events
    app.add_event::<GamePauseEvent>()
        .add_event::<BrickDesctructionEvent>()
        .add_event::<ScoreIncrementEvent>();

    // State independent systems
    app.add_startup_system(spawn_background)
        .add_startup_system(spawn_camera)
        .add_system(on_window_focus)
        .add_system(on_pause)
        .add_system_to_stage(CoreStage::Last, play_sounds);

    // Resources
    app.insert_resource(PlayerProgress::default())
        .insert_resource(StateTransitionTimer(Timer::new(
            Duration::from_secs(2),
            TimerMode::Once,
        )))
        .insert_resource(ScoreIncrementTimer(Timer::new(
            Duration::from_secs_f32(1.),
            TimerMode::Once,
        )))
        .insert_resource(BackgroundAnimationDirection(true))
        .insert_resource(ClearColor(BG_COLOR));

    // Plugins
    app.add_plugin(UiPlugin)
        .add_plugin(BallPlugin)
        .add_plugin(GameInputPlugin)
        .add_plugin(GameAssetsPlugin);

    app.add_state(GameState::Start);

    // Start state
    app.add_system_set(
        SystemSet::on_enter(GameState::Start)
            .with_system(play_music)
            .with_system(spawn_play_text)
            .with_system(spawn_title_text),
    )
    .add_system_set(SystemSet::on_exit(GameState::Start).with_system(despawn::<Text>));

    // Playing state
    app.add_system_set(
        SystemSet::on_enter(GameState::Playing)
            .with_system(reset_bonus_score)
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
            .with_system(on_all_balls_lost)
            .with_system(animate_background)
            .with_system(next_level)
            .with_system(expire_fireballs)
            .with_system(handle_brick_destruction)
            .with_system(update_score)
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
    );

    // Paused state
    app.add_system_set(SystemSet::on_enter(GameState::Paused).with_system(spawn_play_text))
        .add_system_set(SystemSet::on_exit(GameState::Paused).with_system(despawn::<PlayText>));

    // Level transition state
    app.add_system_set(
        SystemSet::on_enter(GameState::LevelCompleted)
            .with_system(spawn_level_complete_text)
            .with_system(spawn_bonus_score_text),
    )
    .add_system_set(SystemSet::on_update(GameState::LevelCompleted).with_system(transition_timer))
    .add_system_set(SystemSet::on_exit(GameState::LevelCompleted).with_system(despawn::<Text>));

    // GameOver state
    app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(spawn_game_over_text))
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(transition_timer))
        .add_system_set(
            SystemSet::on_exit(GameState::GameOver)
                .with_system(despawn::<Text>)
                .with_system(reset_player_progress),
        );

    app.run();
}

fn play_music(assets: Res<GameAssets>, audio: Res<Audio>) {
    audio.play_with_settings(
        assets.audio.music_01.clone(),
        PlaybackSettings {
            repeat: true,
            volume: 0.5,
            speed: 1.,
        },
    );
}

fn reset_bonus_score(mut progress: ResMut<PlayerProgress>) {
    progress.bonus_score = 0.;
}

fn transition_timer(
    mut timer: ResMut<StateTransitionTimer>,
    mut state: ResMut<State<GameState>>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        timer.0.reset();
        if state.current() == &GameState::GameOver {
            state.set(GameState::Start).unwrap()
        } else if state.current() == &GameState::LevelCompleted {
            state.set(GameState::Playing).unwrap()
        }
    }
}

fn increase_ball_speed(mut query: Query<&mut Ball>, time: Res<Time>) {
    for mut ball in query.iter_mut() {
        ball.speed += time.delta_seconds() * BALLS_SPEED_TIME_INCREMENT;
    }
}

fn reset_player_progress(mut player_progress: ResMut<PlayerProgress>) {
    *player_progress = PlayerProgress::default();
}

#[derive(Component)]
struct HasFireBall;

fn spawn_bricks(mut commands: Commands, assets: Res<GameAssets>) {
    for x in 0..BRICK_COLUMNS {
        for y in 0..BRICK_ROWS {
            let brick_sprites = [
                &assets.image.brick_red,
                &assets.image.brick_orange,
                &assets.image.brick_yellow,
                &assets.image.brick_green,
                &assets.image.brick_light_green,
                &assets.image.brick_blue,
            ];

            let mut bundle = (
                Brick {
                    brick_type: BrickType::Regular,
                },
                SpriteBundle {
                    texture: brick_sprites[y].clone(),
                    transform: Transform::from_xyz(
                        x as f32 * BRICK_WIDTH - 4.5 * BRICK_WIDTH,
                        WIN_HEIGHT / 2. - y as f32 * BRICK_HEIGHT - 100.,
                        10.,
                    ),
                    ..default()
                },
                Collider {
                    size: Vec2::new(BRICK_WIDTH, BRICK_HEIGHT),
                },
            );

            if rand::random::<f32>() < FIREBALL_CHANCE {
                bundle.0.brick_type = BrickType::Fireball;
                let parent = commands.spawn(bundle).id();
                let child = commands
                    .spawn(SpriteBundle {
                        texture: assets.image.ball_fire.clone(),
                        transform: Transform::from_xyz(0., 0., 10.),
                        ..default()
                    })
                    .id();

                commands.entity(parent).push_children(&[child]);
            } else {
                commands.spawn(bundle);
            }
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
            transform: Transform::from_xyz(0., -280., 10.),
            texture: assets.image.paddle.clone(),
            ..default()
        })
        .insert(Collider {
            size: Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT),
        });
}
fn play_sounds(
    mut collision_events: EventReader<BallCollisionEvent>,
    mut brick_destruction_events: EventReader<BrickDesctructionEvent>,
    audio: Res<Audio>,
    assets: Res<GameAssets>,
) {
    for _ in brick_destruction_events.iter() {
        audio.play_with_settings(
            assets.audio.drop_004.clone(),
            PlaybackSettings {
                repeat: false,
                volume: 1.,
                speed: rand::random::<f32>() * 0.4 + 0.8,
            },
        );
    }

    for event in collision_events.iter() {
        match event.0 {
            BallCollisionType::Paddle => {
                audio.play(assets.audio.drop_002.clone());
            }
            BallCollisionType::Wall => {
                audio.play_with_settings(
                    assets.audio.drop_003.clone(),
                    PlaybackSettings {
                        volume: 0.3,
                        ..default()
                    },
                );
            }
        }
    }
}

fn update_score(
    mut destruction_events: EventReader<BrickDesctructionEvent>,
    mut score_events: EventWriter<ScoreIncrementEvent>,
    mut player_progress: ResMut<PlayerProgress>,
    mut timer: ResMut<ScoreIncrementTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    for _ in destruction_events.iter() {
        let mut score_increment = BASE_BRICK_SCORE;

        if !timer.0.finished() {
            let bonus = (SCORE_MULTIPLIER_TIMEOUT - timer.0.elapsed_secs()) * SCORE_MULTIPLIER;
            score_increment += bonus;
            player_progress.bonus_score += bonus;
        }
        score_events.send(ScoreIncrementEvent(score_increment));
        player_progress.score += score_increment;
        timer.0.reset();
    }
}

fn on_all_balls_lost(
    mut commands: Commands,
    mut ball_loss_events: EventReader<AllBallsLostEvent>,
    mut player_progress: ResMut<PlayerProgress>,
    mut state: ResMut<State<GameState>>,
) {
    for _ in ball_loss_events.iter() {
        if player_progress.extra_balls_remaining > 0 {
            player_progress.extra_balls_remaining -= 1;
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

fn next_level(
    query: Query<&Brick>,
    mut state: ResMut<State<GameState>>,
    mut progress: ResMut<PlayerProgress>,
) {
    if query.is_empty() {
        progress.level += 1;
        progress.extra_balls_remaining = EXTRA_BALL_COUNT;
        state.set(GameState::LevelCompleted).unwrap();
    }
}

fn on_window_focus(
    mut window_focused: EventReader<WindowFocused>,
    mut pause_event: EventWriter<GamePauseEvent>,
) {
    for window in window_focused.iter() {
        if !window.focused {
            pause_event.send(GamePauseEvent { should_pause: true });
        }
    }
}

fn on_pause(mut pause_event: EventReader<GamePauseEvent>, mut state: ResMut<State<GameState>>) {
    if state.current() != &GameState::GameOver {
        for e in pause_event.iter() {
            if e.should_pause && state.current() != &GameState::Paused {
                state.overwrite_push(GameState::Paused).unwrap();
            } else if !e.should_pause && state.current() == &GameState::Paused {
                state.overwrite_pop().unwrap();
            }
        }
    }
}

fn handle_brick_destruction(
    mut commands: Commands,
    mut events: EventReader<BrickDesctructionEvent>,
    assets: Res<GameAssets>,
) {
    for event in events.iter() {
        if event.brick_type == BrickType::Fireball {
            commands.spawn((
                Ball {
                    direction: Vec2::new(rand::random::<f32>(), rand::random::<f32>()),
                    speed: 400.,
                    curve: 0.,
                    ball_type: BallType::FireBall,
                },
                SpriteBundle {
                    texture: assets.image.ball_fire.clone(),
                    transform: Transform::from_translation(event.position),
                    ..default()
                },
                Collider {
                    size: Vec2::splat(BALL_SIZE),
                },
                FireBall { age: 0. },
            ));
        }
    }
}

fn expire_fireballs(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FireBall)>,
) {
    for (entity, mut fireball) in query.iter_mut() {
        fireball.age += time.delta_seconds();

        if fireball.age > MAX_FIREBALL_AGE {
            commands.entity(entity).despawn();
        }
    }
}
