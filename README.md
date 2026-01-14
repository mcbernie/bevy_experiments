# Fluid Simulation using Compute Shaders

This project explores a fluid simulation implemented using a compute shader pipeline.

The implementation is inspired by:
https://www.youtube.com/watch?v=rSKMYc1CQHE

The project is currently far from reaching the level shown in the video.

## Current State

At the moment, the project implements a basic particle-based simulation.
It’s still fairly simple, but the whole pipeline works and serves as a playground
for further experiments.

A custom material accesses a `StorageBuffer` and renders the particles as cubes.
The cube color is derived from the particle velocity.

Instead of using instancing with multiple draw calls, a single large mesh is created
(`VERTICES * PARTICLE_MAX`). The vertex data is then manipulated directly in the
material’s vertex shader.

Before the 0.18 release, a small `bevy-egui` UI was added to tweak things like
box / border size and gravity. Camera mouse movement is disabled by default
and can be enabled by pressing `L`.

The code is currently a bit chaotic in places.
This is intentional: a lot of things are explored via trial and error,
which is simply how I like to learn and experiment.

The `PARTICLE_MAX` size is defined as a const in `main.rs`.

## It is not optimized (yet)

Right now the compute shader handles collisions in the most naive way possible:
each particle checks against all other particles (O(n²)). This is obviously not great.

The plan is to improve this step by step while learning along the way — e.g. using
spatial hashing / uniform grids, bitmasks / bitpacking, and sorting approaches
(radix / count sort, etc.).

## Planned System Flow (rough idea)

RenderStartup:
- init_sim_pipelines
- init_sim_buffers

Render:
- prepare_sim_bind_groups
- update_uniforms

RenderGraph:
- SimNode_ExternalForces
- SimNode_SpatialHash
- SimNode_Sort
- SimNode_Reorder
- SimNode_Density
- SimNode_Integrate

## Design Notes / Open Questions

The initial idea behind this project was to support multiple independent simulations in a game.
For that reason, buffers and other simulation-related data are stored in components instead of global resources.

This raises the question whether this approach actually makes sense once a RenderGraph node is introduced
that is responsible for dispatching the compute workloads.
In particular: is it reasonable (or even intended) to query components from within a render node,
or is this something that should be avoided?

This becomes even more relevant when multiple compute workloads need to be dispatched in sequence.
Bevy’s *Game of Life* example executes several compute shaders inside a single node,
while the current implementation here performs the work sequentially in a system
running in the `Render` stage of the `RenderApp`.

At the moment, the system-based approach is mainly used because it allows easier access
to component-based simulation data.
Whether this should eventually move into a render node, and how component access should be handled there,
is still an open design question.
