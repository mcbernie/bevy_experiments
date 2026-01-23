/// MarchingCubes 
/// 
/// we need the **densityMap** from the simulation pass to gernerate the mesh
/// and the **scale** 2
/// 
/// 

use bevy::{prelude::*, render::{render_graph::{Node, NodeRunError, RenderGraphContext, RenderLabel}, render_resource::*, renderer::{RenderContext, RenderQueue}}};

mod systems;
mod nodes;
mod assets;

pub use systems::{init_drawing_args_buffer, init_drawing_args_pipeline, prepare_drawing_args};
pub use nodes::{GenerateRenderArgsNode, GenerateRenderArgsLabel};


#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MarchingCubesRenderingLabel;

/* 
#[derive(Default)]
pub struct MarchingCubesRenderingNode;

impl Node for MarchingCubesRenderingNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let queue = world.resource::<RenderQueue>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let gpu = world.resource::<MarchingCubesGpu>();
        let render_args_pipeline = world.resource::<RenderArgsPipeline>();

        let marching = match pipeline_cache.get_compute_pipeline(gpu.marching_pipeline) {
            Some(p) => p,
            None => return Ok(()),
        };

        let args = match pipeline_cache.get_compute_pipeline(gpu.args_pipeline) {
            Some(p) => p,
            None => return Ok(()),
        };

        let mut encoder = render_context
            .render_device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("marching_cubes"),
            });

        // 1. Clear counter
        encoder.clear_buffer(&gpu.triangle_count, 0, None);

        // 2. Marching Cubes Compute
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            pass.set_pipeline(marching);
            pass.set_bind_group(0, &gpu.bind_group, &[]);
            pass.dispatch_workgroups(32, 32, 32);
        }

        // 3. Build indirect args (triangle_count * 3)
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            pass.set_pipeline(args);
            pass.set_bind_group(0, &gpu.bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }

        // 4. Draw
        {
            let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("marching_cubes_draw"),
                ..Default::default()
            });

            pass.set_vertex_buffer(0, gpu.triangle_buffer.slice(..));
            pass.draw_indirect(&gpu.indirect_args, 0);
        }

        queue.submit(Some(encoder.finish()));
        Ok(())
    }
}
    */