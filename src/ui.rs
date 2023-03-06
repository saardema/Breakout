use crate::*;

const UI_Z_VALUE: f32 = 100.;
const BG_ANIM_SPEED: f32 = 0.1;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct PlayText;

#[derive(Component)]
pub struct TitleText;

#[derive(Component)]
pub struct UiBall;

#[derive(Component)]
pub struct ScoreValueText;

#[derive(Component)]
pub struct Background;

pub struct UiPlugin;

#[derive(Resource)]
pub struct BackgroundAnimationDirection(pub bool);

#[derive(Resource)]
pub struct ScoreAnimationTimer(Timer);

pub fn spawn_background(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::Rgba {
                    red: 0.2,
                    green: 0.5,
                    blue: 0.7,
                    alpha: 0.9,
                },
                ..default()
            },
            texture: assets.image.background.clone(),
            transform: Transform::from_xyz(1000., -1000., 0.),
            ..default()
        },
        Background,
    ));
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScoreAnimationTimer(Timer::new(
            Duration::from_secs_f32(SCORE_ANIM_MAX_DURATION),
            TimerMode::Once,
        )));
    }
}

pub fn animate_background(
    mut query: Query<&mut Transform, With<Background>>,
    mut direction: ResMut<BackgroundAnimationDirection>,
) {
    let mut transform = query.single_mut();

    if transform.translation.x >= 1000. {
        direction.0 = false
    } else if transform.translation.x <= -1000. {
        direction.0 = true
    }

    transform.translation.x += if direction.0 {
        BG_ANIM_SPEED
    } else {
        -BG_ANIM_SPEED
    };
}

pub fn spawn_title_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "Breakout!",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 150.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            ..default()
        })
        .insert(Transform::from_xyz(0., 120., UI_Z_VALUE))
        .insert(TitleText);
}

pub fn spawn_play_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "Click to play",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            ..default()
        })
        .insert(Transform::from_xyz(0., -200., 1.))
        .insert(PlayText);
}

pub fn spawn_game_over_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Game over",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 120.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., 0., UI_Z_VALUE),
        ..default()
    });
}

pub fn spawn_level_complete_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Text2dBundle {
        text: Text::from_section(
            "Level complete",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 120.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., 0., UI_Z_VALUE),
        ..default()
    });
}
pub fn spawn_bonus_score_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_progress: Res<PlayerProgress>,
) {
    commands.spawn(Text2dBundle {
        text: Text::from_sections([
            TextSection::new(
                "Bonus: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                player_progress.bonus_score.round().to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 60.0,
                    color: Color::GOLD,
                },
            ),
        ])
        .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_xyz(0., -80., UI_Z_VALUE),
        ..default()
    });
}

pub fn spawn_level_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    "Level: ",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 60.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 60.0,
                    color: Color::GOLD,
                }),
            ]),
            transform: Transform::from_xyz(-5. * BRICK_WIDTH, WIN_HEIGHT / 2. - 10., UI_Z_VALUE),
            ..default()
        },
        LevelText,
    ));
}

pub fn spawn_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_sections([TextSection::new(
                "Score: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            )])
            .with_alignment(TextAlignment::TOP_LEFT),
            transform: Transform::from_xyz(-90., WIN_HEIGHT / 2. - 10., UI_Z_VALUE),
            ..default()
        },
        ScoreText,
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_sections([TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 60.0,
                color: Color::GOLD,
            })])
            .with_alignment(TextAlignment::TOP_LEFT),
            transform: Transform::from_xyz(65., WIN_HEIGHT / 2. - 10., UI_Z_VALUE),
            ..default()
        },
        ScoreText,
        ScoreValueText,
    ));
}

pub fn update_score_text(
    mut query: Query<(&mut Text, &mut Transform), With<ScoreValueText>>,
    player_progress: Res<PlayerProgress>,
    mut timer: ResMut<ScoreAnimationTimer>,
    mut score_increment_events: EventReader<ScoreIncrementEvent>,
    time: Res<Time>,
) {
    let mut boost = 0.;

    timer.0.tick(time.delta());

    if !timer.0.finished() {
        boost = (timer.0.duration().as_secs_f32() - timer.0.elapsed_secs())
            / timer.0.duration().as_secs_f32();
        boost *= boost;
    }

    for score_increment in score_increment_events.iter() {
        if score_increment.0 > 10. {
            timer.0.reset();
            timer.0.set_duration(Duration::from_secs_f32(
                SCORE_ANIM_MAX_DURATION * score_increment.0.min(70.) / 70.,
            ));
        }
    }

    for (mut text, mut transform) in &mut query {
        if player_progress.is_changed() {
            text.sections[0].value = player_progress.score.round().to_string();
        }

        if boost > 0. {
            transform.scale = Vec3::splat(1. + 0.8 * boost);
        }
    }
}

pub fn update_level_text(
    mut query: Query<&mut Text, With<LevelText>>,
    player_progress: Res<PlayerProgress>,
) {
    for mut text in &mut query {
        text.sections[1].value = player_progress.level.to_string();
    }
}

pub fn spawn_ball_count(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_progress: Res<PlayerProgress>,
) {
    for i in 0..player_progress.extra_balls_remaining {
        let x = 5. * BRICK_WIDTH - i as f32 * 30. - BALL_SIZE / 2.;

        commands
            .spawn(SpriteBundle {
                texture: assets.image.ball.clone(),
                transform: Transform::from_xyz(x, WIN_HEIGHT / 2. - 40., UI_Z_VALUE),
                ..default()
            })
            .insert(UiBall);
    }
}

pub fn update_ball_count(
    mut query: Query<&mut Visibility, With<UiBall>>,
    player_progress: Res<PlayerProgress>,
) {
    let mut i = 1;

    for mut visibility in query.iter_mut() {
        visibility.is_visible = i <= player_progress.extra_balls_remaining;
        i += 1;
    }
}
