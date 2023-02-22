use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use crate::{Paddle, PADDLE_WIDTH, WIN_WIDTH};

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(mouse_motion).add_system(cursor_grab_system);
    }
}

fn mouse_motion(
    mut motion_evr: EventReader<MouseMotion>,
    mut q: Query<&mut Transform, With<Paddle>>,
) {
    for ev in motion_evr.iter() {
        let mut transform = q.single_mut();
        transform.translation.x += ev.delta.x;

        if transform.translation.x < -WIN_WIDTH / 2. + PADDLE_WIDTH / 2. {
            transform.translation.x = -WIN_WIDTH / 2. + PADDLE_WIDTH / 2.;
        } else if transform.translation.x > WIN_WIDTH / 2. - PADDLE_WIDTH / 2. {
            transform.translation.x = WIN_WIDTH / 2. - PADDLE_WIDTH / 2.;
        }
    }
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        // if you want to use the cursor, but not let it leave the window,
        // use `Confined` mode:
        window.set_cursor_grab_mode(CursorGrabMode::Confined);

        // for a game that doesn't use the cursor (like a shooter):
        // use `Locked` mode to keep the cursor in one place
        // window.set_cursor_grab_mode(CursorGrabMode::Locked);
        // also hide the cursor
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);
    }
}
