
use bevy::{prelude::*, render::{Render, RenderApp, RenderStartup, RenderSystems, extract_component::ExtractComponentPlugin, gpu_readback::{Readback, ReadbackComplete}}};
use crate::simulation::{components::SimulationBuffers, renderer::spawn_simulation_once};

use super::systems::{init_compute_pipeline, prepare_simulation_bind_groups, run_compute};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            Update,
            spawn_simulation_once.run_if(run_once),
        )
        .add_plugins(ExtractComponentPlugin::<SimulationBuffers>::default());
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            // Extraction synchronisiere Daten von der GameApp zur RenderApp
            .add_systems(RenderStartup,
                init_compute_pipeline,
            )
            .add_systems(Render, 
                prepare_simulation_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups)
            )
            .add_systems(Render,
                run_compute
            );

    }
}
