pub const ATLAS_W: f32 = 1024.0;
pub const ATLAS_H: f32 = 1024.0;

pub const TILE_CONTENT: f32 = 16.0;
pub const PADDING: f32 = 0.001;
pub const CELL: f32 = TILE_CONTENT + 2.0 * PADDING; // 32.0

pub const TILE_GRASS_TOP: (u32, u32) = (21, 5);
pub const TILE_GRASS_SIDE: (u32, u32) = (20, 6);
pub const TILE_DIRT: (u32, u32) = (17, 10);

pub enum UvRot {
    R0,
    R90,
    R180,
    R270,
}

#[derive(Clone, Copy)]
pub struct UvRect {
    pub u0: f32, pub v0: f32,
    pub u1: f32, pub v1: f32,
}

pub fn tile_uv((tx, ty): (u32, u32)) -> UvRect {
    // Start der Zelle
    let cell_x0 = tx as f32 * CELL;
    let cell_y0 = ty as f32 * CELL;

    // Innerer Content-Bereich (Padding abgeschnitten)
    let x0 = cell_x0 + PADDING;
    let y0 = cell_y0 + PADDING;
    let x1 = x0 + TILE_CONTENT;
    let y1 = y0 + TILE_CONTENT;

    // halbes Texel Inset (verhindert Kanten-Sampling)
    let inset = 0.5;

    let u0 = (x0 + inset) / ATLAS_W;
    let u1 = (x1 - inset) / ATLAS_W;

    let v0 = (y0 + inset) / ATLAS_H;
    let v1 = (y1 - inset) / ATLAS_H;

    UvRect { u0, v0, u1, v1 }
}

pub fn push_uvs(rect: UvRect, rot: UvRot, uvs: &mut Vec<[f32; 2]>) {
    let u0 = rect.u0;
    let u1 = rect.u1;
    let v0 = rect.v0;
    let v1 = rect.v1;

    let base = match rot {
        UvRot::R0 => [
            [u0,v0], [u1,v0], [u1,v1], [u0,v1],
        ],
        UvRot::R90 => [
            [u0,v1], [u0,v0], [u1,v0], [u1,v1],
        ],
        UvRot::R180 => [
            [u1,v1], [u0,v1], [u0,v0], [u1,v0],
        ],
        UvRot::R270 => [
            [u1,v0], [u1,v1], [u0,v1], [u0,v0],
        ],
    };

    uvs.extend_from_slice(&base);
}