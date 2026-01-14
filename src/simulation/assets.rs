use bevy::prelude::*; 
use bevy::render::{
    extract_resource::ExtractResource, 
    render_resource::ShaderType
};

#[derive(Reflect, Resource, ExtractResource, ShaderType, Clone)]
#[reflect(Resource)]
pub struct SimulationParams {
    pub box_size: f32,
    pub gravity: f32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        SimulationParams {
            box_size: 3.0,
            gravity: -9.81,
        }
    }
}