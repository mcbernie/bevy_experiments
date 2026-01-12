#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    view_transformations::position_world_to_clip,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<storage, read> positions: array<vec4<f32>>;

fn apply_instance_offset(
    vertex: Vertex,
) -> mat4x4<f32> {
    let offset = positions[vertex.instance_index].xyz;

    var world_from_local =
        mesh_functions::get_world_from_local(vertex.instance_index);

    world_from_local[3] = vec4<f32>(offset, 0.0);

    return world_from_local;
}


@vertex
fn vertex(
    vertex_no_morph: Vertex,
) -> VertexOutput {
    var out: VertexOutput;

    let offset = positions[vertex_no_morph.instance_index].xyz;
    var vertex = vertex_no_morph;
    vertex.position = vertex.position + offset;

    let world_from_local = apply_instance_offset(vertex);

    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        0,
    );

    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    out.uv = vertex.uv;
    out.instance_index = vertex.instance_index;

    return out;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool
) -> @location(0) vec4<f32> {

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    var pos = positions[in.instance_index].xyz;
    return vec4<f32>(pos, 1.0);
    //return vec4<f32>(0.1,0.8,0.0, 1.0);
}