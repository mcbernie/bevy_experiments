use std::num::NonZeroU64;

/// setup bindgroups and pipelines for fluid rendering
/// 
use bevy::{prelude::*, render::{render_resource::{binding_types::{storage_buffer, storage_buffer_read_only, storage_buffer_sized}, *}, renderer::*}};

use crate::{rendering::assets::PreparedRenderArgsBindGroup, simulation::MarchingCubesBuffers};

use super::assets::RenderArgsPipeline;
use super::assets::FluidIndirectArgsBuffer;

#[derive(Resource)]
pub struct FluidPipeline {
    pub pipeline: CachedRenderPipelineId,
}

pub fn init_drawing_args_buffer(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    query: Query<(Entity, &MarchingCubesBuffers), Without<FluidIndirectArgsBuffer>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, _mc_buffers) in &query {

        // create a buffer with the size of DrawIndirectArgs from WGPU struct
        let indirect_args_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("fluid_indirect_args_buffer"),
            size: std::mem::size_of::<DrawIndirectArgs>() as u64,
            usage: BufferUsages::INDIRECT | BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        commands.entity(entity).insert(FluidIndirectArgsBuffer {
            buffer: indirect_args_buffer,
        });
    }
}

// this pipeline / shader count all vertices to draw
// simple but very fast
pub fn init_drawing_args_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {

    // using DrawIndirectArgs struct from WGPU
    // Layout
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "compute_bind_group_layout",
        &[
            storage_buffer_sized(false, NonZeroU64::new(std::mem::size_of::<DrawIndirectArgs>() as u64)).build(0, ShaderStages::COMPUTE),
            storage_buffer_read_only::<u32>(false).build(1, ShaderStages::COMPUTE),
        ],
    );

    let shader = asset_server.load("shaders/render_args.wgsl");

    let pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("render_args_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: shader.clone(),
            entry_point: Some("main".into()),
            ..Default::default()
        }
    );

    commands.insert_resource(RenderArgsPipeline {
        layout: bind_group_layout,
        pipeline,
    });
}

pub fn prepare_drawing_args(
    mut commands: Commands,
    pipeline: Res<RenderArgsPipeline>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    query: Query<(Entity, &FluidIndirectArgsBuffer, &MarchingCubesBuffers), With<FluidIndirectArgsBuffer>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, buffers, mc_buffers) in &query {

        let indirect_args_buffer = &buffers.buffer;

        // we need to create this for each pipeline...?
        let bind_group = render_device.create_bind_group(
            "simulation_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.layout),
            &vec![
                BindGroupEntry {
                    binding: 0,
                    resource: indirect_args_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: mc_buffers.counter_buffer.as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert(
            (
                PreparedRenderArgsBindGroup { bind_group },
            )
        );
    }

}