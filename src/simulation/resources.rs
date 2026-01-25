use bevy::{prelude::*, render::render_resource::{BindGroupLayoutDescriptor, CachedComputePipelineId}};

#[derive(Resource)]
pub struct SimulationComputePipeline {
    pub layout: BindGroupLayoutDescriptor,
    pub write_back_bind_group: BindGroupLayoutDescriptor,
    pub external_forces: CachedComputePipelineId,
    pub spatial_hash: CachedComputePipelineId,
    pub update_positions: CachedComputePipelineId,
    pub reorder: CachedComputePipelineId,
    pub reorder_copy_back: CachedComputePipelineId,
    pub calculate_densities: CachedComputePipelineId,
    pub update_density: CachedComputePipelineId,
    pub calculate_pressure_force: CachedComputePipelineId,
    pub calculate_viscosity: CachedComputePipelineId,
}


#[derive(Resource, Clone)]
pub struct SimStepper {
    pub fixed_dt: f32,        // z.B. 1.0/120.0
    pub max_dt: f32,          // z.B. 1.0/60.0
    pub max_substeps: u32,    // z.B. 8
    pub accumulator: f32,     // bleibt zwischen Frames
    pub steps_this_frame: u32,
}

impl Default for SimStepper {
    fn default() -> Self {
        Self {
            fixed_dt: 1.0 / 240.0,
            max_dt: 1.0 / 60.0,
            max_substeps: 8,
            accumulator: 0.0,
            steps_this_frame: 0,
        }
    }
}

impl SimStepper {
    pub fn update(&mut self, delta_time: f32) {
         let real_dt = delta_time;

        // 1) Mindest-FPS: dt niemals größer als 1/60
        let frame_dt = real_dt.min(self.max_dt);

        // optional: harte Obergrenze gegen Debugger/Alt-Tab
        let frame_dt = frame_dt.min(0.1);

        self.accumulator += frame_dt;

        let mut steps = (self.accumulator / self.fixed_dt).floor() as u32;

        // 2) nicht zu viele Schritte pro Frame (sonst Performance-Tod)
        if steps > self.max_substeps {
            steps = self.max_substeps;

            // 3) Restzeit droppen: Simulation läuft in Low-FPS langsamer, aber stabil
            self.accumulator = self.fixed_dt * self.max_substeps as f32;
        }

        self.accumulator -= steps as f32 * self.fixed_dt;
        self.steps_this_frame = steps;
        
    }
}