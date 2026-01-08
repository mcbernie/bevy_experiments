use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};

use crate::simulation::{material::ParticleMaterial, structs::SharedComputeBuffers};

fn create_particle_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    num_particles: usize,
) -> Handle<Mesh> {
    let mut mesh = Mesh::new(
        PrimitiveTopology::PointList,
        RenderAssetUsages::default(),
    );

    // Dummy-Positionen, werden im Shader ignoriert
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0]; num_particles],
    );

    meshes.add(mesh)
}


pub fn spawn_particles(
    mut commands: Commands,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    compute: Res<SharedComputeBuffers>,
) {
    let mesh = create_particle_mesh(meshes, 1024);

    let material = materials.add(ParticleMaterial {
        positions: compute.positions.clone(),
    });

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
    ));
}