// Constants
const GROUP_SIZE: u32 = 256u;
const ITEMS_PER_GROUP: u32 = 512u; // 2 * GROUP_SIZE

// Buffers
@group(0) @binding(0)
var<storage, read_write> elements: array<u32>;

@group(1) @binding(0)
var<storage, read_write> groupSums: array<u32>;

var<push_constant> item_count_push: u32;
// Shared memory
var<workgroup> temp: array<u32, ITEMS_PER_GROUP>;


// ------------------------------------------------------------
// BlockScan: exclusive prefix sum inside one workgroup
// ------------------------------------------------------------
@compute @workgroup_size(GROUP_SIZE, 1, 1)
fn block_scan(
    @builtin(global_invocation_id) threadGlobal: vec3<u32>,
    @builtin(local_invocation_id)  threadLocal: vec3<u32>,
    @builtin(workgroup_id)         group: vec3<u32>,
) {
    let localA  = threadLocal.x * 2u;
    let localB  = threadLocal.x * 2u + 1u;
    let globalA = threadGlobal.x * 2u;
    let globalB = threadGlobal.x * 2u + 1u;

    let hasA = globalA < item_count_push;
    let hasB = globalB < item_count_push;

    // Load
    temp[localA] = select(0u, elements[globalA], hasA);
    temp[localB] = select(0u, elements[globalB], hasB);

    // Up-sweep
    var offset: u32 = 1u;
    var numActiveThreads: u32 = GROUP_SIZE;

    loop {
        workgroupBarrier();

        if (threadLocal.x < numActiveThreads) {
            let indexA = offset * (localA + 1u) - 1u;
            let indexB = offset * (localB + 1u) - 1u;
            temp[indexB] = temp[indexA] + temp[indexB];
        }

        offset *= 2u;
        numActiveThreads /= 2u;
        if (numActiveThreads == 0u) {
            break;
        }
    }

    // Thread 0
    if (threadLocal.x == 0u) {
        groupSums[group.x] = temp[ITEMS_PER_GROUP - 1u];
        temp[ITEMS_PER_GROUP - 1u] = 0u;
    }

    // Down-sweep
    numActiveThreads = 1u;
    loop {
        workgroupBarrier();
        offset /= 2u;

        if (threadLocal.x < numActiveThreads) {
            let indexA = offset * (localA + 1u) - 1u;
            let indexB = offset * (localB + 1u) - 1u;
            let sum = temp[indexA] + temp[indexB];
            temp[indexA] = temp[indexB];
            temp[indexB] = sum;
        }

        numActiveThreads *= 2u;
        if (numActiveThreads > GROUP_SIZE) {
            break;
        }
    }

    workgroupBarrier();

    if (hasA) {
        elements[globalA] = temp[localA];
    }
    if (hasB) {
        elements[globalB] = temp[localB];
    }
}

// ------------------------------------------------------------
// Block Combine
// ------------------------------------------------------------
@compute @workgroup_size(GROUP_SIZE, 1, 1)
fn block_combine(
    @builtin(global_invocation_id) threadGlobal: vec3<u32>,
    @builtin(workgroup_id)         wid: vec3<u32>,
) {
    let globalA = threadGlobal.x * 2 + 0;
	let globalB = threadGlobal.x * 2 + 1;

	if (globalA < item_count_push) {
        elements[globalA] += groupSums[wid.x];
    }
	if (globalB < item_count_push) {
        elements[globalB] += groupSums[wid.x];
    }
}