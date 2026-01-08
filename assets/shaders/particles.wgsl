struct SimParams {
    num_particles: u32,
    gravity: f32,
    delta_time: f32,
    _pad: f32,
};

@group(0) @binding(0)
var<storage, read_write> positions: array<vec3<f32>>;

@group(0) @binding(1)
var<storage, read_write> velocities: array<vec3<f32>>;

@group(0) @binding(2)
var<uniform> params: SimParams;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.num_particles) {
        return;
    }

    velocities[i] += vec3<f32>(0.0, params.gravity, 0.0) * params.delta_time;
    positions[i] += velocities[i] * params.delta_time;
}
