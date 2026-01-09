#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_functions::{
    mesh_position_local_to_world,
    mesh_position_local_to_clip,
    mesh_normal_local_to_world
}
#import bevy_pbr::mesh_view_bindings::{
    model,
    normal_matrix
}

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var<storage, read> positions: array<vec4<f32>>;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
};

@vertex
fn vertex(
    in: VertexIn,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let local_pos = vec4<f32>(in.position, 1.0);

    // 1. Local → World (Bevy Model Matrix)
    let world_pos =
        mesh_position_local_to_world(local_pos, model);

    // 2. Compute Offset (World Space)
    let offset =
        vec4<f32>(positions[instance_index].xyz, 0.0);

    let final_world_pos = world_pos + offset;

    // 3. Pflichtfelder für PBR
    out.world_position = final_world_pos;

    out.world_normal =
        normalize(mesh_normal_local_to_world(in.normal, normal_matrix));

#ifdef VERTEX_UVS_A
    out.uv = in.uv;
#endif

    // 4. Clip Space
    out.position =
        mesh_position_local_to_clip(local_pos, model) +
        vec4<f32>(offset.xyz, 0.0);

    return out;
}
