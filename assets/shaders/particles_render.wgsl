#import bevy_pbr::{
    mesh_view_types::MeshView,
    vertex_io::VertexOutput,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

@group(0) @binding(0)
var<uniform> mesh_view: MeshView;

// StorageBuffer aus Compute
@group(1) @binding(0)
var<storage, read> positions: array<vec3<f32>>;

struct VertexIn {
    @location(0) position: vec3<f32>, // Quad-Vertex
    @location(1) uv: vec2<f32>,
};

@vertex
fn vertex(
    @builtin(vertex_index) index: u32,
) -> VertexOutput {
    let center = positions[index];

    var out: VertexOutput;
    out.world_position = center;
    out.position = mesh_view.view_proj * vec4<f32>(center, 1.0);
    out.world_normal = vec3<f32>(0.0, 1.0, 0.0);
    out.uv = vec2<f32>(0.0);
    return out;
}


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let pbr = pbr_input_from_standard_material(in, is_front);
    let lit = apply_pbr_lighting(pbr);
    return main_pass_post_lighting_processing(pbr, lit);
}
