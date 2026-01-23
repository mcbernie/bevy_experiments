struct TriangleCounter {
    count: atomic<u32>,
};

struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
};

@group(0) @binding(0)
var<storage, read_write> render_args: DrawIndirectArgs;

@group(0) @binding(1)
var<storage, read> triangle_counter: TriangleCounter;


@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let triangle_count = atomicLoad(&triangle_counter.count);

    render_args.vertex_count   = triangle_count * 3u;
    render_args.instance_count = 1u;
    render_args.first_vertex = 0u; 
    render_args.first_instance = 0u;
}
