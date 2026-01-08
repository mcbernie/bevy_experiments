
@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

struct ParticlePosition {
    pos: vec3<f32>,
    _pad: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<storage, read> positions: array<ParticlePosition>;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> VSOut {
    var out: VSOut;

    // 6 Vertices = 1 Quad
    let particle_index = index / 6u;
    let corner = index % 6u;

    let center = positions[particle_index].pos;

    // Quad-Corners
    let offsets = array<vec2<f32>, 6>(
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0,  1.0),
    );

    let size = 0.55;
    let o = offsets[corner] * size;

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