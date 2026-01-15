use bevy::prelude::*; 
use bevy::render::{
    extract_resource::ExtractResource, 
    render_resource::ShaderType
};
use bevy_inspector_egui::prelude::*;

#[derive(Reflect, Resource, ExtractResource, ShaderType, Clone, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct SimulationParams {
    pub box_size: f32,
    pub gravity: f32,
    pub cell_size: f32,
    pub _pad: f32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        SimulationParams {
            box_size: 3.0,
            gravity: -9.81,
            cell_size: 0.1,
            _pad: 0.0,
        }
    }
}