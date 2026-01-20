use bevy::{prelude::*, render::{render_resource::{BindGroupEntry, BindGroupLayoutDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages, binding_types::{storage_buffer, storage_buffer_read_only}}, renderer::RenderDevice}};

use crate::simulation::{components::InternalSimulationBuffers, gpu_sort::InternalCountSortBuffers, spatial_hash::{components::{PreparedSpatialHashComputeBindGroup}, resources::SpatialHashComputePipeline}};

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
    query: Query<(Entity, &InternalCountSortBuffers, &InternalSimulationBuffers), With<InternalCountSortBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, internal_sort_buffers, internal_simulation_buffers) in &query {

        let sorted_keys = &internal_sort_buffers.sorted_keys;
        let offsets = &internal_simulation_buffers.spatial_offsets;
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
