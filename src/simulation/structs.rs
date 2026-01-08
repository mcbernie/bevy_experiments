use bevy::prelude::*;
use bevy::render::render_resource::{BindGroup, Buffer, CachedComputePipelineId, ShaderType};
use bevy::render::storage::ShaderStorageBuffer;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
pub struct ParticlePosition {
    pub pos: [f32; 3],
    pub _pad: f32,
}

#[derive(Resource)]
pub struct ComputePipelineState {
    pub pipeline_id: CachedComputePipelineId,
}

#[derive(Resource)]
pub struct ComputeBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct ComputeBuffers {
    pub positions: Buffer,
    pub velocities: Buffer,
    pub params: Buffer,
}


// Uniforms müssen immer 16-Byte-aligned sein.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimParams {
    pub num_particles: u32,
    pub gravity: f32,
    pub delta_time: f32,
    pub _pad: f32, // <-- WICHTIG (Alignment!)
}


#[derive(Resource)]
pub struct ReadbackBuffer {
    pub buffer: Buffer,
    pub size: u64,
}

#[derive(Resource)]
pub struct SharedComputeBuffers {
    pub positions: Handle<ShaderStorageBuffer>,
    //pub velocities: Handle<ShaderStorageBuffer>,
}