use bevy::prelude::*;

#[derive(Component)]
pub struct VoxelChunk {
    pub size: UVec3,        // z.B. 16x16x16
    pub density: Vec<f32>, // size.x * size.y * size.z
}

impl VoxelChunk {
    #[inline]
    pub fn idx(&self, x: u32, y: u32, z: u32) -> usize {
        (x + y * self.size.x + z * self.size.x * self.size.y) as usize
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> f32 {
        self.density[self.idx(x, y, z)]
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, v: f32) {
        let i = self.idx(x, y, z);
        self.density[i] = v;
    }
}

#[derive(Component)]
pub struct ChunkCoord(pub IVec3);

#[derive(Component)]
pub struct NeedsMeshing;
