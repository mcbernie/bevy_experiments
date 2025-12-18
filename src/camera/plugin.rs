use bevy::image::TextureFormatPixelInfo;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureViewDescriptor, TextureViewDimension};
use bevy::{core_pipeline::Skybox, input::mouse::MouseMotion, pbr::ScreenSpaceAmbientOcclusion, prelude::*};
use crate::app_state::{AppState, LoadingProgress};
use crate::config::BlocksConfigRes;

use super::components::FlyCam;
use super::skybox::Cubemap;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (load_skybox, asset_loaded).run_if(in_state(AppState::Loading)))
           .add_systems(OnEnter(AppState::InGame), setup_camera)
           .add_systems(Update, (flycam_look, flycam_move).run_if(in_state(AppState::InGame)));
    }
}

fn load_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: If<Res<BlocksConfigRes>>,
) {
    let config: &BlocksConfigRes = &config;

    let skybox_handle = asset_server.load(format!("skybox/{}", &config.0.skybox.texture));

    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });
}

fn setup_camera(
    mut commands: Commands, 
    cube_map: If<Res<Cubemap>>,
) {
    
    let cube_map: &Cubemap = &cube_map;

    commands.spawn((
        Camera3d::default(), 
        Msaa::Off,
        ScreenSpaceAmbientOcclusion::default(),
        Transform::from_xyz(0.0, 10.0, 20.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam {
            speed: 15.0,
            sensitivity: 0.002,
        },
        Skybox {
            image: cube_map.image_handle.clone(),
            brightness: 1000.0,
            ..default()
        }
    ));

}

fn flycam_look(
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let mut delta = Vec2::ZERO;
    for ev in mouse_motion_events.read() {
        delta += ev.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    for (cam, mut transform) in &mut query {
        let yaw = Quat::from_rotation_y(-delta.x * cam.sensitivity);
        let pitch = Quat::from_rotation_x(-delta.y * cam.sensitivity);

        transform.rotation = yaw * transform.rotation;
        transform.rotation = transform.rotation * pitch;
    }
}

fn flycam_move(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    for (cam, mut transform) in &mut query {
        let mut dir = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            dir += transform.forward().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir -= transform.forward().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir -= transform.right().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir += transform.right().as_vec3();
        }
        if keyboard.pressed(KeyCode::Space) {
            dir += Vec3::Y;
        }
        if keyboard.pressed(KeyCode::ControlLeft) {
            dir -= Vec3::Y;
        }

        let mut speed = cam.speed;
        if keyboard.pressed(KeyCode::ShiftLeft) {
            speed *= 3.0;
        }

        if dir != Vec3::ZERO {
            transform.translation += dir.normalize() * speed * time.delta_secs();
        }
    }
}

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: If<ResMut<Cubemap>>,
    mut loaded: ResMut<LoadingProgress>,
) {

    if loaded.skybox_loaded {
        return;
    }
    let cubemap: &mut Cubemap = &mut cubemap; 

    if cubemap.is_loaded {
        return;
    }
    
    if !asset_server.load_state(&cubemap.image_handle).is_loaded() {
        return;
    }

    // Source: Cross PNG
    let image = images.get_mut(&cubemap.image_handle).unwrap();

    let w = image.size().x as u32;
    let h = image.size().y as u32;

    info!("Skybox cross image loaded: {}x{}", w, h);    

    // 4x3 Cross erwartet
    let face = w / 4;
    if face * 4 != w || face * 3 != h {
        panic!("cross.png hat nicht 4x3 layout: {}x{}", w, h);
    }

    let format = image.texture_descriptor.format;
    let bpp = format.pixel_size().unwrap(); // bytes per pixel

    let src_data = image.data.as_ref().expect("cross image has no CPU data");

    // Ziel-Puffer: face x face x 6
    let mut dst_data = vec![0u8; (face * face * 6) as usize * bpp];

    // Helper: (cell_x,cell_y) aus 4x3 cross -> layer im dst
    let mut blit_face = |dst_layer: u32, cell_x: u32, cell_y: u32| {
        let src_x0 = cell_x * face;
        let src_y0 = cell_y * face;

        for y in 0..face {
            let src_row = (src_y0 + y) * w + src_x0;
            let dst_row = (dst_layer * face + y) * face;

            let src_i = (src_row as usize) * bpp;
            let dst_i = (dst_row as usize) * bpp;

            let len = (face as usize) * bpp;
            dst_data[dst_i..dst_i + len].copy_from_slice(&src_data[src_i..src_i + len]);
        }
    };

    // Mapping (ohne Rotation/Flip):
    // 0:+X (2,1)  1:-X (0,1)  2:+Y (1,0)  3:-Y (1,2)  4:+Z (1,1)  5:-Z (3,1)
    blit_face(0, 2, 1);
    blit_face(1, 0, 1);
    blit_face(2, 1, 0);
    blit_face(3, 1, 2);
    blit_face(4, 1, 1);
    blit_face(5, 3, 1);

    // Jetzt das Image-Asset “umdefinieren”: Größe = array(6), Daten = dst, View = Cube
    image.data = Some(dst_data);
    image.texture_descriptor.size = Extent3d {
        width: face,
        height: face,
        depth_or_array_layers: 6,
    };
    image.texture_descriptor.dimension = TextureDimension::D2;

    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..Default::default()
    });

    cubemap.is_loaded = true;
    loaded.skybox_loaded = true;
}