// Constants
const GROUP_SIZE: u32 = 256u;
const ITEMS_PER_GROUP: u32 = 512u; // 2 * GROUP_SIZE

// Buffers
@group(0) @binding(0)
var<storage, read_write> elements: array<u32>;

@group(0) @binding(1)
var<storage, read_write> groupSums: array<u32>;

@group(0) @binding(2)
var<uniform> itemCount: u32;

// Shared memory
var<workgroup> temp: array<u32, 512>;


// ------------------------------------------------------------
// BlockScan: exclusive prefix sum inside one workgroup
// ------------------------------------------------------------
@compute @workgroup_size(256, 1, 1)
fn block_scan(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_id)  lid: vec3<u32>,
    @builtin(workgroup_id)         wid: vec3<u32>,
) {
    let t = lid.x;
    let g = gid.x;
    let group = wid.x;

    let localA  = t * 2u;
    let localB  = t * 2u + 1u;
    let globalA = g * 2u;
    let globalB = g * 2u + 1u;

    let hasA = globalA < itemCount;
    let hasB = globalB < itemCount;

    // Load input into shared memory
    temp[localA] = select(0u, elements[globalA], hasA);
    temp[localB] = select(0u, elements[globalB], hasB);

    // Up-sweep (reduce)
    var offset: u32 = 1u;
    var active: u32 = GROUP_SIZE;

    loop {
        workgroupBarrier();

        if (t < active) {
            let ia = offset * (localA + 1u) - 1u;
            let ib = offset * (localB + 1u) - 1u;
            temp[ib] = temp[ia] + temp[ib];
        }

        offset *= 2u;
        active /= 2u;
        if (active == 0u) { break; }
    }

    // Store block sum and prepare exclusive scan
    if (t == 0u) {
        groupSums[group] = temp[ITEMS_PER_GROUP - 1u];
        temp[ITEMS_PER_GROUP - 1u] = 0u;
    }

    // Down-sweep
    active = 1u;
    loop {
        workgroupBarrier();
        offset /= 2u;

        if (t < active) {
            let ia = offset * (localA + 1u) - 1u;
            let ib = offset * (localB + 1u) - 1u;
            let s = temp[ia] + temp[ib];
            temp[ia] = temp[ib];
            temp[ib] = s;
        }

        active *= 2u;
        if (active > GROUP_SIZE) { break; }
    }

    workgroupBarrier();

    // Write results back
    if (hasA) { elements[globalA] = temp[localA]; }
    if (hasB) { elements[globalB] = temp[localB]; }
}


// ------------------------------------------------------------
// BlockCombine: add scanned group offsets to each element
// ------------------------------------------------------------
@compute @workgroup_size(256, 1, 1)
fn block_combine(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(workgroup_id)         wid: vec3<u32>,
) {
    let g = gid.x;
    let group = wid.x;

    let globalA = g * 2u;
    let globalB = g * 2u + 1u;
    let add = groupSums[group];

    if (globalA < itemCount) {
        elements[globalA] += add;
    }
    if (globalB < itemCount) {
        elements[globalB] += add;
    }
}
