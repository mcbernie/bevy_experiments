# Fluid Simulation using Compute Shaders

[![Short Demonstration Video](https://img.youtube.com/vi/3wkAE7lhHnY/0.jpg)](https://www.youtube.com/watch?v=3wkAE7lhHnY)

This project explores a fluid simulation implemented using a compute shader pipeline.

The implementation is inspired by:
https://www.youtube.com/watch?v=rSKMYc1CQHE

The project is currently far from reaching the level shown in the video.

## Simple "Usage"

Inside `WorldInspector` we can update the SimulationParams inside a **Particle Simulation**
Movement of Camera is done by **WASD**, **Space** for Up, **LShift** for Down and **L** for Lock / unlock mouse to camera.

## Current State

At the moment, the project implements a basic particle-based simulation.
It’s still fairly simple, but the whole pipeline works and serves as a playground
for further experiments.

A custom material accesses a `StorageBuffer` and renders the particles as cubes.
The cube color is derived from the particle velocity.

Instead of using instancing with multiple draw calls, a single large mesh is created
(`VERTICES * PARTICLE_MAX`). The vertex data is then manipulated directly in the
material’s vertex shader.

Camera mouse movement is disabled by default
and can be enabled by pressing `L`.

The code is currently a bit chaotic in places.
This is intentional: a lot of things are explored via trial and error,
which is simply how I like to learn and experiment.

The `PARTICLE_MAX` size is defined as a const in `main.rs`.

## Simulation / Compute Shader – Order of Operations

Begin switchig from RenderNode System to a System which calls all compute shaders.


### Buffer Initialization

- In `simulation/system` all **shared GPU buffers** are created.

### Simulation Nodes (`simulation/node`)

- `StartSimulationNode`
  - Executes the **external_forces** compute shader
  - Then runs the **spatial_hash** compute shader (initial hash build)

### GPU Sorting (`simulation/gpu_sort`)

- `CountSortNode`
  - Executes all compute shaders required for the **count sort algorithm**

- In `simulation/gpu_sort/system.rs`
  - All **internal buffers** required by the count sort process are created

### Spatial Hash Offset Generation

- After the count sort finishes, the **spatial hash offset** is generated
- Implemented in `simulation/spatial_hash`
- `SpatialHashNode`
  - Runs the corresponding compute shader
  - Writes the final offsets

### Finalization

- `FinalSimulationNode` (second render node in `simulation/node`)
  - Final simulation step after all compute passes are complete

---

## Design Notes / Open Questions

The original goal of this project is to support **multiple independent simulations** within a single game.

For this reason:

- Buffers and other simulation-related data are stored in **components**
- No global simulation resources are used

---

## Compute Pass Structure

Currently, a single compute pass is created and multiple pipelines are dispatched within it:

```rust
let mut pass = render_context
    .command_encoder()
    .begin_compute_pass(&ComputePassDescriptor {
        label: Some("begin_simulation_compute"),
        ..Default::default()
    });

for (_, bg) in bind_groups.iter(world) {
    pass.set_pipeline(external_forces);
    pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
    pass.set_bind_group(0, &bg.bind_group, &[]);
    pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

    pass.set_pipeline(spatial_hash);
    pass.set_bind_group(0, &bg.bind_group, &[]);
    pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
}
```

### Alternative Approach

An alternative would be to create a **separate compute pass per pipeline dispatch**:

```rust
for (_, bg) in bind_groups.iter(world) {
    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("compute_external_forces"),
            ..Default::default()
        });
    pass.set_pipeline(external_forces);
    pass.set_push_constants(0, bytemuck::bytes_of(&FIXED_DT));
    pass.set_bind_group(0, &bg.bind_group, &[]);
    pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);

    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor {
            label: Some("compute_spatial_hash"),
            ..Default::default()
        });
    pass.set_pipeline(spatial_hash);
    pass.set_bind_group(0, &bg.bind_group, &[]);
    pass.dispatch_workgroups((PARTICLE_COUNT + 255) / 256, 1, 1);
}
```
