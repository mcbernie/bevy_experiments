use bevy::{camera::primitives::Aabb, prelude::*};
use crate::app_state::{AppState, LoadingProgress};
use super::mesh_builder::build_voxel_surface_mesh;
use super::chunk::*;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(AppState::Loading), load_meshing_data)
        .add_systems(Update, 
            do_meshing.run_if(in_state(AppState::Loading))
        );
    }
}

fn load_meshing_data(
    mut commands: Commands,
) {
    let size = UVec3::new(16, 16, 16);
    let mut density = vec![0.0; (size.x * size.y * size.z) as usize];


    for z in 0..size.z {
        for y in 0..size.y {
            for x in 0..size.x {
                let idx = (x + y*size.x + z*size.x*size.y) as usize;

                // 1) Boden
                if y < 4 {
                    density[idx] = 1.0;
                }

                // 2) Säule in der Mitte
                let cx = size.x / 2;
                let cz = size.z / 2;
                if x == cx && z == cz && y < 10 {
                    density[idx] = 1.0;
                }

                // 3) Schwebende Plattform
                if (x >= 5 && x <= 10) &&
                    (z >= 5 && z <= 10) &&
                    (y >= 10 && y <= 13)
                {
                    density[idx] = 1.0;
                }
            }
        }
    }

    commands.spawn((
        VoxelChunk { size, density },
        ChunkCoord(IVec3::ZERO),
        NeedsMeshing,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

}

fn do_meshing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &VoxelChunk), With<NeedsMeshing>>,
    mut progress: If<ResMut<LoadingProgress>>,
) {
    for (entity, chunk) in &query {
        let mesh = build_voxel_surface_mesh(&chunk);

        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.7, 0.4),
            ..default()
        });

        commands.entity(entity)
            .insert((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                Visibility::Visible,
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Aabb::from_min_max(
                    Vec3::ZERO,
                    Vec3::new(
                        chunk.size.x as f32,
                        chunk.size.y as f32,
                        chunk.size.z as f32,
                    ),
                ),
            ))
            .remove::<NeedsMeshing>();
    }
    progress.meshing_data_loaded = true;
}
