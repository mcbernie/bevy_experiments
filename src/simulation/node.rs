use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{ComputePassDescriptor, PipelineCache};
use bevy::render::renderer::RenderContext;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel};
use bevy::render::storage::GpuShaderStorageBuffer;

use super::components::InternalSimulationBuffers;
use super::resources::SimulationComputePipeline;
use crate::{FIXED_DT, PARTICLE_COUNT, ReadbackBuffer};

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

            // 1. External forces / integration
            pass.set_pipeline(external_forces);
            pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            // 2. Spatial hash
            pass.set_pipeline(spatial_hash);
            pass.set_bind_group(0, &bg.bind_group, &[]);
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

        


        let Some(mut bind_groups) = world.try_query::<(Entity, &PreparedSimulationBindGroup, &InternalSimulationBuffers)>()
            else { return Ok(()); };

        let Some(calculate_densities) =
            pipeline_cache.get_compute_pipeline(pipelines.calculate_densities)
        else { return Ok(()); };

        let Some(calculate_pressure_force) =
            pipeline_cache.get_compute_pipeline(pipelines.calculate_pressure_force)
        else { return Ok(()); };

        let Some(calculate_viscosity) =
            pipeline_cache.get_compute_pipeline(pipelines.calculate_viscosity)
        else { return Ok(()); };

        let Some(update_positions) =
            pipeline_cache.get_compute_pipeline(pipelines.update_positions)
        else { return Ok(()); };

        let Some(reorder) =
            pipeline_cache.get_compute_pipeline(pipelines.reorder)
        else { return Ok(()); };

        let Some(reorder_copy_back) =
            pipeline_cache.get_compute_pipeline(pipelines.reorder_copy_back)
        else { return Ok(()); };

        let readback_buffer_handle = world.resource::<ReadbackBuffer>();
        let buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();
        let Some(readback_buffer_storage) = buffers.get(&readback_buffer_handle.handle) else {
            return Ok(());
        };
        let readback_buffer = &readback_buffer_storage.buffer; 
        

        for (_, bg, ib) in bind_groups.iter(world) {
            
            render_context.command_encoder().copy_buffer_to_buffer(
                &ib.spatial_keys, 
                0, 
                &readback_buffer, 
                0, 
                ib.spatial_keys.size()
            );
            
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("final_simulation_compute"),
                    ..Default::default()
                });
            

            pass.set_pipeline(reorder);
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_bind_group(1, &bg.write_back_bind_group, &[]);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            pass.set_pipeline(reorder_copy_back);
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_bind_group(1, &bg.write_back_bind_group, &[]);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            // calculate densities
            pass.set_pipeline(calculate_densities);
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            // calculate pressure forces
            pass.set_pipeline(calculate_pressure_force);
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            // calculate viscosity forces
            pass.set_pipeline(calculate_viscosity);
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

            // finaly update positions
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.set_pipeline(update_positions);
            pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
            pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

        }

        Ok(())
    }
}