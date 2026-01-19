use bevy::{prelude::*, render::render_resource::{BindGroup, Buffer}};

#[derive(Component)]
pub struct PreparedSpatialHashComputeBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component, Clone)]
pub struct InternalSpatialHashBuffers {
    pub spatial_offsets: Buffer,
}