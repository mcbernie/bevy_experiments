use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::{
    render_resource::ShaderType
};
use bevy_inspector_egui::bevy_egui::EguiContexts;
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

pub fn simulation_params_ui_systems(mut contexts: EguiContexts, mut simulations: Query<&mut SimulationParams>) {
    use bevy_inspector_egui::egui;

    for mut params in simulations.iter_mut() {
        egui::Window::new("Simulation Parameters").show(contexts.ctx_mut().unwrap(), |ui| {
            ui.add(egui::Slider::new(&mut params.gravity, -5.0..=5.0).text("Gravity"));
            ui.add(egui::Slider::new(&mut params.smoothing_radius, 0.01..=0.5).text("Smoothing Radius"));
            ui.add(egui::Slider::new(&mut params.collision_damping, 0.0..=1.0).text("Collision Damping"));

            ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            ui.label("Bounding: ");
            ui.add(egui::DragValue::new(&mut params.bounds_size.x).speed(0.1).prefix("X: "));
            ui.add(egui::DragValue::new(&mut params.bounds_size.y).speed(0.1).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut params.bounds_size.z).speed(0.1).prefix("Z: "));
        });
    }
}