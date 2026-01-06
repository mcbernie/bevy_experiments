// signed distance field generation and manipulation
use super::chunk::VoxelChunk;



fn smooth_sdf(chunk: &mut VoxelChunk) {
    let mut copy = chunk.density.clone();

    let sx = chunk.size.x as i32;
    let sy = chunk.size.y as i32;
    let sz = chunk.size.z as i32;

    for z in 1..sz - 1 {
        for y in 1..sy - 1 {
            for x in 1..sx - 1 {
                let mut sum = 0.0;
                let mut count = 0.0;

                for dz in -1..=1 {
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let ix = (x + dx) as u32;
                            let iy = (y + dy) as u32;
                            let iz = (z + dz) as u32;
                            sum += chunk.get(ix, iy, iz);
                            count += 1.0;
                        }
                    }
                }

                let idx = chunk.idx(x as u32, y as u32, z as u32);
                copy[idx] = sum / count;
            }
        }
    }

    chunk.density = copy;
}
