use bevy::{
    asset::RenderAssetUsages, color::palettes::css::{GREEN, LIME, RED}, mesh::PrimitiveTopology, prelude::*, render::storage::ShaderStorageBuffer
};

use crate::{JITTER_STRENGTH, PARTICLE_SPAWN_DENSITY, simulation::{self, assets::SimulationParams, components::TransformData, material::ParticleMaterial, spawn::Spawner}};

use super::{
    components::SimulationBuffers
};

pub fn spawn_simulation_once(
    mut commands: Commands,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {

    let position = Transform::from_xyz(0.0, 6.0, 0.0).with_scale(Vec3::new(16.0, 12.0, 8.0));
    let mut simulation_params = SimulationParams::default();

    warn!("Setting simulation bounds size to: {:?}", position.scale);
    warn!("Setting simulation centre to: {:?}", position.translation);

    simulation_params.bounds_size = position.scale;
    simulation_params.centre = position.translation;

    info!("Spawning particle simulation.");

    let spawner = Spawner::new(
        PARTICLE_SPAWN_DENSITY,
        Vec3::ZERO,
        JITTER_STRENGTH,
        simulation::spawn::SpawnRegion::new(Vec3::new(0.0,6.0,0.0), 4.0),
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

    let vertex_count = spawn_data.points.len() as usize * 6;
    let v_positions = vec![[0.0, 0.0, 0.0]; vertex_count];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_positions);

    let mesh = meshes.add(mesh);

    let material = materials.add(
        ParticleMaterial {
            positions: positions.clone(), // currently using only one buffer for rendering
            velocities: velocities.clone(),
            debug_buffer: debug_buffer_handle.clone(),
            color: Vec4::new(0.2, 0.5, 1.0, 1.0),
            radius: 0.5,
        }
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
        position,
        TransformData {
            scale: position.scale,
            translation: position.translation,
        },
        GlobalTransform::IDENTITY,
        InheritedVisibility::VISIBLE,
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
    ));
}


pub fn update_simulation_gizmo(
    mut gizmos: Gizmos,
    query: Query<(&SimulationParams, &Transform)>,
) {
    for (params, transform) in &query {
        let center = transform.translation;

        //simulation::spawn::SpawnRegion::new(Vec3::new(0.0,6.0,0.0), 2.0),
        gizmos.cube(
            Transform {
                translation: Vec3::new(0.0, 6.0, 0.0),
                rotation: transform.rotation,
                scale: Vec3::ONE * 4.0,
            },
            RED,
        );

        gizmos.cube(
            Transform {
                translation: center,
                rotation: transform.rotation,
                scale: transform.scale,
            },
            LIME,
        );
    }
}