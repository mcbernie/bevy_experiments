#import simulation::math::{
    SimParams,
    Triangle,
    Vertex,
};


// lookup tables
const offsets: array<u32, 256> = array<u32, 256>(
    0, 0, 3, 6, 12, 15, 21, 27, 36, 39, 45, 51, 60, 66, 75, 84, 90, 93, 99, 105, 114, 120, 129, 138, 150, 156, 165, 174, 186, 195, 207, 219, 228, 231, 237, 243, 252, 258, 267, 276, 288, 294, 303, 312, 324, 333, 345, 357, 366, 372, 381, 390, 396, 405, 417, 429, 438, 447, 459, 471, 480, 492, 507, 522, 528, 531, 537, 543, 552, 558, 567, 576, 588, 594, 603, 612, 624, 633, 645, 657, 666, 672, 681, 690, 702, 711, 723, 735, 750, 759, 771, 783, 798, 810, 825, 840, 852, 858, 867, 876, 888, 897, 909, 915, 924, 933, 945, 957, 972, 984, 999, 1008, 1014, 1023, 1035, 1047, 1056, 1068, 1083, 1092, 1098, 1110, 1125, 1140, 1152, 1167, 1173, 1185, 1188, 1191, 1197, 1203, 1212, 1218, 1227, 1236, 1248, 1254, 1263, 1272, 1284, 1293, 1305, 1317, 1326, 1332, 1341, 1350, 1362, 1371, 1383, 1395, 1410, 1419, 1425, 1437, 1446, 1458, 1467, 1482, 1488, 1494, 1503, 1512, 1524, 1533, 1545, 1557, 1572, 1581, 1593, 1605, 1620, 1632, 1647, 1662, 1674, 1683, 1695, 1707, 1716, 1728, 1743, 1758, 1770, 1782, 1791, 1806, 1812, 1827, 1839, 1845, 1848, 1854, 1863, 1872, 1884, 1893, 1905, 1917, 1932, 1941, 1953, 1965, 1980, 1986, 1995, 2004, 2010, 2019, 2031, 2043, 2058, 2070, 2085, 2100, 2106, 2118, 2127, 2142, 2154, 2163, 2169, 2181, 2184, 2193, 2205, 2217, 2232, 2244, 2259, 2268, 2280, 2292, 2307, 2322, 2328, 2337, 2349, 2355, 2358, 2364, 2373, 2382, 2388, 2397, 2409, 2415, 2418, 2427, 2433, 2445, 2448, 2454, 2457, 2460
);

const lengths: array<u32, 256> = array<u32, 256>(
    0, 3, 3, 6, 3, 6, 6, 9, 3, 6, 6, 9, 6, 9, 9, 6, 3, 6, 6, 9, 6, 9, 9, 12, 6, 9, 9, 12, 9, 12, 12, 9, 3, 6, 6, 9, 6, 9, 9, 12, 6, 9, 9, 12, 9, 12, 12, 9, 6, 9, 9, 6, 9, 12, 12, 9, 9, 12, 12, 9, 12, 15, 15, 6, 3, 6, 6, 9, 6, 9, 9, 12, 6, 9, 9, 12, 9, 12, 12, 9, 6, 9, 9, 12, 9, 12, 12, 15, 9, 12, 12, 15, 12, 15, 15, 12, 6, 9, 9, 12, 9, 12, 6, 9, 9, 12, 12, 15, 12, 15, 9, 6, 9, 12, 12, 9, 12, 15, 9, 6, 12, 15, 15, 12, 15, 6, 12, 3, 3, 6, 6, 9, 6, 9, 9, 12, 6, 9, 9, 12, 9, 12, 12, 9, 6, 9, 9, 12, 9, 12, 12, 15, 9, 6, 12, 9, 12, 9, 15, 6, 6, 9, 9, 12, 9, 12, 12, 15, 9, 12, 12, 15, 12, 15, 15, 12, 9, 12, 12, 9, 12, 15, 15, 12, 12, 9, 15, 6, 15, 12, 6, 3, 6, 9, 9, 12, 9, 12, 12, 15, 9, 12, 12, 15, 6, 9, 9, 6, 9, 12, 12, 15, 12, 15, 15, 6, 12, 9, 15, 12, 9, 6, 12, 3, 9, 12, 12, 15, 12, 15, 9, 12, 12, 15, 15, 6, 9, 12, 6, 3, 6, 9, 9, 6, 9, 12, 6, 3, 9, 6, 12, 3, 6, 3, 3, 0
);

const corner_index_a_from_edge: array<u32, 12> =
    array<u32, 12>(0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3);

const corner_index_b_from_edge: array<u32, 12> =
    array<u32, 12>(1, 2, 3, 0, 5, 6, 7, 4, 4, 5, 6, 7);

// ============================================================
// buffers & resources
// ============================================================
struct TriangleCounter {
    count: atomic<u32>,
};

@group(0) @binding(0)
var<storage, read_write> triangles: array<Triangle>;

@group(0) @binding(1)
var<storage, read_write> triangle_counter: TriangleCounter;

@group(0) @binding(2)
var<storage, read> lut: array<u32>;

@group(0) @binding(3)
var density_map: texture_3d<f32>;

@group(0) @binding(4)
var linear_clamp_sampler: sampler;

@group(0) @binding(5)
var<uniform> density_map_size: vec4<u32>;

@group(1) @binding(0) var<uniform> params : SimParams;


// configurable (derzeit konstant)
const iso_level: f32 = 0.0;

// ============================================================
// helpers
// ============================================================
//fn coord_to_world(coord: vec3<i32>) -> vec3<f32> {
//    let size_f = vec3<f32>(density_map_size.xyz) - vec3<f32>(1.0);
//    return vec3<f32>(coord) / size_f - vec3<f32>(0.5);
//}
fn coord_to_world(coord: vec3<i32>) -> vec3<f32> {
    let grid_size = vec3<f32>(density_map_size.xyz);
    let voxel_size = params.bounds_size / grid_size;

    return (vec3<f32>(coord) + 0.5) * voxel_size
         - params.bounds_size * 0.5;

    //return (vec3<f32>(coord) / vec3<f32>(density_map_size.xyz - vec3<u32>(1))) - vec3<f32>(0.5);
}


fn sample_density(coord: vec3<i32>) -> f32 {
    let min_c = any(coord <= vec3<i32>(0));
    let max_c = any(coord >= vec3<i32>(density_map_size.xyz) - vec3<i32>(1));
    if (min_c || max_c) {
        return iso_level;
    }

    let uvw = vec3<f32>(coord)
        / (vec3<f32>(density_map_size.xyz) - vec3<f32>(1.0));

    return -textureSampleLevel(density_map, linear_clamp_sampler, uvw, 0.0).x;
}

fn calculate_normal(coord: vec3<i32>) -> vec3<f32> {
    let dx = sample_density(coord + vec3<i32>(1, 0, 0))
           - sample_density(coord - vec3<i32>(1, 0, 0));
    let dy = sample_density(coord + vec3<i32>(0, 1, 0))
           - sample_density(coord - vec3<i32>(0, 1, 0));
    let dz = sample_density(coord + vec3<i32>(0, 0, 1))
           - sample_density(coord - vec3<i32>(0, 0, 1));

    return normalize(vec3<f32>(dx, dy, dz));
}

fn create_vertex(coord_a: vec3<i32>, coord_b: vec3<i32>) -> Vertex {
    let pos_a = coord_to_world(coord_a);
    let pos_b = coord_to_world(coord_b);

    let density_a = sample_density(coord_a);
    let density_b = sample_density(coord_b);

    let t = (iso_level - density_a) / (density_b - density_a);
    let position = pos_a + t * (pos_b - pos_a);

    let normal_a = calculate_normal(coord_a);
    let normal_b = calculate_normal(coord_b);
    let normal = normalize(normal_a + t * (normal_b - normal_a));
    
    let scale = vec3<f32>(1.0, 1.0, 1.0);//params.bounds_size.xyz;

    return Vertex(vec4<f32>(position * scale, 0.0), vec4<f32>(normal, 0.0));
}

// ============================================================
// compute entry
// ============================================================
@compute @workgroup_size(8, 8, 8)
fn process_cube(@builtin(global_invocation_id) id: vec3<u32>) {
    let cid = vec3<i32>(id);
    let num_cubes = vec3<i32>(density_map_size.xyz) - vec3<i32>(1);

    if (cid.x >= num_cubes.x || cid.y >= num_cubes.y || cid.z >= num_cubes.z) {
        return;
    }

    var corner_coords: array<vec3<i32>, 8>;
    corner_coords[0] = cid + vec3<i32>(0, 0, 0);
    corner_coords[1] = cid + vec3<i32>(1, 0, 0);
    corner_coords[2] = cid + vec3<i32>(1, 0, 1);
    corner_coords[3] = cid + vec3<i32>(0, 0, 1);
    corner_coords[4] = cid + vec3<i32>(0, 1, 0);
    corner_coords[5] = cid + vec3<i32>(1, 1, 0);
    corner_coords[6] = cid + vec3<i32>(1, 1, 1);
    corner_coords[7] = cid + vec3<i32>(0, 1, 1);

    var cube_configuration: u32 = 0u;
    for (var i: u32 = 0u; i < 8u; i = i + 1u) {
        if (sample_density(corner_coords[i]) < iso_level) {
            cube_configuration |= (1u << i);
        }
    }

    let num_indices = lengths[cube_configuration];
    let offset = offsets[cube_configuration];

    var i: u32 = 0u;
    loop {
        if (i >= num_indices) { break; }

        let v0 = lut[offset + i];
        let v1 = lut[offset + i + 1u];
        let v2 = lut[offset + i + 2u];

        let a0 = corner_index_a_from_edge[v0];
        let b0 = corner_index_b_from_edge[v0];
        let a1 = corner_index_a_from_edge[v1];
        let b1 = corner_index_b_from_edge[v1];
        let a2 = corner_index_a_from_edge[v2];
        let b2 = corner_index_b_from_edge[v2];

        let vertex_a = create_vertex(corner_coords[a0], corner_coords[b0]);
        let vertex_b = create_vertex(corner_coords[a1], corner_coords[b1]);
        let vertex_c = create_vertex(corner_coords[a2], corner_coords[b2]);

        let index = atomicAdd(&triangle_counter.count, 1u);
        triangles[index] = Triangle(vertex_c, vertex_b, vertex_a);

        i = i + 3u;
    }
}