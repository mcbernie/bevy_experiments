#define_import_path simulation::math

const PI: f32 = 3.14159265359;

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

@group(0) @binding(7) var<uniform> params : SimParams;

fn get_cell_3d(pos: vec3<f32>, r: f32) -> vec3<i32> {
    return vec3<i32>(floor(pos / r));
}

fn hash_cell_3d(cell: vec3<i32>) -> u32 {
    let blockSize : u32 = 50u;
    let ucell = vec3<u32>(cell + vec3<i32>(25));
    let local = ucell % blockSize;
    let block = ucell / blockSize;
    return local.x + blockSize * (local.y + blockSize * local.z)
        + block.x * 15823u + block.y * 9737333u + block.z * 440817757u;
}

fn key_from_hash(h: u32, size: u32) -> u32 {
    return h % size;
}

const OFFSETS_3D : array<vec3<i32>, 27> = array<vec3<i32>, 27>(
    vec3<i32>(-1, -1, -1),
    vec3<i32>( 0, -1, -1),
    vec3<i32>( 1, -1, -1),

    vec3<i32>(-1,  0, -1),
    vec3<i32>( 0,  0, -1),
    vec3<i32>( 1,  0, -1),

    vec3<i32>(-1,  1, -1),
    vec3<i32>( 0,  1, -1),
    vec3<i32>( 1,  1, -1),

    vec3<i32>(-1, -1,  0),
    vec3<i32>( 0, -1,  0),
    vec3<i32>( 1, -1,  0),

    vec3<i32>(-1,  0,  0),
    vec3<i32>( 0,  0,  0),
    vec3<i32>( 1,  0,  0),

    vec3<i32>(-1,  1,  0),
    vec3<i32>( 0,  1,  0),
    vec3<i32>( 1,  1,  0),

    vec3<i32>(-1, -1,  1),
    vec3<i32>( 0, -1,  1),
    vec3<i32>( 1, -1,  1),

    vec3<i32>(-1,  0,  1),
    vec3<i32>( 0,  0,  1),
    vec3<i32>( 1,  0,  1),

    vec3<i32>(-1,  1,  1),
    vec3<i32>( 0,  1,  1),
    vec3<i32>( 1,  1,  1),
);


// ------------------------------------------------------------
// Linear Kernel
// ------------------------------------------------------------
fn linear_kernel(dst: f32, radius: f32) -> f32 {
    if (dst < radius) {
        return 1.0 - dst / radius;
    }
    return 0.0;
}

// ------------------------------------------------------------
// Poly6 Smoothing Kernel
// ------------------------------------------------------------
fn smoothing_kernel_poly6(dst: f32, radius: f32) -> f32 {
    if (dst < radius) {
        let scale = 315.0 / (64.0 * PI * pow(abs(radius), 9.0));
        let v = radius * radius - dst * dst;
        return v * v * v * scale;
    }
    return 0.0;
}

// ------------------------------------------------------------
// Spiky Kernel (power 3)
// ------------------------------------------------------------
fn spiky_kernel_pow3(dst: f32, radius: f32) -> f32 {
    if (dst < radius) {
        let v = radius - dst;
        return v * v * v * params.spiky_pow_three;
    }
    return 0.0;
}

// ------------------------------------------------------------
// Spiky Kernel (power 2)
// ------------------------------------------------------------
fn spiky_kernel_pow2(dst: f32, radius: f32) -> f32 {
    if (dst < radius) {
        let v = radius - dst;
        return v * v * params.spiky_pow_two;
    }
    return 0.0;
}

// ------------------------------------------------------------
// Derivative Spiky (power 3)
// ------------------------------------------------------------
fn derivative_spiky_pow3(dst: f32, radius: f32) -> f32 {
    if (dst <= radius) {
        let v = radius - dst;
        return -v * v * params.spiky_pow_three_grad;
    }
    return 0.0;
}

// ------------------------------------------------------------
// Derivative Spiky (power 2)
// ------------------------------------------------------------
fn derivative_spiky_pow2(dst: f32, radius: f32) -> f32 {
    if (dst <= radius) {
        let v = radius - dst;
        return -v * params.spiky_pow_two_grad;
    }
    return 0.0;
}

// ------------------------------------------------------------
// Density Kernel (aktuell: SpikyPow2)
// ------------------------------------------------------------
fn density_kernel(dst: f32, radius: f32) -> f32 {
    // return smoothing_kernel_poly6(dst, radius);
    return spiky_kernel_pow2(dst, radius);
}

// ------------------------------------------------------------
// Near Density Kernel
// ------------------------------------------------------------
fn near_density_kernel(dst: f32, radius: f32) -> f32 {
    return spiky_kernel_pow3(dst, radius);
}

// ------------------------------------------------------------
// Density Derivative
// ------------------------------------------------------------
fn density_derivative(dst: f32, radius: f32) -> f32 {
    return derivative_spiky_pow2(dst, radius);
}

// ------------------------------------------------------------
// Near Density Derivative
// ------------------------------------------------------------
fn near_density_derivative(dst: f32, radius: f32) -> f32 {
    return derivative_spiky_pow3(dst, radius);
}
