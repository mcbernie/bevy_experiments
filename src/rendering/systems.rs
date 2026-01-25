use std::num::NonZeroU64;

/// setup bindgroups and pipelines for fluid rendering
/// 
use bevy::{prelude::*, render::{render_resource::{binding_types::{storage_buffer, storage_buffer_read_only, storage_buffer_read_only_sized, storage_buffer_sized, uniform_buffer, uniform_buffer_sized}, *}, renderer::*}};

use crate::{rendering::assets::{MarchingCubesRenderResources, ModelData, PreparedRenderArgsBindGroup, SimRenderPipeline, ViewData}, simulation::{MarchingCubesBuffers, SimulationParams, marching_cubes::Triangle}};

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

pub fn init_render_pipeline(
    mut commands: Commands,
) {

    // using DrawIndirectArgs struct from WGPU
    // Layout
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "render_args_bind_group_layout",
        &[
            storage_buffer_sized(false, NonZeroU64::new(std::mem::size_of::<DrawIndirectArgs>() as u64)).build(0, ShaderStages::VERTEX | ShaderStages::FRAGMENT),
            storage_buffer_read_only::<u32>(false).build(1, ShaderStages::VERTEX | ShaderStages::FRAGMENT),
        ],
    );

    let model_view_bind_group_layout = BindGroupLayoutDescriptor::new(
        "model_view_bind_group_layout",
        &[
            storage_buffer_read_only_sized(false, NonZeroU64::new(std::mem::size_of::<ViewData>() as u64)).build(0, ShaderStages::VERTEX | ShaderStages::FRAGMENT),
            storage_buffer_read_only_sized(false, NonZeroU64::new(std::mem::size_of::<ModelData>() as u64)).build(1, ShaderStages::VERTEX | ShaderStages::FRAGMENT),
        ],
    );


    // we only create the layouts....
    commands.insert_resource(SimRenderPipeline {
        layout: bind_group_layout,
        model_view_layout: model_view_bind_group_layout,
    });
}


pub fn prepare_marching_cubes_render_resources(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
    existing: Option<Res<MarchingCubesRenderResources>>,

) {

    if existing.is_some() {
        return;
    }

    let shader = asset_server.load("shaders/fluid_marching_cubes.wgsl");

    let model_view_layout_desc = BindGroupLayoutDescriptor::new(
        "model_view_layout_desc",
        &[
            uniform_buffer_sized(false, NonZeroU64::new(std::mem::size_of::<ViewData>() as u64)).build(0, ShaderStages::VERTEX | ShaderStages::FRAGMENT),
            uniform_buffer_sized(false, NonZeroU64::new(std::mem::size_of::<ModelData>() as u64)).build(1, ShaderStages::VERTEX | ShaderStages::FRAGMENT),
        ],
    );

    let model_view_layout = render_device.create_bind_group_layout(
        Some("model_view_layout".into()), 
        &model_view_layout_desc.entries
    );

    let triangle_layout_desc = BindGroupLayoutDescriptor::new(
        "triangle_layout_desc",
        &[
            storage_buffer_read_only::<Triangle>(false).build(0, ShaderStages::VERTEX),
        ],
    );

    let simulation_settings_layout_desc = BindGroupLayoutDescriptor::new(
        "simulation_settings_layout_desc",
        &[
            uniform_buffer::<SimulationParams>(false).build(0, ShaderStages::VERTEX | ShaderStages::COMPUTE),
        ],
    );



    let pipeline_id = pipeline_cache.queue_render_pipeline(
        RenderPipelineDescriptor {
            label: Some("marching_cubes_pipeline".into()),
            layout: vec![
                model_view_layout_desc.clone(),
                triangle_layout_desc.clone(),
                simulation_settings_layout_desc.clone(),
            ],
            vertex: VertexState {
                shader: shader.clone(),
                entry_point: Some("vertex".into()),
                buffers: vec![],
                ..Default::default()
            },
            fragment: Some(FragmentState {
                shader,
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                ..Default::default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                cull_mode: Some(Face::Back),
                front_face: FrontFace::Ccw,
                ..default()
            },
            depth_stencil: None,
            //depth_stencil: Some(DepthStencilState {
            //    format: TextureFormat::Depth32Float,
            //    depth_write_enabled: true,
            //    depth_compare: CompareFunction::Less,
            //    stencil: default(),
            //    bias: default(),
            //}),
            ..Default::default()
        },
    );

    let view_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("marching_cubes_view_uniform"),
        size: std::mem::size_of::<ViewData>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let model_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("marching_cubes_model_uniform"),
        size: std::mem::size_of::<ModelData>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // move it into a component....
    let model_view_bind_group = render_device.create_bind_group(
        "marching_cubes_view_and_model_bind_group",
        &model_view_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: view_buffer.as_entire_binding(),
        },
        BindGroupEntry {
            binding: 1,
            resource: model_buffer.as_entire_binding(),
        }],
    );

    commands.insert_resource(MarchingCubesRenderResources {
        pipeline: pipeline_id,
        view_buffer,
        model_buffer,
        model_view_bind_group,
    });
}
