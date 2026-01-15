use bevy::prelude::*;
use bevy::render::render_resource::{ComputePassDescriptor, PipelineCache};
use bevy::render::renderer::RenderContext;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel};

use crate::{FIXED_DT, PARTICLE_COUNT};

use super::systems::SimulationComputePipeline;
use super::components::PreparedSimulationBindGroup;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SimulationSystemLabel;

#[derive(Default)]
pub struct SimulationNode;


impl Node for SimulationNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<SimulationComputePipeline>();
        let mut bind_groups = world.try_query::<(Entity, &PreparedSimulationBindGroup)>().unwrap();

        let Some(spatial_hash) =
            pipeline_cache.get_compute_pipeline(pipelines.spatial_hash_pipeline)
        else { return Ok(()); };

        let Some(main_pipeline) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_pipeline)
        else { return Ok(()); };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("simulation_compute"),
                ..Default::default()
            });

        for (_, bg) in bind_groups.iter(world) {
            pass.set_bind_group(0, &bg.bind_group, &[]);

            // 1. External forces / integration
            pass.set_pipeline(main_pipeline);
            pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            // 2. Spatial hash
            pass.set_pipeline(spatial_hash);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
        }
        Ok(())
    }
}