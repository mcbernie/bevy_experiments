
use bevy::{
    asset::{load_internal_asset, uuid_handle}, prelude::*, render::{
        Render, RenderApp, RenderStartup, RenderSystems, extract_component::ExtractComponentPlugin, graph::CameraDriverLabel, render_graph::RenderGraph
    }
};

use crate::simulation::{node::{SimulationGraphLabel, SimulationNode}, systems::{SimulationRunGuard, run_simulation}};

use super::{
    assets::SimulationParams,
    components::SimulationBuffers,
    gpu_sort::{
        CountSortLabel,
        CountSortNode,
        init_count_sort_compute_pipeline,
        init_count_sort_system,
        prepare_count_sort_bind_groups,
    },
    node::{
        FinalSimulationNode,
        FinalSimulationSystemLabel,
        StartSimulationNode,
        StartSimulationSystemLabel,
    },
    renderer::{
        spawn_simulation_once,
        update_simulation_gizmo,
    },
    systems::{
        init_compute_pipeline,
        init_simulation_system,
        prepare_simulation_bind_groups,
        update_simulation_uniform,
    },
    spatial_hash::{
        SpatialHashNode,
        SpatialHashSystemLabel,
        init_spatial_hash_compute_pipeline,
        prepare_spatial_hash_bind_groups,
    },
};

const MATH_SHADER_HELPER: Handle<Shader> =
    uuid_handle!("990ee5ac-3d4a-4593-841f-6b46f02abcb4");

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {

        app.add_plugins(ExtractComponentPlugin::<SimulationParams>::default())
        .add_systems(
            Update,
            (
                spawn_simulation_once.run_if(run_once),
                update_simulation_gizmo,
            ),
        )
        .register_type::<SimulationParams>()
        .add_plugins(ExtractComponentPlugin::<SimulationBuffers>::default());

        // make math.wgsl shader available for other shaders (#import)
        load_internal_asset!(
            app,
            MATH_SHADER_HELPER,
            "shaders/math.wgsl",
            Shader::from_wgsl
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(SimulationRunGuard::default())
            .add_systems(RenderStartup,
                (init_compute_pipeline, init_count_sort_compute_pipeline, init_spatial_hash_compute_pipeline),
            )
            .add_systems(
                Render,
                (
                    update_simulation_uniform,
                )
            )
            .add_systems(Render, (
                //swap_simulation_buffers.before(RenderSystems::PrepareBindGroups),
                init_simulation_system
                    .in_set(RenderSystems::PrepareBindGroups)
                    .before(prepare_simulation_bind_groups),
                init_count_sort_system
                    .in_set(RenderSystems::PrepareBindGroups)
                    .before(prepare_count_sort_bind_groups).after(init_simulation_system),

                prepare_simulation_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups),
                prepare_count_sort_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups),
                prepare_spatial_hash_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups),

            ));
            // System based rendering of the simulation
            //.add_systems(Render, 
            //    run_simulation.in_set(RenderSystems::Queue),
            //);
        
        // render graph based simulation
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        // next try, put all different nodes to one node
        render_graph.add_node(SimulationGraphLabel, SimulationNode::default());
        render_graph.add_node_edge(SimulationGraphLabel, CameraDriverLabel);
        
        //render_graph.add_node(StartSimulationSystemLabel, StartSimulationNode::default());
        //render_graph.add_node(CountSortLabel, CountSortNode::default());
        //render_graph.add_node(SpatialHashSystemLabel, SpatialHashNode::default());
        //render_graph.add_node(FinalSimulationSystemLabel, FinalSimulationNode::default());

        //// Begin the simulation
        //render_graph.add_node_edge(StartSimulationSystemLabel, CountSortLabel);
        //// run the count-sort algorithm
        //render_graph.add_node_edge(CountSortLabel, SpatialHashSystemLabel);

        //// finish the simulation step by calculate the spatial_hash
        //render_graph.add_node_edge(SpatialHashSystemLabel, FinalSimulationSystemLabel);

        //// do all the simulation on sorted positions and update positions and velocities
        //render_graph.add_node_edge(FinalSimulationSystemLabel, CameraDriverLabel);

    }
}
