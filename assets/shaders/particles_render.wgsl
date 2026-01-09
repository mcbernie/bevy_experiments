// ===============================
// Imports
// ===============================

#import bevy_pbr::forward_io::VertexOutput

#ifdef FORWARD_PIPELINE
    #import bevy_pbr::mesh_view_bindings::mesh
    #import bevy_render::view::view
    #import bevy_pbr::mesh_functions::get_world_from_local
#endif


// ===============================
// Compute / Instancing Buffer
// ===============================

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var<storage, read> positions: array<vec4<f32>>;


// ===============================
// Vertex Input
// ===============================

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};


// ===============================
// Vertex Shader
// ===============================

@vertex
fn vertex(
    in: VertexIn,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

#ifdef FORWARD_PIPELINE
    // --------------------------------
    // Local → World (Bevy Transform)
    // --------------------------------
    let world_pos_mesh =
        get_world_from_local(vec4<f32>(in.position, 1.0));

    // --------------------------------
    // Compute offset (World Space)
    // --------------------------------
    let instance_offset =
        vec4<f32>(positions[instance_index].xyz, 0.0);

    let world_pos = world_pos_mesh + instance_offset;

    // --------------------------------
    // Outputs für Bevy-PBR
    // --------------------------------
    out.world_position = world_pos;

    out.world_normal =
        normalize(
            (mesh.normal_from_local * vec4<f32>(in.normal, 0.0)).xyz
        );

    out.uv = in.uv;

    // Clip Space
    out.position = view.view_proj * world_pos;

#else
    // --------------------------------
    // Fallback für Shadow / Prepass /
    // Depth / andere Varianten
    // --------------------------------
    out.position = vec4<f32>(0.0);
#endif

    return out;
}
