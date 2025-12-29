use bevy::prelude::*;
use crate::config::BlocksConfigRes;
use super::chunk::{Block, CHUNK_SIZE, ChunkPos};
use super::plugin::VoxelWorld;
use super::chunk::ChunkData;

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
