use bevy::{
    prelude::*, 
    render::{
        storage::ShaderStorageBuffer
    }
};

use crate::simulation::{components::{SimulationBuffers, WaterSimulation}};

use super::{
    material::ParticleMaterial, 
};

pub fn spawn_simulation_once(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    const PARTICLE_COUNT: usize = 1024;

    info!("Spawning particle simulation.");

    use rand::Rng;

    let mut rng = rand::thread_rng();

    let mut pos_data = Vec::with_capacity(PARTICLE_COUNT);
    let mut vel_data = Vec::with_capacity(PARTICLE_COUNT);

    for _ in 0..PARTICLE_COUNT {
        let x = rng.gen_range(-0.8..0.8);
        let y = rng.gen_range(-0.8..0.8);

        pos_data.push([x, y, 0.0, 1.0]);
        vel_data.push([0.0, 0.0, 0.0, 0.0]);
    }

    let positions = storage_buffers.add(
        ShaderStorageBuffer::from(pos_data)
    );

    let velocities = storage_buffers.add(
        ShaderStorageBuffer::from(vel_data)
    );


    // --- Dummy Mesh (Vertex Index wird benutzt) ---
    let mesh = meshes.add(Rectangle::new(0.5, 0.5));

    // --- Material liest direkt aus dem Compute-Buffer ---
    let material = materials.add(
        ParticleMaterial {
            positions: positions.clone(),
        }
    );

    // --- Entity ---
    let mut childs = commands.spawn((
        WaterSimulation {
            particle_count: PARTICLE_COUNT as u32,
        },
        SimulationBuffers {
            positions,
            velocities,
        },
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
    ));
    for p in 0..PARTICLE_COUNT {
        childs.with_children(|parent| {
            parent.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Name::new(format!("Particle {}", p)),
                InheritedVisibility::VISIBLE,
            ));
        });
    }
}