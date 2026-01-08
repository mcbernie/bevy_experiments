use bevy::{
    pbr::MaterialExtension, prelude::*, reflect::TypePath, render::render_resource::AsBindGroup, shader::ShaderRef
};

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct CheckerMaterial {
    #[uniform(100)]
    pub scale: f32,
    #[uniform(101)]
    pub color_a: LinearRgba,
    #[uniform(102)]
    pub color_b: LinearRgba,
}

impl MaterialExtension for CheckerMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/checker.wgsl".into()
    }
}