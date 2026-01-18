use bevy::{prelude::*, render::render_resource::BindGroup};

#[derive(Component)]
pub struct PreparedCountSortComputeBindGroup {
    pub sort_bind_group: BindGroup,
    pub scan_bind_group: BindGroup,
}