use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use crate::*;

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Start).with_system(click_to_start))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(paddle_motion)
                    .with_system(launch_ball),
            )
            .add_system(window_focus);
    }
}

fn paddle_motion(
    mut motion_evr: EventReader<MouseMotion>,
    mut q: Query<(&mut Transform, &mut Paddle)>,
) {
    for ev in motion_evr.iter() {
        let (mut transform, mut paddle) = q.single_mut();
        transform.translation.x += ev.delta.x;

        paddle.speed = ev.delta.x;

        transform.translation.x = transform.translation.x.clamp(
            -WIN_WIDTH / 2. + PADDLE_WIDTH / 2.,
            WIN_WIDTH / 2. - PADDLE_WIDTH / 2.,
        );
    }
}

fn click_to_start(btn: Res<Input<MouseButton>>, mut state: ResMut<State<GameState>>) {
    if btn.just_pressed(MouseButton::Left) {
        state.set(GameState::Playing).unwrap();
    }
}

fn window_focus(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut pause_event: EventWriter<GamePauseEvent>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_grab_mode(CursorGrabMode::Confined);
        window.set_cursor_visibility(false);

        pause_event.send(GamePauseEvent {
            should_pause: false,
        })
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);

        pause_event.send(GamePauseEvent { should_pause: true })
    }
}

fn launch_ball(
    mut commands: Commands,
    query: Query<Entity, With<AttachedToPaddle>>,
    kb: Res<Input<KeyCode>>,
) {
    if kb.just_pressed(KeyCode::Space) {
        if let Ok(entity) = query.get_single() {
            commands.entity(entity).remove::<AttachedToPaddle>();
        }
    }
}
