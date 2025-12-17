use bevy::prelude::*;

pub const CHUNK_SIZE: IVec3 = IVec3::new(16, 16, 16);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Air,
    Solid,
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
    pub fn new() -> Self {
        let mut blocks = vec![Block::Air; (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize];

        for z in 0..CHUNK_SIZE.z {
            for x in 0..CHUNK_SIZE.x {
                blocks[Self::idx(x, 0, z)] = Block::Solid;
            }
        }
        for y in 1..6 {
            blocks[Self::idx(8, y, 8)] = Block::Solid;
        }

        Self { blocks }
    }

    #[inline]
    fn idx(x: i32, y: i32, z: i32) -> usize {
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

pub fn chunk_origin_world(pos: ChunkPos) -> Vec3 {
    Vec3::new(
        (pos.0.x * CHUNK_SIZE.x) as f32,
        (pos.0.y * CHUNK_SIZE.y) as f32,
        (pos.0.z * CHUNK_SIZE.z) as f32,
    )
}