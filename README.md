# Fluid base Simulation with computeshaders

i want to create a fluid simulation based on a compute shader pipeline.
as source for help i heavly search bevy source and docs.rs. Inspiration comes from https://www.youtube.com/watch?v=rSKMYc1CQHE 

## State

currently i got a simple stupid particle animation and simulation. (but it works)

## System-Flow (rough idea)

RenderStartup:
  init_sim_pipelines
  init_sim_buffers

Render:
  prepare_sim_bind_groups
  update_uniforms

RenderGraph:
  SimNode_ExternalForces
  SimNode_SpatialHash
  SimNode_Sort
  SimNode_Reorder
  SimNode_Density
  SimNode_Integrate
