
use bevy::{prelude::*, render::{Render, RenderApp, RenderStartup, RenderSystems, extract_component::ExtractComponentPlugin, extract_resource::ExtractResourcePlugin, render_graph::RenderGraph}};
//use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};

use super::assets::SimulationParams;
use super::components::SimulationBuffers; 
use super::renderer::spawn_simulation_once;
use super::systems::{
    SimulationTime, 
    SimulationUniform, 
    update_simulation_uniform,
    init_compute_pipeline, 
    prepare_simulation_bind_groups, 
    run_compute
};

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
        .add_plugins(ExtractComponentPlugin::<SimulationBuffers>::default());
        //.add_plugins(EguiPlugin::default())
        //.add_plugins(ResourceInspectorPlugin::<SimulationParams>::default());

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
                (
                    update_simulation_uniform,
                    run_compute,
                )
            )
            .add_systems(Render, 
                prepare_simulation_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups)
            );
        
        // instead of using a compute system, i try to create a graph node
        // can a node works if we need to query components short before dispatch_workgroups?
        //let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        //render_graph.add_node(SimulationSystemLabel, SimulationNode::default());
        //render_graph.add_node_edge(SimulationSystemLabel, bevy::render::graph::CameraDriverLabel).unwrap();

    }
}
