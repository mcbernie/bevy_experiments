// -----------------------------------------------------------------------------
// Uniforms
// -----------------------------------------------------------------------------

struct ViewData {
    clip_from_world: mat4x4<f32>,
    world_from_view: mat4x4<f32>,
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

struct Vertex {
    position: vec4<f32>,
    normal: vec4<f32>,
};

struct Triangle {
    a: Vertex,
    b: Vertex,
    c: Vertex,
};

@group(1) @binding(0)
var<storage, read> triangles: array<Triangle>;

struct SimParams {
    num_particles : u32,
    gravity : f32,
    smoothing_radius : f32,
    target_density : f32,
    pressure_multiplier : f32,
    near_pressure_multiplier : f32,
    collision_damping : f32,
    viscosity_strength : f32,
    bounds_size : vec3<f32>,
    spiky_pow_two : f32,
    spiky_pow_three : f32,
    spiky_pow_two_grad: f32,
    spiky_pow_three_grad: f32,
};

@group(2) @binding(0) var<uniform> params : SimParams;

// -----------------------------------------------------------------------------
// Vertex output
// -----------------------------------------------------------------------------

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

// -----------------------------------------------------------------------------
// Vertex shader
// -----------------------------------------------------------------------------

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOut {
    let tri_index = vertex_index / 3u;
    let vert      = vertex_index % 3u;

    let tri = triangles[tri_index];

    var world_pos: vec4<f32>;
    var normal: vec4<f32>;
    if (vert == 0u) {
        world_pos = tri.a.position;
        normal = tri.a.normal;
    } else if (vert == 1u) {
        world_pos = tri.b.position;
        normal = tri.b.normal;
    } else {
        world_pos = tri.c.position;
        normal = tri.c.normal;
    }

    var out: VertexOut;
    world_pos = vec4<f32>(world_pos.xyz, 1.0);
    out.clip_position = view.clip_from_world * world_pos;
    out.world_pos = world_pos.xyz;
    out.normal = normalize(normal.xyz);
    //out.clip_position = view.clip_from_world * world_pos;
    return out;
}





// -----------------------------------------------------------------------------
// Fragment shader
// -----------------------------------------------------------------------------

@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    let N = normalize(in.normal);
    let V = normalize(view.world_from_view[3].xyz - in.world_pos);

    // Licht (for testing)
    let light_dir = normalize(vec3<f32>(-0.4, -1.0, -0.2));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);

    let NdotL = max(dot(N, -light_dir), 0.0);
    let diffuse = NdotL * light_color;

    // Fresnel 
    let fresnel = pow(1.0 - max(dot(N, V), 0.0), 5.0);

    // Wasserfarbe
    let deep_water = vec3<f32>(0.0, 0.15, 0.35);
    let shallow_water = vec3<f32>(0.1, 0.4, 0.6);

    let color = mix(deep_water, shallow_water, diffuse)
              + fresnel * vec3<f32>(0.6, 0.8, 1.0);

    // Transparenz
    let alpha = 0.4 + fresnel * 0.4;

    return vec4<f32>(color, alpha);
}