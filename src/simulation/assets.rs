use bevy::{prelude::*, render::{extract_resource::ExtractResource, render_resource::ShaderType}};
use bevy_inspector_egui::prelude::*;


#[derive(Reflect, Resource, InspectorOptions, ExtractResource, ShaderType, Clone)]
#[reflect(Resource, InspectorOptions)]
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