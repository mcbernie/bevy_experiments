use bevy::camera::Viewport;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::Node;
use bevy::render::render_graph::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::*;
use bevy::render::storage::GpuShaderStorageBuffer;
use bevy::render::view::ExtractedView;
use bevy::render::view::ViewDepthTexture;
use bevy::render::view::ViewTarget;
use bevy::render::render_resource::LoadOp;

use crate::ReadbackBuffer;
use crate::rendering::assets::FluidIndirectArgsBuffer;
use crate::rendering::assets::MarchingCubesRenderResources;
use crate::rendering::assets::ModelData;
use crate::rendering::assets::PreparedRenderArgsBindGroup;
use crate::rendering::assets::RenderArgsPipeline;
use crate::rendering::assets::ViewData;
use crate::simulation::MarchingCubesBuffers;
use crate::simulation::SimulationUniform;
use crate::simulation::marching_cubes::MarchingCubesBindGroup;
use crate::simulation::marching_cubes::Triangle;

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

        let encoder = render_context.command_encoder();

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


#[derive(Default)]
pub struct MarchingCubesRenderNode;

impl ViewNode for MarchingCubesRenderNode {
    type ViewQuery = (
        &'static ExtractedView,
        &'static ViewTarget,
        &'static ViewDepthTexture,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view, target, depth): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {

        // ------------------------------------------------------------
        // Global render resources
        // ------------------------------------------------------------
        let render_resources = world.resource::<MarchingCubesRenderResources>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let render_queue = world.resource::<RenderQueue>();

        let Some(pipeline) =
            pipeline_cache.get_render_pipeline(render_resources.pipeline)
        else {
            return Ok(()); // Pipeline noch nicht ready
        };

        // ------------------------------------------------------------
        // View / Model Uniforms aktualisieren
        // ------------------------------------------------------------
        let clip_from_world = view.clip_from_world.unwrap_or_else(|| {
            view.clip_from_view * view.world_from_view.to_matrix().inverse()
        });

        let view_data = ViewData {
            clip_from_world: clip_from_world.to_cols_array_2d(),
            world_from_view: view.world_from_view.to_matrix().to_cols_array_2d(),
        };

        render_queue.write_buffer(
            &render_resources.view_buffer,
            0,
            bytemuck::bytes_of(&view_data),
        );

        // erstmal Identität – später Chunk-Transform möglich
        let model_data = ModelData {
            model: Mat4::IDENTITY.to_cols_array_2d(),
        };

        render_queue.write_buffer(
            &render_resources.model_buffer,
            0,
            bytemuck::bytes_of(&model_data),
        );

        // ------------------------------------------------------------
        // Render pass
        // ------------------------------------------------------------
        

        // ------------------------------------------------------------
        // Alle Marching-Cubes-Entities rendern
        // ------------------------------------------------------------
        let Some(mut query) = world.try_query::<(
            &MarchingCubesBindGroup,
            &MarchingCubesBuffers,
            &FluidIndirectArgsBuffer,
            &SimulationUniform
        )>() else {
            return Ok(());
        };

        //let readback_buffer_handle = world.resource::<ReadbackBuffer>();
        //let buffers = world.resource::<RenderAssets<GpuShaderStorageBuffer>>();
        //let Some(readback_buffer_storage) = buffers.get(&readback_buffer_handle.handle) else {
        //    return Ok(());
        //};
        //let readback_buffer = &readback_buffer_storage.buffer; 

        for (mc_bind_groups, buffers, indirect_args, simulation_uniform) in query.iter(world) {
            

            //render_context.command_encoder().copy_buffer_to_buffer(
            //    &buffers.triangle_buffer, 
            //    0, 
            //    &readback_buffer, 
            //    0, 
            //    100 * std::mem::size_of::<Triangle>() as u64
            //);

            let mut pass = render_context.begin_tracked_render_pass(
            RenderPassDescriptor {
                label: Some("marching_cubes_indirect_pass"),
                color_attachments: &[Some(target.get_color_attachment())],
                //color_attachments: &[Some(RenderPassColorAttachment {
                //    view: target.main_texture_view(),
                //    resolve_target: None,
                //    ops: Operations {
                //        //load: LoadOp::Clear(LinearRgba::BLACK.into()),
                //        load: LoadOp::Load,
                //        store: StoreOp::Store,
                //    },
                //    depth_slice: None,
                //})],
                depth_stencil_attachment: None,
                //depth_stencil_attachment: Some(
                //    depth.get_attachment(StoreOp::Store),
                //),
                timestamp_writes: None,
                occlusion_query_set: None,
            },
            );

            let vp = &view.viewport;
            let viewport = Viewport {
                physical_position: UVec2::new(vp.x, vp.y),
                physical_size: UVec2::new(vp.z, vp.w),
                depth: 0.0..1.0,
            };
            pass.set_camera_viewport(&viewport);

            pass.set_render_pipeline(pipeline);

            // BindGroup 0: View + Model
            pass.set_bind_group(
                0,
                &render_resources.model_view_bind_group,
                &[],
            );
            // BindGroup 1: Triangles (storage buffer)
            pass.set_bind_group(
                1,
                &mc_bind_groups.single_triangle_bind_group,
                &[],
            );

            pass.set_bind_group(
                2,
                &simulation_uniform.bind_group,
                &[],
            );

            pass.draw_indirect(&indirect_args.buffer, 0);
            //pass.draw(0..480, 0..1);

        }

        Ok(())
    }
}