use bevy::{
    prelude::*, 
    render::{
        render_resource::{
            AsBindGroup
        }, 
        storage::ShaderStorageBuffer
    }, 
    shader::ShaderRef
};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ParticleMaterial {
    #[storage(0, read_only, visibility(vertex))]
    pub positions: Handle<ShaderStorageBuffer>,
}

impl Material for ParticleMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/particles_render.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/particles_render.wgsl".into()
    }
}
