use bevy::{
    asset::Handle,
    ecs::system::Res,
    render::render_resource::{
        BindGroupLayoutDescriptor,
        CachedComputePipelineId,
        ComputePipelineDescriptor,
        PipelineCache,
        PushConstantRange,
        ShaderStages,
    },
    shader::Shader,
};


/// Hilfsfunktion zum Erstellen von Compute Pipelines
#[derive(Default, Clone)]
pub struct CreatePipelineArgs {
    pub for_shader: Handle<Shader>,
    pub name: &'static str,
    pub entry_point: &'static str,
    pub bind_group_layout: BindGroupLayoutDescriptor,
    pub with_delta_push: bool,
}

pub fn create_pipeline(
    args: CreatePipelineArgs,
    pipeline_cache: &Res<PipelineCache>,
) -> CachedComputePipelineId {

    let CreatePipelineArgs {
        for_shader,
        name,
        entry_point,
        bind_group_layout,
        with_delta_push,
    } = args;

    let mut final_name = name.to_string();
    if name.is_empty() {
        final_name = format!("{}_pipeline", entry_point).into();
    }

    let mut descriptior = ComputePipelineDescriptor {
        label: Some(final_name.into()),
        layout: vec![bind_group_layout.clone()],
        shader: for_shader.clone(),
        entry_point: Some(entry_point.into()),
        ..Default::default()
    };

    if with_delta_push {
        // füge einen Push Constant Bereich für delta_time hinzu
        descriptior.push_constant_ranges = vec![
            PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..4,
            }
        ];
    }

    pipeline_cache.queue_compute_pipeline(
        descriptior
    )
}
        