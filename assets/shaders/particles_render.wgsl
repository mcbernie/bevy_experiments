#import bevy_render::view::View

const QUAD_OFFSETS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(-0.5, -0.5),
    vec2( 0.5, -0.5),
    vec2( 0.5,  0.5),

    vec2(-0.5, -0.5),
    vec2( 0.5,  0.5),
    vec2(-0.5,  0.5),
);


@group(0) @binding(0)
var<uniform> view: View;

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<storage, read> positions: array<vec4<f32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var<storage, read> velocities: array<vec4<f32>>;

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(1) uv: vec2<f32>,          
    @location(2) velocity: vec3<f32>,    
};

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOut {
    let particle_index = vertex_index / 6u;
    let corner = vertex_index % 6u;

    let center = positions[particle_index].xyz;
    let velocity = velocities[particle_index].xyz;

    let size = 0.05;
    let offset_2d = QUAD_OFFSETS[corner] * size;

    // Kameraachsen
    let right = normalize(view.world_from_view[0].xyz);
    let up    = normalize(view.world_from_view[1].xyz);

    let world_pos =
        center
        + right * offset_2d.x
        + up    * offset_2d.y;

    var out: VertexOut;
    out.uv = QUAD_OFFSETS[corner] + vec2(0.5);
    out.velocity = velocity;
    out.clip_pos = view.clip_from_world * vec4(world_pos, 1.0);
    return out;
}


struct FragmentOut {
    @location(0) color: vec4<f32>
};

@fragment
fn fragment(in: VertexOut) -> FragmentOut {
    // Fake-Normal (Kugelannahme aus UV)
    let n = normalize(vec3(in.uv * 2.0 - 1.0, 1.0));

    // Simple Lichtannahme (von oben vorne)
    let light_dir = normalize(vec3(0.3, 0.8, 0.4));
    let diffuse = clamp(dot(n, light_dir), 0.0, 1.0);

    // Geschwindigkeit → Farbe
    let speed = length(in.velocity);
    let t = clamp(speed / 4.0, 0.0, 1.0);
    let base_color = speed_to_color(t);

    // Wasserartiger Look
    let color = base_color * (0.4 + 0.6 * diffuse);

    var out: FragmentOut;
    out.color = vec4(color, 0.9);
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
