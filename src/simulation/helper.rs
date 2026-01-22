use bevy::{
    asset::Handle, ecs::system::Res, prelude::*, render::{render_resource::*, renderer::RenderDevice}, shader::Shader
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
        

pub fn dispatch_compute(
    pass: &mut ComputePass,
    pipeline: &ComputePipeline,
    bind_groups: &[&BindGroup],
    push_constants: Option<&[u8]>,
    workgroups: u32,
) {
    pass.set_pipeline(pipeline);

    for (i, bg) in bind_groups.iter().enumerate() {
        pass.set_bind_group(i as u32, *bg, &[]);
    }

    if let Some(bytes) = push_constants {
        pass.set_push_constants(0, bytes);
    }

    pass.dispatch_workgroups(workgroups, 1, 1);
}

pub fn random_in_unit_sphere(rng: &mut impl rand::Rng) -> Vec3 {
    loop {
        let v = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );
        if v.length_squared() <= 1.0 {
            return v;
        }
    }
}

/// create a 3D texture for density map storage
pub fn create_density_texture(
    device: &RenderDevice,
    size: Extent3d,
) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("density_map"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R16Float,
        usage: TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}