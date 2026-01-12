use bevy::{
    camera::visibility::NoFrustumCulling, pbr::{ExtendedMaterial, wireframe::NoWireframe}, prelude::*, render::storage::ShaderStorageBuffer
};

use crate::simulation::{components::{SimulationBuffers, WaterSimulation}};

use super::{
    material::ParticleMaterial, 
};

pub fn spawn_simulation_once(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ParticleMaterial>>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    const PARTICLE_COUNT: usize = 6400;

    info!("Spawning particle simulation.");

    use rand::Rng;

    let mut rng = rand::rng();

    let mut pos_data = Vec::with_capacity(PARTICLE_COUNT);
    let mut vel_data = Vec::with_capacity(PARTICLE_COUNT);

    for _ in 0..PARTICLE_COUNT {
        let x = rng.random_range(-0.8..0.8);
        let y = rng.random_range(-0.8..0.8);
        let z = rng.random_range(-0.8..0.8);

        pos_data.push([x, y, z, 0.0]);
        vel_data.push([0.0, 0.0, 0.0, 0.0]);
    }

    let positions = storage_buffers.add(
        ShaderStorageBuffer::from(pos_data.clone())
    );

    let velocities = storage_buffers.add(
        ShaderStorageBuffer::from(vel_data)
    );


    // --- Dummy Mesh (Vertex Index wird benutzt) ---
    let mesh = meshes.add(Sphere::new(0.05));

    // --- Material liest direkt aus dem Compute-Buffer ---
    let material = materials.add(
            ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::srgba(0.2, 0.7, 1.0, 1.0),
                    ..Default::default()
                },
                extension: ParticleMaterial {
                    positions: positions.clone(),
                    velocities: velocities.clone(),
                }
            }
        );

    // --- Entity ---
    commands.spawn((
        WaterSimulation {
            particle_count: PARTICLE_COUNT as u32,
        },
        SimulationBuffers {
            positions,
            velocities,
        },
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
        InheritedVisibility::VISIBLE,
    )).with_children(|parent| {
        for _ in 0..PARTICLE_COUNT {
            parent.spawn((
                NoWireframe,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                NoFrustumCulling,
            ));
        }
    });
}