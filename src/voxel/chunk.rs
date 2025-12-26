use bevy::prelude::*;

use crate::{config::BlocksConfigRes, voxel::meshing::{FaceDir, face_kind, tile_for}};

pub const CHUNK_SIZE: IVec3 = IVec3::new(16, 16, 16);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub IVec3); // in Chunk-Koordinaten (nicht World Units)

#[derive(Component)]
pub struct ChunkData {
    pub blocks: Vec<Block>,
}

#[derive(Component)]
pub struct ChunkDirty;

#[derive(Component)]
pub struct ChunkMesh; // Marker: diese Entity ist das Mesh des Chunks (optional)

impl ChunkData {
    #[inline]
    pub fn idx(x: i32, y: i32, z: i32) -> usize {
        // wo liegt x, y, z im Vektor
        // z ist die plane
        // y ist eine reihe
        // und x ist ein block in der reihe
        // x + (CHUNK.x * y) + (CHUNK.x * CHUNK.y * z)
        (x + CHUNK_SIZE.x * (y + CHUNK_SIZE.y * z)) as usize
    }

    #[inline]
    pub fn get_local(&self, x: i32, y: i32, z: i32) -> Block {
        if x < 0 || y < 0 || z < 0 || x >= CHUNK_SIZE.x || y >= CHUNK_SIZE.y || z >= CHUNK_SIZE.z {
            return Block::Air;
        }
        self.blocks[Self::idx(x, y, z)]
    }
}

/// Sehr simpel: pro Block + Richtung eine ID.
/// Wichtig: Greedy darf nur Flächen zusammenfassen, deren ID identisch ist.
#[inline]
pub fn face_id(cfg: &BlocksConfigRes, block: Block, dir: FaceDir) -> u32 {
    let face = face_kind(dir);
    let tile = tile_for(cfg, block, face);
    // tile ist (u32,u32) -> packen wir in u32
    let t = (tile.0 & 0xFFFF) | ((tile.1 & 0xFFFF) << 16);

    let d = match dir {
        FaceDir::PosX => 1,
        FaceDir::NegX => 2,
        FaceDir::PosY => 3,
        FaceDir::NegY => 4,
        FaceDir::PosZ => 5,
        FaceDir::NegZ => 6,
    };

    (t as u32) ^ ((d as u32) << 24)
}

pub fn chunk_origin_world(pos: ChunkPos) -> Vec3 {
    Vec3::new(
        (pos.0.x * CHUNK_SIZE.x) as f32,
        (pos.0.y * CHUNK_SIZE.y) as f32,
        (pos.0.z * CHUNK_SIZE.z) as f32,
    )
}

pub fn world_to_chunk_pos(world: Vec3) -> ChunkPos {
    // Block-Koordinaten (floor für negatives)
    let bx = world.x.floor() as i32;
    let by = world.y.floor() as i32;
    let bz = world.z.floor() as i32;

    // Euclid-Division: -1..-16 landet korrekt in Chunk -1
    ChunkPos(IVec3::new(
        bx.div_euclid(CHUNK_SIZE.x),
        by.div_euclid(CHUNK_SIZE.y),
        bz.div_euclid(CHUNK_SIZE.z),
    ))
}