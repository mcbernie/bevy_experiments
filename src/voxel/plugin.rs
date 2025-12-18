use bevy::image::ImageSampler;
use bevy::prelude::*;

use crate::app_state::{AppState, LoadingProgress};
use crate::config::BlocksConfigRes;
use crate::voxel::chunk_stream::{ChunkLoadQueue, ChunkStreamConfig, RequestChunkLoad, StreamTimer, chunk_stream_tick_system, handle_chunk_load_requests_system};

use super::meshing::build_chunk_mesh_with_neighbors;
use super::chunk::{Block, CHUNK_SIZE, ChunkData, ChunkDirty, ChunkPos, chunk_origin_world};
use super::components::ChunkMeshChild;


#[derive(Resource)]
pub struct VoxelMaterials {
    pub blocks: Handle<StandardMaterial>,
}

use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct VoxelWorld {
    pub chunks: HashMap<ChunkPos, Entity>,
}

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VoxelWorld>()
        .insert_resource(ChunkStreamConfig {
            view_radius: 4,
            unload_radius: 6,
            tick_seconds: 0.2,
            y_min: 0,
            y_max: 0,
            load_budget: 2,
        })
        .insert_resource(ChunkLoadQueue::default())
        .insert_resource(StreamTimer(Timer::from_seconds(0.2, TimerMode::Repeating)))
        .add_message::<RequestChunkLoad>()
        .add_systems(Update, (setup_voxel_materials, poll_voxel_loaded).run_if(in_state(AppState::Loading)))
        .add_systems(OnEnter(AppState::InGame), spawn_chunks)
        .add_systems(
            Update,
            (chunk_stream_tick_system,handle_chunk_load_requests_system,remesh_dirty_chunks).run_if(in_state(AppState::InGame)),
        );
    }
}

fn spawn_chunks(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
) {
    for cz in -1..=1 {
        for cx in -1..=1 {
            let pos = ChunkPos(IVec3::new(cx, 0, cz));
            let data = ChunkData { blocks: make_test_blocks() };

            let e = commands.spawn((
                pos,
                data,
                ChunkDirty, // beim Start direkt meshen
                Transform::from_translation(chunk_origin_world(pos)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            )).id();


            world.chunks.insert(pos, e);
        }
    }
}

fn setup_voxel_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    loaded: Option<Res<BlocksConfigRes>>,
) {

    let Some(cfg) = loaded else { return; }; 

    let texture_path = &cfg.0.atlas.texture;
    let tex: Handle<Image> = asset_server.load(format!("textures/{}", texture_path));

    // Nearest: wichtig für Pixelart
    if let Some(img) = images.get_mut(&tex) {
        img.sampler = ImageSampler::nearest();
    }

    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(tex),
        perceptual_roughness: 1.0,
        unlit: true,
        ..default()
    });

    commands.insert_resource(VoxelMaterials { blocks: mat });
}

fn poll_voxel_loaded(
    mats: Option<Res<VoxelMaterials>>,
    loaded_cfg: Option<Res<BlocksConfigRes>>,
    mut progress: ResMut<LoadingProgress>,
) {
    let Some(_cfg) = loaded_cfg else { return; };

    // Atlas-Texture geladen?
    if mats.is_some() && !progress.atlas_loaded {
        progress.atlas_loaded = true;
    }
}

fn remesh_dirty_chunks(
    mut commands: Commands,
    world: Res<VoxelWorld>,
    all_chunks: Query<&ChunkData>,
    voxel_mats: Res<VoxelMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    dirty: Query<(Entity, &ChunkPos, &ChunkData, Option<&Children>), With<ChunkDirty>>,
    //chunks: Query<(Entity, &ChunkPos, &ChunkData, Option<&Children>), With<ChunkDirty>>,
    mesh_children: Query<Entity, With<ChunkMeshChild>>,
    cfg: Res<BlocksConfigRes>,
) {
    for (chunk_e, &chunk_pos, data, children_opt) in &dirty {
        let mesh = build_chunk_mesh_with_neighbors(&cfg, &world, &all_chunks, chunk_pos, data);
        let mesh_handle = meshes.add(mesh);

        // vorhandenes Mesh-Kind suchen
        let mut existing_child: Option<Entity> = None;
        if let Some(children) = children_opt {
            for &c in children {
                if mesh_children.get(c).is_ok() {
                    existing_child = Some(c);
                    break;
                }
            }
        }

        match existing_child {
            Some(child) => {
                commands.entity(child).insert(Mesh3d(mesh_handle));
            }
            None => {
                commands.entity(chunk_e).with_children(|p| {
                    p.spawn((
                        ChunkMeshChild,
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(voxel_mats.blocks.clone()),
                        Transform::IDENTITY,
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                    ));
                });
            }
        }

        commands.entity(chunk_e).remove::<ChunkDirty>();
    }
}

#[inline]
pub fn block_index(x: i32, y: i32, z: i32) -> usize {
    (x + CHUNK_SIZE.x * (y + CHUNK_SIZE.y * z)) as usize
}

pub fn make_test_blocks() -> Vec<Block> {
    let size = (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize;
    let mut blocks = vec![Block::Air; size];

    // Boden (y == 0)
    for z in 0..CHUNK_SIZE.z {
        for x in 0..CHUNK_SIZE.x {
            let idx = block_index(x, 0, z);
            blocks[idx] = Block::Grass;
        }
    }

    // kleine Säule in der Mitte
    let cx = CHUNK_SIZE.x / 2;
    let cz = CHUNK_SIZE.z / 2;
    for y in 1..5 {
        let idx = block_index(cx, y, cz);
        blocks[idx] = Block::Grass;
    }

    blocks
}