use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::{Buffer, UniformBuffer};
use bevy::render::{
    render_resource::BindGroup, 
    storage::ShaderStorageBuffer, 
};

use crate::simulation::assets::SimulationParams;

// created inside main app, used in render app
// connection between simulation and rendering
#[derive(Component, ExtractComponent, Clone)]
pub struct SimulationBuffers {
    pub positions: Handle<ShaderStorageBuffer>,
    pub velocities: Handle<ShaderStorageBuffer>,
    pub debug_buffer: Handle<ShaderStorageBuffer>,
    pub active_index: u32, // 0 oder 1 für double buffering
}

#[derive(Component, Clone)]
pub struct InternalSimulationBuffers {
    pub predicted_positions: Buffer,
    pub spatial_keys: Buffer,
    pub spatial_offsets: Buffer,
    pub spatial_indices: Buffer,
    pub sorted_indices: Buffer,
    pub sort_target_position: Buffer,
    pub sort_target_predicted_positions: Buffer,
    pub sort_target_velocity: Buffer,
}

#[derive(Component, ExtractComponent, Clone)]
pub struct AdvancedSimulationBuffers {
    pub position_sorted: Buffer,
    pub velocities_sorted: Buffer,
    pub spatial_sort_counts: Buffer,
    pub spatial_sort_offsets: Buffer,
    pub spatial_sorted_indices: Buffer,
    pub write_offsets: Buffer,

    pub spatial_keys: Buffer,
    pub spatial_indices: Buffer,
    pub spatial_offsets: Buffer
}

// lebt nur in der RenderApp
#[derive(Component)]
pub struct PreparedSimulationBindGroup {
    pub bind_group: BindGroup,

    pub write_back_bind_group: BindGroup,
}

// lebt nur in der RenderApp
#[derive(Component)]
pub struct SimulationUniform (
    pub UniformBuffer<SimulationParams>
);


