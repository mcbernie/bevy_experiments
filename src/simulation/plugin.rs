
use bevy::{prelude::*, render::{Render, RenderApp, RenderStartup, RenderSystems, extract_component::ExtractComponentPlugin, extract_resource::ExtractResourcePlugin}};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};
use crate::simulation::{assets::SimulationParams, components::SimulationBuffers, renderer::spawn_simulation_once, systems::{SimulationTime, SimulationUniform, update_simulation_uniform}};

use super::systems::{init_compute_pipeline, prepare_simulation_bind_groups, run_compute};

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {

        app.add_plugins(ExtractResourcePlugin::<SimulationParams>::default())
        .add_systems(
            Update,
            spawn_simulation_once.run_if(run_once),
        )
        .register_type::<SimulationParams>()
        .insert_resource(SimulationParams::default())
        .add_plugins(EguiPlugin::default())
        .add_plugins(ExtractComponentPlugin::<SimulationBuffers>::default())
        .add_plugins(ResourceInspectorPlugin::<SimulationParams>::default());

        let render_app = app.sub_app_mut(RenderApp);

        render_app.insert_resource(SimulationUniform { buffer: None });
        render_app.insert_resource(SimulationTime { accumulator: 0.0 });
        render_app
            // Extraction synchronisiere Daten von der GameApp zur RenderApp
            .add_systems(RenderStartup,
                init_compute_pipeline,
            )
            .add_systems(
                Render,
                update_simulation_uniform,
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
