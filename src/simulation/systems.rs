use bevy::{prelude::*, render::{renderer::{RenderDevice, RenderQueue}}};

use crate::simulation::{assets::SimulationParams, components::SimulationUniform};

/// Update the simulation uniform buffer if parameters have changed
/// which represents the simulation parameters like gravity, box size, etc.
pub fn update_simulation_uniform(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut query: Query<(Entity, &SimulationParams, &mut SimulationUniform), Changed<SimulationParams>>,
) {

    for (_entity, params, mut uniform) in &mut query {
        uniform.0.set(params.clone());
        uniform.0.write_buffer(&render_device, &render_queue);
    }
}