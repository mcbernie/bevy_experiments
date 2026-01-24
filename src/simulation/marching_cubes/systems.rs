use std::num::NonZeroU64;

use bevy::prelude::*;
use bevy::render::render_resource::binding_types::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};

use crate::DENSITY_TEXTURE_RES;
use crate::simulation::assets::SimulationParams;
use crate::simulation::components::{DensityMap, InternalSimulationBuffers, SimulationUniform};
use crate::simulation::gpu_sort::InternalCountSortBuffers;
use super::components::{MarchingCubesBindGroup, MarchingCubesBuffers, Triangle};
use super::resources::{MarchingCubesLut, MarchingCubesPipeline};

pub fn init_marching_cubes_simulation_system(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    query: Query<(Entity, &InternalCountSortBuffers,), Without<MarchingCubesBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, _internal_buffers) in &query {
        // ----------------------------------------------------
        // 1. Resolution bestimmen (Beispiel)
        // ----------------------------------------------------
        let resolution = DENSITY_TEXTURE_RES;
        // MUSS >= 2 sein
        if resolution < 2 {
            continue;
        }

        let voxels_per_axis = resolution - 1;
        let num_voxels = voxels_per_axis * voxels_per_axis * voxels_per_axis;

        // max 5 triangles per voxel
        let max_triangles = num_voxels * 5;

        // ----------------------------------------------------
        // 2. Triangle-Buffer
        // ----------------------------------------------------
        warn!("create triangle buffer with size for {} triangles", max_triangles);
        let triangle_buffer_size =
            max_triangles as u64 * std::mem::size_of::<Triangle>() as u64;

        let triangle_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("marching_cubes_triangle_buffer"),
            size: triangle_buffer_size,
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // ----------------------------------------------------
        // 3. Atomic Counter Buffer
        // ----------------------------------------------------
        let counter_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("marching_cubes_counter_buffer"),
            contents: bytemuck::cast_slice(&[0u32]),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        });

        // ----------------------------------------------------
        // 4. In Component packen
        // ----------------------------------------------------
        let buffers = MarchingCubesBuffers {
            triangle_buffer,
            counter_buffer,
            max_triangles,
        };

        commands.entity(entity).insert(buffers);
    }
}

pub fn init_marching_cubes_pipeline(
    mut commands: Commands,
    pipeline_cache: ResMut<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    let shader = asset_server.load("shaders/marching_cubes.wgsl");

    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "marching_cubes_layout",
        &[
            // triangles
            storage_buffer::<Triangle>(false).build(0, ShaderStages::COMPUTE),
            // atomic counter
            storage_buffer::<u32>(false).build(1, ShaderStages::COMPUTE),
            // lut
            storage_buffer_read_only::<u32>(false).build(2, ShaderStages::COMPUTE),
            // density texture
            texture_3d(TextureSampleType::Float { filterable: true }).build(3, ShaderStages::COMPUTE),
            sampler(SamplerBindingType::Filtering).build(4, ShaderStages::COMPUTE),
            // params
            uniform_buffer_sized(false, NonZeroU64::new(std::mem::size_of::<[u32;4]>() as u64)).build(5, ShaderStages::COMPUTE),
        ],
    );

    let simulation_settings_layout_desc = BindGroupLayoutDescriptor::new(
        "simulation_settings_layout_desc",
        &[
            uniform_buffer::<SimulationParams>(false).build(0, ShaderStages::VERTEX | ShaderStages::COMPUTE),
        ],
    );

    let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("marching_cubes_pipeline".into()),
        layout: vec![bind_group_layout.clone(), simulation_settings_layout_desc],
        shader,
        entry_point: Some("process_cube".into()),
        shader_defs: vec![],
        ..Default::default()
    });

    commands.insert_resource(MarchingCubesPipeline {
        pipeline_id,
        bind_group_layout,
    });
}


pub fn init_marching_cubes_lut(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
) {
    let lut: Vec<i32> = include_str!("marching_cubes_lut.txt")
        .trim()
        .split(',')
        .map(|v| v.parse().unwrap())
        .collect();

    let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("marching_cubes_lut"),
        contents: bytemuck::cast_slice(&lut),
        usage: BufferUsages::STORAGE,
    });

    commands.insert_resource(MarchingCubesLut {
        buffer,
        len: lut.len() as u32,
    });
}

pub fn prepare_marching_cubes_bind_group(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<MarchingCubesPipeline>,
    pipeline_cache: Res<PipelineCache>,
    lut: Res<MarchingCubesLut>,
    query: Query<(Entity, &MarchingCubesBuffers, &DensityMap, &InternalSimulationBuffers)>,
) {

    let triangle_layout_desc = BindGroupLayoutDescriptor::new(
        "triangle_layout_desc",
        &[
            storage_buffer_read_only::<Triangle>(false).build(0, ShaderStages::VERTEX),
        ],
    );

    for (entity, buffers, density, internal_buffers) in &query {
        let bind_group = render_device.create_bind_group(
            "marching_cubes_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.bind_group_layout),
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffers.triangle_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffers.counter_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: lut.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: density.view.into_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: density.sampler.into_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: internal_buffers.density_map_size.as_entire_binding(),
                }
            ],
        );

        let triangle_bind_group = render_device.create_bind_group(
            "marching_cubes_triangle_bind_group",
            &pipeline_cache.get_bind_group_layout(&triangle_layout_desc),
            &[BindGroupEntry {
                binding: 0,
                resource: buffers.triangle_buffer.as_entire_binding(),
            }],
        );

        commands.entity(entity).insert(MarchingCubesBindGroup { 
            bind_group,
            single_triangle_bind_group: triangle_bind_group,
        });
    }
}
