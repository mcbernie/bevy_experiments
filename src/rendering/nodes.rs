use bevy::prelude::*;
use bevy::render::render_graph::Node;
use bevy::render::render_graph::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::*;

use crate::rendering::assets::PreparedRenderArgsBindGroup;
use crate::rendering::assets::RenderArgsPipeline;

#[derive(RenderLabel, Hash, PartialEq, Eq, Clone, Debug)]
pub struct GenerateRenderArgsLabel;


#[derive(Default)]
pub struct GenerateRenderArgsNode;


impl Node for GenerateRenderArgsNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<RenderArgsPipeline>();

        let Some(mut bind_groups) = world.try_query::<(Entity, &PreparedRenderArgsBindGroup)>()
        else {
            return Ok(());
        };

        let args = match pipeline_cache.get_compute_pipeline(pipelines.pipeline) {
            Some(p) => p,
            None => return Ok(()),
        };

        let mut encoder = render_context
            .render_device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("generate_render_args_encoder"),
            });

        for (_entity, bg) in bind_groups.iter(world) {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("generate_render_args_pass"),
                ..Default::default()
            });
            pass.set_pipeline(&args);
            pass.set_bind_group(0, &bg.bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }


        Ok(())
    }
}