use bevy::{
    prelude::*, 
    render::render_resource::{
        BindGroupLayoutDescriptor, 
        CachedComputePipelineId
    }
};

#[derive(Resource)]
pub struct CountSortComputePipeline {
    pub sort_layout: BindGroupLayoutDescriptor,
    pub clear_counts: CachedComputePipelineId,
    pub calculate_counts: CachedComputePipelineId,
    pub scatter_output: CachedComputePipelineId,
    pub copy_back: CachedComputePipelineId,

    pub scan_layout: BindGroupLayoutDescriptor,
    pub block_scan: CachedComputePipelineId,
    pub block_combine: CachedComputePipelineId,
}
