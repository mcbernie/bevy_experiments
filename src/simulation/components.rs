use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::Buffer;
use bevy::render::{
    render_resource::BindGroup, 
    storage::ShaderStorageBuffer, 
};

#[derive(Component, ExtractComponent, Clone)]
pub struct SimulationBuffers {
    pub positions: Handle<ShaderStorageBuffer>,
    pub velocities: Handle<ShaderStorageBuffer>,
}

#[derive(Component, ExtractComponent, Clone)]
pub struct AdvancedSimulationBuffers {
    pub spatial_keys: Buffer,
    pub spatial_sort_counts: Buffer,
    pub spatial_sort_offsets: Buffer,
}

// lebt nur in der RenderApp
#[derive(Component)]
pub struct PreparedSimulationBindGroup {
    pub bind_group: BindGroup,
}


