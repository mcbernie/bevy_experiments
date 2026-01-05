use bevy::{asset::RenderAssetUsages, mesh::{Indices, PrimitiveTopology}, prelude::*};
use super::chunk::VoxelChunk;
struct MeshBuilder {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl MeshBuilder {
    fn add_quad(
        &mut self,
        a: Vec3,
        b: Vec3,
        c: Vec3,
        d: Vec3,
        normal: Vec3,
    ) {
        let base = self.positions.len() as u32;

        self.positions.extend([a, b, c, d]);
        self.normals.extend([normal; 4]);

        self.indices.extend([
            base, base + 2, base + 1,
            base, base + 3, base + 2,
        ]);
    }

    fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.positions.into_iter().map(|v| [v.x, v.y, v.z]).collect::<Vec<_>>(),
        );

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            self.normals.into_iter().map(|n| [n.x, n.y, n.z]).collect::<Vec<_>>(),
        );

        mesh.insert_indices(
            Indices::U32(self.indices),
        );

        mesh
    }
}


pub fn build_voxel_surface_mesh(chunk: &VoxelChunk) -> Mesh {
    let mut mb = MeshBuilder::new();
    let size = chunk.size;

    for z in 0..size.z as i32 {
        for y in 0..size.y as i32 {
            for x in 0..size.x as i32 {
                if !is_solid(chunk, x, y, z) {
                    continue;
                }

                let base = Vec3::new(x as f32, y as f32, z as f32);

                // +Y (top)
                if !is_solid(chunk, x, y + 1, z) {
                    mb.add_quad(
                        base + Vec3::new(0.0, 1.0, 0.0),
                        base + Vec3::new(1.0, 1.0, 0.0),
                        base + Vec3::new(1.0, 1.0, 1.0),
                        base + Vec3::new(0.0, 1.0, 1.0),
                        Vec3::Y,
                    );
                }

                // -Y (bottom)
                if !is_solid(chunk, x, y - 1, z) {
                    mb.add_quad(
                        base + Vec3::new(0.0, 0.0, 1.0),
                        base + Vec3::new(1.0, 0.0, 1.0),
                        base + Vec3::new(1.0, 0.0, 0.0),
                        base + Vec3::new(0.0, 0.0, 0.0),
                        -Vec3::Y,
                    );
                }

                // +X
                if !is_solid(chunk, x + 1, y, z) {
                    mb.add_quad(
                        base + Vec3::new(1.0, 0.0, 0.0),
                        base + Vec3::new(1.0, 0.0, 1.0),
                        base + Vec3::new(1.0, 1.0, 1.0),
                        base + Vec3::new(1.0, 1.0, 0.0),
                        Vec3::X,
                    );
                }

                // -X
                if !is_solid(chunk, x - 1, y, z) {
                    mb.add_quad(
                        base + Vec3::new(0.0, 0.0, 1.0),
                        base + Vec3::new(0.0, 0.0, 0.0),
                        base + Vec3::new(0.0, 1.0, 0.0),
                        base + Vec3::new(0.0, 1.0, 1.0),
                        -Vec3::X,
                    );
                }

                // +Z
                if !is_solid(chunk, x, y, z + 1) {
                    mb.add_quad(
                        base + Vec3::new(1.0, 0.0, 1.0),
                        base + Vec3::new(0.0, 0.0, 1.0),
                        base + Vec3::new(0.0, 1.0, 1.0),
                        base + Vec3::new(1.0, 1.0, 1.0),
                        Vec3::Z,
                    );
                }

                // -Z
                if !is_solid(chunk, x, y, z - 1) {
                    mb.add_quad(
                        base + Vec3::new(0.0, 0.0, 0.0),
                        base + Vec3::new(1.0, 0.0, 0.0),
                        base + Vec3::new(1.0, 1.0, 0.0),
                        base + Vec3::new(0.0, 1.0, 0.0),
                        -Vec3::Z,
                    );
                }
            }
        }
    }

    mb.into_mesh()
}



fn is_solid(chunk: &VoxelChunk, x: i32, y: i32, z: i32) -> bool {
    let size = chunk.size;

    if x < 0 || y < 0 || z < 0 {
        return false;
    }
    if x >= size.x as i32 || y >= size.y as i32 || z >= size.z as i32 {
        return false;
    }

    let idx =
        x as usize +
        y as usize * size.x as usize +
        z as usize * size.x as usize * size.y as usize;

    chunk.density[idx] > 0.0
}
