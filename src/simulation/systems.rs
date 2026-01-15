/// Hier kommen alle wichtigen Systeme rein die notwendig für die Simulation sind.
use bevy::prelude::*;

use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy::render::render_resource::{
    BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferDescriptor, BufferUsages, CachedComputePipelineId, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderStages, UniformBuffer
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::storage::GpuShaderStorageBuffer;

use crate::{FIXED_DT, PARTICLE_COUNT};
use crate::simulation::assets::SimulationParams;
use crate::simulation::components::{AdvancedSimulationBuffers, PreparedSimulationBindGroup, SimulationBuffers};


#[derive(Resource)]
pub struct SimulationTime {
    pub accumulator: f32,
}

#[derive(Resource)]
pub struct SimulationComputePipeline {
    pub compute_pipeline: CachedComputePipelineId,
    pub spatial_hash_pipeline: CachedComputePipelineId,
    pub layout: BindGroupLayoutDescriptor,
}

#[derive(Component)]
pub struct SimulationUniform {
    pub buffer: Option<UniformBuffer<SimulationParams>>,
}

/// Update the simulation uniform buffer if parameters have changed
/// which represents the simulation parameters like gravity, box size, etc.
pub fn update_simulation_uniform(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut query: Query<(Entity, &SimulationParams, &mut SimulationUniform), Changed<SimulationParams>>,
) {

    for (_entity, params, mut uniform) in &mut query {
        if let Some(buffer) = uniform.buffer.as_mut() {
            buffer.set(params.clone());
            buffer.write_buffer(&render_device, &render_queue);
        } else {
            let mut buffer = UniformBuffer::from(params.clone());
            buffer.write_buffer(&render_device, &render_queue);
            uniform.buffer = Some(buffer);
        }
    }
}

/// init the compute pipeline for the simulation
pub fn init_compute_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {

    let shader: Handle<Shader> = asset_server.load("shaders/particles.wgsl");
    let spatial_hash_shader: Handle<Shader> = asset_server.load("shaders/sim_spatial.wgsl");

    // Layout
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "compute_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            [
                // will replaced by i think `storage_buffer` method. but works for now
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: u32::MAX,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                uniform_buffer::<SimulationParams>(false).build(u32::MAX, ShaderStages::COMPUTE),
            ],
        )
    );

    // my first compute pipeline, where all the strange stuff happens
    let main_compute_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("simulation_compute_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: vec![
                PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..4, // für f32 (delta_time)
                }
            ],
            shader,
            entry_point: Some("main".into()),
            ..Default::default()
        }
    );

    // the next big thing: spatial hash pipeline
    let spatial_hash_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_hash_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_hash_shader,
            entry_point: Some("update_spatial_hash".into()),
            ..Default::default()
        }
    );

    /*let spatial_hash_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_hash_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_hash_shader,
            entry_point: Some("update_spatial_hash".into()),
            ..Default::default()
        }
    );*/

    // jetzt wollen wir natürlich den pipeline handle / id speichern und
    // auch das bind_group_layout
    commands.insert_resource(SimulationComputePipeline {
        compute_pipeline: main_compute_pipeline,
        spatial_hash_pipeline: spatial_hash_pipeline,
        layout: bind_group_layout,
    });

}


/// RenderApp - Erstelle eine "echte" BindGroup für die Simulation
pub fn prepare_simulation_bind_groups(
    mut commands: Commands,
    pipeline: Res<SimulationComputePipeline>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    storage_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>, // <- Da legen wir den StorageBuffer ab
    query: Query<(Entity, &SimulationBuffers, &SimulationParams), Without<PreparedSimulationBindGroup>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, buffers, params) in &query {
        warn!("run !");
        let positions = storage_buffers.get(&buffers.positions).unwrap();
        let velocities = storage_buffers.get(&buffers.velocities).unwrap();
        let mut uniform_buffer = UniformBuffer::from(params.clone());
        uniform_buffer.write_buffer(&render_device, &render_queue);

        //let n = params.num_particles as usize;
        let n = PARTICLE_COUNT as usize;

        let spatial_keys = render_device.create_buffer(&BufferDescriptor {
            label: Some("spatial_keys"),
            size: (n * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spatial_counts = render_device.create_buffer(&BufferDescriptor {
            label: Some("spatial_counts"),
            size: (n * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spatial_offsets = render_device.create_buffer(&BufferDescriptor {
            label: Some("spatial_offsets"),
            size: (n * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // we need to create this for each pipeline...?
        let bind_group = render_device.create_bind_group(
            "particle_compute_pipeline_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.layout),
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: positions.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: velocities.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: spatial_keys.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: spatial_counts.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: spatial_offsets.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: uniform_buffer.buffer().unwrap().as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert(
            (
                PreparedSimulationBindGroup { bind_group },
                SimulationUniform { buffer: Some(uniform_buffer) },
                AdvancedSimulationBuffers {
                    spatial_keys,
                    spatial_sort_counts: spatial_counts,
                    spatial_sort_offsets: spatial_offsets,
                },
            )
        );
    }
}

// each frame, run the compute shader to update the simulation
pub fn run_compute(
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<SimulationComputePipeline>,
    //time: Res<Time>,
    //mut sim_time: ResMut<SimulationTime>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<&PreparedSimulationBindGroup>,
) {

    if query.is_empty() {
        return;
    }
    let Some(main_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.compute_pipeline)
    else {
        return;
    };
    let Some(spatial_hash_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.spatial_hash_pipeline)
    else {
        return;
    };

    let mut encoder = render_device.create_command_encoder(
        &CommandEncoderDescriptor {
            label: Some("simulation_compute_encoder"),
            ..Default::default()
        }
    );

    {
        let mut pass = encoder.begin_compute_pass(
            &ComputePassDescriptor {
                label: Some("spatial_hash_compute_pass"),
                ..Default::default()
            }
        );
        pass.set_pipeline(spatial_hash_pipeline);
        for simulation_bind_groups in query.iter() {
                pass.set_bind_group(0, &simulation_bind_groups.bind_group, &[]);
                pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
        }
    }

    {
        let mut pass = encoder.begin_compute_pass(
            &ComputePassDescriptor {
                label: Some("simulation_compute_pass"),
                ..Default::default()
            }
        );
        pass.set_pipeline(main_pipeline);
        for simulation_bind_groups in query.iter() {
                pass.set_bind_group(0, &simulation_bind_groups.bind_group, &[]);
                pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
                pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
        }
    }

    render_queue.submit(Some(encoder.finish()));

}