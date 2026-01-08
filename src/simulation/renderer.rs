use bevy::{
    asset::RenderAssetUsages, 
    mesh::PrimitiveTopology, 
    prelude::*, 
    render::{
        render_resource::ShaderType, 
        storage::ShaderStorageBuffer
    }
};

use crate::simulation::structs::ParticlePosition;

use super::{
    material::ParticleMaterial, 
    structs::SharedComputeBuffers
};

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
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 1.0, 0.0]; num_particles],
    );

    meshes.add(mesh)
}

fn create_point_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    num_particles: usize,
) -> Handle<Mesh> {

    let mut mesh = Mesh::new(
        PrimitiveTopology::PointList,
        RenderAssetUsages::default(),
    );

    // Dummy-Vertices, werden ignoriert
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0]; num_particles],
    );

    meshes.add(mesh)
}

fn create_billboard_mesh(
    meshes: &mut Assets<Mesh>,
    num_particles: usize,
) -> Handle<Mesh> {

    let mut positions = Vec::with_capacity(num_particles * 6);

    for _ in 0..num_particles {
        // Dummy-Vertices – werden im Shader ersetzt
        positions.extend_from_slice(&[
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

    meshes.add(mesh)
}



pub fn init_compute_buffers(
    mut commands: Commands,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let num_particles = 1024;


    let positions_data = vec![
        ParticlePosition { pos: [0.0; 3], _pad: 0.0 };
        num_particles
    ];

    let positions = storage_buffers.add(
        ShaderStorageBuffer::from(
            &positions_data,
        )
    );

    info!("Initialized compute buffers.");
    commands.insert_resource(SharedComputeBuffers {
        positions,
    });
}

pub fn spawn_particles(
    mut commands: Commands,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    compute: Res<SharedComputeBuffers>,
) {
    
    let mesh = create_billboard_mesh(&mut meshes, 1024);
    //let mesh = create_particle_mesh(meshes, 1024);

    let material = materials.add(ParticleMaterial {
        positions: compute.positions.clone(),
    });

    info!("Spawning particles.");   
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
    ));
}