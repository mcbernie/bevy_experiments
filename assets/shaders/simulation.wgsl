@group(0) @binding(0) var<storage, read_write> positions : array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities : array<vec4<f32>>;

@group(0) @binding(2) var<storage, read_write> predicted_positions : array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> spatial_keys : array<u32>;
@group(0) @binding(4) var<storage, read_write> spatial_offsets : array<u32>;
@group(0) @binding(5) var<storage, read> sorted_indices : array<u32>;

struct SimParams {
    numParticles : u32,
    gravity : f32,
    smoothingRadius : f32,
    //targetDensity : f32,
    //pressureMultiplier : f32,
    //nearPressureMultiplier : f32,
    //viscosityStrength : f32,
    collisionDamping : f32,
    bounds_size : vec3<f32>,
};

@group(0) @binding(6) var<uniform> params : SimParams;

var<push_constant> delta_time: f32;

fn getCell3D(pos: vec3<f32>, r: f32) -> vec3<i32> {
    return vec3<i32>(floor(pos / r));
}

fn hashCell3D(cell: vec3<i32>) -> u32 {
    let blockSize : u32 = 50u;
    let ucell = vec3<u32>(cell + vec3<i32>(25));
    let local = ucell % blockSize;
    let block = ucell / blockSize;
    return local.x + blockSize * (local.y + blockSize * local.z)
        + block.x * 15823u + block.y * 9737333u + block.z * 440817757u;
}

fn keyFromHash(h: u32, size: u32) -> u32 {
    return h % size;
}

@compute @workgroup_size(256)
fn external_forces(@builtin(global_invocation_id) id : vec3<u32>) {
    if (id.x >= params.numParticles) { return; }

    velocities[id.x] += vec4<f32>(0.0, params.gravity * delta_time, 0.0, 0.0);
    predicted_positions[id.x] = vec4<f32>(positions[id.x].xyz + velocities[id.x].xyz * (1.0 / 120.0), 0.0);
}

@compute @workgroup_size(256)
fn update_spatial(@builtin(global_invocation_id) id : vec3<u32>) {
    if (id.x >= params.numParticles) { return; }

    let cell = getCell3D(predicted_positions[id.x].xyz, params.smoothingRadius);
    let hash = hashCell3D(cell);
    spatial_keys[id.x] = keyFromHash(hash, params.numParticles);
}

@compute @workgroup_size(256)
fn update_positions(@builtin(global_invocation_id) id : vec3<u32>) {
    if (id.x >= params.numParticles) { return; }

    var vel = velocities[id.x].xyz;
    var pos = positions[id.x].xyz;
    pos += vel * delta_time;

    resolve_particle_collisions(id.x, &pos, &vel);

    resolve_collisions(&pos, &vel, params.collisionDamping);


    // Write results
    positions[id.x] = vec4<f32>(pos, 0.0);
    velocities[id.x] = vec4<f32>(vel, 0.0);
}

fn resolve_collisions(
    pos: ptr<function, vec3<f32>>,
    vel: ptr<function, vec3<f32>>,
    collisionDamping: f32,
) {
    // Position / Velocity in lokalen Raum transformieren
    let halfSize = params.bounds_size * 0.5;
    let minBound = vec3<f32>(-halfSize.x, 0.2, -halfSize.z);
    let maxBound = vec3<f32>( halfSize.x, params.bounds_size.y, halfSize.z);

    var p = *pos;
    var v = *vel;

    if (p.x < minBound.x) {
        p.x = minBound.x;
        v.x *= -collisionDamping;
    } else if (p.x > maxBound.x) {
        p.x = maxBound.x;
        v.x *= -collisionDamping;
    }

    // Y (Boden + Decke)
    if (p.y < minBound.y) {
        p.y = minBound.y;
        v.y *= -collisionDamping;
    } else if (p.y > maxBound.y) {
        p.y = maxBound.y;
        v.y *= -collisionDamping;
    }

    // Z
    if (p.z < minBound.z) {
        p.z = minBound.z;
        v.z *= -collisionDamping;
    } else if (p.z > maxBound.z) {
        p.z = maxBound.z;
        v.z *= -collisionDamping;
    }

    *pos = p.xyz;
    *vel = v.xyz;
}

fn resolve_particle_collisions(
    index: u32,
    pos: ptr<function, vec3<f32>>,
    vel: ptr<function, vec3<f32>>,
) {
    let r = 0.05;
    let min_dist = r * 2.0;
    let min_dist_sq = min_dist * min_dist;

    var p = *pos;
    var v = *vel;

    for (var i: u32 = 0u; i < params.numParticles; i = i + 1u) {
        if (i == index) {
            continue;
        }

        let other_p = positions[i].xyz;
        let delta = p - other_p;
        let dist_sq = dot(delta, delta);

        if (dist_sq > 0.0 && dist_sq < min_dist_sq) {
            let dist = sqrt(dist_sq);
            let n = delta / dist;

            let penetration = min_dist - dist;

            // Position minimal korrigieren (schwer = wenig Bewegung)
            p += n * penetration * 0.7;

            // Velocity nur entlang Normalen dämpfen
            let vn = dot(v, n);
            if (vn < 0.0) {
                v -= (1.0 + 0.1) * vn * n;
            }
        }
    }

    *pos = p;
    *vel = v;
}
