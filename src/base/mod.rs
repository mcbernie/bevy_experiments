use bevy::prelude::*;

pub fn create_plane_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(16))),
        MeshMaterial3d(materials.add(Color::srgb(0.4, 0.7, 0.4))),
        Transform::from_translation(Vec3::ZERO),
    ));

}