use bevy::{
    platform::collections::HashMap,
    prelude::*,
    render::{
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BufferDescriptor, BufferUsages, ComputePass, ComputePipeline, ComputePipelineDescriptor, PipelineCache, PushConstantRange, ShaderStages, binding_types::{storage_buffer, uniform_buffer}
        },
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::{
    WORKGROUP_SIZE, simulation::{
        assets::SimulationParams,
        components::InternalSimulationBuffers,
        gpu_sort::{
            components::{InternalCountSortBuffers, PreparedCountSortComputeBindGroup},
            helper::calc_num_groups,
        },
        helper::{CreatePipelineArgs, create_pipeline, dispatch_compute},
    }
};

use super::resources::CountSortComputePipeline;

struct ScanLevel {
    item_count: u32,
    num_groups: u32,
    elements_bg: BindGroup,   // Buffer, der gerade gescannt wird
    group_sums_bg: BindGroup, // passender groupSums-Buffer
}

pub fn run_count_sort_compute(
    queue: &RenderQueue,
    pass: &mut ComputePass,
    clear_counts: &ComputePipeline,
    count: &ComputePipeline,
    scan: &ComputePipeline,
    combine: &ComputePipeline,
    scatter_output: &ComputePipeline,
    copy_back: &ComputePipeline,
    bg: &PreparedCountSortComputeBindGroup,
    buffers: &InternalCountSortBuffers,
    particle_count: u32,
) {
    let wg_size = WORKGROUP_SIZE;
    
    let calced_wg_size = (particle_count + (WORKGROUP_SIZE - 1)) / WORKGROUP_SIZE;//calc_num_groups(particle_count, wg_size);
    queue.write_buffer(&buffers.num_items, 0, bytemuck::bytes_of(&particle_count));

    dispatch_compute(pass, clear_counts, &[&bg.sort_bind_group], None, calced_wg_size);
    dispatch_compute(pass, count, &[&bg.sort_bind_group], None, calced_wg_size);
    let mut levels: Vec<ScanLevel> = Vec::new();

    let mut current_item_count = particle_count;
    let mut current_elements_bg = bg.scan_bind_group.clone(); // elements

    // ---- the scan pass ----
    loop {
        let num_groups = calc_num_groups(current_item_count, wg_size);

        let group_sums_bg = bg.per_group_count
            .get(&num_groups)
            .expect("missing group_sums bind group")
            .clone();

        dispatch_compute(pass, scan, &[
            &current_elements_bg,
            &group_sums_bg,
        ], Some(bytemuck::bytes_of(&current_item_count)), num_groups);

        levels.push(ScanLevel {
            item_count: current_item_count,
            num_groups,
            elements_bg: current_elements_bg.clone(),
            group_sums_bg: group_sums_bg.clone(),
        });

        if num_groups <= 1 {
            break;
        }

        current_elements_bg = group_sums_bg;
        current_item_count = num_groups;
    }

    for level in levels.iter().rev().skip(1) {
        dispatch_compute(pass, combine, &[
            &level.elements_bg,
            &level.group_sums_bg,
        ], Some(bytemuck::bytes_of(&level.item_count)), level.num_groups);
    }

    // 4) scatter output
    dispatch_compute(pass, scatter_output, &[
        &bg.sort_bind_group,
    ], None, calced_wg_size);

    // 5) copy back
    dispatch_compute(pass, copy_back, &[
        &bg.sort_bind_group,
    ], None, calced_wg_size);
}   

pub fn init_count_sort_compute_pipeline(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    // Layout
    let sort_layout = BindGroupLayoutDescriptor::new(
        "count_sort_layout",
        &[
            storage_buffer::<Vec<u32>>(false).build(0, ShaderStages::COMPUTE), // input_items
            storage_buffer::<Vec<u32>>(false).build(1, ShaderStages::COMPUTE), // input_keys
            storage_buffer::<Vec<u32>>(false).build(2, ShaderStages::COMPUTE), // sorted_items
            storage_buffer::<Vec<u32>>(false).build(3, ShaderStages::COMPUTE), // sorted_keys
            storage_buffer::<Vec<u32>>(false).build(4, ShaderStages::COMPUTE), // counts
            uniform_buffer::<u32>(false).build(5, ShaderStages::COMPUTE), // num_inputs
        ],
    );

    let scan_layout = BindGroupLayoutDescriptor::new(
        "scan_layout",
        &[
            storage_buffer::<Vec<u32>>(false).build(0, ShaderStages::COMPUTE), // elements (count_buffer from count_sort)
        ],
    );

    let group_layout = BindGroupLayoutDescriptor::new(
        "group_layout",
        &[
            storage_buffer::<Vec<u32>>(false).build(0, ShaderStages::COMPUTE), // group_sums
        ],
    );

    let count_layout = BindGroupLayoutDescriptor::new(
        "count_layout",
        &[
            uniform_buffer::<u32>(false).build(0, ShaderStages::COMPUTE), // item_count
        ],
    );

    let count_sort_shader: Handle<Shader> = asset_server.load("shaders/count_sort_compute.wgsl");
    let scan_shader: Handle<Shader> = asset_server.load("shaders/scan_compute.wgsl");

    let clear_counts = create_pipeline(
        CreatePipelineArgs {
            for_shader: count_sort_shader.clone(),
            entry_point: "clear_counts",
            bind_group_layout: sort_layout.clone(),
            ..Default::default()
        },
        &pipeline_cache,
    );

    let count = create_pipeline(
        CreatePipelineArgs {
            for_shader: count_sort_shader.clone(),
            entry_point: "calculate_counts",
            bind_group_layout: sort_layout.clone(),
            ..Default::default()
        },
        &pipeline_cache,
    );
    
    let scan = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("scan_pipeline".into()),
            layout: vec![
                scan_layout.clone(),
                group_layout.clone(),
            ],
            push_constant_ranges: vec![
                PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..4,
                }
            ],
            shader: scan_shader.clone(),
            entry_point: Some("block_scan".into()),
            ..Default::default()
        }
    );

    let combine = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("combine_pipeline".into()),
            layout: vec![
                scan_layout.clone(),
                group_layout.clone(),
            ],
            push_constant_ranges: vec![
                PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..4,
                }
            ],
            shader: scan_shader.clone(),
            entry_point: Some("block_combine".into()),
            ..Default::default()
        }
    );
    
    let scatter_output = create_pipeline(
        CreatePipelineArgs {
            for_shader: count_sort_shader.clone(),
            entry_point: "scatter_output",
            bind_group_layout: sort_layout.clone(),
            ..Default::default()
        },
        &pipeline_cache,
    );

    let copy_back = create_pipeline(
        CreatePipelineArgs {
            for_shader: count_sort_shader.clone(),
            entry_point: "copy_back",
            bind_group_layout: sort_layout.clone(),
            ..Default::default()
        },
        &pipeline_cache,
    );

    commands.insert_resource(CountSortComputePipeline {
        sort_layout,
        scan_layout,
        count_layout,
        group_layout,
        clear_counts,
        count,
        scan,
        combine,
        scatter_output,
        copy_back,
    });

}

pub fn init_count_sort_system(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    //render_queue: Res<RenderQueue>,
    query: Query<(Entity, &SimulationParams), Without<InternalCountSortBuffers>>,
) {

    if query.is_empty() {
        return;
    }

    for (entity, params) in &query {
        let particle_count = params.particle_count;
        let internal_buffers = create_internal_count_sort_buffers(&render_device, particle_count);

        warn!("Created InternalCountSortBuffers for entity {:?}", entity);
        commands.entity(entity).insert((
            internal_buffers,
        ));
    }
}

/// RenderApp - Erstelle eine "echte" BindGroup für count_sort and scan
pub fn prepare_count_sort_bind_groups(
    mut commands: Commands,
    pipeline: Res<CountSortComputePipeline>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    query: Query<(Entity, &InternalCountSortBuffers, &InternalSimulationBuffers), With<InternalCountSortBuffers>>,
) {
    if query.is_empty() {
        return;
    }

    for (entity, buffers, sim_buffers) in &query {

        let num_items = buffers.num_items.as_entire_binding();
        let input_items = sim_buffers.spatial_indices.as_entire_binding(); // spatial indices
        let input_keys = sim_buffers.spatial_keys.as_entire_binding();
        let sorted_items = sim_buffers.sorted_indices.as_entire_binding();
        let sorted_keys = buffers.sorted_keys.as_entire_binding();
        let counts = buffers.counts.as_entire_binding();
        //let group_sums = buffers.group_sums.as_entire_binding();

        //let uniform_buffer = simulation_uniform.buffer.as_ref().unwrap();

        // we need to create this for each pipeline...?
        let sort_bind_group = render_device.create_bind_group(
            "count_sort_bind_group",
            &pipeline_cache.get_bind_group_layout(&pipeline.sort_layout),
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: input_items.clone(), // Spatial Indices
                },
                BindGroupEntry {
                    binding: 1,
                    resource: input_keys.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: sorted_items.clone(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: sorted_keys.clone(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: counts.clone(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: num_items.clone(),
                },
            ],
        );

        let mut groups = HashMap::new();

        for (&num_groups, group_sum_buffer) in &buffers.group_sums {
            let group_sums = group_sum_buffer.as_entire_binding();
            let group_bind_group = render_device.create_bind_group(
                "count_sort_per_group_bind_group",
                &pipeline_cache.get_bind_group_layout(&pipeline.group_layout),
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: group_sums,
                    },
                ],
            );
            groups.insert(num_groups, group_bind_group);
        }

        commands.entity(entity).insert(
            (
                PreparedCountSortComputeBindGroup { 
                    sort_bind_group,
                    scan_bind_group: render_device.create_bind_group(
                        "scan_bind_group",
                        &pipeline_cache.get_bind_group_layout(&pipeline.scan_layout),
                        &[
                            BindGroupEntry {
                                binding: 0,
                                resource: counts.clone(),
                            },
                        ],
                    ),
                    per_group_count: groups,
                    count_bind_group: render_device.create_bind_group(
                        "count_bind_group",
                        &pipeline_cache.get_bind_group_layout(&pipeline.count_layout),
                        &[
                            BindGroupEntry {
                                binding: 0,
                                resource: num_items.clone(),
                            },
                        ],
                    ),
                 },
            )
        );
    }
}



/// create all required internal buffers for count_sort
pub fn create_internal_count_sort_buffers(
    render_device: &RenderDevice,
    num_items: u32,
) -> InternalCountSortBuffers {
    let buffer_size_u32 = (num_items as usize) * std::mem::size_of::<u32>();

    let sorted_keys = render_device.create_buffer(&BufferDescriptor {
        label: Some("sorted_keys_buffer"),
        size: buffer_size_u32 as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let counts = render_device.create_buffer(&BufferDescriptor {
        label: Some("counts_buffer"),
        size: buffer_size_u32 as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let uniform_num_items = render_device.create_buffer(&BufferDescriptor {
        label: Some("num_items_uniform"),
        size: std::mem::size_of::<u32>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut count_items = num_items;
    let mut group_sums = HashMap::new();

    loop {
        let num_groups = calc_num_groups(count_items, WORKGROUP_SIZE);
        let group_sum_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some(&format!("group_sums_buffer_{}", num_groups)),
            size: (num_groups as usize * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        group_sums.insert(num_groups, group_sum_buffer);
        if num_groups <= 1 {
            break;
        }
        count_items = num_groups;
    }

    InternalCountSortBuffers {
        //sorted_items,
        sorted_keys,
        counts,
        group_sums,
        num_items: uniform_num_items,
    }
}