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

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var<storage, read> spatial_keys: array<u32>;

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(1) uv: vec2<f32>,          
    @location(2) velocity: vec3<f32>,    
    @location(3) particle_index: u32,    
};

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32
) -> VertexOut {
    let particle_index = vertex_index / 6u;
    let corner = vertex_index % 6u;

    let center = positions[particle_index].xyz;
    let velocity = velocities[particle_index].xyz;

    let size = 0.1;
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
    out.particle_index = particle_index;
    return out;
}


struct FragmentOut {
    @location(0) color: vec4<f32>
};

@fragment
fn fragment(in: VertexOut) -> FragmentOut {
    // we want a sphere, not a square
    // so discard pixels outside the circle
    let p = in.uv * 2.0 - 1.0;
    let r2 = dot(p, p);
    if (r2 > 1.0) {
        discard;
    }

    // normal for a sphere ( fake normal )
    let z = sqrt(1.0 - r2);
    let n = normalize(vec3(p.x, p.y, z));


    // simple light, not using real lighting system
    let light_dir = normalize(vec3(0.3, 0.8, 0.4));
    let diffuse = clamp(dot(n, light_dir), 0.0, 1.0);

    // color based on velocity
    let speed = length(in.velocity);
    let t = clamp(speed / 4.0, 0.0, 1.0);
    let base_color = speed_to_color(t);
    let color = base_color * (0.4 + 0.6 * diffuse);



    let key = spatial_keys[in.particle_index];
    let h_color = hash_color(key);

    var out: FragmentOut;
    out.color = vec4(color, 1.0);
    //out.color = vec4(h_color, 1.0);
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

fn hash_color(k: u32) -> vec3<f32> {
    let r = f32((k * 16807u) & 255u) / 255.0;
    let g = f32((k * 48271u) & 255u) / 255.0;
    let b = f32((k * 69621u) & 255u) / 255.0;
    return vec3(r, g, b);
}