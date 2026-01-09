
@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;


@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<storage, read> positions: array<vec4<f32>>;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VSOut {
    var out: VSOut;

     if (instance_index >= arrayLength(&positions)) {
        out.pos = vec4<f32>(0.0);
        return out;
    }

    // Partikelposition aus Compute-Buffer
    let center = positions[instance_index].xyz;

    // Quad-Corners
    let offsets = array<vec2<f32>, 6>(
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0,  1.0),
    );

    let size = 0.05; // MUSS zur Physik passen
    let o = offsets[vertex_index % 6u] * size;

    let world = vec4<f32>(
        center.x + o.x,
        center.y + o.y,
        center.z,
        1.0
    );

    out.pos = view_proj * world;
    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}