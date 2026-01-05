use bevy::prelude::*;

#[derive(Component)]
pub struct VoxelChunk {
    pub size: UVec3,        // z.B. 16x16x16
    pub density: Vec<f32>, // size.x * size.y * size.z
}

#[derive(Component)]
pub struct ChunkCoord(pub IVec3);

#[derive(Component)]
pub struct NeedsMeshing;
