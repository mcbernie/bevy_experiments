mod custom_material;
use bevy::{pbr::ExtendedMaterial, prelude::*};

pub use custom_material::CheckerMaterial;

pub fn create_plane_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, CheckerMaterial>>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(16))),
        MeshMaterial3d(materials.add(
            ExtendedMaterial {
                base: StandardMaterial::default(),
                extension: CheckerMaterial {
                    scale: 17.0,
                    color_a: LinearRgba { red: 0.1, green: 0.1, blue: 0.1, alpha: 1.0 },
                    color_b: LinearRgba { red: 0.9, green: 0.9, blue: 0.9, alpha: 1.0 },
                }
            }
        )),
        Transform::from_translation(Vec3::ZERO),
    ));

}