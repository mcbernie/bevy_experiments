/// Hier kommen alle wichtigen Systeme rein die notwendig für die Simulation sind.
use bevy::prelude::*;

use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, CachedComputePipelineId, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderStages, UniformBuffer
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::storage::GpuShaderStorageBuffer;

use crate::simulation::components::{SimulationBuffers, PreparedSimulationBindGroup};


#[derive(Resource)]
pub struct SimulationComputePipeline {
    pub pipeline: CachedComputePipelineId,
    pub layout: BindGroupLayout,
}

// RenderApp - Initialisiere die Compute Pipeline
pub fn init_compute_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
) {

    let shader = asset_server.load("shaders/particles.wgsl");

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
        ],
    );


    let pipeline_id = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("particle_compute_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: vec![
                PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..4, // für f32 (delta_time)
                }
            ],
            shader,
            shader_defs: vec![],
            entry_point: Some("main".into()),
            zero_initialize_workgroup_memory: true,
        }
    );

    // jetzt wollen wir natürlich den pipeline handle / id speichern und
    // auch das bind_group_layout
    commands.insert_resource(SimulationComputePipeline {
        pipeline: pipeline_id,
        layout: bind_group_layout,
    });

}


/// RenderApp - Erstelle eine "echte" BindGroup für die Simulation
pub fn prepare_simulation_bind_groups(
    mut commands: Commands,
    pipeline: Res<SimulationComputePipeline>,
    render_device: Res<RenderDevice>,
    storage_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>, // <- Da legen wir den StorageBuffer ab
    // nur die Entities die noch keine PreparedSimulationBindGroup haben
    query: Query<(Entity, &SimulationBuffers), Without<PreparedSimulationBindGroup>>,
) {
    for (entity, buffers) in &query {
        let positions = storage_buffers.get(&buffers.positions).unwrap();
        let velocities = storage_buffers.get(&buffers.velocities).unwrap();

        let bind_group = render_device.create_bind_group(
            "particle_compute_pipeline_bind_group",
            &pipeline.layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: positions.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: velocities.buffer.as_entire_binding(),
                }
            ],
        );

        commands.entity(entity).insert(
            PreparedSimulationBindGroup { bind_group }
        );
    }
}

// RenderApp - Führe die Compute Pipeline aus
pub fn run_compute(
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<SimulationComputePipeline>,
    time: Res<Time>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<&PreparedSimulationBindGroup>,
) {

    let delta_time = time.delta_secs();

    let pipeline = match pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
        Some(p) => p,
        None => return, // Pipeline noch nicht bereit
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
                label: Some("simulation_compute_pass"),
                ..Default::default()
            }
        );

        pass.set_pipeline(pipeline);
        for bind_group in query.iter() {
            pass.set_bind_group(0, &bind_group.bind_group, &[]);

            // delta time pushen
            pass.set_push_constants(0, bytemuck::bytes_of(&delta_time));
            pass.dispatch_workgroups(6400 / 256, 1, 1);
        }
    }

    render_queue.submit(Some(encoder.finish()));
}