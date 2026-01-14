use bevy::prelude::*;

#[derive(Component)]
pub struct FlyCam {
    pub speed: f32,
    pub sensitivity: f32,
}

#[derive(Component)]
pub struct MouseLockedToFlyCam;