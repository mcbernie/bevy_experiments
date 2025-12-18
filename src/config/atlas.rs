use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct BlocksConfig {
    pub atlas: AtlasInfo,
    pub skybox: SkyboxInfo,
    pub blocks: HashMap<String, BlockDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkyboxInfo {
    pub texture: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtlasInfo {
    pub size: (u32, u32),
    pub tile_size: (u32, u32),
    pub texture: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockDef {
    pub all: Option<(u32, u32)>,
    pub top: Option<(u32, u32)>,
    pub bottom: Option<(u32, u32)>,
    pub side: Option<(u32, u32)>,
}
