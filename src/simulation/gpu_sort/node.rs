use bevy::{
    prelude::*,
    render::{render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel}, render_resource::{ComputePassDescriptor, PipelineCache}, renderer::{RenderContext, RenderQueue}},
};
use bevy_inspector_egui::bevy_egui::render;

use crate::{PARTICLE_COUNT, WORKGROUP_SIZE, simulation::{assets::SimulationParams, gpu_sort::{components::InternalCountSortBuffers, helper::calc_num_groups}}};
use super::{components::PreparedCountSortComputeBindGroup, resources::CountSortComputePipeline};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct CountSortLabel;

#[derive(Default)]
pub struct CountSortNode;

impl Node for CountSortNode {

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let queue = world.resource::<RenderQueue>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<CountSortComputePipeline>();

        let Some(mut bind_groups) = world.try_query::<(Entity, &PreparedCountSortComputeBindGroup, &InternalCountSortBuffers, &SimulationParams)>()
        else {
            return Ok(());
        };

        let Some(clear_counts) =
            pipeline_cache.get_compute_pipeline(pipeline.clear_counts)
        else { return Ok(()); };

        let Some(count) =
            pipeline_cache.get_compute_pipeline(pipeline.count)
        else { return Ok(()); };

        let Some(scan) =
            pipeline_cache.get_compute_pipeline(pipeline.scan)
        else { return Ok(()); };

        let Some(combine) =
            pipeline_cache.get_compute_pipeline(pipeline.combine)
        else { return Ok(()); };

        let Some(scatter_output) =
            pipeline_cache.get_compute_pipeline(pipeline.scatter_output)
        else { return Ok(()); };

        let Some(copy_back) =
            pipeline_cache.get_compute_pipeline(pipeline.copy_back)
        else { return Ok(()); };



        for (_, bg, buffers, params) in bind_groups.iter(world) {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("count_sort_compute_pass"),
                    ..Default::default()
                });

            // 1) clear counts
            pass.set_bind_group(0, &bg.sort_bind_group, &[]);
            pass.set_pipeline(&clear_counts);
            pass.dispatch_workgroups((params.particle_count + 255) / 256, 1, 1);
            
            // 2) count
            pass.set_bind_group(0, &bg.sort_bind_group, &[]);
            pass.set_pipeline(&count);
            pass.dispatch_workgroups((params.particle_count + 255) / 256, 1, 1);

            // scan ->
            // 3) scan

            let mut current_count = params.particle_count;

            loop {
                let num_groups = calc_num_groups(current_count, WORKGROUP_SIZE);

                // ---- Scan ----
                pass.set_pipeline(&scan);
                pass.set_bind_group(0, &bg.scan_bind_group, &[]);
                pass.set_bind_group(1, bg.per_group_count.get(&num_groups).unwrap(), &[]);
                
                let count: u32 = current_count;
                queue.write_buffer(&buffers.num_items, 0, bytemuck::bytes_of(&count));

                pass.set_bind_group(2, &bg.count_bind_group, &[]);
                pass.dispatch_workgroups(num_groups, 1, 1);

                if num_groups <= 1 {
                    break;
                }

                // ---- Recurse on group sums ----
                current_count = num_groups;

                // ---- Combine ----
                pass.set_pipeline(&combine);
                pass.set_bind_group(0, &bg.scan_bind_group, &[]);
                pass.set_bind_group(1, bg.per_group_count.get(&num_groups).unwrap(), &[]);

                let count: u32 = params.particle_count;
                queue.write_buffer(&buffers.num_items, 0, bytemuck::bytes_of(&count));

                pass.set_bind_group(2, &bg.count_bind_group, &[]);
                pass.dispatch_workgroups(num_groups, 1, 1);
            }

            let count: u32 = params.particle_count;
            queue.write_buffer(&buffers.num_items, 0, bytemuck::bytes_of(&count));

            // 4) scatter output
            pass.set_pipeline(&scatter_output);
            pass.set_bind_group(0, &bg.sort_bind_group, &[]);

            // 5) copy back
            pass.set_pipeline(&copy_back);
            pass.set_bind_group(0, &bg.sort_bind_group, &[]);

        }


        Ok(())
    }
}