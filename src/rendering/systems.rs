/// setup bindgroups and pipelines for fluid rendering
/// 
use bevy::{prelude::*, render::renderer::*, render::render_resource::*};

#[derive(Resource)]
pub struct FluidPipeline {
    pub pipeline: CachedRenderPipelineId,
}

fn prepare_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    let shader = asset_server.load("fluid_marching_cubes.wgsl");

    let pipeline = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("fluid_pipeline".into()),
        layout: Vec::new(), // ← extrem wichtig
        vertex: VertexState {
            shader: shader.clone(),
            entry_point: Some("vertex_main".into()),
            buffers: vec![],
            shader_defs: vec![],
        },
        fragment: Some(FragmentState {
            shader,
            entry_point: Some("fragment_main".into()),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::bevy_default(),
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            shader_defs: vec![],
        }),
        primitive: PrimitiveState::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
        ..Default::default()

    });

    commands.insert_resource(FluidPipeline {
        pipeline,
    });
}