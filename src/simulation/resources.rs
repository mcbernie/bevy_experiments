use bevy::{prelude::*, render::render_resource::{BindGroupLayoutDescriptor, CachedComputePipelineId}};

#[derive(Resource)]
pub struct SimulationComputePipeline {
    pub layout: BindGroupLayoutDescriptor,
    pub write_back_bind_group: BindGroupLayoutDescriptor,
    pub external_forces: CachedComputePipelineId,
    pub spatial_hash: CachedComputePipelineId,
    pub update_positions: CachedComputePipelineId,
    pub reorder: CachedComputePipelineId,
    pub reorder_copy_back: CachedComputePipelineId,
}