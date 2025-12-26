use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use crate::{config::BlocksConfigRes, voxel::{chunk::{Block, CHUNK_SIZE, ChunkData, ChunkPos, face_id}, meshing::{FaceDir, effective_block_kind, face_kind, get_block_world, tile_for}, plugin::VoxelWorld, tile::{UvRot, push_uvs, tile_uv}}};


pub fn build_chunk_mesh_greedy_z(
    cfg: &BlocksConfigRes,
    world: &VoxelWorld,
    all_chunks: &Query<&ChunkData>,
    chunk_pos: ChunkPos,
    data: &ChunkData,
) -> Mesh {
    let mut positions = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];

    let sx = CHUNK_SIZE.x;
    let sy = CHUNK_SIZE.y;
    let sz = CHUNK_SIZE.z;

    // Masken pro Z-Grenzfläche (zwischen z-1 und z)
    let mut mask_id: Vec<u32> = vec![0; (sx * sy) as usize];
    let mut mask_block: Vec<Block> = vec![Block::Air; (sx * sy) as usize];
    let mut mask_dir: Vec<FaceDir> = vec![FaceDir::PosZ; (sx * sy) as usize];

    // z läuft über Grenzflächen: 0..=sz
    for z in 0..=sz {
        // 1) Maske bauen
        for y in 0..sy {
            for x in 0..sx {
                let i = (x + y * sx) as usize;

                let a = if z == 0 {
                    Block::Air
                } else {
                    data.get_local(x, y, z - 1)
                };

                let b = if z == sz {
                    Block::Air
                } else {
                    data.get_local(x, y, z)
                };

                // Wenn du Neighbor-Chunks auch für Greedy willst:
                // ersetze a/b durch get_block_world(world, all_chunks, chunk_pos, x, y, z-1) / z
                // (das ist später Schritt 2)

                let (id, blk, dir) = if a != Block::Air && b == Block::Air {
                    // sichtbare PosZ Face am Block a
                    let is_surface = get_block_world(world, all_chunks, chunk_pos, x, y + 1, z - 1) == Block::Air;
                    let eff = effective_block_kind(a, is_surface);
                    (face_id(cfg, eff, FaceDir::PosZ), eff, FaceDir::PosZ)
                } else if b != Block::Air && a == Block::Air {
                    // sichtbare NegZ Face am Block b
                    let is_surface = get_block_world(world, all_chunks, chunk_pos, x, y + 1, z) == Block::Air;
                    let eff = effective_block_kind(b, is_surface);
                    (face_id(cfg, eff, FaceDir::NegZ), eff, FaceDir::NegZ)
                } else {
                    (0, Block::Air, FaceDir::PosZ)
                };

                mask_id[i] = id;
                mask_block[i] = blk;
                mask_dir[i] = dir;
            }
        }

        // 2) Greedy rectangles auf der Maske (X×Y)
        let mut y = 0;
        while y < sy {
            let mut x = 0;
            while x < sx {
                let i0 = (x + y * sx) as usize;
                let id0 = mask_id[i0];
                if id0 == 0 { x += 1; continue; }

                let blk0 = mask_block[i0];
                let dir0 = mask_dir[i0];

                // Breite w
                let mut w = 1;
                while x + w < sx {
                    let ii = (x + w + y * sx) as usize;
                    if mask_id[ii] != id0 { break; }
                    w += 1;
                }

                // Höhe h
                let mut h = 1;
                'outer: while y + h < sy {
                    for xx in 0..w {
                        let ii = (x + xx + (y + h) * sx) as usize;
                        if mask_id[ii] != id0 { break 'outer; }
                    }
                    h += 1;
                }

                // auf 0 setzen
                for yy in 0..h {
                    for xx in 0..w {
                        let ii = (x + xx + (y + yy) * sx) as usize;
                        mask_id[ii] = 0;
                    }
                }

                // Quad emittieren:
                // Bei PosZ/NegZ ist die Face-Ebene bei z (Grenzfläche),
                // und spannt x..x+w, y..y+h.
                // Für PosZ: Block liegt bei z-1, Face bei z
                // Für NegZ: Block liegt bei z, Face bei z
                // -> wir setzen immer z als Grenzflächen-z, und push_face_rect nutzt z als "start z".
                // Für Z-Faces braucht push_face_rect: x,y,z und w,h und dir.
                push_face_rect(
                    cfg,
                    blk0,
                    dir0,
                    x,
                    y,
                    z,   // Grenzfläche
                    w,   // X-Ausdehnung
                    h,   // Y-Ausdehnung
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut indices,
                );

                x += w;
            }
            y += 1;
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn push_face_rect(
    cfg: &BlocksConfigRes,
    block: Block,
    dir: FaceDir,
    x: i32,
    y: i32,
    z: i32,
    w: i32, // Breite in der 2D-Maske
    h: i32, // Höhe  in der 2D-Maske
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let base = positions.len() as u32;

    let x0 = x as f32;
    let y0 = y as f32;
    let z0 = z as f32;

    // je nach Richtung bestimmen wir, welche Achsen w/h belegen
    let (x1, y1, z1) = match dir {
        FaceDir::PosX | FaceDir::NegX => (x0 + 1.0, y0 + w as f32, z0 + h as f32),
        FaceDir::PosY | FaceDir::NegY => (x0 + w as f32, y0 + 1.0, z0 + h as f32),
        FaceDir::PosZ | FaceDir::NegZ => (x0 + w as f32, y0 + h as f32, z0 + 1.0),
    };

    // 4 Vertices pro Face, außen gesehen CCW (wie bei dir)
    let (p0, p1, p2, p3, n) = match dir {
        FaceDir::PosX => (
            [x1, y0, z0],
            [x1, y0, z1],
            [x1, y1, z1],
            [x1, y1, z0],
            [1.0, 0.0, 0.0],
        ),
        FaceDir::NegX => (
            [x0, y0, z1],
            [x0, y0, z0],
            [x0, y1, z0],
            [x0, y1, z1],
            [-1.0, 0.0, 0.0],
        ),
        FaceDir::PosY => (
            [x0, y1, z0],
            [x1, y1, z0],
            [x1, y1, z1],
            [x0, y1, z1],
            [0.0, 1.0, 0.0],
        ),
        FaceDir::NegY => (
            [x0, y0, z1],
            [x1, y0, z1],
            [x1, y0, z0],
            [x0, y0, z0],
            [0.0, -1.0, 0.0],
        ),
        FaceDir::PosZ => (
            [x1, y0, z1],
            [x0, y0, z1],
            [x0, y1, z1],
            [x1, y1, z1],
            [0.0, 0.0, 1.0],
        ),
        FaceDir::NegZ => (
            [x0, y0, z0],
            [x1, y0, z0],
            [x1, y1, z0],
            [x0, y1, z0],
            [0.0, 0.0, -1.0],
        ),
    };

    let face = face_kind(dir);
    let tile = tile_for(cfg, block, face);

    let rot = match dir {
        FaceDir::PosX | FaceDir::NegX | FaceDir::PosZ | FaceDir::NegZ => UvRot::R180,
        _ => UvRot::R0,
    };

    // TODO: UVs müssen für (w,h) gekachelt werden.
    // Übergangslösung: erstmal wie vorher (streckt).
    let rect = (tile_uv(tile), rot);
    push_uvs(rect.0, rect.1, uvs);

    positions.extend_from_slice(&[p0, p1, p2, p3]);
    normals.extend_from_slice(&[n, n, n, n]);

    indices.extend_from_slice(&[
        base, base + 2, base + 1,
        base, base + 3, base + 2,
    ]);
}
