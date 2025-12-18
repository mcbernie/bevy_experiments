use bevy::prelude::*;
use std::collections::HashMap;

use crate::voxel::plugin::VoxelWorld;

use super::chunk::{ChunkPos, ChunkData, Block};

#[derive(Component)]
pub struct ChunkModified;

#[derive(Resource, Default)]
pub struct ChunkSaveStore {
    // kompletter Chunk-Blockbuffer
    pub saved: HashMap<ChunkPos, Vec<Block>>,
}

impl ChunkSaveStore {
    pub fn load_chunk(&self, pos: ChunkPos) -> Option<ChunkData> {
        self.saved.get(&pos).map(|blocks| ChunkData { blocks: blocks.clone() })
    }

    pub fn save_chunk(&mut self, pos: ChunkPos, data: &ChunkData) {
        self.saved.insert(pos, data.blocks.clone());
    }
}

#[derive(Message, Clone, Copy)]
pub struct RequestChunkUnload(pub ChunkPos, pub Entity);

pub fn handle_chunk_unload_requests_system(
    mut commands: Commands,
    mut ev: MessageReader<RequestChunkUnload>,
    mut world: ResMut<VoxelWorld>,
    mut store: ResMut<ChunkSaveStore>,
    q_data: Query<&ChunkData>,
    q_modified: Query<(), With<ChunkModified>>,
) {
    for RequestChunkUnload(pos, ent) in ev.read().copied() {
        // aus map nehmen
        world.chunks.remove(&pos);

        // nur speichern wenn modified
        if q_modified.get(ent).is_ok() {
            if let Ok(data) = q_data.get(ent) {
                store.save_chunk(pos, data);
            }
        }

        commands.entity(ent).despawn();
    }
}