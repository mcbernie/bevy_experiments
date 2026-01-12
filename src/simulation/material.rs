use bevy::{
    pbr::MaterialExtension, prelude::*, render::{
        render_resource::AsBindGroup, 
        storage::ShaderStorageBuffer
    }, shader::ShaderRef
};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ParticleMaterial {
    #[storage(100, read_only)]
    pub positions: Handle<ShaderStorageBuffer>,
    #[storage(101, read_only)]
    pub velocities: Handle<ShaderStorageBuffer>,
}

impl MaterialExtension for ParticleMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/particles_render.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/particles_render.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "shaders/particles_prepass.wgsl".into()
    }
}
