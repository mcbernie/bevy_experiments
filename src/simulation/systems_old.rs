/// Hier kommen alle wichtigen Systeme rein die notwendig für die Simulation sind.
use bevy::prelude::*;

use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{storage_buffer, uniform_buffer};
use bevy::render::render_resource::{
    BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferDescriptor, BufferUsages, CachedComputePipelineId, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderStages, UniformBuffer
};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::storage::GpuShaderStorageBuffer;

use crate::{FIXED_DT, PARTICLE_COUNT};
use crate::simulation::assets::SimulationParams;
use crate::simulation::components::{AdvancedSimulationBuffers, PreparedSimulationBindGroup, SimulationBuffers};


#[derive(Resource, Default)]
pub struct SimulationSwapState {
    pub active: u32, // 0 oder 1
}

#[derive(Resource)]
pub struct SimulationTime {
    pub accumulator: f32,
}

#[derive(Resource)]
pub struct SimulationComputePipeline {
    pub compute_pipeline: CachedComputePipelineId,
    pub spatial_hash_pipeline: CachedComputePipelineId,
    pub spatial_count_pipeline: CachedComputePipelineId,
    pub spatial_clear_pipeline: CachedComputePipelineId,
    pub spatial_prefix_scan_pipeline: CachedComputePipelineId,
    pub spatial_copy_offsets_pipeline: CachedComputePipelineId,
    pub spatial_scatter_pipeline: CachedComputePipelineId,
    pub spatial_reorder_pipeline: CachedComputePipelineId,
    pub collision_pipeline: CachedComputePipelineId,
    pub layout: BindGroupLayoutDescriptor,
}


/// Update the simulation uniform buffer if parameters have changed
/// which represents the simulation parameters like gravity, box size, etc.
pub fn update_simulation_uniform(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut query: Query<(Entity, &SimulationParams, &mut SimulationUniform), Changed<SimulationParams>>,
) {

    for (_entity, params, mut uniform) in &mut query {
        if let Some(buffer) = uniform.buffer.as_mut() {
            buffer.set(params.clone());
            buffer.write_buffer(&render_device, &render_queue);
        } else {
            let mut buffer = UniformBuffer::from(params.clone());
            buffer.write_buffer(&render_device, &render_queue);
            uniform.buffer = Some(buffer);
        }
    }
}

/// init the compute pipeline for the simulation
pub fn init_compute_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {


    // Layout
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "compute_bind_group_layout",
        &[
            // will replaced by i think `storage_buffer` method. but works for now
            // positions_in
            storage_buffer::<Vec4>(false).build(0, ShaderStages::COMPUTE),
            // velocities_in
            storage_buffer::<Vec4>(false).build(1, ShaderStages::COMPUTE),
            // positions_out
            storage_buffer::<Vec4>(false).build(2, ShaderStages::COMPUTE),
            // velocities_out
            storage_buffer::<Vec4>(false).build(3, ShaderStages::COMPUTE),
            // positions_sorted
            storage_buffer::<Vec4>(false).build(4, ShaderStages::COMPUTE),
            // velocities_sorted
            storage_buffer::<Vec4>(false).build(5, ShaderStages::COMPUTE),

            // spatial keys
            storage_buffer::<u32>(false).build(6, ShaderStages::COMPUTE),
            // spatial counts
            storage_buffer::<u32>(false).build(7, ShaderStages::COMPUTE),
            // spatial offsets
            storage_buffer::<u32>(false).build(8, ShaderStages::COMPUTE),
            // spacial sorted indices
            storage_buffer::<u32>(false).build(9, ShaderStages::COMPUTE),
            // write offsets
            storage_buffer::<u32>(false).build(10, ShaderStages::COMPUTE),
            uniform_buffer::<SimulationParams>(false).build(11, ShaderStages::COMPUTE),
        ],
    );
    
    

    let shader: Handle<Shader> = asset_server.load("shaders/particles.wgsl");
    // my first compute pipeline, where all the strange stuff happens
    let main_compute_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("simulation_compute_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: vec![
                PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..4, // für f32 (delta_time)
                }
            ],
            shader: shader.clone(),
            entry_point: Some("external_forces".into()),
            ..Default::default()
        }
    );

    let spatial_hash_shader: Handle<Shader> = asset_server.load("shaders/sim_spatial.wgsl");
    let spatial_hash_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_hash_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_hash_shader,
            entry_point: Some("update_spatial_hash".into()),
            ..Default::default()
        }
    );

    let spatial_sort_shader: Handle<Shader> = asset_server.load("shaders/sim_sort.wgsl");
    let spatial_clear_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_clear_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_sort_shader.clone(),
            entry_point: Some("clear_counts".into()),
            ..Default::default()
        }
    );
    
    let spatial_count_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_count_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_sort_shader.clone(),
            entry_point: Some("calculate_counts".into()),
            ..Default::default()
        }
    );

    let spatial_prefix_scan_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_prefix_scan_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_sort_shader.clone(),
            entry_point: Some("prefix_scan".into()),
            ..Default::default()
        }
    );

    let spatial_copy_offsets_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_copy_offsets_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_sort_shader.clone(),
            entry_point: Some("copy_offsets".into()),
            ..Default::default()
        }
    );

    let spatial_scatter_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_scatter_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_sort_shader.clone(),
            entry_point: Some("scatter".into()),
            ..Default::default()
        }
    );

    let spatial_reorder_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("spatial_reorder_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: spatial_sort_shader.clone(),
            entry_point: Some("reorder".into()),
            ..Default::default()
        }
    );

    let collision_pipeline = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("collision_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            shader: shader.clone(),
            entry_point: Some("collision".into()),
            ..Default::default()
        }
    );

    // jetzt wollen wir natürlich den pipeline handle / id speichern und
    // auch das bind_group_layout
    commands.insert_resource(SimulationComputePipeline {
        compute_pipeline: main_compute_pipeline,
        spatial_hash_pipeline: spatial_hash_pipeline,
        spatial_count_pipeline,
        spatial_clear_pipeline,
        spatial_prefix_scan_pipeline,
        spatial_copy_offsets_pipeline,
        spatial_scatter_pipeline,
        spatial_reorder_pipeline,
        layout: bind_group_layout,
        collision_pipeline,
    });

}

pub fn init_per_component(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<(Entity, &SimulationParams), Without<AdvancedSimulationBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, params) in &query {
        let mut uniform_buffer = UniformBuffer::from(params.clone());
        uniform_buffer.write_buffer(&render_device, &render_queue);

        //let n = params.num_particles as usize;
        let n = PARTICLE_COUNT as usize;

        let position_sorted = render_device.create_buffer(&BufferDescriptor {
            label: Some("positions_sorted"),
            size: (n * std::mem::size_of::<[f32; 4]>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let velocities_sorted = render_device.create_buffer(&BufferDescriptor {
            label: Some("velocities_sorted"),
            size: (n * std::mem::size_of::<[f32; 4]>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        let spatial_counts = render_device.create_buffer(&BufferDescriptor {
            label: Some("spatial_counts"),
            size: (n * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spatial_offsets = render_device.create_buffer(&BufferDescriptor {
            label: Some("spatial_offsets"),
            size: (n * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spatial_sorted_indices = render_device.create_buffer(&BufferDescriptor {
            label: Some("spatial_sorted_indices"),
            size: (n * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let write_offsets = render_device.create_buffer(&BufferDescriptor {
            label: Some("write_offsets"),
            size: (n * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });


        commands.entity(entity).insert(
            (
                SimulationUniform { buffer: Some(uniform_buffer) },
                AdvancedSimulationBuffers {
                    velocities_sorted,
                    position_sorted,
                    spatial_sort_counts: spatial_counts,
                    spatial_sort_offsets: spatial_offsets,
                    spatial_sorted_indices,
                    write_offsets
                },
            )
        );
    }

}


/// RenderApp - Erstelle eine "echte" BindGroup für die Simulation
pub fn prepare_simulation_bind_groups(
    mut commands: Commands,
    pipeline: Res<SimulationComputePipeline>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    storage_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>, // <- Da legen wir den StorageBuffer ab
    swap: Res<SimulationSwapState>,
    query: Query<(Entity, &SimulationBuffers, &AdvancedSimulationBuffers, &SimulationUniform), With<AdvancedSimulationBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, buffers, advanced_buffers, simulation_uniform) in &query {
        let read = swap.active as usize;
        let write = 1 - read;

        let positions_in  = storage_buffers.get(&buffers.positions[read]).unwrap();
        let positions_out = storage_buffers.get(&buffers.positions[write]).unwrap();
        let velocities_in  = storage_buffers.get(&buffers.velocities[read]).unwrap();
        let velocities_out = storage_buffers.get(&buffers.velocities[write]).unwrap();

        let positions_sorted = &advanced_buffers.position_sorted;
        let velocities_sorted = &advanced_buffers.velocities_sorted;

        let spatial_counts = &advanced_buffers.spatial_sort_counts;
        let spatial_offsets = &advanced_buffers.spatial_sort_offsets;
        let spatial_sorted_indices = &advanced_buffers.spatial_sorted_indices;
        let write_offsets = &advanced_buffers.write_offsets;

        let uniform_buffer = simulation_uniform.buffer.as_ref().unwrap();


        let spatial_keys = storage_buffers.get(&buffers.spatial_keys).unwrap();

        // we need to create this for each pipeline...?
        let bind_group = render_device.create_bind_group(
            "particle_compute_pipeline_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.layout),
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: positions_in.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: velocities_in.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: positions_out.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: velocities_out.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: positions_sorted.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: velocities_sorted.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: spatial_keys.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: spatial_counts.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: spatial_offsets.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: spatial_sorted_indices.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: write_offsets.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 11,
                    resource: uniform_buffer.buffer().unwrap().as_entire_binding(),
                },
            ],
        );

        commands.entity(entity).insert(
            (
                PreparedSimulationBindGroup { bind_group },
            )
        );
    }
}

pub fn swap_simulation_buffers(mut swap: ResMut<SimulationSwapState>) {
    swap.active ^= 1;
}