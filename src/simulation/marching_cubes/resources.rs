use bevy::{prelude::*, render::render_resource::*};

#[derive(Resource)]
pub struct MarchingCubesLut {
    pub buffer: Buffer,
    pub len: u32,
}

#[derive(Resource)]
pub struct MarchingCubesPipeline {
    pub pipeline_id: CachedComputePipelineId,
    pub bind_group_layout: BindGroupLayoutDescriptor,
}