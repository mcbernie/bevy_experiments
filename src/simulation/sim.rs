use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets, render_resource::{
            BindGroupEntry, BindGroupLayoutDescriptor, BufferDescriptor, BufferUsages, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderStages, UniformBuffer, binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer}
        }, renderer::{RenderDevice, RenderQueue}, storage::GpuShaderStorageBuffer
    },
};

use crate::simulation::components::{
    PreparedSimulationBindGroup, 
    SimulationBuffers
};

use super::{
    assets::SimulationParams,
    components::{InternalSimulationBuffers, SimulationUniform},
    helper::{CreatePipelineArgs, create_pipeline},
    resources::SimulationComputePipeline,
};


/// create all required internal buffers for the simulation
/// multiple compute pipelines will use them
pub fn create_internal_simulation_buffers(
    render_device: &RenderDevice,
    particle_count: u32,
) -> InternalSimulationBuffers {
    let buffer_size = (particle_count as usize) * std::mem::size_of::<Vec4>();
    let buffer_size_u32 = (particle_count as usize) * std::mem::size_of::<u32>();

    let predicted_positions = render_device.create_buffer(&BufferDescriptor {
        label: Some("predicted_positions_buffer"),
        size: buffer_size as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let spatial_keys = render_device.create_buffer(&BufferDescriptor {
        label: Some("spatial_keys_buffer"),
        size: buffer_size_u32 as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let spatial_offsets = render_device.create_buffer(&BufferDescriptor {
        label: Some("spatial_offsets_buffer"),
        size: buffer_size_u32 as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let sorted_indices = render_device.create_buffer(&BufferDescriptor {
        label: Some("sorted_indices_buffer"),
        size: buffer_size_u32 as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    InternalSimulationBuffers {
        predicted_positions,
        spatial_keys,
        spatial_offsets,
        sorted_indices,
    }
}


// when no InternalSimulationBuffers exist, create them, and create SimulationUniform
// should only run once and only before prepare step
// TODO: if we change the particle count, we would need to recreate them 
pub fn init_simulation_system(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<(Entity, &SimulationParams), Without<InternalSimulationBuffers>>,
) {

    if query.is_empty() {
        return;
    }

    for (entity, params) in &query {
        let particle_count = params.particle_count;
        let internal_buffers = create_internal_simulation_buffers(&render_device, particle_count);

        let mut uniform_buffer = UniformBuffer::from(params.clone());
        uniform_buffer.write_buffer(&render_device, &render_queue);

        warn!("Created InternalSimulationBuffers and SimulationUniform for entity {:?}", entity);
        commands.entity(entity).insert((
            internal_buffers,
            SimulationUniform (
                uniform_buffer,
            ),
        ));
    }
}

/// runs in renderer world to create pipelines and bind group layouts
pub fn init_compute_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {

    // Layout
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "compute_bind_group_layout",
        &[
            // will replaced by i think `storage_buffer` method. but works for now
            // positions
            storage_buffer::<Vec4>(false).build(0, ShaderStages::COMPUTE),
            // velocities
            storage_buffer::<Vec4>(false).build(1, ShaderStages::COMPUTE),
            // predicted positions
            storage_buffer::<Vec4>(false).build(2, ShaderStages::COMPUTE),
            // spatial keys
            storage_buffer::<u32>(false).build(3, ShaderStages::COMPUTE),
            // spatial offesets
            storage_buffer::<u32>(false).build(4, ShaderStages::COMPUTE),
            // sorted indices
            storage_buffer_read_only::<u32>(false).build(5, ShaderStages::COMPUTE),
            uniform_buffer::<SimulationParams>(false).build(6, ShaderStages::COMPUTE),
        ],
    );

    let shader: Handle<Shader> = asset_server.load("shaders/simulation.wgsl");

    let external_forces = create_pipeline(
        CreatePipelineArgs {
            for_shader: shader.clone(),
            entry_point: "external_forces",
            bind_group_layout: bind_group_layout.clone(),
            with_delta_push: true,
            ..Default::default()
        },
        &pipeline_cache,
    );

    // next step in simulation is
    let spatial_hash = create_pipeline(
        CreatePipelineArgs {
            for_shader: shader.clone(),
            entry_point: "update_spatial",
            bind_group_layout: bind_group_layout.clone(),
            ..Default::default()
        },
        &pipeline_cache,
    );
    
    // last step: copy the positions back 
    let update_positions = create_pipeline(
        CreatePipelineArgs {
            for_shader: shader.clone(),
            entry_point: "update_positions",
            bind_group_layout: bind_group_layout.clone(),
            with_delta_push: true,
            ..Default::default()
        },
        &pipeline_cache,
    );
    
    warn!("insert simulationcomputepipeline resource 
         external_forces: {:?}, spatial_hash: {:?}, update_positions: {:?}",
        external_forces, spatial_hash, update_positions
    );
    // jetzt wollen wir natürlich den pipeline handle / id speichern und
    // auch das bind_group_layout
    commands.insert_resource(SimulationComputePipeline {
        external_forces,
        spatial_hash,
        update_positions,
        layout: bind_group_layout,
    });

}


/// RenderApp - Erstelle eine "echte" BindGroup für die Simulation
pub fn prepare_simulation_bind_groups(
    mut commands: Commands,
    pipeline: Res<SimulationComputePipeline>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    storage_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>, // <- Da legen wir den StorageBuffer ab
    query: Query<(Entity, &SimulationBuffers, &InternalSimulationBuffers, &SimulationUniform), With<InternalSimulationBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, buffers, internal_buffers, simulation_uniform) in &query {

        let positions  = storage_buffers.get(&buffers.positions).unwrap();
        let velocities = storage_buffers.get(&buffers.velocities).unwrap();

        let predicted_positions = &internal_buffers.predicted_positions;
        let spatial_keys = &internal_buffers.spatial_keys;
        let spatial_offsets = &internal_buffers.spatial_offsets;
        let sorted_indices = &internal_buffers.sorted_indices;

        let uniform_buffer = &simulation_uniform.0;

        // we need to create this for each pipeline...?
        let bind_group = render_device.create_bind_group(
            "simulation_bind_group",
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
                    resource: predicted_positions.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: spatial_keys.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: spatial_offsets.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: sorted_indices.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: uniform_buffer.buffer().unwrap().as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert(
            (
                PreparedSimulationBindGroup { bind_group },
            )
        );
    }
}