use bevy::{color::palettes::css::WHITE, pbr::wireframe::{WireframeConfig, WireframePlugin}, prelude::*};

use crate::app_state::AppState;

mod app_state;
mod config;
mod voxel;
mod camera;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WireframePlugin::default(),
        ))
        .init_state::<AppState>()
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: true,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: WHITE.into(),
        })
        .add_plugins((config::AtlasConfigPlugin, voxel::VoxelPlugin))
        .add_plugins(camera::CameraPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_colors)
        .insert_resource(ClearColor(Color::BLACK))
        .run();
}


// setup system
fn setup_scene(
    mut commands: Commands,
) {

    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
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