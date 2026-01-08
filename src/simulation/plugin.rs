use std::time::Duration;

use bevy::{prelude::*, render::{Render, RenderApp, RenderStartup}, time::common_conditions::on_timer};
use crate::simulation::pipelines::ReadbackState;

use super::pipelines;
use super::renderer;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum SimulationStartupSet {
    InitBuffers,
    SpawnEntities,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {

        app
            .configure_sets(
                Startup,
                (
                    SimulationStartupSet::InitBuffers,
                    SimulationStartupSet::SpawnEntities,
                )
                .chain(), // ← wichtig!
            )
            .add_systems(Startup, renderer::init_compute_buffers.in_set(SimulationStartupSet::InitBuffers))
            .add_systems(Startup, renderer::spawn_particles.in_set(SimulationStartupSet::SpawnEntities));
            ;

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(ReadbackState::default())
            .add_systems(RenderStartup, (
                pipelines::init_compute,
            ))
            .add_systems(Render,
                pipelines::update_params.before(pipelines::run_compute),
            )
            .add_systems(Render,
                pipelines::run_compute,
            )
            .add_systems(
                Render,
                pipelines::read_positions.run_if(on_timer(Duration::from_secs(1))),
            )
            ;

    }
}