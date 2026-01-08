use bevy::{
    prelude::*, 
    render::{
        render_resource::{
            BindGroupEntry, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferDescriptor, BufferInitDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, MapMode, PipelineCache, PollType, ShaderStages, ShaderType
        }, 
        renderer::{RenderDevice, RenderQueue}, storage::ShaderStorageBuffer
    }
};
use bytemuck::cast_slice;

use crate::simulation::structs::{ComputeBindGroup, ComputeBuffers, ReadbackBuffer, SharedComputeBuffers};
use super::structs::{ComputePipelineState, SimParams};

#[derive(Resource, Default)]
pub struct ReadbackState {
    pub busy: bool,
}

#[repr(C)]
#[derive(Clone, Copy, ShaderType)]
struct ParticlePosition {
    pos: Vec3, // Bevy-Vec3 ist korrekt gepaddet
}

pub fn init_compute_buffers(
    mut commands: Commands,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let num_particles = 1024;

    let positions_data = vec![
        ParticlePosition { pos: Vec3::ZERO };
        num_particles
    ];

    let positions = storage_buffers.add(
        ShaderStorageBuffer::from(&positions_data)
    );

    commands.insert_resource(SharedComputeBuffers {
        positions,
    });
}


pub fn update_params(
    time: Res<Time>,
    buffers: Res<ComputeBuffers>,
    render_queue: Res<RenderQueue>,
) {
    let params = SimParams {
        num_particles: 1024,
        gravity: -9.81,
        delta_time: time.delta_secs(),
        _pad: 0.0,
    };

    render_queue.write_buffer(
        &buffers.params,
        0,
        bytemuck::bytes_of(&params),
    );
}


pub fn init_compute(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {
    let num_particles: u32 = 1024;

    // -----------------------------------------
    // Positions Buffer
    // -----------------------------------------
    let positions_data = vec![[0.0f32; 3]; num_particles as usize];

    let positions = render_device.create_buffer_with_data(
        &BufferInitDescriptor {
            label: Some("positions_buffer"),
            contents: bytemuck::cast_slice(&positions_data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        },
    );

    // -----------------------------------------
    // Velocities Buffer
    // -----------------------------------------
    let velocities_data = vec![[0.0f32; 3]; num_particles as usize];

    let velocities = render_device.create_buffer_with_data(
        &BufferInitDescriptor {
            label: Some("velocities_buffer"),
            contents: bytemuck::cast_slice(&velocities_data),
            usage: BufferUsages::STORAGE,
        },
    );

    // -----------------------------------------
    // Uniform Buffer
    // -----------------------------------------
    let params = SimParams {
        num_particles,
        gravity: -9.81,
        delta_time: 1.0 / 60.0,
        _pad: 0.0,
    };

    let params_buffer = render_device.create_buffer_with_data(
        &BufferInitDescriptor {
            label: Some("params_buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        },
    );

    let buffers = ComputeBuffers {
        positions,
        velocities,
        params: params_buffer,
    };

    let shader: Handle<Shader> =
        asset_server.load("shaders/particles.wgsl");

    // Layout
    let bind_group_layout = render_device.create_bind_group_layout(
        "compute_bind_group_layout",
        &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    );

    let bind_group = render_device.create_bind_group(
        "compute_bind_group",
        &bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: buffers.positions.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: buffers.velocities.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: buffers.params.as_entire_binding(),
            },
        ],
    );

    commands.insert_resource(ComputeBindGroup(bind_group));

    let pipeline_id = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("particle_compute".into()),
            layout: vec![bind_group_layout],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Some("main".into()),
            zero_initialize_workgroup_memory: true,
        }
    );

    commands.insert_resource(buffers);
    commands.insert_resource(ComputePipelineState {
        pipeline_id,
    });

    let readback_size = (num_particles as u64) * 12; // vec3<f32>

    let readback_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("positions_readback"),
        size: readback_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    commands.insert_resource(ReadbackBuffer {
        buffer: readback_buffer,
        size: readback_size,
    });

}

pub fn run_compute(
    pipeline_cache: Res<PipelineCache>,
    pipeline_state: Res<ComputePipelineState>,
    compute_bind_group: Res<ComputeBindGroup>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    readback: Res<ReadbackBuffer>,
    buffers: Res<ComputeBuffers>,
    state: Res<ReadbackState>,
) {
    if state.busy {
        return; // 🚫 GPU wird gerade gelesen
    }
    let Some(pipeline) =
        pipeline_cache.get_compute_pipeline(pipeline_state.pipeline_id)
    else {
        return;
    };

    let mut encoder =
        render_device.create_command_encoder(&CommandEncoderDescriptor::default());

    {
        let mut pass =
            encoder.begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &compute_bind_group.0, &[]);
        pass.dispatch_workgroups(16, 1, 1);
    }

    encoder.copy_buffer_to_buffer(
        &buffers.positions,
        0,
        &readback.buffer,
        0,
        readback.size,
    );

    render_queue.submit(Some(encoder.finish()));
}

pub fn read_positions(
    mut state: ResMut<ReadbackState>,
    readback: Res<ReadbackBuffer>,
    render_device: Res<RenderDevice>,
) {

    state.busy = true;
    let slice = readback.buffer.slice(..);

    slice.map_async(MapMode::Read, |_| {});

    let _ = render_device.wgpu_device().poll(PollType::Wait);

    let data = slice.get_mapped_range();

    let positions: &[[f32; 3]] =
        bytemuck::cast_slice(&data);

    // DEBUG
    println!("p0 = {:?}", positions[0]);

    drop(data);
    readback.buffer.unmap();
    let _ = render_device.wgpu_device().poll(PollType::Wait);
    state.busy = false;
}
