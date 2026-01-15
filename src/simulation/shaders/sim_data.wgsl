#define_import_path simulation::data

@group(0) @binding(0)
var<storage, read_write> positions: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> velocities: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read_write> spatial_keys: array<u32>;

@group(0) @binding(3)
var<storage, read_write> spatial_counts: array<u32>;

@group(0) @binding(4)
var<storage, read_write> spatial_offsets: array<u32>;

struct SimulationParams {
    box_size: f32,
    gravity: f32,
    particle_radius: f32,
    cell_size: f32,
    _pad: f32,
};

@group(0) @binding(5)
var<uniform> params: SimulationParams;

struct Push {
    delta_time: f32,
};

var<push_constant> push: Push;