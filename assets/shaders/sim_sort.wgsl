#import simulation::data::{
    spatial_keys,
    spatial_counts,
    spatial_offsets,
    spatial_sorted_indices,
    write_offsets,
    positions_in,
    positions_sorted,
    velocities_in,
    velocities_sorted
};

@compute @workgroup_size(256)
fn clear_counts(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&spatial_counts)) {
        return;
    }
    spatial_counts[i] = 0u;
}

@compute @workgroup_size(256)
fn calculate_counts(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&spatial_keys)) {
        return;
    }

    let key = spatial_keys[i];
    atomicAdd(&spatial_counts[key], 1u);
}

@compute @workgroup_size(256)
fn prefix_scan(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&spatial_counts)) {
        return;
    }

    var sum = 0u;
    var j = 0u;
    loop {
        if (j >= i) {
            break;
        }
        sum += spatial_counts[j];
        j += 1u;
    }

    spatial_offsets[i] = sum;
}

@compute @workgroup_size(256)
fn copy_offsets(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&spatial_offsets)) { return; }
    write_offsets[i] = spatial_offsets[i];
}

@compute @workgroup_size(256)
fn scatter(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&spatial_keys)) { return; }

    let key = spatial_keys[i];
    let dst = atomicAdd(&write_offsets[key], 1u);
    spatial_sorted_indices[dst] = i;
}

@compute
@workgroup_size(256)
fn reorder(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&spatial_sorted_indices)) {
        return;
    }

    let src = spatial_sorted_indices[i];

    positions_sorted[i]  = positions_in[src];
    velocities_sorted[i] = velocities_in[src];
}
