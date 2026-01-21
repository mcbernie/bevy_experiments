use bevy::{
    asset::RenderAssetUsages, color::palettes::css::LIME, mesh::PrimitiveTopology, prelude::*, render::storage::ShaderStorageBuffer
};

use crate::{PARTICLE_COUNT, simulation::assets::SimulationParams};

use super::{
    material::ParticleMaterial, 
    components::SimulationBuffers
};

pub fn spawn_simulation_once(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {

    info!("Spawning particle simulation.");
    let mut pos_data = Vec::with_capacity(PARTICLE_COUNT as usize);
    let mut vel_data = Vec::with_capacity(PARTICLE_COUNT as usize);

    // create a grid of particles inside the bound box
    let simulation_params = SimulationParams::default();
    let bound_box =simulation_params.bounds_size;

    let volume = bound_box.x * bound_box.y * bound_box.z;
    let volume_per_particle = volume / PARTICLE_COUNT as f32;
    let spacing = volume_per_particle.cbrt();

    let nx = (bound_box.x / spacing).floor() as usize;
    let ny = (bound_box.y / spacing).floor() as usize;
    let nz = (bound_box.z / spacing).floor() as usize;

    let mut count = 0;

    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                if count >= PARTICLE_COUNT { break; }

                let pos = Vec3::new(
                    (x as f32 + 0.5) * spacing - bound_box.x * 0.5,
                    (y as f32 + 0.5) * spacing,
                    (z as f32 + 0.5) * spacing - bound_box.z * 0.5,
                );

                pos_data.push([pos.x, pos.y, pos.z, 0.0]);
                vel_data.push([0.0, 0.0, 0.0, 0.0]);

                count += 1;
            }
        }
    }


    let positions = storage_buffers.add(ShaderStorageBuffer::from(pos_data.clone()));
    let velocities = storage_buffers.add(ShaderStorageBuffer::from(vel_data.clone()));

    // currently not used...
    let debug_buffer = ShaderStorageBuffer::from(vec![0u32; PARTICLE_COUNT as usize]);
    let debug_buffer_handle = storage_buffers.add(
        debug_buffer
    );

    let vertex_count = PARTICLE_COUNT as usize * 6;
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
        Transform::IDENTITY,
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
        let size = params.bounds_size;

        let center = transform.translation
            + Vec3::new(0.0, params.bounds_size.y * 0.5, 0.0);

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