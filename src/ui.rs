use crate::*;

pub fn despawn_text(mut commands: Commands, query: Query<(Entity, &Text)>) {
    for (entity, _) in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
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
            margin: UiRect {
                left: Val::Px(10.),
                top: Val::Px(10.),
                ..default()
            },
            ..default()
        }),
        ScoreText,
    ));
}

pub fn text_update_system(
    mut query: Query<&mut Text, With<ScoreText>>,
    player_progress: Res<PlayerProgress>,
) {
    for mut text in &mut query {
        // Update the value of the second section
        text.sections[1].value = player_progress.score.to_string();
    }
}

#[derive(Component)]
pub struct UiBall;

pub fn spawn_ball_count(
    mut commands: Commands,
    assets: Res<GameAssets>,
    player_progress: Res<PlayerProgress>,
) {
    for i in 0..player_progress.ball_count {
        let x = WIN_WIDTH / 2. - i as f32 * 40. - 20.;
        commands
            .spawn(SpriteBundle {
                texture: assets.image.ball.clone(),
                transform: Transform::from_xyz(x, WIN_HEIGHT / 2. - 20., 0.),
                ..default()
            })
            .insert(UiBall);
    }
}

pub fn despawn_ball_count(mut commands: Commands, entities: Query<Entity, With<UiBall>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
