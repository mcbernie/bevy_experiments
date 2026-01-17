@group(0) @binding(0)
var<storage, read> sorted_keys: array<u32>;

@group(0) @binding(1)
var<storage, read_write> offsets: array<u32>;

@compute @workgroup_size(256, 1, 1)
fn initialize_offsets(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= arrayLength(&sorted_keys)) {
        return;
    }

    let length = arrayLength(&sorted_keys);

    offsets[id.x] = length;
}


@compute @workgroup_size(256, 1, 1)
fn calculate_offsets(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= arrayLength(&sorted_keys)) {
        return;
    }

    let i: u32 = id.x;
    let null_value: u32 = arrayLength(&sorted_keys);

    let key: u32 = sorted_keys[i];
    let keyPrev: u32 = select(
        sorted_keys[i - 1],
        null_value,
        i == 0u
    );

    if (key != keyPrev) {
        offsets[key] = i;
    }
}
