use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use crate::*;

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(mouse_motion)
                .with_system(launch_ball),
        )
        .add_system(cursor_grab)
        .add_system(keyboard_input);
    }
}

fn keyboard_input(kb: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if kb.just_pressed(KeyCode::Escape) && state.current() != &GameState::PreGame {
        state.set(GameState::PreGame).unwrap();
    }

    if kb.just_pressed(KeyCode::Space) && state.current() == &GameState::PreGame {
        state.set(GameState::InGame).unwrap();
    }
}

fn mouse_motion(
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

fn cursor_grab(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        // if you want to use the cursor, but not let it leave the window,
        // use `Confined` mode:
        // window.set_cursor_grab_mode(CursorGrabMode::Confined);

        // for a game that doesn't use the cursor (like a shooter):
        // use `Locked` mode to keep the cursor in one place
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
        // also hide the cursor
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);
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
