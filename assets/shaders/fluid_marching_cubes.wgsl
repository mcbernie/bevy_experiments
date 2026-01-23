// -----------------------------------------------------------------------------
// Uniforms
// -----------------------------------------------------------------------------

struct ViewData {
    clip_from_world: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> view: ViewData;

struct ModelData {
    model: mat4x4<f32>,
};

@group(0) @binding(1)
var<uniform> model: ModelData;

// -----------------------------------------------------------------------------
// Geometry storage buffer
// -----------------------------------------------------------------------------

struct Triangle {
    a: vec4<f32>,
    b: vec4<f32>,
    c: vec4<f32>,
};

@group(1) @binding(0)
var<storage, read> triangles: array<Triangle>;

// -----------------------------------------------------------------------------
// Vertex output
// -----------------------------------------------------------------------------

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
};

// -----------------------------------------------------------------------------
// Vertex shader
// -----------------------------------------------------------------------------

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    let tri_index = vertex_index / 3u;
    let vert_in_tri = vertex_index % 3u;

    let tri = triangles[tri_index];

    var local_pos: vec3<f32>;
    if (vert_in_tri == 0u) {
        local_pos = tri.a.xyz;
    } else if (vert_in_tri == 1u) {
        local_pos = tri.b.xyz;
    } else {
        local_pos = tri.c.xyz;
    }

    let world_pos = model.model * vec4<f32>(local_pos, 1.0);

    var out: VertexOut;
    out.clip_position = view.clip_from_world * world_pos;
    return out;
}

// -----------------------------------------------------------------------------
// Fragment shader
// -----------------------------------------------------------------------------

@fragment
fn fragment() -> @location(0) vec4<f32> {
    // simple debug color
    return vec4<f32>(0.9, 0.9, 0.9, 1.0);
}
