use bevy::{
    prelude::*, render::{
        render_asset::RenderAssets,
        render_resource::{binding_types::*, *},
        renderer::{RenderDevice, RenderQueue},
        storage::GpuShaderStorageBuffer,
    }
};

use crate::{DENSITY_TEXTURE_RES, simulation::{components::{DensityMap, TransformData}, helper::{create_density_texture, density_map_extent}, resources::SimStepper}};

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

pub fn update_sim_stepper(
    time: Res<Time>,
    mut stepper: ResMut<SimStepper>,
) {
    let mut frame_dt = time.delta_secs();

    frame_dt = frame_dt.min(0.1);
    stepper.update(frame_dt);

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
        uniform.buffer.set(update_params);
        uniform.buffer.write_buffer(&render_device, &render_queue);
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

    let density_map_size = render_device.create_buffer(&BufferDescriptor {
        label: Some("density_map_size_buffer"),
        size: std::mem::size_of::<[u32; 4]>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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
        density_map_size,
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

    let sim_uniform_layout_desc = BindGroupLayoutDescriptor::new(
        "sim_uniform_layout_desc",
        &[
            uniform_buffer::<SimulationParams>(false).build(0, ShaderStages::COMPUTE | ShaderStages::VERTEX),
        ],
    );
    let sim_uniform_layout = render_device.create_bind_group_layout(
        Some("sim_uniform_layout".into()), 
        &sim_uniform_layout_desc.entries
    );

    for (entity, params) in &query {
        let particle_count = params.particle_count;
        let internal_buffers = create_internal_simulation_buffers(&render_device, particle_count);

        let mut uniform_buffer = UniformBuffer::from(params.clone());
        uniform_buffer.write_buffer(&render_device, &render_queue);

        let simulation_uniform_bindgroup = render_device.create_bind_group(
            "simulation_uniform_bindgroup",
            &sim_uniform_layout,
            &[BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer().unwrap().as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert((
            internal_buffers,
            SimulationUniform {
                buffer: uniform_buffer,
                bind_group: simulation_uniform_bindgroup,
            },
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
            texture_storage_3d(TextureFormat::R16Float, StorageTextureAccess::WriteOnly).build(8, ShaderStages::COMPUTE),
            uniform_buffer::<[u32; 4]>(false).build(9, ShaderStages::COMPUTE),
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

    let update_density = create_pipeline(
        CreatePipelineArgs {
            for_shader: shader.clone(),
            entry_point: "update_density",
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
        update_density,
        update_positions,
        reorder,
        reorder_copy_back,
        calculate_pressure_force,
        calculate_viscosity,
        write_back_bind_group: write_back_bind_group_layout,
        layout: bind_group_layout,
    });

}

pub fn prepare_density_map(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<(Entity, &TransformData, &SimulationParams, &InternalSimulationBuffers, Option<&DensityMap>)>,
) {
    for (entity, transform, params, buffers, existing) in &query {
        let desired_extent =
            density_map_extent(transform.scale, DENSITY_TEXTURE_RES);

        let needs_resize = match existing {
            Some(d) => d.extent != desired_extent,
            None => true,
        };

        if !needs_resize {
            continue;
        }

        let texture = create_density_texture(&render_device, desired_extent);
        let view = texture.create_view(&TextureViewDescriptor::default());

        // update densitymapsizebuffer
        let density_map_size = [
            desired_extent.width,
            desired_extent.height,
            desired_extent.depth_or_array_layers,
            0,
        ];

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("density_map_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        render_queue.write_buffer(&buffers.density_map_size, 0, bytemuck::cast_slice(&density_map_size));

        commands.entity(entity).insert(DensityMap {
            texture,
            view,
            sampler,
            extent: desired_extent,
        });
    }
}


/// RenderApp - Erstelle eine "echte" BindGroup für die Simulation
/// gets-called on every frame ?!
pub fn prepare_simulation_bind_groups(
    mut commands: Commands,
    pipeline: Res<SimulationComputePipeline>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    storage_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>, // <- Da legen wir den StorageBuffer ab
    query: Query<(Entity, &SimulationBuffers, &InternalSimulationBuffers, &SimulationUniform, &DensityMap), With<InternalSimulationBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, buffers, internal_buffers, simulation_uniform, density_map) in &query {

        let positions  = storage_buffers.get(&buffers.positions).unwrap();
        let velocities = storage_buffers.get(&buffers.velocities).unwrap();

        let predicted_positions = &internal_buffers.predicted_positions;
        let spatial_keys = &internal_buffers.spatial_keys;
        let spatial_offsets = &internal_buffers.spatial_offsets;
        let sorted_indices = &internal_buffers.sorted_indices;
        let densities = &internal_buffers.densities;
        let density_map_size = &internal_buffers.density_map_size;

        let sort_target_position = &internal_buffers.sort_target_position;
        let sort_target_predicted_positions = &internal_buffers.sort_target_predicted_positions;
        let sort_target_velocities = &internal_buffers.sort_target_velocity;

        let uniform_buffer = &simulation_uniform.buffer;

        let entries =vec![
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
            BindGroupEntry {
                binding: 8,
                resource: BindingResource::TextureView(&density_map.view),
            },
            BindGroupEntry {
                binding: 9,
                resource: density_map_size.as_entire_binding(),
            }
        ]; 


        // we need to create this for each pipeline...?
        let bind_group = render_device.create_bind_group(
            "simulation_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.layout),
            &entries,
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