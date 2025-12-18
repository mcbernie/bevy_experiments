use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::voxel::{chunk::{Block, CHUNK_SIZE, ChunkData, chunk_origin_world}, plugin::VoxelWorld};

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

#[derive(Resource, Default)]
pub struct ChunkLoadQueue {
    pub queued: HashSet<ChunkPos>,
    pub fifo: VecDeque<ChunkPos>,
}

#[derive(Resource)]
pub struct StreamTimer(pub Timer);

#[derive(Message, Clone, Copy)]
pub struct RequestChunkLoad(pub ChunkPos);


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

    // Quelle: ich nehme Kamera. Wenn du Player hast, nimm With<Player>
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

    // 1) missing -> queue
    let wanted = wanted_chunks(center, cfg.view_radius, cfg.y_min, cfg.y_max);
    for pos in wanted.iter().copied() {
        if world.chunks.contains_key(&pos) { continue; }
        if queue.queued.insert(pos) {
            queue.fifo.push_back(pos);
        }
    }

    // 2) unload far (mit Hysterese)
    let mut to_unload = Vec::new();
    for (&pos, &ent) in world.chunks.iter() {
        let d = chebyshev_dist(pos, center);
        if d.x > cfg.unload_radius || d.z > cfg.unload_radius || d.y > (cfg.unload_radius.max(2)) {
            to_unload.push((pos, ent));
        }
    }

    for (pos, ent) in to_unload {
        world.chunks.remove(&pos);
        commands.entity(ent).despawn_children();
        commands.entity(ent).despawn();

        // wichtig: Nachbarn remeshen, weil Seiten wieder sichtbar werden können
        mark_neighbors_dirty(&mut commands, &world, pos);
    }

    // 3) budgeted load requests
    for _ in 0..cfg.load_budget {
        let Some(pos) = queue.fifo.pop_front() else { break; };
        queue.queued.remove(&pos);
        ev_load.write(RequestChunkLoad(pos));
    }
}

fn generate_chunk_data(pos: ChunkPos) -> ChunkData {
    // Platzhalter: mach hier später Noise / Terrain rein
    let mut blocks = vec![Block::Air; (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize];

    // Beispiel: simple Ebene bei world_y == 0
    // Chunk y==0 komplett Dirt/Grass
    if pos.0.y == 0 {
        for z in 0..CHUNK_SIZE.z {
            for y in 0..CHUNK_SIZE.y {
                for x in 0..CHUNK_SIZE.x {
                    let idx = ChunkData::idx(x, y, z);
                    blocks[idx] = if y == CHUNK_SIZE.y - 1 { Block::Grass } else { Block::Dirt };
                }
            }
        }
    }

    ChunkData { blocks }
}

pub fn handle_chunk_load_requests_system(
    mut commands: Commands,
    mut ev: MessageReader<RequestChunkLoad>,
    mut world: ResMut<VoxelWorld>,
) {
    for RequestChunkLoad(pos) in ev.read().copied() {
        if world.chunks.contains_key(&pos) {
            continue;
        }

        let data = generate_chunk_data(pos);
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