#define_import_path simulation::data

@group(0) @binding(0)
var<storage, read_write> positions_in: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> velocities_in: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read_write> positions_out: array<vec4<f32>>;

@group(0) @binding(3)
var<storage, read_write> velocities_out: array<vec4<f32>>;

@group(0) @binding(4)
var<storage, read_write> positions_sorted: array<vec4<f32>>;

@group(0) @binding(5)
var<storage, read_write> velocities_sorted: array<vec4<f32>>;

@group(0) @binding(6)
var<storage, read_write> spatial_keys: array<u32>;

@group(0) @binding(7)
var<storage, read_write> spatial_counts: array<atomic<u32>>;

@group(0) @binding(8)
var<storage, read_write> spatial_offsets: array<u32>;

@group(0) @binding(9)
var<storage, read_write> spatial_sorted_indices: array<u32>;

@group(0) @binding(10)
var<storage, read_write> write_offsets: array<atomic<u32>>;

struct SimulationParams {
    box_size: f32,
    gravity: f32,
    particle_radius: f32,
    cell_size: f32,
    _pad: f32,
};

@group(0) @binding(11)
var<uniform> params: SimulationParams;

struct Push {
    delta_time: f32,
};

var<push_constant> push: Push;


fn cell_from_pos(p: vec3<f32>, cell_size: f32) -> vec3<i32> {
    return vec3<i32>(floor(p / cell_size));
}

// 3D-Hash
fn hash_cell(c: vec3<i32>) -> u32 {
    return u32(
        c.x * 73856093 ^
        c.y * 19349663 ^
        c.z * 83492791
    );
}