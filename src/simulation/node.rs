use bevy::prelude::*;
use bevy::render::render_resource::{ComputePassDescriptor, PipelineCache};
use bevy::render::renderer::RenderContext;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel};

use crate::simulation::resources::SimulationComputePipeline;
use crate::{FIXED_DT, PARTICLE_COUNT};

use super::components::PreparedSimulationBindGroup;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct StartSimulationSystemLabel;

#[derive(Default)]
pub struct StartSimulationNode;

impl Node for StartSimulationNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<SimulationComputePipeline>();
        let Some(mut bind_groups) = world.try_query::<(Entity, &PreparedSimulationBindGroup)>()
        else {
            return Ok(());
        };

        let Some(external_forces) = pipeline_cache.get_compute_pipeline(pipelines.external_forces)
            else { return Ok(()); };
        
        let Some(spatial_hash) =
            pipeline_cache.get_compute_pipeline(pipelines.spatial_hash)
        else { return Ok(()); };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("begin_simulation_compute"),
                ..Default::default()
            });

        for (_, bg) in bind_groups.iter(world) {
            pass.set_bind_group(0, &bg.bind_group, &[]);

            // 1. External forces / integration
            pass.set_pipeline(external_forces);
            pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            // 2. Spatial hash
            pass.set_pipeline(spatial_hash);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
            
        }

        Ok(())
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct FinalSimulationSystemLabel;

#[derive(Default)]
pub struct FinalSimulationNode;

impl Node for FinalSimulationNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<SimulationComputePipeline>();
        let Some(mut bind_groups) = world.try_query::<(Entity, &PreparedSimulationBindGroup)>()
        else {
            return Ok(());
        };


        let Some(update_positions) =
            pipeline_cache.get_compute_pipeline(pipelines.update_positions)
        else { return Ok(()); };

        let Some(reorder) =
            pipeline_cache.get_compute_pipeline(pipelines.reorder)
        else { return Ok(()); };

        let Some(reorder_copy_back) =
            pipeline_cache.get_compute_pipeline(pipelines.reorder_copy_back)
        else { return Ok(()); };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("final_simulation_compute"),
                ..Default::default()
            });

        for (_, bg) in bind_groups.iter(world) {


            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_bind_group(1, &bg.write_back_bind_group, &[]);
            pass.set_pipeline(reorder);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_bind_group(1, &bg.write_back_bind_group, &[]);
            pass.set_pipeline(reorder_copy_back);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_pipeline(update_positions);
            pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

        }

        Ok(())
    }
}