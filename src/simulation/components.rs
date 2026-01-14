use bevy::prelude::*; 
use bevy::render::extract_component::ExtractComponent;
use bevy::render::{
    render_resource::BindGroup, 
    storage::ShaderStorageBuffer, 
};

#[derive(Component, ExtractComponent, Clone)]
pub struct SimulationBuffers {
    pub positions: Handle<ShaderStorageBuffer>,
    pub velocities: Handle<ShaderStorageBuffer>,
    pub spatial_keys: Handle<ShaderStorageBuffer>,
}

// lebt nur in der RenderApp
#[derive(Component)]
pub struct PreparedSimulationBindGroup {
    pub bind_group: BindGroup,
}


