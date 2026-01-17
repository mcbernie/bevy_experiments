use bevy::{prelude::*, render::render_resource::{BindGroupLayoutDescriptor, CachedComputePipelineId}};

#[derive(Resource)]
pub struct SimulationComputePipeline {
    pub layout: BindGroupLayoutDescriptor,
    pub external_forces: CachedComputePipelineId,
    pub spatial_hash: CachedComputePipelineId,
    pub update_positions: CachedComputePipelineId,
}