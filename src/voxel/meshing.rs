use bevy::mesh::Indices;
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology};
use bevy::prelude::*;

use crate::config::BlocksConfigRes;

use super::chunk::{Block, CHUNK_SIZE, ChunkPos};
use super::plugin::VoxelWorld;

use super::chunk::ChunkData;
use super::tile::{UvRot, push_uvs, tile_uv};


#[derive(Clone, Copy)]
pub enum FaceDir {
    PosX, // rechts
    NegX, // links
    PosY, // hinten
    NegY, // vorne
    PosZ, // oben
    NegZ, // unten
}

#[derive(Clone, Copy)]
pub enum BlockFace { Top, Bottom, Side }

pub fn build_chunk_mesh_with_neighbors(
    cfg: &BlocksConfigRes,
    world: &VoxelWorld,
    all_chunks: &Query<&ChunkData>,
    chunk_pos: ChunkPos,
    data: &ChunkData,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for z in 0..CHUNK_SIZE.z {
        for y in 0..CHUNK_SIZE.y {
            for x in 0..CHUNK_SIZE.x {
                let raw = data.get_local(x, y, z);
                if raw == Block::Air { continue; }

                let is_surface = get_block_world(world, all_chunks, chunk_pos, x, y + 1, z) == Block::Air;
                let block = effective_block_kind(raw, is_surface);

                // X+ (rechts ist luft, also sichtbare seite)
                if get_block_world(&world, &all_chunks, chunk_pos, x+1, y, z) == Block::Air {
                    push_face(&cfg, block, FaceDir::PosX, x, y, z, &mut positions, &mut normals, &mut uvs, &mut indices);
                }

                // X- (links ist luft, also sichtbare seite)
                if get_block_world(&world, &all_chunks, chunk_pos, x - 1, y, z) == Block::Air {
                    push_face(&cfg, block, FaceDir::NegX, x, y, z, &mut positions, &mut normals, &mut uvs, &mut indices);
                }

                // Y+ (oben ist luft, also sichtbare seite)
                if get_block_world(&world, &all_chunks, chunk_pos, x, y + 1, z) == Block::Air {
                    push_face(&cfg, block, FaceDir::PosY, x, y, z, &mut positions, &mut normals, &mut uvs, &mut indices);
                }

                // Y- (vorne ist luft, also sichtbare seite)
                if get_block_world(&world, &all_chunks, chunk_pos, x, y - 1, z) == Block::Air {
                    push_face(&cfg, block, FaceDir::NegY, x, y, z, &mut positions, &mut normals, &mut uvs, &mut indices);
                }

                // Z+ (hinten ist luft, also sichtbare seite)
                if get_block_world(&world, &all_chunks, chunk_pos, x, y, z + 1) == Block::Air {
                    push_face(&cfg, block, FaceDir::PosZ, x, y, z, &mut positions, &mut normals, &mut uvs, &mut indices);
                }

                // Z- (vorne ist luft, also sichtbare seite)
                if get_block_world(&world, &all_chunks, chunk_pos, x, y, z - 1) == Block::Air {
                    push_face(&cfg, block, FaceDir::NegZ, x, y, z, &mut positions, &mut normals, &mut uvs, &mut indices);
                }

            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    

    mesh
}

fn push_face(
    cfg: &BlocksConfigRes,
    block: Block, 
    dir: FaceDir,
    x: i32,
    y: i32,
    z: i32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    
    let base = positions.len() as u32;

    let x0 = x as f32;
    let x1 = x as f32 + 1.0;
    let y0 = y as f32;
    let y1 = y as f32 + 1.0;
    let z0 = z as f32;
    let z1 = z as f32 + 1.0;

    // 4 Vertices pro Face, außen gesehen CCW
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

    let rect = (tile_uv(tile), rot);
    push_uvs(rect.0, rect.1, uvs);

    positions.extend_from_slice(&[p0, p1, p2, p3]);
    normals.extend_from_slice(&[n, n, n, n]);

    // Triangles (CCW)
    indices.extend_from_slice(&[
        base, base + 2, base + 1,
        base, base + 3, base + 2,
    ]);
}

fn neighbor_coord(base: ChunkPos, x: i32, y: i32, z: i32) -> (ChunkPos, IVec3) {
    let sx = CHUNK_SIZE.x;
    let sy = CHUNK_SIZE.y;
    let sz = CHUNK_SIZE.z;

    let ox = x.div_euclid(sx);
    let oy = y.div_euclid(sy);
    let oz = z.div_euclid(sz);

    let lx = x.rem_euclid(sx);
    let ly = y.rem_euclid(sy);
    let lz = z.rem_euclid(sz);

    (ChunkPos(base.0 + IVec3::new(ox, oy, oz)), IVec3::new(lx, ly, lz))
}

pub fn get_block_world(
    world: &VoxelWorld,
    all_chunks: &Query<&ChunkData>,
    base_chunk: ChunkPos,
    x: i32,
    y: i32,
    z: i32,
) -> Block {
    let (cp, local) = neighbor_coord(base_chunk, x, y, z);

    let Some(&e) = world.chunks.get(&cp) else {
        return Block::Air; // außerhalb geladener Welt = Luft (oder später: "Unknown")
    };

    let Ok(data) = all_chunks.get(e) else {
        return Block::Air;
    };

    data.get_local(local.x, local.y, local.z) // deine lokale get()-Methode, ohne "out of bounds = Air"
}

pub fn effective_block_kind(
    block: Block,
    above_is_air: bool,
) -> Block {
    match block {
        Block::Grass if !above_is_air => Block::Dirt,
        Block::Dirt if above_is_air => Block::Grass,
        other => other,
    }
}

pub fn face_kind(dir: FaceDir) -> BlockFace {
    match dir {
        FaceDir::PosY => BlockFace::Top,
        FaceDir::NegY => BlockFace::Bottom,
        _ => BlockFace::Side,
    }
}

pub fn tile_for(cfg: &BlocksConfigRes, block: Block, face: BlockFace) -> (u32, u32) {
    let key = match block {
        Block::Grass => "grass",
        Block::Dirt => "dirt",
        Block::Stone => "stone",
        Block::Air => "air",
    };

    let def = cfg.0.blocks.get(key).expect("block missing in config");

    // Fallback: all -> specific
    if let Some(all) = def.all { return all; }

    match face {
        BlockFace::Top => def.top.expect("missing top"),
        BlockFace::Bottom => def.bottom.expect("missing bottom"),
        BlockFace::Side => def.side.expect("missing side"),
    }
}
