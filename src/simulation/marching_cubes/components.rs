use bevy::{prelude::*, render::render_resource::{BindGroup, Buffer, ShaderType}};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
pub struct Triangle {
    pub vertex_a: Vertex,
    pub vertex_b: Vertex,
    pub vertex_c: Vertex,
}

#[derive(Component)]
pub struct MarchingCubesBuffers {
    pub triangle_buffer: Buffer,
    pub counter_buffer: Buffer,
    pub max_triangles: u32,
}

#[derive(Component)]
pub struct MarchingCubesBindGroup {
    pub bind_group: BindGroup,
}