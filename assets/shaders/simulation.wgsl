#import simulation::math::{
    params,
    get_cell_3d,
    hash_cell_3d,
    key_from_hash,
    OFFSETS_3D,
    density_kernel,
    near_density_kernel,
    density_derivative,
    near_density_derivative,
    smoothing_kernel_poly6,
};

@group(0) @binding(0) var<storage, read_write> positions : array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities : array<vec4<f32>>;

@group(0) @binding(2) var<storage, read_write> predicted_positions : array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> spatial_keys : array<u32>;
@group(0) @binding(4) var<storage, read_write> spatial_offsets : array<u32>;
@group(0) @binding(5) var<storage, read> sorted_indices : array<u32>;

@group(0) @binding(6) var<storage, read_write> densities: array<vec2<f32>>;

@group(0) @binding(8) var density_map : texture_storage_3d<r16float, write>;
@group(0) @binding(9) var<uniform> density_map_size : vec4<u32>;

var<push_constant> delta_time: f32;

@group(1) @binding(0)
var<storage, read_write> sort_target_positions: array<vec4<f32>>;
@group(1) @binding(1)
var<storage, read_write> sort_target_predicted_positions: array<vec4<f32>>;
@group(1) @binding(2)
var<storage, read_write> sort_target_velocities: array<vec4<f32>>;



const WORKGROUP_SIZE: u32 = 256u;

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn external_forces(@builtin(global_invocation_id) id : vec3<u32>) {
    if (id.x >= params.num_particles) { return; }

    velocities[id.x] += vec4<f32>(0.0, params.gravity * delta_time, 0.0, 0.0);
    predicted_positions[id.x] = vec4<f32>(positions[id.x].xyz + velocities[id.x].xyz * delta_time, 0.0);
}

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn update_spatial(@builtin(global_invocation_id) id : vec3<u32>) {
    if (id.x >= params.num_particles) { return; }

    let cell = get_cell_3d(predicted_positions[id.x].xyz, params.smoothing_radius);
    let hash = hash_cell_3d(cell);
    spatial_keys[id.x] = key_from_hash(hash, params.num_particles);
}

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn calculate_densities(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.num_particles) {
        return;
    }

    let pos = predicted_positions[id];
    let d = calculate_densities_at_point(pos.xyz);
    //densities[id] = pos.xy;
    densities[id] = d;
}

@compute @workgroup_size(8, 8, 8)
fn update_density(@builtin(global_invocation_id) id : vec3<u32>) {
    if (any(id >= density_map_size.xyz)) {
        return;
    }

    let tex_pos = vec3<f32>(id) / (vec3<f32>(density_map_size.xyz) - 1.0);
    let world_pos = (tex_pos - 0.5) * params.bounds_size;

    let density = calculate_densities_at_point(world_pos);
    textureStore(density_map, vec3<i32>(id), vec4<f32>(density.x, 0.0, 0.0, 0.0));
}



fn pressure_from_density(density: f32) -> f32 {
    return (density - params.target_density) * params.pressure_multiplier;
}

fn near_pressure_from_density(nearDensity: f32) -> f32 {
    return nearDensity * params.near_pressure_multiplier;
}

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn calculate_pressure_force(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.num_particles) {
        return;
    }

    let density = densities[id].x;
    let near_density = densities[id].y;

    let pressure = pressure_from_density(density);
    let near_pressure = near_pressure_from_density(near_density);

    var pressure_force: vec3<f32> = vec3<f32>(0.0);
    let velocity = velocities[id].xyz;
    let pos = predicted_positions[id].xyz;

    let origin_cell = get_cell_3d(pos, params.smoothing_radius);
    let sqr_radius = params.smoothing_radius * params.smoothing_radius;

    var neighbour_count: i32 = 0;

    // neighbour search
    for (var i: u32 = 0u; i < 27u; i = i + 1u) {
        let cell = origin_cell + OFFSETS_3D[i];
        let hash = hash_cell_3d(cell);
        let key = key_from_hash(hash, params.num_particles);

        var curr_index = spatial_offsets[key];

        loop {
            if (curr_index >= params.num_particles) {
                break;
            }

            let neighbour_index = curr_index;
            curr_index += 1u;

            if (neighbour_index == id) {
                continue;
            }

            if (spatial_keys[neighbour_index] != key) {
                break;
            }

            let neighbour_pos = predicted_positions[neighbour_index].xyz;
            let offset = neighbour_pos - pos;
            let sqr_dist = dot(offset, offset);

            if (sqr_dist > sqr_radius) {
                continue;
            }

            let dist = sqrt(sqr_dist);
            let dir =
                select(vec3<f32>(0.0, 1.0, 0.0), offset / dist, dist > 0.0);

            neighbour_count += 1;

            let density_n = densities[neighbour_index].x;
            let near_density_n = densities[neighbour_index].y;

            let pressure_n = pressure_from_density(density_n);
            let near_pressure_n = near_pressure_from_density(near_density_n);

            let shared_pressure = (pressure + pressure_n) * 0.5;
            let shared_near_pressure = (near_pressure + near_pressure_n) * 0.5;

            pressure_force +=
                dir * density_derivative(dist, params.smoothing_radius)
                * shared_pressure / density_n;

            pressure_force +=
                dir * near_density_derivative(dist, params.smoothing_radius)
                * shared_near_pressure / near_density_n;
        }
    }

    let acceleration = pressure_force / density;
    var velocity_new = velocity + acceleration * delta_time;

    // airborne damping (wie Original)
    if (neighbour_count < 8) {
        velocity_new -= velocity_new * delta_time * 0.75;
    }

    velocities[id] = vec4<f32>(velocity_new, 0.0);
}

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn calculate_viscosity(
    @builtin(global_invocation_id) id: vec3<u32>,
) {
    let i = id.x;
    if (i >= params.num_particles) {
        return;
    }

    let pos = predicted_positions[i].xyz;
    let origin_cell = get_cell_3d(pos, params.smoothing_radius);
    let sqr_radius = params.smoothing_radius * params.smoothing_radius;

    var viscosity_force = vec3<f32>(0.0);
    let velocity = velocities[i].xyz;

    // neighbour search (3x3x3)
    for (var n = 0u; n < 27u; n++) {
        let neighbour_cell = origin_cell + OFFSETS_3D[n];
        let hash = hash_cell_3d(neighbour_cell);
        let key  = key_from_hash(hash, params.num_particles);

        var curr_index = spatial_offsets[key];

        loop {
            if (curr_index >= params.num_particles) {
                break;
            }

            let neighbour_index = curr_index;
            curr_index += 1u;

            let neighbour_key = spatial_keys[neighbour_index];
            if (neighbour_key != key) {
                break;
            }

            // self check
            if (neighbour_index == i) {
                continue;
            }

            let neighbour_pos = predicted_positions[neighbour_index].xyz;
            let delta = neighbour_pos - pos;
            let sqr_dst = dot(delta, delta);

            if (sqr_dst > sqr_radius) {
                continue;
            }

            let dst = sqrt(sqr_dst);
            let neighbour_velocity = velocities[neighbour_index].xyz;

            viscosity_force +=
                (neighbour_velocity - velocity)
                * smoothing_kernel_poly6(dst, params.smoothing_radius);
        }
    }

    velocities[i] += vec4<f32>(viscosity_force * params.viscosity_strength * delta_time, 0.0);
}


@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn update_positions(@builtin(global_invocation_id) id : vec3<u32>) {
    if (id.x >= params.num_particles) { return; }

    var vel = velocities[id.x].xyz;
    var pos = positions[id.x].xyz;
    pos += vel * delta_time;


    resolve_collisions(&pos, &vel, params.collision_damping);

    let max = 30.0;

    // Write results
    positions[id.x] = vec4<f32>(pos, 0.0);
    if (vel.y >= max) {
        vel.y = max;
    }
    if (vel.x >= max) {
        vel.x = max;
    }

    if (vel.z >= max) {
        vel.z = max;
    }

    if (vel.y <= -max) {
        vel.y = -max;
    }
    if (vel.x <= -max) {
        vel.x = -max;
    }

    if (vel.z <= -max) {
        vel.z = -max;
    }
    velocities[id.x] = vec4<f32>(vel, 0.0);
}

fn resolve_collisions(
    pos: ptr<function, vec3<f32>>,
    vel: ptr<function, vec3<f32>>,
    collision_damping: f32,
) {
    // Position / Velocity in lokalen Raum transformieren
    var pos_local = (params.world_to_local * vec4<f32>(*pos, 1.0)).xyz;
    var vel_local = (params.world_to_local * vec4<f32>(*vel, 0.0)).xyz;

    let half_size = vec3<f32>(0.5);
    let edge_dst = half_size - abs(pos_local);

    if (edge_dst.x <= 0.0)
    {
        pos_local.x = half_size.x * sign(pos_local.x);
        vel_local.x *= -1.0 * collision_damping;
    }

    if (edge_dst.y <= 0.0)
    {
        pos_local.y = half_size.y * sign(pos_local.y);
        vel_local.y *= -1.0 * collision_damping;
    }

    if (edge_dst.z <= 0.0)
    {
        pos_local.z = half_size.z * sign(pos_local.z);
        vel_local.z *= -1.0 * collision_damping;
    }

    *pos = (params.local_to_world * vec4<f32>(pos_local, 1.0)).xyz;
    *vel = (params.local_to_world * vec4<f32>(vel_local, 0.0)).xyz;
}

/*fn resolve_collisions(
    pos: ptr<function, vec3<f32>>,
    vel: ptr<function, vec3<f32>>,
    collision_damping: f32,
) {
    // Position / Velocity in lokalen Raum transformieren
    var pos_local = vec4<f32>(*pos, 1.0).xyz;
    var vel_local = vec4<f32>(*vel, 0.0).xyz;    

    let half_size = vec3<f32>(0.5, 0.5, 0.5);
    let edge_dst = half_size - abs(pos_local);

    if (edge_dst.x <= 0)
    {
        pos_local.x = half_size.x * sign(pos_local.x);
        vel_local.x *= -1 * collision_damping;
    }
    if (edge_dst.y <= 0)
    {
        pos_local.y = half_size.y * sign(pos_local.y);
        vel_local.y *= -1 * collision_damping;
    }
    if (edge_dst.z <= 0)
    {
        pos_local.z = half_size.z * sign(pos_local.z);
        vel_local.z *= -1 * collision_damping;
    }

    // Transform resolved position/velocity back to world space
    //pos = mul(localToWorld, float4(pos_local, 1)).xyz;
    //vel = mul(localToWorld, float4(vel_local, 0)).xyz;
    *pos = pos_local;
    *vel = vel_local;
}*/

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn reorder(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    let i = id.x;

    if (i >= params.num_particles) {
        return;
    }

    let sorted_index = sorted_indices[i];

    sort_target_positions[i] = positions[sorted_index];
    sort_target_predicted_positions[i] = predicted_positions[sorted_index];
    sort_target_velocities[i] = velocities[sorted_index];
}

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn reorder_copy_back(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    let i = id.x;

    if (i >= params.num_particles) {
        return;
    }

    positions[i] = sort_target_positions[i];
    predicted_positions[i] = sort_target_predicted_positions[i];
    velocities[i] = sort_target_velocities[i];
}

fn calculate_densities_at_point(pos: vec3<f32>) -> vec2<f32> {
    let origin_cell = get_cell_3d(pos, params.smoothing_radius);
    let sqr_radius = params.smoothing_radius * params.smoothing_radius;

    var density: f32 = 0.0;
    var near_density: f32 = 0.0;

    // neighbour search (27 surrounding cells)
    for (var i: u32 = 0u; i < 27u; i = i + 1u) {
        let cell = origin_cell + OFFSETS_3D[i];
        let hash = hash_cell_3d(cell);
        let key = key_from_hash(hash, params.num_particles);

        var curr_index = spatial_offsets[key];

        loop {
            if (curr_index >= params.num_particles) {
                break;
            }

            let neighbour_index = curr_index;
            curr_index = curr_index + 1u;

            let neighbour_key = spatial_keys[neighbour_index];
            if (neighbour_key != key) {
                break;
            }

            let neighbour_pos = predicted_positions[neighbour_index].xyz;
            let offset = neighbour_pos - pos;
            let sqr_dist = dot(offset, offset);

            if (sqr_dist > sqr_radius) {
                continue;
            }

            let dist = sqrt(sqr_dist);
            density += density_kernel(dist, params.smoothing_radius);
            near_density += near_density_kernel(dist, params.smoothing_radius);
        }
    }

    return vec2<f32>(density, near_density);
}
