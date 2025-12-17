use bevy::prelude::*;

#[derive(Component)]
pub struct ChunkMeshChild; // sitzt auf dem Kind-Entity


#[derive(Component)]
pub struct ChunkMeshTag;

#[derive(Bundle)]
pub struct ChunkBundle {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl ChunkBundle {
    pub fn new(mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> Self {
        Self {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
            //transform: Transform::from_xyz(-8.0, 0.0, -8.0),
            transform: Transform::IDENTITY,
            global_transform: default(),
            visibility: default(),
            inherited_visibility: default(),
            view_visibility: default(),
        }
    }
}
