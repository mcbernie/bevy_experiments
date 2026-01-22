use bevy::{prelude::*, render::{render_graph::RenderLabel, render_resource::*, renderer::RenderContext}};
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext};
use crate::simulation::{components::DensityMap, marching_cubes::{components::MarchingCubesBindGroup, resources::MarchingCubesPipeline}};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MarchingCubesLabel;

#[derive(Default)]
pub struct MarchingCubesNode;

impl Node for MarchingCubesNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<MarchingCubesPipeline>();

        let Some(pipeline) =
            pipeline_cache.get_compute_pipeline(pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_pipeline(pipeline);

        let Some(mut query) = world.try_query::<(&MarchingCubesBindGroup, Option<&DensityMap>)>() 
        else {
            return Ok(());
        };

        for (bind_group, density_map) in query.iter(world) {
            let Some(density_map) = density_map else {
                continue;
            };
            let cubes_x = density_map.extent.width  - 1;
            let cubes_y = density_map.extent.height - 1;
            let cubes_z = density_map.extent.depth_or_array_layers - 1;

            let gx = (cubes_x + 7) / 8;
            let gy = (cubes_y + 7) / 8;
            let gz = (cubes_z + 7) / 8;
            pass.set_bind_group(0, &bind_group.bind_group, &[]);
            pass.dispatch_workgroups(gx, gy, gz);
        }

        Ok(())
    }
}
