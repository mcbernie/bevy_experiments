use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::ShaderType}};


#[derive(Resource, Clone, ExtractResource, ShaderType)]
pub struct SimulationParams {
    pub particle_count: u32,
    pub gravity: f32,
    pub delta_time: f32,
}