use bevy::prelude::*;
use bevy::render::render_resource::*;

#[derive(Resource)]
pub struct MarchingCubesGpu {
    pub triangle_buffer: Buffer,
    pub triangle_count: Buffer,
    pub indirect_args: Buffer,

    pub marching_pipeline: CachedComputePipelineId,
    pub args_pipeline: CachedComputePipelineId,

    pub bind_group: BindGroup,
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
