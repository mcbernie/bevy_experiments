use bevy::prelude::*;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel};
use bevy::render::render_resource::{ComputePassDescriptor, PipelineCache};
use bevy::render::renderer::RenderContext;

use crate::PARTICLE_COUNT;
use crate::simulation::spatial_hash::components::PreparedSpatialHashComputeBindGroup;
use crate::simulation::spatial_hash::resources::SpatialHashComputePipeline;

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
                label: Some("initialize_spatial_hash_offsets_compute_pass"),
                ..Default::default()
            });

        for (_, bg) in bind_groups.iter(world) {
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_pipeline(initalize_offsets);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_pipeline(calculate_offsets);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
        }

        Ok(())
    }
}
