use bevy::{
    color::palettes::css::LIME, prelude::*, render::storage::ShaderStorageBuffer
};

use crate::{JITTER_STRENGTH, PARTICLE_SPAWN_DENSITY, simulation::{self, assets::SimulationParams, spawn::Spawner}};

use super::{
    components::SimulationBuffers
};

pub fn spawn_simulation_once(
    mut commands: Commands,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut simulation_params = SimulationParams::default();

    simulation_params.bounds_size = Vec3::new(40.0, 80.0, 40.0);

    info!("Spawning particle simulation.");

    let spawner = Spawner::new(
        PARTICLE_SPAWN_DENSITY,
        Vec3::ZERO,
        JITTER_STRENGTH,
        simulation::spawn::SpawnRegion::new(Vec3::new(0.0, 15.0,0.0), 5.0),
    );

    let spawn_data = spawner.spawn();
    simulation_params.particle_count = spawn_data.points.len() as u32;


    let positions = storage_buffers.add(ShaderStorageBuffer::from(spawn_data.points.clone()));
    let velocities = storage_buffers.add(ShaderStorageBuffer::from(spawn_data.velocities.clone()));

    // currently not used...
    let debug_buffer = ShaderStorageBuffer::from(vec![0u32; simulation_params.particle_count as usize]);
    let debug_buffer_handle = storage_buffers.add(
        debug_buffer
    );

    // --- Entity ---
    commands.spawn((
        Name::new("Particle Simulation"),
        SimulationBuffers {
            positions,
            velocities,
            debug_buffer: debug_buffer_handle.clone(),
        },
        simulation_params,
        Transform::IDENTITY,
        GlobalTransform::IDENTITY,
        InheritedVisibility::VISIBLE,
        //Mesh3d(mesh.clone()),
        //MeshMaterial3d(material.clone()),
    ));
}


pub fn update_simulation_gizmo(
    mut gizmos: Gizmos,
    query: Query<(&SimulationParams, &Transform)>,
) {
    for (params, transform) in &query {
        let size = params.bounds_size;

        let center = transform.translation;

        gizmos.cube(
            Transform {
                translation: center,
                rotation: transform.rotation,
                scale: size,
            },
            LIME,
        );
    }
}