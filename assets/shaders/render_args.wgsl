@group(0) @binding(0)
var<storage, read_write> render_args: array<u32>;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Index count: output from marching cubes is triangle buffer, where each triangle entry contains 3 vertices.
    // Therefore, the number of indices is 3x the number of elements (which at this stage has been copied into the render args)
    render_args[0] = render_args[0] * 3u;
    render_args[1] = 1u; // Instance count
    render_args[2] = 0u; // Submesh index
    render_args[3] = 0u; // Base vertex
    render_args[4] = 0u; // Offset
}
