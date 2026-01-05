use bevy::prelude::*;

use crate::app_state::{AppState, LoadingProgress};

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(AppState::Loading), load_meshing_data)
        .add_systems(Update, 
            do_meshing.run_if(in_state(AppState::Loading))
        );
    }
}

fn load_meshing_data(
    mut commands: Commands,
    mut progress: ResMut<LoadingProgress>,
) {
    // Placeholder for loading meshing data
    progress.meshing_data_loaded = true;
}

fn do_meshing(
    mut commands: Commands,
) {
    return;
}