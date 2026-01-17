use bevy::{
    mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout}, 
    pbr::{MaterialPipeline, MaterialPipelineKey}, 
    prelude::*, 
    render::{
        render_resource::{
            AsBindGroup, 
            RenderPipelineDescriptor, 
            SpecializedMeshPipelineError, 
            VertexStepMode
        }, 
        storage::ShaderStorageBuffer
    }, 
    shader::ShaderRef
};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ParticleMaterial {
    #[storage(0, read_only)]
    pub positions: Handle<ShaderStorageBuffer>,
    #[storage(1, read_only)]
    pub velocities: Handle<ShaderStorageBuffer>,
    
    // for debugging
    #[storage(2, read_only)]
    pub debug_buffer: Handle<ShaderStorageBuffer>,

    #[uniform(3)]
    pub color: Vec4,
    #[uniform(4)]
    pub radius: f32,
}

impl Material for ParticleMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/particles_render.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/particles_render.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers = vec![
            VertexBufferLayout {
                array_stride: 0,
                step_mode: VertexStepMode::Vertex,
                attributes: vec![],
            }
        ];
        Ok(())
    }

}
