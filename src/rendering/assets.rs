use bevy::prelude::*;
use bevy::render::render_resource::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewData {
    pub clip_from_world: [[f32; 4]; 4],
    pub world_from_view: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelData {
    pub model: [[f32; 4]; 4],
}

#[derive(Resource)]
pub struct MarchingCubesRenderResources {
    pub pipeline: CachedRenderPipelineId,   
    pub view_buffer: Buffer,
    pub model_buffer: Buffer,
    pub model_view_bind_group: BindGroup,
}

#[derive(Component)]
pub struct FluidIndirectArgsBuffer {
    pub buffer: Buffer,
}

#[derive(Resource)]
pub struct RenderArgsPipeline {
    pub pipeline: CachedComputePipelineId,
    pub layout: BindGroupLayoutDescriptor,
}

#[derive(Component)]
pub struct PreparedRenderArgsBindGroup {
    pub bind_group: BindGroup,
}


#[derive(Resource)]
pub struct SimRenderPipeline {
    pub layout: BindGroupLayoutDescriptor,
    pub model_view_layout: BindGroupLayoutDescriptor,
}