use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::Buffer;
use bevy::render::{
    render_resource::BindGroup, 
    storage::ShaderStorageBuffer, 
};

#[derive(Component, ExtractComponent, Clone)]
pub struct SimulationBuffers {
    pub positions: [Handle<ShaderStorageBuffer>; 2],
    pub velocities: [Handle<ShaderStorageBuffer>; 2],
    pub spatial_keys: Handle<ShaderStorageBuffer>,
    pub active_index: u32, // 0 oder 1 für double buffering
}

#[derive(Component, ExtractComponent, Clone)]
pub struct AdvancedSimulationBuffers {
    pub position_sorted: Buffer,
    pub velocities_sorted: Buffer,
    pub spatial_sort_counts: Buffer,
    pub spatial_sort_offsets: Buffer,
    pub spatial_sorted_indices: Buffer,
    pub write_offsets: Buffer,
}

// lebt nur in der RenderApp
#[derive(Component)]
pub struct PreparedSimulationBindGroup {
    pub bind_group: BindGroup,
}


