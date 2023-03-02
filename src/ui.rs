use crate::*;

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

pub fn spawn_title_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "Breakout!",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 120.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            ..default()
        })
        .insert(Transform::from_xyz(0., 0., 1.))
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
    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "Game over",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 120.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            ..default()
        })
        .insert(Transform::from_xyz(0., 0., 1.));
}

pub fn spawn_level_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_sections([
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
        ])
        .with_style(Style {
            margin: UiRect {
                left: Val::Px(10.),
                top: Val::Px(10.),
                ..default()
            },
            ..default()
        }),
        LevelText,
    ));
}

pub fn spawn_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
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
        ])
        .with_style(Style {
            // position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            margin: UiRect {
                // left: Val::Px(10.),
                top: Val::Px(10.),
                ..default()
            },
            ..default()
        }),
        ScoreText,
    ));
}

pub fn update_score_text(
    mut query: Query<&mut Text, With<ScoreText>>,
    player_progress: Res<PlayerProgress>,
) {
    for mut text in &mut query {
        // Update the value of the second section
        text.sections[1].value = player_progress.score.to_string();
    }
}

pub fn update_level_text(
    mut query: Query<&mut Text, With<LevelText>>,
    player_progress: Res<PlayerProgress>,
) {
    for mut text in &mut query {
        // Update the value of the second section
        text.sections[1].value = player_progress.level.to_string();
    }
}

pub fn spawn_ball_count(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_progress: Res<PlayerProgress>,
) {
    const MARGIN: f32 = 20.;

    for i in 0..player_progress.balls_remaining {
        let x = WIN_WIDTH / 2. - i as f32 * 40. - MARGIN;
        commands
            .spawn(SpriteBundle {
                texture: assets.image.ball.clone(),
                transform: Transform::from_xyz(x, WIN_HEIGHT / 2. - MARGIN, 0.),
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
        visibility.is_visible = i <= player_progress.balls_remaining;
        i += 1;
    }
}
