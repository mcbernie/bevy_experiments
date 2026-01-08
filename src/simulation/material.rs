use bevy::{prelude::*, render::{render_resource::AsBindGroup, storage::ShaderStorageBuffer}, shader::ShaderRef};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ParticleMaterial {
    #[storage(0, read_only)]
    pub positions: Handle<ShaderStorageBuffer>,
}

impl Material for ParticleMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/particle_billboard.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/particle_billboard.wgsl".into()
    }
}
