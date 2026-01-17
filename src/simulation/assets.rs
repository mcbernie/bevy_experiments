use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::{
    render_resource::ShaderType
};
use bevy_inspector_egui::prelude::*;

use crate::PARTICLE_COUNT;

#[derive(Reflect, Component, ExtractComponent, ShaderType, Clone, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct SimulationParams {
    pub particle_count: u32,
    pub gravity: f32,
    pub smoothing_radius: f32,
    pub collision_damping: f32,
    pub bounds_size: Vec3,
}

impl Default for SimulationParams {
    fn default() -> Self {
        SimulationParams {
            particle_count: PARTICLE_COUNT,
            gravity: -0.81,
            smoothing_radius: 0.1,
            collision_damping: 0.5,
            bounds_size: Vec3::splat(2.0),
        }
    }
}