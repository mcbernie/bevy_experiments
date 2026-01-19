use bevy::{platform::collections::HashMap, prelude::*, render::render_resource::{BindGroup, Buffer, UniformBuffer}};

#[derive(Component)]
pub struct PreparedCountSortComputeBindGroup {
    pub sort_bind_group: BindGroup,
    pub scan_bind_group: BindGroup,
    pub per_group_count: HashMap<u32, BindGroup>,
    pub count_bind_group: BindGroup,
}

#[derive(Component, Clone)]
pub struct InternalCountSortBuffers {
    pub input_items: Buffer,
    pub input_keys: Buffer,
    pub sorted_items: Buffer,
    pub sorted_keys: Buffer,
    pub counts: Buffer, // and elements for scan
    pub group_sums: HashMap<u32, Buffer>, // for scan
    pub num_items: Buffer,
}