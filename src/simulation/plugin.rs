use std::time::Duration;

use bevy::{prelude::*, render::{Render, RenderApp, RenderStartup}, time::common_conditions::on_timer};
use crate::simulation::pipelines::ReadbackState;

use super::pipelines;
use super::renderer;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(Startup, renderer::spawn_particles);

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
            );

    }
}