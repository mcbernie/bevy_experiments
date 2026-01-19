use bevy::{prelude::*, render::{render_resource::{BindGroupEntry, BindGroupLayoutDescriptor, BufferDescriptor, BufferUsages, ComputePipelineDescriptor, PipelineCache, ShaderStages, binding_types::{storage_buffer, storage_buffer_read_only}}, renderer::RenderDevice}};

use crate::simulation::{assets::SimulationParams, gpu_sort::InternalCountSortBuffers, spatial_hash::{components::{InternalSpatialHashBuffers, PreparedSpatialHashComputeBindGroup}, resources::SpatialHashComputePipeline}};

pub fn init_spatial_hash_system(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    //render_queue: Res<RenderQueue>,
    query: Query<(Entity, &SimulationParams), Without<InternalSpatialHashBuffers>>,
) {

    if query.is_empty() {
        return;
    }

    for (entity, params) in &query {
        let particle_count = params.particle_count;
        let internal_buffers = create_internal_spatial_hash_buffers(&render_device, particle_count);

        warn!("Created InternalCountSortBuffers for entity {:?}", entity);
        commands.entity(entity).insert((
            internal_buffers,
        ));
    }
}

pub fn init_spatial_hash_compute_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {

    // Layout
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "spatial_hash_bind_group_layout",
        &[
            storage_buffer_read_only::<Vec<u32>>(false).build(0, ShaderStages::COMPUTE),
            storage_buffer::<Vec<u32>>(false).build(1, ShaderStages::COMPUTE),
        ],
    );

    let shader: Handle<Shader> = asset_server.load("shaders/spatial_offsets_compute.wgsl");

    let initialize_offsets = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("initialize_offsets_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: shader.clone(),
            entry_point: Some("initialize_offsets".into()),
            ..Default::default()
        }
    );

    let calculate_offsets = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("calculate_offsets_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: shader.clone(),
            entry_point: Some("calculate_offsets".into()),
            ..Default::default()
        }
    );

    // jetzt wollen wir natürlich den pipeline handle / id speichern und
    // auch das bind_group_layout
    commands.insert_resource(SpatialHashComputePipeline {
        initialize_offsets,
        calculate_offsets,
        layout: bind_group_layout,
    });

}


/// RenderApp - Erstelle eine "echte" BindGroup für die Simulation
pub fn prepare_spatial_hash_bind_groups(
    mut commands: Commands,
    pipeline: Res<SpatialHashComputePipeline>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    query: Query<(Entity, &InternalCountSortBuffers, &InternalSpatialHashBuffers), With<InternalCountSortBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, internal_sort_buffers, internal_spatial_hash_buffers) in &query {

        let sorted_keys = &internal_sort_buffers.sorted_keys;
        let offsets = &internal_spatial_hash_buffers.spatial_offsets;
        //let uniform_buffer = simulation_uniform.buffer.as_ref().unwrap();

        // we need to create this for each pipeline...?
        let bind_group = render_device.create_bind_group(
            "spatial_hash_compute_pipeline_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.layout),
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: sorted_keys.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: offsets.as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert(
            (
                PreparedSpatialHashComputeBindGroup { bind_group },
            )
        );
    }
}

pub fn create_internal_spatial_hash_buffers(
    render_device: &RenderDevice,
    num_items: u32,
) -> InternalSpatialHashBuffers {
    let buffer_size_u32 = (num_items as usize) * std::mem::size_of::<u32>();

    let spatial_offsets = render_device.create_buffer(&BufferDescriptor {
        label: Some("spatial_offsets_buffer"),
        size: buffer_size_u32 as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    InternalSpatialHashBuffers {
        spatial_offsets,
    }
}