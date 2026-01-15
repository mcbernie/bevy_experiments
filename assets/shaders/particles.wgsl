#import simulation::data::{
    positions_in,
    velocities_in,
    positions_out,
    velocities_out,
    positions_sorted,
    velocities_sorted,
    params,
    push,
    spatial_counts,
    spatial_offsets,
    hash_cell,
    cell_from_pos,
    spatial_sorted_indices,

};

const RESTITUTION: f32 = 0.9;

@compute
@workgroup_size(256)
fn external_forces(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&positions_in)) {
        return;
    }

    let min_bound = vec3<f32>(-params.box_size / 2.0 + params.particle_radius, 0.0 + params.particle_radius, -params.box_size / 2.0 + params.particle_radius);
    let max_bound = vec3<f32>( params.box_size / 2.0 - params.particle_radius, params.box_size - params.particle_radius, params.box_size / 2.0 - params.particle_radius);

    let dt = push.delta_time;

    // --- READ ---
    var pos = positions_in[i].xyz;
    var vel = velocities_in[i].xyz;

    // 1. Gravity
    vel += vec3<f32>(vec3<f32>(0.0, params.gravity, 0.0) * dt);

    // 2. Integrate
    pos += vel * dt;

    // 3. World bounds + bounce
    for (var axis: u32 = 0u; axis < 3u; axis++) {
        if (pos[axis] < min_bound[axis]) {
            pos[axis] = min_bound[axis];
            vel[axis] *= -RESTITUTION;
        }

        if (pos[axis] > max_bound[axis]) {
            pos[axis] = max_bound[axis];
            vel[axis] *= -RESTITUTION;
        }
    }

    positions_out[i]  = vec4<f32>(pos, positions_in[i].w);
    velocities_out[i] = vec4<f32>(vel, velocities_in[i].w);
}

@compute
@workgroup_size(256)
fn collision(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&positions_sorted)) {
        return;
    }

    // Position & Velocity des SORTIERTEN Eintrags
    //var pos = positions_sorted[i].xyz;
    //var vel = velocities_sorted[i].xyz;

    // Original-Partikelindex merken!
    let original = spatial_sorted_indices[i];

    // BASIS = integrierte Position
    var pos = positions_out[original].xyz;
    var vel = velocities_out[original].xyz;

    // Zelle bestimmen
    let cell = cell_from_pos(pos, params.cell_size);
    let key  = hash_cell(cell) % arrayLength(&spatial_counts);

    let start = spatial_offsets[key];
    let count = spatial_counts[key];
    let end   = start + count;

    let radius   = params.particle_radius;
    let min_dist = radius * 2.0;

    // Kollisionen
    for (var j = start; j < end; j++) {
        if (j == i) {
            continue;
        }

        let other_pos = positions_sorted[j].xyz;

        let delta = pos - other_pos;
        let dist  = length(delta);

        if (dist > 0.0001 && dist < min_dist) {
            let normal = delta / dist;
            let penetration = min_dist - dist;

            // Positionskorrektur (PBD-Style)
            pos += normal * penetration * 0.5;

            // minimale Reaktion, damit man was sieht
            vel += normal * penetration * 5.0;
        }
    }

    // WICHTIG: zurück auf ORIGINALEN INDEX schreiben
    positions_out[original] =
        vec4<f32>(pos, positions_sorted[i].w);

    velocities_out[original] =
        vec4<f32>(vel, velocities_sorted[i].w);
}



