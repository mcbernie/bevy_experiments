use bevy::{
    asset::RenderAssetUsages, color::palettes::css::LIME, mesh::PrimitiveTopology, prelude::*, render::storage::ShaderStorageBuffer
};

use crate::{JITTER_STRENGTH, PARTICLE_COUNT, simulation::{self, assets::SimulationParams, helper::random_in_unit_sphere}};

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
    let mut simulation_params = SimulationParams::default();

    info!("Spawning particle simulation.");
    let mut pos_data = Vec::with_capacity(PARTICLE_COUNT as usize);
    let mut vel_data = Vec::with_capacity(PARTICLE_COUNT as usize);

    let particle_count: usize = PARTICLE_COUNT as usize;
    let smoothing_radius: f32 = simulation_params.smoothing_radius;
    let jitter_strength: f32 = JITTER_STRENGTH; // z. B. 0.05

    let particles_per_axis = (particle_count as f32).cbrt().ceil() as usize;
    let particles_per_axis = particles_per_axis.max(1);

    let spacing = smoothing_radius * 0.9;

    let size_x = (particles_per_axis - 1) as f32 * spacing;
    let size_y = (particles_per_axis - 1) as f32 * spacing;
    let size_z = (particles_per_axis - 1) as f32 * spacing;

    // set bounds based on particle grid size
    let bounds = Vec3::new(size_x, size_y, size_z);
    simulation_params.bounds_size = bounds;


    let mut rng = rand::rng();
    let mut spawned = 0;

    for x in 0..particles_per_axis {
        for y in 0..particles_per_axis {
            for z in 0..particles_per_axis {
                if spawned >= particle_count {
                    break;
                }

                let tx = x as f32 / (particles_per_axis - 1).max(1) as f32;
                let ty = y as f32 / (particles_per_axis - 1).max(1) as f32;
                let tz = z as f32 / (particles_per_axis - 1).max(1) as f32;

                let px = (tx - 0.5) * bounds.x;
                let py = ty * bounds.y;
                let pz = (tz - 0.5) * bounds.z;

                let jitter = random_in_unit_sphere(&mut rng) * jitter_strength * spacing;

                pos_data.push([px + jitter.x, py + jitter.y, pz + jitter.z, 0.0]);
                vel_data.push([0.0, 0.0, 0.0, 0.0]);

                spawned += 1;
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