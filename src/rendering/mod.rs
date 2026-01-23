/// MarchingCubes 
/// 
/// we need the **densityMap** from the simulation pass to gernerate the mesh
/// and the **scale** 2
/// 
/// 

use bevy::render::render_graph::RenderLabel;

mod systems;
mod nodes;
mod assets;

pub use systems::{init_drawing_args_buffer, init_drawing_args_pipeline, prepare_drawing_args};
pub use systems::{init_render_pipeline, prepare_marching_cubes_render_resources};
pub use nodes::{GenerateRenderArgsNode, GenerateRenderArgsLabel, MarchingCubesRenderNode};


#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MarchingCubesRenderingLabel;