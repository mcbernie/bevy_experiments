struct View {
    view_proj : mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> view : View;

struct Vertex {
    position : vec3<f32>,
    normal   : vec3<f32>,
};

@group(1) @binding(0)
var<storage, read> vertex_buffer : array<Vertex>;

struct MeshUniform {
    model : mat4x4<f32>,
};

@group(2) @binding(0)
var<uniform> mesh : MeshUniform;


struct VertexOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) normal : vec3<f32>,
};


@vertex
fn vertex_main(@builtin(vertex_index) vertex_id : u32) -> VertexOut {
    let v = vertex_buffer[vertex_id];

    var out : VertexOut;
    out.clip_position = view.view_proj * mesh.model * vec4<f32>(v.position, 1.0);
    out.normal = (model * vec4<f32>(v.normal, 0.0)).xyz;
    return out;
}

@fragment
fn fragment_main(in : VertexOut) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.4, 1.0, 0.3)); // feste Weltlichtquelle
    let n = normalize(in.normal);
    let shading = dot(light_dir, n) * 0.5 + 0.5;

    return material_color * shading;
}
