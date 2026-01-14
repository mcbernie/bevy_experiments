/// Hier kommen alle wichtigen Systeme rein die notwendig für die Simulation sind.
use bevy::prelude::*;

use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy::render::render_resource::{
    BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries, BindGroupLayoutEntry, BindingType, BufferBindingType, CachedComputePipelineId, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderStages, UniformBuffer
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::storage::GpuShaderStorageBuffer;

use crate::PARTICLE_COUNT;
use crate::simulation::assets::SimulationParams;
use crate::simulation::components::{SimulationBuffers, PreparedSimulationBindGroup};

const FIXED_DT: f32 = 1.0 / 120.0;
const MAX_STEPS: u32 = 4;

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

#[derive(Resource)]
pub struct SimulationUniform {
    pub buffer: Option<UniformBuffer<SimulationParams>>,
}

/// Update the simulation uniform buffer if parameters have changed
/// which represents the simulation parameters like gravity, box size, etc.
pub fn update_simulation_uniform(
    mut uniform: ResMut<SimulationUniform>,
    params: Option<Res<SimulationParams>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    if params.is_none() {
        return;
    }
    let params = params.unwrap();
    if uniform.buffer.is_none() || params.is_changed() {

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
            // doenst need push constants
            push_constant_ranges: vec![
                PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..4, // für f32 (delta_time)
                }
            ],
            shader: spatial_hash_shader,
            entry_point: Some("update_spatial_hash".into()),
            ..Default::default()
        }
    );

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
    pipeline_cache: Res<PipelineCache>,
    settings_uniform: Res<SimulationUniform>,
    storage_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>, // <- Da legen wir den StorageBuffer ab
    query: Query<(Entity, &SimulationBuffers), Without<PreparedSimulationBindGroup>>,
) {
    if query.is_empty() {
        return;
    }
    if settings_uniform.buffer.is_none() {
        return;
    }

    let uniform = settings_uniform
        .buffer
        .as_ref()
        .expect("SimulationUniform not initialized yet");

    for (entity, buffers) in &query {
        let positions = storage_buffers.get(&buffers.positions).unwrap();
        let velocities = storage_buffers.get(&buffers.velocities).unwrap();
        let spatial_keys = storage_buffers.get(&buffers.spatial_keys).unwrap();


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
                    resource: spatial_keys.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: uniform.buffer().unwrap().as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert(
            PreparedSimulationBindGroup { bind_group }
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
        // nichts zu tun
        return;
    }
    let pipeline = match pipeline_cache.get_compute_pipeline(pipeline.compute_pipeline) {
        Some(p) => p,
        None => return, // Pipeline noch nicht bereit
    };

    // this could later help to be more stable in terms of time steps
    //sim_time.accumulator += time.delta_secs();
    //let mut steps = 0;
    //while sim_time.accumulator >= FIXED_DT && steps < MAX_STEPS {
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
            for simulation_bind_groups in query.iter() {
                pass.set_bind_group(0, &simulation_bind_groups.bind_group, &[]);

                // delta time pushen
                pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
                pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
            }
        }

        render_queue.submit(Some(encoder.finish()));

    //    sim_time.accumulator -= FIXED_DT;
    //    steps += 1;
    //}
}