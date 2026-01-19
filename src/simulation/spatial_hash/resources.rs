use bevy::{prelude::*, render::render_resource::{BindGroupLayoutDescriptor, CachedComputePipelineId}};

#[derive(Resource)]
pub struct SpatialHashComputePipeline {
    pub initialize_offsets: CachedComputePipelineId,
    pub calculate_offsets: CachedComputePipelineId,
    pub layout: BindGroupLayoutDescriptor,
}