use bevy::prelude::*;

#[derive(Resource)]
pub struct Cubemap {
    pub is_loaded: bool,
    pub index: usize,
    pub image_handle: Handle<Image>,
}
