use bevy::prelude::*;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel};
use bevy::render::render_resource::binding_types::{storage_buffer, storage_buffer_read_only};
use bevy::render::render_resource::{BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, ShaderStages};
use bevy::render::renderer::{RenderContext, RenderDevice};

use crate::PARTICLE_COUNT;
use crate::simulation::components::AdvancedSimulationBuffers;
use crate::simulation::gpu_sort::InternalCountSortBuffers;


#[derive(Resource)]
pub struct SpatialHashComputePipeline {
    pub initialize_offsets: CachedComputePipelineId,
    pub calculate_offsets: CachedComputePipelineId,
    pub layout: BindGroupLayoutDescriptor,
}

#[derive(Component)]
pub struct PreparedSpatialHashComputeBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SpatialHashSystemLabel;

#[derive(Default)]
pub struct SpatialHashNode;

impl Node for SpatialHashNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline_resource = world.resource::<SpatialHashComputePipeline>();

        let Some(mut bind_groups) = world.try_query::<(Entity, &PreparedSpatialHashComputeBindGroup)>()
        else {
            return Ok(());
        };

        let Some(initalize_offsets) =
            pipeline_cache.get_compute_pipeline(pipeline_resource.initialize_offsets)
        else { return Ok(()); };

        let Some(calculate_offsets) =
            pipeline_cache.get_compute_pipeline(pipeline_resource.calculate_offsets)
        else { return Ok(()); };


        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("spatial_hash_compute_pass"),
                ..Default::default()
            });

        for (_, bg) in bind_groups.iter(world) {
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_pipeline(initalize_offsets);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
        }

        for (_, bg) in bind_groups.iter(world) {
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_pipeline(calculate_offsets);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
        }

        Ok(())
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
    query: Query<(Entity, &InternalCountSortBuffers), With<InternalCountSortBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, internal_sort_buffers) in &query {

        let sorted_keys = &internal_sort_buffers.sorted_keys;
        let offsets = &advanced_buffers.spatial_sort_offsets;
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