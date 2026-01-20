use bevy::{prelude::*, render::render_resource::BindGroup};

#[derive(Component)]
pub struct PreparedSpatialHashComputeBindGroup {
    pub bind_group: BindGroup,
}