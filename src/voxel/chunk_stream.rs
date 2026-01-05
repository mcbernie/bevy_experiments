use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::voxel::{chunk::{Block, CHUNK_SIZE, ChunkData, chunk_origin_world}, chunk_queue::{ChunkLoadQueue, QueuedChunk}, chunk_store::ChunkSaveStore, plugin::VoxelWorld};

use super::chunk::{ChunkDirty, world_to_chunk_pos, ChunkPos};

#[derive(Resource)]
pub struct ChunkStreamConfig {
    pub view_radius: i32,     // in Chunks
    pub unload_radius: i32,   // view + hysterese (z.B. +2)
    pub tick_seconds: f32,    // z.B. 0.2
    pub y_min: i32,           // welche Chunk-Ebenen laden (z.B. 0..0)
    pub y_max: i32,
    pub load_budget: usize,   // wie viele Chunks pro Tick
}

#[derive(Resource)]
pub struct StreamTimer(pub Timer);

#[derive(Message, Clone, Copy)]
pub struct RequestChunkLoad(pub ChunkPos);

fn chunk_priority(
    center: ChunkPos,
    forward: Vec3,
    pos: ChunkPos,
) -> f32 {
    let d = (pos.0 - center.0).as_vec3();

    let dist = d.length();                 // Nähe
    let dir = d.normalize_or_zero();
    let facing = forward.dot(dir);         // [-1..1], vorne = 1

    // Gewichtung:
    dist * 1.0 - facing * 2.0
}


fn wanted_chunks(center: ChunkPos, r: i32, y_min: i32, y_max: i32) -> HashSet<ChunkPos> {
    let mut set = HashSet::new();
    for x in (center.0.x - r)..=(center.0.x + r) {
        for z in (center.0.z - r)..=(center.0.z + r) {
            for y in y_min..=y_max {
                set.insert(ChunkPos(IVec3::new(x, y, z)));
            }
        }
    }
    set
}

fn chebyshev_dist(a: ChunkPos, b: ChunkPos) -> IVec3 {
    (a.0 - b.0).abs()
}

pub fn chunk_stream_tick_system(
    time: Res<Time>,
    cfg: Res<ChunkStreamConfig>,
    mut timer: ResMut<StreamTimer>,

    cam_q: Query<&GlobalTransform, With<Camera3d>>,

    mut world: ResMut<VoxelWorld>,
    mut queue: ResMut<ChunkLoadQueue>,
    mut ev_load: MessageWriter<RequestChunkLoad>,
    mut commands: Commands,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let Ok(cam_tf) = cam_q.single() else { return; };

    let center = world_to_chunk_pos(cam_tf.translation());
    let forward = cam_tf.forward();

    // 1) fehlende Chunks → Heap
    let wanted = wanted_chunks(center, cfg.view_radius, cfg.y_min, cfg.y_max);

    for pos in wanted.iter().copied() {
        if world.chunks.contains_key(&pos) {
            continue;
        }

        if queue.queued.insert(pos) {
            let score = chunk_priority(center, *forward, pos);
            queue.heap.push(QueuedChunk { pos, score });
        }
    }

    // 2) Unload mit Hysterese
    let mut to_unload = Vec::new();
    for (&pos, &ent) in world.chunks.iter() {
        let d = chebyshev_dist(pos, center);
        if d.x > cfg.unload_radius
            || d.z > cfg.unload_radius
            || d.y > cfg.unload_radius.max(2)
        {
            to_unload.push((pos, ent));
        }
    }

    for (pos, ent) in to_unload {
        world.chunks.remove(&pos);
        commands.entity(ent).despawn_children();
        commands.entity(ent).despawn();

        mark_neighbors_dirty(&mut commands, &world, pos);
    }

    // 3) Budgetiertes Laden (Heap!)
    for _ in 0..cfg.load_budget {
        let Some(entry) = queue.heap.pop() else { break; };
        queue.queued.remove(&entry.pos);
        ev_load.write(RequestChunkLoad(entry.pos));
    }
}


use noise::{NoiseFn, Perlin};

pub fn generate_chunk_data(pos: ChunkPos) -> ChunkData {
    let mut blocks = vec![Block::Air; (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize];

    // Weltweite Chunk-Offsets
    let chunk_world_x = pos.0.x * CHUNK_SIZE.x;
    let chunk_world_y = pos.0.y * CHUNK_SIZE.y;
    let chunk_world_z = pos.0.z * CHUNK_SIZE.z;

    // Perlin Noise
    let perlin = Perlin::new(42); // Seed

    // Terrain-Parameter
    let base_height = 5;     // Grundniveau
    let height_scale = 6.0;  // Höhenvariation
    let noise_scale = 0.05;   // Frequenz

    for z in 0..CHUNK_SIZE.z {
        for x in 0..CHUNK_SIZE.x {
            // World-Koordinaten (wichtig!)
            let wx = (chunk_world_x + x) as f64;
            let wz = (chunk_world_z + z) as f64;

            let n = perlin.get([wx * noise_scale, wz * noise_scale]);
            let height = base_height + (n * height_scale) as i32;
            if height <= 0 {
                debug!("Negative Höhe bei Chunk {:?}, x={}, z={}: height={}", pos, wx, wz, height);
            }

            // 0 ist immer Boden!!
            let idx = ChunkData::idx(x, 0, z);
            blocks[idx] = Block::Stone;

            for y in 1..CHUNK_SIZE.y {
                let wy = chunk_world_y + y;
                let idx = ChunkData::idx(x, y, z);

                blocks[idx] = if wy > height {
                    Block::Air
                } else if wy == height {
                    Block::Grass
                } else if wy > height - 4 {
                    Block::Dirt
                } else {
                    Block::Stone
                };
            }
        }
    }

    ChunkData { blocks }
}

pub fn handle_chunk_load_requests_system(
    mut commands: Commands,
    mut ev: MessageReader<RequestChunkLoad>,
    mut world: ResMut<VoxelWorld>,
    store: ResMut<ChunkSaveStore>,
) {
    for RequestChunkLoad(pos) in ev.read().copied() {
        if world.chunks.contains_key(&pos) {
            continue;
        }

        let data = store
            .load_chunk(pos)
            .unwrap_or_else(|| generate_chunk_data(pos));

        let origin = chunk_origin_world(pos);

        let ent = commands.spawn((
            pos,
            data,
            ChunkDirty,
            Transform::from_translation(origin),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        )).id();

        world.chunks.insert(pos, ent);

        // Nachbarn ebenfalls dirty: ihre Seiten ändern sich jetzt
        mark_neighbors_dirty(&mut commands, &world, pos);
    }
}

fn neighbors_6(pos: ChunkPos) -> [ChunkPos; 6] {
    let p = pos.0;
    [
        ChunkPos(p + IVec3::X),
        ChunkPos(p - IVec3::X),
        ChunkPos(p + IVec3::Y),
        ChunkPos(p - IVec3::Y),
        ChunkPos(p + IVec3::Z),
        ChunkPos(p - IVec3::Z),
    ]
}

fn mark_neighbors_dirty(commands: &mut Commands, world: &VoxelWorld, pos: ChunkPos) {
    for n in neighbors_6(pos) {
        if let Some(&e) = world.chunks.get(&n) {
            commands.entity(e).insert(ChunkDirty);
        }
    }
}