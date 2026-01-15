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

    use rand::Rng;

    let mut rng = rand::rng();

    let mut pos_data = Vec::with_capacity(PARTICLE_COUNT as usize);
    let mut vel_data = Vec::with_capacity(PARTICLE_COUNT as usize);

    for _ in 0..PARTICLE_COUNT {
        let x = rng.random_range(-0.8..0.8);
        let y = rng.random_range(-0.8..0.8);
        let z = rng.random_range(-0.8..0.8);

        pos_data.push([x, y, z, 0.0]);
        vel_data.push([0.0, 0.0, 0.0, 0.0]);
    }

    let positions = [storage_buffers.add(
            ShaderStorageBuffer::from(pos_data.clone())
        ),
        storage_buffers.add(
            ShaderStorageBuffer::from(pos_data.clone())
        )
    ];

    let velocities = [storage_buffers.add(
            ShaderStorageBuffer::from(vel_data.clone())
        ),
        storage_buffers.add(
            ShaderStorageBuffer::from(vel_data.clone())
        )
    ];

    let spk_buffer = ShaderStorageBuffer::from(vec![0u32; PARTICLE_COUNT as usize]);

    let spatial_keys = storage_buffers.add(
        spk_buffer
    );

    let vertex_count = PARTICLE_COUNT as usize * 6;
    let v_positions = vec![[0.0, 0.0, 0.0]; vertex_count];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_positions);

    let mesh = meshes.add(mesh);

    let material = materials.add(
        ParticleMaterial {
            positions: positions[0].clone(), // currently using only one buffer for rendering
            velocities: velocities[0].clone(),
            spatial_keys: spatial_keys.clone(),
            color: Vec4::new(0.2, 0.5, 1.0, 1.0),
            radius: 0.05,
        }
    );

    // --- Entity ---
    commands.spawn((
        Name::new("Particle Simulation"),
        SimulationBuffers {
            positions,
            velocities,
            spatial_keys,
            active_index: 0,
        },
        SimulationParams::default(),
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
        let size = Vec3::splat(params.box_size);

        let center = transform.translation
            + Vec3::new(0.0, params.box_size * 0.5, 0.0);

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