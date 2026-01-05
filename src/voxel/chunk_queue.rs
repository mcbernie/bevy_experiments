use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{HashSet, BinaryHeap};

use crate::voxel::chunk::ChunkPos;

#[derive(Clone, Copy)]
pub struct QueuedChunk {
    pub pos: ChunkPos,
    pub score: f32,
}

impl Eq for QueuedChunk {}
impl PartialEq for QueuedChunk {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Ord for QueuedChunk {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap ist Max-Heap → Score umdrehen
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for QueuedChunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Resource, Default)]
pub struct ChunkLoadQueue {
    pub queued: HashSet<ChunkPos>,
    pub heap: BinaryHeap<QueuedChunk>,
}
