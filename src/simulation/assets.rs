use std::f32::consts::PI;

use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::{
    render_resource::ShaderType
};
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::prelude::*;


#[derive(Reflect, Component, ExtractComponent, ShaderType, Clone, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct SimulationParams {
    pub particle_count: u32,
    pub gravity: f32,
    pub smoothing_radius: f32,
    pub collision_damping: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub near_pressure_multiplier: f32,
    pub viscosity_strength: f32,
    pub bounds_size: Vec3,
    pub k_spiky_pow2 : f32,
    pub k_spiky_pow3 : f32,
    pub k_spiky_pow2_grad: f32,
    pub k_spiky_pow3_grad: f32,
}

impl SimulationParams {
    pub fn update_derived_constants(&mut self) {
        self.k_spiky_pow2 = 15.0 / (PI * f32::powf(self.smoothing_radius, 5.0));
        self.k_spiky_pow3 = 15.0 / (PI * f32::powf(self.smoothing_radius, 6.0));
        self.k_spiky_pow2_grad = 15.0 / (PI * f32::powf(self.smoothing_radius, 5.0));
        self.k_spiky_pow3_grad = 45.0 / (PI * f32::powf(self.smoothing_radius, 6.0));
    }
}

impl Default for SimulationParams {
    fn default() -> Self {
        let smoothing_radius = 0.2;
        SimulationParams {
            particle_count: 100,
            gravity: -10.0,
            smoothing_radius: smoothing_radius.clone(),
            collision_damping: 0.95,
            target_density: 630.0,
            pressure_multiplier: 288.0,
            near_pressure_multiplier: 2.25,
            viscosity_strength: 0.001,
            bounds_size: Vec3::new(10.0, 8.0, 4.0),
            k_spiky_pow2: 15.0 / (PI * f32::powf(smoothing_radius, 5.0)),
            k_spiky_pow3: 15.0 / (PI * f32::powf(smoothing_radius, 6.0)),
            k_spiky_pow2_grad: 15.0 / (PI * f32::powf(smoothing_radius, 5.0)),
            k_spiky_pow3_grad: 45.0 / (PI * f32::powf(smoothing_radius, 6.0)),
        }
    }
}

pub fn simulation_params_ui_systems(mut contexts: EguiContexts, mut simulations: Query<&mut SimulationParams>) {
    use bevy_inspector_egui::egui;

    for mut params in simulations.iter_mut() {
        egui::Window::new("Simulation Parameters").show(contexts.ctx_mut().unwrap(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut params.particle_count.to_string()).desired_width(100.0).hint_text("Particle Count"));
            ui.add(egui::Slider::new(&mut params.gravity, -10.0..=5.0).text("Gravity"));
            ui.add(egui::Slider::new(&mut params.smoothing_radius, 0.01..=0.5).text("Smoothing Radius"));
            ui.add(egui::Slider::new(&mut params.collision_damping, 0.0..=10.0).text("Collision Damping"));
            ui.add(egui::Slider::new(&mut params.target_density, 0.0..=1000.0).text("Target Density"));
            ui.add(egui::Slider::new(&mut params.pressure_multiplier, 0.0..=500.0).text("Pressure Multiplier"));
            ui.add(egui::Slider::new(&mut params.near_pressure_multiplier, 0.0..=5.0).text("Near Pressure Multiplier"));

            ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            ui.label("Bounding: ");
            ui.add(egui::DragValue::new(&mut params.bounds_size.x).speed(0.1).prefix("X: "));
            ui.add(egui::DragValue::new(&mut params.bounds_size.y).speed(0.1).prefix("Y: "));
            ui.add(egui::DragValue::new(&mut params.bounds_size.z).speed(0.1).prefix("Z: "));
        });
    }
}