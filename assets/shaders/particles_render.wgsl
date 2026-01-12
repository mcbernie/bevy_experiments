#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    skinning,
    morph::morph,
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    view_transformations::position_world_to_clip,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<storage, read> positions: array<vec4<f32>>;

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var<storage, read> velocities: array<vec4<f32>>;

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
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    var velocity = velocities[in.instance_index].xyz;
    let dir = normalize(velocity);
    let speed = length(velocity);

    let t = clamp(speed / 4.0, 0.0, 1.0);

    let color = speed_to_color(t);
    out.color = vec4<f32>(color.r, color.g, color.b, 1.0);
    // we can optionally modify the lit color before post-processing is applied
    //out.color = vec4<f32>(out.color.x, out.color.y, map(pos.x,0.0, 1.0), 1.0);
        // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    //out.color = out.color * 2.0;
#endif

    return out;
}

fn speed_to_color(t: f32) -> vec3<f32> {
    let c0 = vec3<f32>(0.85, 0.95, 1.0); // weißblau (fast still)
    let c1 = vec3<f32>(1.0, 1.0, 0.0);  // gelb (mittel)
    let c2 = vec3<f32>(1.0, 0.0, 0.0);  // rot (schnell)

    if (t < 0.5) {
        // 0.0 → 0.5 : weißblau → gelb
        let k = t / 0.5;
        return mix(c0, c1, k);
    } else {
        // 0.5 → 1.0 : gelb → rot
        let k = (t - 0.5) / 0.5;
        return mix(c1, c2, k);
    }
}
