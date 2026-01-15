#import simulation::data::{
    spatial_keys,
    spatial_counts,
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