const GROUP_SIZE: u32 = 256u;

@group(0) @binding(0)
var<storage, read_write> inputItems: array<u32>;

@group(0) @binding(1)
var<storage, read_write> inputKeys: array<u32>;

@group(0) @binding(2)
var<storage, read_write> sortedItems: array<u32>;

@group(0) @binding(3)
var<storage, read_write> sortedKeys: array<u32>;

// Counts muss atomic sein wegen atomicAdd
@group(0) @binding(4)
var<storage, read_write> counts: array<atomic<u32>>;

@group(0) @binding(5)
var<uniform> numInputs: u32;


@compute @workgroup_size(256, 1, 1)
fn clear_counts(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= numInputs) {
        return;
    }

    atomicStore(&counts[id.x], 0u);
    inputItems[id.x] = id.x;
}

@compute @workgroup_size(256, 1, 1)
fn calculate_counts(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= numInputs) {
        return;
    }

    let key: u32 = inputKeys[id.x];
    atomicAdd(&counts[key], 1u);
}

@compute @workgroup_size(256, 1, 1)
fn scatter_output(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= numInputs) {
        return;
    }

    let key: u32 = inputKeys[id.x];

    // atomicAdd gibt den alten Wert zurück → sortedIndex
    let sortedIndex: u32 = atomicAdd(&counts[key], 1u);

    sortedItems[sortedIndex] = inputItems[id.x];
    sortedKeys[sortedIndex]  = key;
}

@compute @workgroup_size(256, 1, 1)
fn copy_back(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= numInputs) {
        return;
    }

    inputItems[id.x] = sortedItems[id.x];
    inputKeys[id.x]  = sortedKeys[id.x];
}
