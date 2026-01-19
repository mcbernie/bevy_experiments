use bevy::{
    prelude::*, 
    render::render_resource::{
        BindGroupLayoutDescriptor, 
        CachedComputePipelineId
    }
};

#[derive(Resource)]
pub struct CountSortComputePipeline {
    pub scan_layout: BindGroupLayoutDescriptor,
    pub sort_layout: BindGroupLayoutDescriptor,
    pub count_layout: BindGroupLayoutDescriptor,
    pub group_layout: BindGroupLayoutDescriptor,

    // count_sort
    pub clear_counts: CachedComputePipelineId, // 1) <- in
    pub count: CachedComputePipelineId, // 2)

    pub scan: CachedComputePipelineId, // 3)
    pub combine: CachedComputePipelineId, // 4)

    pub scatter_output: CachedComputePipelineId, // 5)
    pub copy_back: CachedComputePipelineId, // 6) -> out

}
