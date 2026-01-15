use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::{
    render_resource::ShaderType
};
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;

#[derive(Reflect, Component, ExtractComponent, ShaderType, Clone, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct SimulationParams {
    #[inspector(min = 1.0, max = 20.0, display = NumberDisplay::Slider)]
    pub box_size: f32,
    pub gravity: f32,
    pub particle_radius: f32,
    pub cell_size: f32,
    #[reflect(ignore)]
    pub _pad: f32,
}

impl Default for SimulationParams {
    fn default() -> Self {
        SimulationParams {
            box_size: 3.0,
            gravity: -9.81,
            particle_radius: 0.05,
            cell_size: 0.1,
            _pad: 0.0,
        }
    }
}