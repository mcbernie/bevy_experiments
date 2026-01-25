use bevy::prelude::*;
use bevy::render::renderer::RenderQueue;
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel},
    render_resource::{ComputePassDescriptor, PipelineCache},
    renderer::RenderContext,
};
use bevy::time::Stopwatch;

use super::{
    components::{InternalSimulationBuffers, PreparedSimulationBindGroup},
    resources::SimulationComputePipeline,
};
use crate::simulation::assets::SimulationParams;
use crate::simulation::components::DensityMap;
use crate::simulation::gpu_sort::{CountSortComputePipeline, InternalCountSortBuffers, PreparedCountSortComputeBindGroup, run_count_sort_compute};
use crate::simulation::helper::dispatch_compute;
use crate::simulation::resources::SimStepper;
use crate::simulation::spatial_hash::{PreparedSpatialHashComputeBindGroup, SpatialHashComputePipeline, run_spatial_hash_compute_pipeline};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SimulationGraphLabel;

#[derive(Default)]
pub struct SimulationNode {
    first_run: bool,
    counter: u64,
}

impl Node for SimulationNode {
    fn update(&mut self, _world: &mut World) {

        if !self.first_run && self.counter == 100 {
            self.first_run = true;
        }
        if !self.first_run {
            self.counter += 1;
        }
    }
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {

        let stepper = world.resource::<SimStepper>();
        let queue = world.resource::<RenderQueue>();

        let Some(mut entries) = world.try_query::<(
            Entity, 
            &PreparedSimulationBindGroup, 
            &PreparedCountSortComputeBindGroup,
            &PreparedSpatialHashComputeBindGroup, 
            &InternalCountSortBuffers, 
            &InternalSimulationBuffers,
            &SimulationParams,
        )>()
            else { return Ok(()); };
        
        let Some(mut density_query) = world.try_query::<(
            Entity, 
            &PreparedSimulationBindGroup, 
            Option<&DensityMap>,
        )>()
            else { return Ok(()); };

        let pipeline_cache = world.resource::<PipelineCache>();
        let simulation_pipelines = world.resource::<SimulationComputePipeline>();

        let Some(external_forces) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.external_forces)
        else { return Ok(()); };

        let Some(spatial_hash) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.spatial_hash)
        else { return Ok(()); };

        let Some(reorder) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.reorder)
        else { return Ok(()); };

        let Some(reorder_copy_back) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.reorder_copy_back)
        else { return Ok(()); };

        let Some(calculate_densities) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.calculate_densities)
        else { return Ok(()); };

        let Some(update_density) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.update_density)
        else { return Ok(()); };

        let Some(calculate_pressure_force) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.calculate_pressure_force)
        else { return Ok(()); };

        let Some(calculate_viscosity) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.calculate_viscosity)
        else { return Ok(()); };

        let Some(update_positions) =
            pipeline_cache.get_compute_pipeline(simulation_pipelines.update_positions)
        else { return Ok(()); };

        // spatial hash pipelines
        let spatial_hash_pipelines = world.resource::<SpatialHashComputePipeline>();
        
        let Some(sh_initialize_offsets) =
            pipeline_cache.get_compute_pipeline(spatial_hash_pipelines.initialize_offsets)
        else { return Ok(()); };
        let Some(sh_calculate_offsets) =
            pipeline_cache.get_compute_pipeline(spatial_hash_pipelines.calculate_offsets)
        else { return Ok(()); };

        // count sort pipelines
        let count_sort_pipelines = world.resource::<CountSortComputePipeline>();

        let Some(cs_clear_counts) =
            pipeline_cache.get_compute_pipeline(count_sort_pipelines.clear_counts)
        else { return Ok(()); };

        let Some(cs_count) =
            pipeline_cache.get_compute_pipeline(count_sort_pipelines.count)
        else { return Ok(()); };

        let Some(cs_scan) =
            pipeline_cache.get_compute_pipeline(count_sort_pipelines.scan)
        else { return Ok(()); };

        let Some(cs_combine) =
            pipeline_cache.get_compute_pipeline(count_sort_pipelines.combine)
        else { return Ok(()); };

        let Some(cs_scatter_output) =
            pipeline_cache.get_compute_pipeline(count_sort_pipelines.scatter_output)
        else { return Ok(()); };

        let Some(cs_copy_back) =
            pipeline_cache.get_compute_pipeline(count_sort_pipelines.copy_back)
        else { return Ok(()); };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("begin_simulation_compute"),
                ..Default::default()
            });

        
        let dt = stepper.fixed_dt;

        // when starting the simulation, we need to stabilize the simulation
        let delta: f32 = 1.0 / 120.0; //if !self.first_run { 0.0 } else { dt };
        let substeps = 3; //if !self.first_run { 1 } else { stepper.steps_this_frame.max(2) };

        let substeps_delta = delta / substeps as f32;
        //let delta: f32 = 0.004;

        for _i in 0..substeps {
            for (
                _entity, 
                simulation_bg, 
                count_sort_bg, 
                spatial_hash_bg, 
                internal_count_sort_buffers,
                _internal_simulation_buffers,
                params,
            ) in entries.iter(world) {

                let particle_count = params.particle_count;
                let wg_size = (particle_count + 255) / 256;
                let binary_delta = Some(bytemuck::bytes_of(&substeps_delta));

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
                    queue, 
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

        // finaly update density texture
        for (_entity, simulation_bg, density_map) in density_query.iter(world) {
            // skip if no density map is present
            let Some(density_map) = density_map else {
                continue;
            };

            let extent = density_map.extent;

            let gx = (extent.width  + 7) / 8;
            let gy = (extent.height + 7) / 8;
            let gz = (extent.depth_or_array_layers + 7) / 8;

            pass.set_pipeline(update_density);
            pass.set_bind_group(0, &simulation_bg.bind_group, &[]);
            pass.dispatch_workgroups(gx, gy, gz);

        }
        
        Ok(())
    }
}