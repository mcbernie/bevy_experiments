#import simulation::data::{
    positions,
    params,
    spatial_keys
};

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

@compute @workgroup_size(256)
fn update_spatial_hash(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&positions)) {
        return;
    }

    let p = positions[i].xyz;
    let cell = cell_from_pos(p, params.cell_size);
    let h = hash_cell(cell);

    // Key im erlaubten Bereich halten
    spatial_keys[i] = h;
}
