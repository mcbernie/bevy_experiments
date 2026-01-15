#import simulation::data::{
    positions_in,
    params,
    spatial_keys,
    spatial_counts,
    hash_cell,
    cell_from_pos,
};


fn linear_cell_id(cell: vec3<i32>) -> u32 {
    // DEBUG: einfache Projektion
    // z.B. nur X/Y, damit man es sieht
    return u32(cell.x + cell.y * 64);
}

@compute @workgroup_size(256)
fn update_spatial_hash(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&positions_in)) {
        return;
    }

    let p = positions_in[i].xyz;
    let cell = cell_from_pos(p, params.cell_size);
    // let h = hash_cell(cell);
    let h = hash_cell(cell) % arrayLength(&spatial_counts);
    //let h = linear_cell_id(cell);

    spatial_keys[i] = h;
}
