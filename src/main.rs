use std::time::Duration;

use assets::*;
use ball::*;
use bevy::{
    prelude::*,
    sprite::collide_aabb::*,
    window::{WindowFocused, WindowResolution},
};
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

pub struct BrickDesctructionEvent {
    position: Vec3,
    brick_type: BrickType,
}

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

#[derive(PartialEq, Clone)]
pub enum BrickType {
    Regular,
    Fireball,
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

#[derive(States, Hash, Clone, PartialEq, Eq, Debug, Default)]
enum GameState {
    #[default]
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

    app.add_state::<GameState>();

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
        .insert_resource(ClearColor(BG_COLOR));

    // Plugins
    app.add_plugin(GameAssetsPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(UiPlugin)
        .add_plugin(BallPlugin)
        .add_plugin(GameInputPlugin);

    // Events
    app.add_event::<GamePauseEvent>()
        .add_event::<BrickDesctructionEvent>()
        .add_event::<ScoreIncrementEvent>();

    // State independent systems
    app.add_startup_system(spawn_camera)
        .add_startup_system(configure_window)
        .add_system(on_window_focus)
        // .add_system(on_pause)
        .add_startup_system(play_music)
        .add_system(play_sounds);

    // Playing state
    app.add_systems(
        (reset_bonus_score, spawn_bricks, spawn_paddle, spawn_ball)
            .in_schedule(OnEnter(GameState::Playing)),
    )
    .add_systems(
        (on_all_balls_lost, next_level, handle_powerups, update_score)
            .in_set(OnUpdate(GameState::Playing)),
    )
    .add_systems(
        (
            despawn::<Ball>,
            despawn::<UiBall>,
            despawn::<Paddle>,
            despawn::<Brick>,
        )
            .in_schedule(OnExit(GameState::Playing)),
    );

    // Level transition state
    app.add_system(transition_timer.in_set(OnUpdate(GameState::LevelCompleted)));

    // GameOver state
    app.add_systems((
        transition_timer.in_set(OnUpdate(GameState::GameOver)),
        reset_player_progress.in_schedule(OnExit(GameState::GameOver)),
    ));

    app.run();
}

fn configure_window(mut query: Query<&mut Window>) {
    if let Ok(mut window) = query.get_single_mut() {
        window.resolution = WindowResolution::new(WIN_WIDTH, WIN_HEIGHT);
        window.title = "Breakout!".to_string();
        window.resizable = false;
    }
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
    state: ResMut<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        timer.0.reset();
        if state.0 == GameState::GameOver {
            *next_state = NextState(Some(GameState::Start));
        } else if state.0 == GameState::LevelCompleted {
            *next_state = NextState(Some(GameState::Playing));
        }
    }
}

fn reset_player_progress(mut player_progress: ResMut<PlayerProgress>) {
    *player_progress = PlayerProgress::default();
}

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
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _ in ball_loss_events.iter() {
        if player_progress.extra_balls_remaining > 0 {
            player_progress.extra_balls_remaining -= 1;
            commands.add(SpawnBallCommand);
        } else {
            *next_state = NextState(Some(GameState::GameOver));
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
    mut progress: ResMut<PlayerProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if query.is_empty() {
        progress.level += 1;
        progress.extra_balls_remaining = EXTRA_BALL_COUNT;
        *next_state = NextState(Some(GameState::LevelCompleted));
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

fn on_pause(
    mut pause_event: EventReader<GamePauseEvent>,
    state: ResMut<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if state.0 != GameState::GameOver {
        for e in pause_event.iter() {
            if e.should_pause && state.0 != GameState::Paused {
                *next_state = NextState(Some(GameState::Paused));
            } else if !e.should_pause && state.0 == GameState::Paused {
                *next_state = NextState(Some(GameState::Playing));
            }
        }
    }
}

fn handle_powerups(
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
