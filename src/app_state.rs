use bevy::prelude::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    InGame,
}

#[derive(Resource, Default)]
pub struct LoadingProgress {
    pub config_loaded: bool,
    pub atlas_loaded: bool,
    pub skybox_loaded: bool,
}

#[derive(Component)]
pub struct LoadingUiRoot;

pub fn spawn_loading_ui(mut commands: Commands) {
    commands.spawn((
        LoadingUiRoot,
        Text::new("Loading..."),
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

pub fn despawn_loading_ui(mut commands: Commands, q: Query<Entity, With<LoadingUiRoot>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}