@group(0) @binding(0)
var<storage, read_write> positions: array<vec4<f32>>;

@group(0) @binding(1)
var<storage, read_write> velocities: array<vec4<f32>>;

struct SimulationParams {
    box_size: f32,
    gravity: f32,
};

@group(0) @binding(2)
var<uniform> params: SimulationParams;

struct Push {
    delta_time: f32,
};

var<push_constant> push: Push;

const GRAVITY: vec3<f32> = vec3<f32>(0.0, -1.0, 0.0);
const RADIUS: f32 = 0.05;
const RESTITUTION: f32 = 0.9;
const BOX_SIZE: f32 = 2.0;

//const MIN_BOUND: vec3<f32> = vec3<f32>(-BOX_SIZE / 2.0 + RADIUS, 0.0 + RADIUS, -BOX_SIZE / 2.0 + RADIUS);
//const MAX_BOUND: vec3<f32> = vec3<f32>( BOX_SIZE / 2.0 - RADIUS, BOX_SIZE - RADIUS, BOX_SIZE / 2.0 - RADIUS);

fn set_pos(i: u32, new_pos: vec3<f32>) {
    positions[i] = vec4<f32>(new_pos, positions[i].w);
}

fn set_vel(i: u32, new_vel: vec3<f32>) {
    velocities[i] = vec4<f32>(new_vel, velocities[i].w);
}


@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;

    let min_bound = vec3<f32>(-params.box_size / 2.0 + RADIUS, 0.0 + RADIUS, -params.box_size / 2.0 + RADIUS);
    let max_bound = vec3<f32>( params.box_size / 2.0 - RADIUS, params.box_size - RADIUS, params.box_size / 2.0 - RADIUS);

    if (i >= arrayLength(&positions)) {
        return;
    }

    let dt = push.delta_time;

    /* ---------------------------------
       1. Gravity
    --------------------------------- */
    velocities[i] += vec4<f32>(vec3<f32>(0.0, params.gravity, 0.0) * dt, 0.0);

    /* ---------------------------------
       2. Integrate position
    --------------------------------- */
    positions[i] += velocities[i] * dt;

    /* ---------------------------------
       3. World bounds + bounce
    --------------------------------- */
    for (var axis: u32 = 0u; axis < 3u; axis++) {
        if (positions[i][axis] < min_bound[axis]) {
            positions[i][axis] = min_bound[axis];
            velocities[i][axis] *= -RESTITUTION;
        }

        if (positions[i][axis] > max_bound[axis]) {
            positions[i][axis] = max_bound[axis];
            velocities[i][axis] *= -RESTITUTION;
        }
    }

    /* ---------------------------------
       4. Particle–Particle collisions
       (naiv O(n²), absichtlich simpel)
    --------------------------------- */
    for (var j: u32 = 0u; j < arrayLength(&positions); j++) {
        if (j == i) {
            continue;
        }

        let pi = positions[i].xyz;
        let pj = positions[j].xyz;

        var delta = pi - pj;
        var dist = length(delta);
        let min_dist = RADIUS * 2.0;

        if (dist < min_dist && dist > 0.0001) {
            let normal = delta / dist;
            let penetration = min_dist - dist;

            // Position nur für i korrigieren
            let new_pi = pi + normal * (penetration * 0.5);
            set_pos(i, new_pi);

            let vi = velocities[i].xyz;
            let vj = velocities[j].xyz;
            let rel_vel = vi - vj;
            let vel_n = dot(rel_vel, normal);

            if (vel_n < 0.0) {
                let impulse = -(1.0 + RESTITUTION) * vel_n * 0.5;
                set_vel(i, vi + normal * impulse);
            }
        }

    }
}
