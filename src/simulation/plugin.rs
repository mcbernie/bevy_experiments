
use bevy::{
    asset::{load_internal_asset, uuid_handle}, prelude::*, render::{
        Render, RenderApp, RenderStartup, RenderSystems, extract_component::ExtractComponentPlugin, graph::CameraDriverLabel, render_graph::RenderGraph
    }
};

use crate::{rendering::{GenerateRenderArgsLabel, GenerateRenderArgsNode, init_drawing_args_buffer, init_drawing_args_pipeline, prepare_drawing_args}, simulation::{marching_cubes::{MarchingCubesLabel, MarchingCubesNode, init_marching_cubes_lut, init_marching_cubes_pipeline, init_marching_cubes_simulation_system, prepare_marching_cubes_bind_group}, node::{SimulationGraphLabel, SimulationNode}, systems::prepare_density_map}};

use super::{
    assets::SimulationParams,
    components::SimulationBuffers,
    gpu_sort::{
        init_count_sort_compute_pipeline,
        init_count_sort_system,
        prepare_count_sort_bind_groups,
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
            .add_systems(RenderStartup,
                (
                    init_compute_pipeline, 
                    init_count_sort_compute_pipeline, 
                    init_spatial_hash_compute_pipeline,
                    init_marching_cubes_lut,
                    init_marching_cubes_pipeline,
                    init_drawing_args_pipeline,
                ),
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
                init_marching_cubes_simulation_system
                    .in_set(RenderSystems::PrepareBindGroups)
                    .before(prepare_marching_cubes_bind_group).after(init_count_sort_system),
                init_drawing_args_buffer
                    .in_set(RenderSystems::PrepareBindGroups)
                    .before(prepare_drawing_args).after(init_marching_cubes_simulation_system),

                // update or create density map texture
                prepare_density_map.in_set(RenderSystems::PrepareBindGroups)
                    .before(prepare_simulation_bind_groups),

                prepare_simulation_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups),
                prepare_count_sort_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups),
                prepare_spatial_hash_bind_groups
                    .in_set(RenderSystems::PrepareBindGroups),
                prepare_marching_cubes_bind_group
                    .in_set(RenderSystems::PrepareBindGroups),
                prepare_drawing_args
                    .in_set(RenderSystems::PrepareBindGroups),

            ));
        
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();

        render_graph.add_node(SimulationGraphLabel, SimulationNode::default());
        render_graph.add_node(MarchingCubesLabel, MarchingCubesNode::default());
        render_graph.add_node(GenerateRenderArgsLabel, GenerateRenderArgsNode::default());
        render_graph.add_node_edge(SimulationGraphLabel, MarchingCubesLabel);
        render_graph.add_node_edge(MarchingCubesLabel, GenerateRenderArgsLabel);
        render_graph.add_node_edge(GenerateRenderArgsLabel, CameraDriverLabel);
        

    }
}
