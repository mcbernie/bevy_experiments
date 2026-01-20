use core::num;

use bevy::{
    prelude::*,
    render::{render_asset::RenderAssets, render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel}, render_resource::{BindGroup, ComputePassDescriptor, PipelineCache}, renderer::{RenderContext, RenderQueue}, storage::GpuShaderStorageBuffer},
};

use crate::{ReadbackBuffer, WORKGROUP_SIZE, simulation::{assets::SimulationParams, gpu_sort::{components::InternalCountSortBuffers, helper::calc_num_groups}}};
use super::{components::PreparedCountSortComputeBindGroup, resources::CountSortComputePipeline};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct CountSortLabel;

#[derive(Default)]
pub struct CountSortNode;

struct ScanLevel {
    item_count: u32,
    num_groups: u32,
    elements_bg: BindGroup,   // Buffer, der gerade gescannt wird
    group_sums_bg: BindGroup, // passender groupSums-Buffer
}

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

        //let readback_buffer_handle = world.resource::<ReadbackBuffer>();
        //let buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();
        //let Some(readback_buffer_storage) = buffers.get(&readback_buffer_handle.handle) else {
        //    return Ok(());
        //};
        //let readback_buffer = &readback_buffer_storage.buffer; 


        for (_, bg, buffers, params) in bind_groups.iter(world) {
            {
                queue.write_buffer(&buffers.num_items, 0, bytemuck::bytes_of(&params.particle_count));

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
            }

            // scan ->
            // 3) scan

            let mut levels: Vec<ScanLevel> = Vec::new();

            let mut current_item_count = params.particle_count;
            let mut current_elements_bg = bg.scan_bind_group.clone(); // elements

            //let num_groups = calc_num_groups(current_item_count, WORKGROUP_SIZE);
            //let source = buffers.group_sums.get(&num_groups).unwrap();
            //render_context.command_encoder().copy_buffer_to_buffer(
            //    &buffers.counts, 
            //    0, 
            //    &readback_buffer, 
            //    0, 
            //    buffers.counts.size()
            //);

            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("scan_level"),
                    ..Default::default()
                });

            loop {
                let num_groups = calc_num_groups(current_item_count, WORKGROUP_SIZE);

                let group_sums_bg = bg.per_group_count
                    .get(&num_groups)
                    .expect("missing group_sums bind group")
                    .clone();


                // ---- scan ----
                {
                    pass.set_pipeline(&scan);
                    pass.set_push_constants(0, bytemuck::bytes_of(&current_item_count));
                    pass.set_bind_group(0, &current_elements_bg, &[]);
                    pass.set_bind_group(1, &group_sums_bg, &[]);
                    pass.dispatch_workgroups(num_groups, 1, 1);
                }


                // Merke dieses Level
                levels.push(ScanLevel {
                    item_count: current_item_count,
                    num_groups,
                    elements_bg: current_elements_bg.clone(),
                    group_sums_bg: group_sums_bg.clone(),
                });

                // Abbruchbedingung (Unity: if numGroups <= 1)
                if num_groups <= 1 {
                    break;
                }

                // nächstes Level arbeitet auf groupSums
                current_elements_bg = group_sums_bg;
                current_item_count = num_groups;
            }

            // Von oben nach unten, Level 0 überspringen
            for level in levels.iter().rev().skip(1) {
                //let mut pass = render_context
                //    .command_encoder()
                //    .begin_compute_pass(&ComputePassDescriptor {
                //        label: Some("combine_level"),
                //        ..Default::default()
                //    });

                pass.set_pipeline(&combine);
                pass.set_push_constants(0, bytemuck::bytes_of(&level.item_count));
                pass.set_bind_group(0, &level.elements_bg, &[]);
                pass.set_bind_group(1, &level.group_sums_bg, &[]);
                pass.dispatch_workgroups(level.num_groups, 1, 1);
            }

            //let mut pass = render_context
            //    .command_encoder()
            //    .begin_compute_pass(&ComputePassDescriptor {
            //        label: Some("finalize"),
            //        ..Default::default()
            //    });

            // 4) scatter output
            pass.set_pipeline(&scatter_output);
            pass.set_bind_group(0, &bg.sort_bind_group, &[]);
            pass.dispatch_workgroups((params.particle_count + 255) / 256, 1, 1);

            // 5) copy back
            pass.set_pipeline(&copy_back);
            pass.set_bind_group(0, &bg.sort_bind_group, &[]);
            pass.dispatch_workgroups((params.particle_count + 255) / 256, 1, 1);
        }


        Ok(())
    }
}