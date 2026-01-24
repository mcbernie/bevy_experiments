use bevy::{
    dev_tools::fps_overlay::{
        FpsOverlayConfig, 
        FpsOverlayPlugin, 
        FrameTimeGraphConfig
    }, 
    pbr::{
        ExtendedMaterial, 
        wireframe::{WireframeConfig, WireframePlugin}
    }, 
    prelude::*, render::{extract_resource::{ExtractResource, ExtractResourcePlugin}, gpu_readback::{Readback, ReadbackComplete}, render_resource::BufferUsages, storage::ShaderStorageBuffer}
};

use bevy_inspector_egui::{bevy_egui::{EguiPlugin, EguiPrimaryContextPass}};


use crate::{
    app_state::{
        AppState, 
        LoadingProgress, 
        despawn_loading_ui,
        spawn_loading_ui
    }, 
    base::create_plane_mesh, simulation::{marching_cubes::{Triangle, Vertex}, material::ParticleMaterial, simulation_params_ui_systems}
};

pub const WORKGROUP_SIZE: u32 = 256; // currently fixed in compute shader
pub const JITTER_STRENGTH: f32 = 0.035;
pub const PARTICLE_SPAWN_DENSITY: u32 = 600;
pub const DENSITY_TEXTURE_RES: u32 = 150;

mod app_state;
mod config;
mod camera;
mod base;
mod simulation;
mod rendering;

#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackBuffer { 
    pub handle: Handle<ShaderStorageBuffer>,
    pub mapped: bool,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Particle Simulation".to_string(),
                    //present_mode: bevy::window::PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            //ExtractResourcePlugin::<ReadbackBuffer>::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 16.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        ..default()
                    },

                    text_color: Color::WHITE,
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 144.0,
                    },
                },
            },
            MaterialPlugin::<ExtendedMaterial<StandardMaterial,base::CheckerMaterial>>::default(),
            MaterialPlugin::<ParticleMaterial>::default(),
            WireframePlugin::default(),
        ))
        .add_plugins(EguiPlugin::default())
        //.add_plugins(WorldInspectorPlugin::new())
        .init_state::<AppState>()
        //.insert_resource(WireframeConfig {
        //    global: true,
        //    default_color: WHITE.into(),
        //})
        .add_systems(OnEnter(AppState::Loading), spawn_loading_ui)
        .add_systems(OnExit(AppState::Loading), despawn_loading_ui)
        .add_plugins(config::SystemConfigPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(simulation::SimulationPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, create_plane_mesh)
        .add_systems(Update, update_colors)
        .add_systems(Update, exit_on_esc)
        .add_systems(Update, advance_to_ingame_when_ready.run_if(in_state(AppState::Loading)))
        .add_systems(EguiPrimaryContextPass, simulation_params_ui_systems)

        .insert_resource(ClearColor(Color::BLACK))
        .run();
}


// setup system
fn setup_scene(
    mut commands: Commands,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>, // for readback buffer debugging...
) {

    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, 0.7, 0.0)),
    ));

    // Text used to show controls
    commands.spawn((
        Text::default(),
        TextFont {
            font_size: 14.0,
            ..Default::default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));

}

/// This system let's you toggle various wireframe settings
fn update_colors(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<WireframeConfig>,
    mut text: Single<&mut Text>,
) {
    text.0 = format!(
        "Controls
---------------
Z - Toggle global

WireframeConfig
-------------
Global: {}",
        config.global
    );

    // Toggle showing a wireframe on all meshes
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        config.global = !config.global;
    }

}

fn exit_on_esc(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: MessageWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_exit_events.write(AppExit::Success);
    }
}

fn advance_to_ingame_when_ready(
    progress: Res<LoadingProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if progress.config_loaded && progress.skybox_loaded {
        next_state.set(AppState::InGame);
    }
}