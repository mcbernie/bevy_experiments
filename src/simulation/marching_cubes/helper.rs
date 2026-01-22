use bevy::render::{render_resource::*, renderer::RenderDevice};

use super::components::{MarchingCubesBuffers, Triangle};

pub fn create_triangle_buffers(
    resolution: u32,
    render_device: &RenderDevice,
) -> MarchingCubesBuffers {
    let voxels = (resolution - 1).pow(3);
    let max_triangles = voxels * 5;

    let triangle_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("marching_cubes_triangles"),
        size: max_triangles as u64 * std::mem::size_of::<Triangle>() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
        mapped_at_creation: false,
    });

    let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("marching_cubes_counter"),
        contents: bytemuck::cast_slice(&[0u32]),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    MarchingCubesBuffers {
        triangle_buffer,
        counter_buffer,
        max_triangles,
    }
}