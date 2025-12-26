use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

use crate::{
    config::BlocksConfigRes,
    voxel::{
        chunk::{face_id, Block, CHUNK_SIZE, ChunkData, ChunkPos},
        meshing::{face_kind, get_block_world, tile_for, effective_block_kind, FaceDir},
        plugin::VoxelWorld,
        tile::{tile_uv, push_uvs, UvRot},
    },
};

/// Greedy meshing über alle 3 Achsen.
/// Idee:
/// - wir sweepen jede Achse separat
/// - pro Grenzfläche bauen wir eine 2D-Maske
/// - auf dieser Maske laufen wir greedy rectangles
pub fn build_chunk_mesh_greedy_all_axes(
    cfg: &BlocksConfigRes,
    world: &VoxelWorld,
    all_chunks: &Query<&ChunkData>,
    chunk_pos: ChunkPos,
    data: &ChunkData,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut indices: Vec<u32> = vec![];

    // Greedy für Z, dann X, dann Y (Reihenfolge egal)
    greedy_axis(
        2, // Z
        cfg, world, all_chunks, chunk_pos, data,
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );
    greedy_axis(
        0, // X
        cfg, world, all_chunks, chunk_pos, data,
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );
    greedy_axis(
        1, // Y
        cfg, world, all_chunks, chunk_pos, data,
        &mut positions, &mut normals, &mut uvs, &mut indices,
    );

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Eine Achse greedy meshen.
/// axis: 0=X, 1=Y, 2=Z
fn greedy_axis(
    axis: usize,
    cfg: &BlocksConfigRes,
    world: &VoxelWorld,
    all_chunks: &Query<&ChunkData>,
    chunk_pos: ChunkPos,
    data: &ChunkData,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let size = [CHUNK_SIZE.x, CHUNK_SIZE.y, CHUNK_SIZE.z];

    // Die zwei Achsen in der Maske (U,V) sind die "anderen beiden".
    let (u_axis, v_axis) = match axis {
        0 => (1, 2), // X sweep -> Maske ist YZ
        1 => (0, 2), // Y sweep -> Maske ist XZ
        _ => (0, 1), // Z sweep -> Maske ist XY
    };

    let su = size[u_axis];
    let sv = size[v_axis];
    let sd = size[axis];

    // Maskenfelder (pro cell in UxV)
    let mut mask_id: Vec<u32> = vec![0; (su * sv) as usize];
    let mut mask_block: Vec<Block> = vec![Block::Air; (su * sv) as usize];
    let mut mask_dir: Vec<FaceDir> = vec![FaceDir::PosZ; (su * sv) as usize];

    // slice läuft über Grenzflächen: 0..=sd
    for slice in 0..=sd {
        // 1) Maske bauen
        for vv in 0..sv {
            for uu in 0..su {
                let i = (uu + vv * su) as usize;

                // Wir brauchen die 3D-Koordinate (x,y,z) für a und b.
                // a liegt bei slice-1 entlang axis, b bei slice entlang axis.
                let a_pos = axis_uvd_to_xyz(axis, u_axis, v_axis, uu, vv, slice - 1);
                let b_pos = axis_uvd_to_xyz(axis, u_axis, v_axis, uu, vv, slice);

                let a = if slice == 0 {
                    Block::Air
                } else {
                    get_block(world, all_chunks, chunk_pos, data, a_pos)
                };
                let b = if slice == sd {
                    Block::Air
                } else {
                    get_block(world, all_chunks, chunk_pos, data, b_pos)
                };

                let (id, blk, dir) = if a != Block::Air && b == Block::Air {
                    // sichtbare Face in +axis Richtung am Block a
                    let dir = axis_pos_dir(axis);

                    // Surface-Detection: bei dir war "oben = y+1" Logik drin.
                    // Das bleibt hier identisch, nur mit Weltkoordinaten.
                    let is_surface = {
                        let (x, y, z) = a_pos;
                        get_block_world(world, all_chunks, chunk_pos, x, y + 1, z) == Block::Air
                    };
                    let eff = effective_block_kind(a, is_surface);
                    (face_id(cfg, eff, dir), eff, dir)
                } else if b != Block::Air && a == Block::Air {
                    // sichtbare Face in -axis Richtung am Block b
                    let dir = axis_neg_dir(axis);
                    let is_surface = {
                        let (x, y, z) = b_pos;
                        get_block_world(world, all_chunks, chunk_pos, x, y + 1, z) == Block::Air
                    };
                    let eff = effective_block_kind(b, is_surface);
                    (face_id(cfg, eff, dir), eff, dir)
                } else {
                    (0, Block::Air, FaceDir::PosZ)
                };

                mask_id[i] = id;
                mask_block[i] = blk;
                mask_dir[i] = dir;
            }
        }

        // 2) Greedy rectangles auf der Maske (U×V)
        let mut v = 0;
        while v < sv {
            let mut u = 0;
            while u < su {
                let i0 = (u + v * su) as usize;
                let id0 = mask_id[i0];
                if id0 == 0 {
                    u += 1;
                    continue;
                }

                let blk0 = mask_block[i0];
                let dir0 = mask_dir[i0];

                // Breite w in U-Richtung
                let mut w = 1;
                while u + w < su {
                    let ii = (u + w + v * su) as usize;
                    if mask_id[ii] != id0 {
                        break;
                    }
                    w += 1;
                }

                // Höhe h in V-Richtung
                let mut h = 1;
                'outer: while v + h < sv {
                    for du in 0..w {
                        let ii = (u + du + (v + h) * su) as usize;
                        if mask_id[ii] != id0 {
                            break 'outer;
                        }
                    }
                    h += 1;
                }

                // verbrauchte Zellen leeren
                for dv in 0..h {
                    for du in 0..w {
                        let ii = (u + du + (v + dv) * su) as usize;
                        mask_id[ii] = 0;
                    }
                }

                // Quad emittieren: liegt auf der Grenzfläche bei `slice` entlang `axis`,
                // und spannt in U/V die Bereiche u..u+w und v..v+h auf.
                emit_greedy_quad(
                    cfg,
                    blk0,
                    dir0,
                    axis,
                    u_axis,
                    v_axis,
                    u,
                    v,
                    slice,
                    w,
                    h,
                    positions,
                    normals,
                    uvs,
                    indices,
                );

                u += w;
            }
            v += 1;
        }
    }
}

/// Block holen.
/// - lokal innerhalb des Chunks nehmen wir data.get_local()
/// - außerhalb nutzen wir get_block_world(...).
fn get_block(
    world: &VoxelWorld,
    all_chunks: &Query<&ChunkData>,
    chunk_pos: ChunkPos,
    data: &ChunkData,
    (x, y, z): (i32, i32, i32),
) -> Block {
    // innerhalb des eigenen chunks:
    if (0..CHUNK_SIZE.x).contains(&x) && (0..CHUNK_SIZE.y).contains(&y) && (0..CHUNK_SIZE.z).contains(&z) {
        data.get_local(x, y, z)
    } else {
        // neighbor-chunks:
        get_block_world(world, all_chunks, chunk_pos, x, y, z)
    }
}

/// Mappt (U,V,D) auf (x,y,z), abhängig von axis.
/// d = Position entlang axis.
/// uu/vv sind die Maskenkoordinaten.
fn axis_uvd_to_xyz(
    axis: usize,
    u_axis: usize,
    v_axis: usize,
    uu: i32,
    vv: i32,
    d: i32,
) -> (i32, i32, i32) {
    let mut xyz = [0i32; 3];
    xyz[axis] = d;
    xyz[u_axis] = uu;
    xyz[v_axis] = vv;
    (xyz[0], xyz[1], xyz[2])
}

fn axis_pos_dir(axis: usize) -> FaceDir {
    match axis {
        0 => FaceDir::PosX,
        1 => FaceDir::PosY,
        _ => FaceDir::PosZ,
    }
}
fn axis_neg_dir(axis: usize) -> FaceDir {
    match axis {
        0 => FaceDir::NegX,
        1 => FaceDir::NegY,
        _ => FaceDir::NegZ,
    }
}

/// Emit eines greedy-Quads.
/// Es liegt auf der Grenzfläche bei `d = slice` entlang `axis`.
/// In U/V spannt es `u..u+w` und `v..v+h`.
fn emit_greedy_quad(
    cfg: &BlocksConfigRes,
    block: Block,
    dir: FaceDir,
    axis: usize,
    u_axis: usize,
    v_axis: usize,
    u: i32,
    v: i32,
    d: i32, // slice (Grenzfläche)
    w: i32,
    h: i32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let base = positions.len() as u32;

    // Wir bauen 4 Ecken im (U,V) Rechteck und setzen axis-Koordinate auf d.
    // Danach ordnen wir die Punkte je nach dir so an, dass "außen" CCW ist.
    let p_uvd = [
        (u,     v,     d),
        (u + w, v,     d),
        (u + w, v + h, d),
        (u,     v + h, d),
    ];

    // In xyz umrechnen
    let mut p_xyz = [[0.0f32; 3]; 4];
    for (i, (uu, vv, dd)) in p_uvd.iter().copied().enumerate() {
        let (x, y, z) = axis_uvd_to_xyz(axis, u_axis, v_axis, uu, vv, dd);
        p_xyz[i] = [x as f32, y as f32, z as f32];
    }

    // Normal & Vertex-Reihenfolge abhängig von FaceDir.
    // Wichtig: je nach Richtung muss die Quad-Winding gedreht werden.
    let (n, order) = match dir {
        FaceDir::PosX => ([ 1.0,  0.0,  0.0], [0, 3, 2, 1]),
        FaceDir::NegX => ([-1.0,  0.0,  0.0], [0, 1, 2, 3]),

        FaceDir::PosY => ([ 0.0,  1.0,  0.0], [0, 1, 2, 3]),
        FaceDir::NegY => ([ 0.0, -1.0,  0.0], [0, 3, 2, 1]),

        FaceDir::PosZ => ([ 0.0,  0.0,  1.0], [0, 3, 2, 1]),
        FaceDir::NegZ => ([ 0.0,  0.0, -1.0], [0, 1, 2, 3]),
    };

    let p0 = p_xyz[order[0]];
    let p1 = p_xyz[order[1]];
    let p2 = p_xyz[order[2]];
    let p3 = p_xyz[order[3]];

    // Tile bestimmen
    let face = face_kind(dir);
    let tile = tile_for(cfg, block, face);

    // Rotation: wie bei dir (kannst du später feiner machen pro Richtung)
    let rot = match dir {
        FaceDir::PosZ | FaceDir::PosY => UvRot::R90,
        FaceDir::NegY | FaceDir::NegZ => UvRot::R180,
        FaceDir::PosX => UvRot::R180,
        _ => UvRot::R90,
    };

    // UVs: derzeit wie vorher (eine Textur über das ganze Quad gestreckt).
    // Wenn du wirklich kacheln willst, musst du das bewusst lösen (siehe Hinweis unten).
    let rect = (tile_uv(tile), rot);
    push_uvs(rect.0, rect.1, uvs);

    positions.extend_from_slice(&[p0, p1, p2, p3]);
    normals.extend_from_slice(&[n, n, n, n]);

    indices.extend_from_slice(&[
        base, base + 2, base + 1,
        base, base + 3, base + 2,
    ]);
}
