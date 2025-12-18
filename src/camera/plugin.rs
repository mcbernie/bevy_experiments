use bevy::{input::mouse::MouseMotion, prelude::*};
use crate::app_state::AppState;

use super::components::FlyCam;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_camera)
            .add_systems(Update, (flycam_look, flycam_move).run_if(in_state(AppState::InGame)));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(), 
        Transform::from_xyz(0.0, 10.0, 20.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam {
            speed: 15.0,
            sensitivity: 0.002,
        },
    ));
}

fn flycam_look(
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let mut delta = Vec2::ZERO;
    for ev in mouse_motion_events.read() {
        delta += ev.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    for (cam, mut transform) in &mut query {
        let yaw = Quat::from_rotation_y(-delta.x * cam.sensitivity);
        let pitch = Quat::from_rotation_x(-delta.y * cam.sensitivity);

        transform.rotation = yaw * transform.rotation;
        transform.rotation = transform.rotation * pitch;
    }
}

fn flycam_move(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    for (cam, mut transform) in &mut query {
        let mut dir = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            dir += transform.forward().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir -= transform.forward().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir -= transform.right().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir += transform.right().as_vec3();
        }
        if keyboard.pressed(KeyCode::Space) {
            dir += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ControlLeft) {
            dir -= Vec3::Y;
        }

        let mut speed = cam.speed;
        if keyboard.pressed(KeyCode::ShiftLeft) {
            speed *= 3.0;
        }

        if dir != Vec3::ZERO {
            transform.translation += dir.normalize() * speed * time.delta_secs();
        }
    }
}