use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            BindGroupEntry, BindGroupLayoutDescriptor, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages, UniformBuffer, binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer}
        },
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
    }, ui::update,
};

use crate::{FIXED_DT, PARTICLE_COUNT, simulation::{gpu_sort::{CountSortComputePipeline, InternalCountSortBuffers, PreparedCountSortComputeBindGroup, run_count_sort_compute}, helper::dispatch_compute, spatial_hash::{self, PreparedSpatialHashComputeBindGroup, SpatialHashComputePipeline, run_spatial_hash_compute_pipeline}}};

use super::{
    assets::SimulationParams,
    components::{
        InternalSimulationBuffers,
        SimulationUniform,
        PreparedSimulationBindGroup,
        SimulationBuffers,
    },
    helper::{CreatePipelineArgs, create_pipeline},
    resources::SimulationComputePipeline,
};

#[derive(Resource, Default)]
pub struct SimulationRunGuard {
   pub last_simulated_time: f64,
}

pub fn run_simulation(
    mut guard: ResMut<SimulationRunGuard>,
    time: Res<Time>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    // all pipelines
    simulation_pipelines: Res<SimulationComputePipeline>,
    count_sort_pipelines: Res<CountSortComputePipeline>,
    spatial_hash_pipelines: Res<SpatialHashComputePipeline>,


    query: Query<(Entity, 
        &PreparedSimulationBindGroup, 
        &PreparedCountSortComputeBindGroup,
        &PreparedSpatialHashComputeBindGroup, 
        &InternalCountSortBuffers, 
        &InternalSimulationBuffers,
        &SimulationParams
    )>
) {
    let now = time.elapsed_secs_f64();

    if guard.last_simulated_time == now {
        warn!("Skipping simulation step, already simulated for this frame");
        return; // dieses Render-Frame schon simuliert
    }

    let external_forces = match pipeline_cache.get_compute_pipeline(simulation_pipelines.external_forces) {
        Some(p) => p,
        None => return,
    };
    let spatial_hash = match pipeline_cache.get_compute_pipeline(simulation_pipelines.spatial_hash) {
        Some(p) => p,
        None => return,
    };
    let reorder = match pipeline_cache.get_compute_pipeline(simulation_pipelines.reorder) {
        Some(p) => p,
        None => return,
    };
    let reorder_copy_back = match pipeline_cache.get_compute_pipeline(simulation_pipelines.reorder_copy_back) {
        Some(p) => p,
        None => return,
    };
    let calculate_densities = match pipeline_cache.get_compute_pipeline(simulation_pipelines.calculate_densities) {
        Some(p) => p,
        None => return,
    };
    let calculate_pressure_force = match pipeline_cache.get_compute_pipeline(simulation_pipelines.calculate_pressure_force) {
        Some(p) => p,
        None => return,
    };
    let calculate_viscosity = match pipeline_cache.get_compute_pipeline(simulation_pipelines.calculate_viscosity) {
        Some(p) => p,
        None => return,
    };
    let update_positions = match pipeline_cache.get_compute_pipeline(simulation_pipelines.update_positions) {
        Some(p) => p,
        None => return,
    };

    // spatial hash pipelines
    let sh_initialize_offsets = match pipeline_cache.get_compute_pipeline(spatial_hash_pipelines.initialize_offsets) {
        Some(p) => p,
        None => return,
    };
    let sh_calculate_offsets = match pipeline_cache.get_compute_pipeline(spatial_hash_pipelines.calculate_offsets) {
        Some(p) => p,
        None => return,
    };

    // count sort pipelines
    let cs_clear_counts = match pipeline_cache.get_compute_pipeline(count_sort_pipelines.clear_counts) {
        Some(p) => p,
        None => return,
    };
    let cs_count = match pipeline_cache.get_compute_pipeline(count_sort_pipelines.count) {
        Some(p) => p,
        None => return,
    };
    let cs_scan = match pipeline_cache.get_compute_pipeline(count_sort_pipelines.scan) {
        Some(p) => p,
        None => return,
    };
    let cs_combine = match pipeline_cache.get_compute_pipeline(count_sort_pipelines.combine) {
        Some(p) => p,
        None => return,
    };
    let cs_scatter_output = match pipeline_cache.get_compute_pipeline(count_sort_pipelines.scatter_output) {
        Some(p) => p,
        None => return,
    };
    let cs_copy_back = match pipeline_cache.get_compute_pipeline(count_sort_pipelines.copy_back) {
        Some(p) => p,
        None => return,
    };

    let max_timestep_fps = 60.0;
    let max_delta_time = if max_timestep_fps > 0.0 { 1.0 / max_timestep_fps } else { f32::INFINITY }; // If framerate dips too low, run the simulation slower than real-time
    let active_time_scale = 1.0; // Time.timeScale equivalent
    let iterations_per_frame = 3.0; // number of simulation steps per frame
    let mut dt = f32::min(time.delta_secs() * active_time_scale, max_delta_time);

    if guard.last_simulated_time == 0.0 {
        // first run, avoid large delta time
        dt = 0.0
    }
    if dt >= 0.014 {
        warn!("Running simulation step with dt = {}", dt);
    }
    let sub_step_delta_time = dt / iterations_per_frame;

    for (
        _entity, 
        simulation_bg, 
        count_sort_bg, 
        spatial_hash_bg, 
        internal_count_sort_buffers,
        _internal_simulation_buffers,
        params
    ) in &query {

        let mut encoder = render_device
            .create_command_encoder(
                &CommandEncoderDescriptor { 
                    label: Some("simulation_encoder"),
                }
            );
        for _i in 0..(iterations_per_frame as u32) {
            let particle_count = params.particle_count;
            let wg_size = (particle_count + 255) / 256;

            //let delta = time.delta_secs();
            //let delta = time.delta_secs() / 100.0;
            let delta = sub_step_delta_time;
            let binary_delta = Some(bytemuck::bytes_of(&delta));

            // all the simulations step here

            {

                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("final_simulation_compute"),
                    ..Default::default()
                });


                dispatch_compute(
                    &mut pass, 
                    external_forces, 
                    &[&simulation_bg.bind_group], 
                    binary_delta, 
                    wg_size
                );

                // 2. Spatial hash
                dispatch_compute(
                    &mut pass, 
                    spatial_hash, 
                    &[&simulation_bg.bind_group], 
                    None, 
                    wg_size
                );

                // run count_sort
                run_count_sort_compute(
                    &render_queue, 
                    &mut pass,
                    &cs_clear_counts,
                    &cs_count,
                    &cs_scan,
                    &cs_combine,
                    &cs_scatter_output,
                    &cs_copy_back,
                    &count_sort_bg,
                    &internal_count_sort_buffers,
                    particle_count,
                );

                // run spatial_hash
                run_spatial_hash_compute_pipeline(
                    &mut pass,
                    &sh_initialize_offsets,
                    &sh_calculate_offsets,
                    &spatial_hash_bg,
                    wg_size,
                );

                dispatch_compute(
                    &mut pass, 
                    reorder, 
                    &[&simulation_bg.bind_group, &simulation_bg.write_back_bind_group], 
                    None, 
                    wg_size
                );

                dispatch_compute(
                    &mut pass, 
                    reorder_copy_back, 
                    &[&simulation_bg.bind_group, &simulation_bg.write_back_bind_group], 
                    None, 
                    wg_size
                );

                // calculate densities
                dispatch_compute(
                    &mut pass, 
                    calculate_densities, 
                    &[&simulation_bg.bind_group], 
                    None, 
                    wg_size
                );

                // calculate pressure forces
                dispatch_compute(
                    &mut pass, 
                    calculate_pressure_force, 
                    &[&simulation_bg.bind_group], 
                    binary_delta, 
                    wg_size
                );

                // calculate viscosity forces
                dispatch_compute(
                    &mut pass, 
                    calculate_viscosity, 
                    &[&simulation_bg.bind_group], 
                    binary_delta, 
                    wg_size
                );

                // finaly update positions
                dispatch_compute(
                    &mut pass, 
                    update_positions, 
                    &[&simulation_bg.bind_group], 
                    binary_delta, 
                    wg_size
                );
            }
        }

        render_queue.submit(Some(encoder.finish()));

    }

    guard.last_simulated_time = now;
}

/// Update the simulation uniform buffer if parameters have changed
/// which represents the simulation parameters like gravity, box size, etc.
pub fn update_simulation_uniform(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut query: Query<(Entity, &SimulationParams, &mut SimulationUniform), Changed<SimulationParams>>,
) {

    for (_entity, params, mut uniform) in &mut query {
        let mut update_params = params.clone();
        update_params.update_derived_constants();
        uniform.0.set(update_params);
        uniform.0.write_buffer(&render_device, &render_queue);
    }
}

/// create all required internal buffers for the simulation
/// multiple compute pipelines will use them
pub fn create_internal_simulation_buffers(
    render_device: &RenderDevice,
    particle_count: u32,
) -> InternalSimulationBuffers {
    let buffer_size = (particle_count as usize) * std::mem::size_of::<Vec4>();
    let buffer_size_u32 = (particle_count as usize) * std::mem::size_of::<u32>();
    let buffer_size_vec2 = (particle_count as usize) * std::mem::size_of::<Vec2>();

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

    let spatial_indices = render_device.create_buffer(&BufferDescriptor {
        label: Some("spatial_indices_buffer"),
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

    let sort_target_position = render_device.create_buffer(&BufferDescriptor {
        label: Some("sort_target_position_buffer"),
        size: buffer_size as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let sort_target_predicted_positions = render_device.create_buffer(&BufferDescriptor {
        label: Some("sort_target_predicted_positions_buffer"),
        size: buffer_size as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let sort_target_velocity = render_device.create_buffer(&BufferDescriptor {
        label: Some("sort_target_velocity_buffer"),
        size: buffer_size as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let densities = render_device.create_buffer(&BufferDescriptor {
        label: Some("densities_buffer"),
        size: buffer_size_vec2 as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    InternalSimulationBuffers {
        predicted_positions,
        spatial_keys,
        spatial_indices,
        spatial_offsets,
        sorted_indices,
        sort_target_position,
        sort_target_predicted_positions,
        sort_target_velocity,
        densities,
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
            // densities
            storage_buffer::<Vec2>(false).build(6, ShaderStages::COMPUTE),
            uniform_buffer::<SimulationParams>(false).build(7, ShaderStages::COMPUTE),
        ],
    );
    
    let write_back_bind_group_layout = BindGroupLayoutDescriptor::new(
        "write_back_bind_group_layout",
        &[
            // will replaced by i think `storage_buffer` method. but works for now
            // sort_target_position
            storage_buffer::<Vec4>(false).build(0, ShaderStages::COMPUTE),
            // sort_target_predicted_positions
            storage_buffer::<Vec4>(false).build(1, ShaderStages::COMPUTE),
            // predicted sort_target_velocities
            storage_buffer::<Vec4>(false).build(2, ShaderStages::COMPUTE),
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

    let calculate_densities = create_pipeline(
        CreatePipelineArgs {
            for_shader: shader.clone(),
            entry_point: "calculate_densities",
            bind_group_layout: bind_group_layout.clone(),
            ..Default::default()
        },
        &pipeline_cache,
    );

    let calculate_pressure_force = create_pipeline(
        CreatePipelineArgs {
            for_shader: shader.clone(),
            entry_point: "calculate_pressure_force",
            bind_group_layout: bind_group_layout.clone(),
            with_delta_push: true,
            ..Default::default()
        },
        &pipeline_cache,
    );

    let calculate_viscosity = create_pipeline(
        CreatePipelineArgs {
            for_shader: shader.clone(),
            entry_point: "calculate_viscosity",
            bind_group_layout: bind_group_layout.clone(),
            with_delta_push: true,
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
    
    let reorder = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("reorder_pipeline".into()),
            layout: vec![bind_group_layout.clone(), write_back_bind_group_layout.clone()],
            shader: shader.clone(),
            entry_point: Some("reorder".into()),
            ..Default::default()
        }
    );

    let reorder_copy_back = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("reorder_copy_back_pipeline".into()),
            layout: vec![bind_group_layout.clone(), write_back_bind_group_layout.clone()],
            shader: shader.clone(),
            entry_point: Some("reorder_copy_back".into()),
            ..Default::default()
        }
    );
    
    warn!("insert simulationcomputepipeline resource 
         external_forces: {:?}, spatial_hash: {:?}, calculate_densities: {:?}",
        external_forces, spatial_hash, calculate_densities
    );
    // jetzt wollen wir natürlich den pipeline handle / id speichern und
    // auch das bind_group_layout
    commands.insert_resource(SimulationComputePipeline {
        external_forces,
        spatial_hash,
        calculate_densities,
        update_positions,
        reorder,
        reorder_copy_back,
        calculate_pressure_force,
        calculate_viscosity,
        write_back_bind_group: write_back_bind_group_layout,
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
        let densities = &internal_buffers.densities;

        let sort_target_position = &internal_buffers.sort_target_position;
        let sort_target_predicted_positions = &internal_buffers.sort_target_predicted_positions;
        let sort_target_velocities = &internal_buffers.sort_target_velocity;

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
                    resource: densities.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: uniform_buffer.buffer().unwrap().as_entire_binding(),
                },
            ],
        );

        let write_back_bind_group = render_device.create_bind_group(
            "simulation_write_back_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.write_back_bind_group),
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: sort_target_position.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: sort_target_predicted_positions.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: sort_target_velocities.as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert(
            (
                PreparedSimulationBindGroup { bind_group, write_back_bind_group },
            )
        );
    }
}