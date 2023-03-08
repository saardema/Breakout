use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

use crate::*;

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(click_to_start.in_set(OnUpdate(GameState::Start)))
            .add_systems((paddle_motion, launch_ball).in_set(OnUpdate(GameState::Playing)))
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

fn click_to_start(btn: Res<Input<MouseButton>>, mut next_state: ResMut<NextState<GameState>>) {
    if btn.just_pressed(MouseButton::Left) {
        *next_state = NextState(Some(GameState::Playing));
    }
}

fn window_focus(
    mut windows: Query<&mut Window>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
    mut pause_event: EventWriter<GamePauseEvent>,
) {
    for mut window in windows.iter_mut() {
        if btn.just_pressed(MouseButton::Left) {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;

            pause_event.send(GamePauseEvent {
                should_pause: false,
            })
        }

        if key.just_pressed(KeyCode::Escape) {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;

            pause_event.send(GamePauseEvent { should_pause: true })
        }
    }
}

fn launch_ball(
    mut commands: Commands,
    query: Query<Entity, With<AttachedToPaddle>>,
    kb: Res<Input<KeyCode>>,
) {
    if kb.just_pressed(KeyCode::Space) {
        for entity in &query {
            commands.entity(entity).remove::<AttachedToPaddle>();
        }
        // if let Ok(entity) = query.get_single() {
        //     commands.entity(entity).remove::<AttachedToPaddle>();
        // }
    }
}
