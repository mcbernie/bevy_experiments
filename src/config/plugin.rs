use bevy::prelude::*;
use bevy::asset::{AssetLoader, io::Reader, LoadContext};
use crate::app_state::{AppState, LoadingProgress};

use super::configuration::SystemConfig;

pub struct SystemConfigPlugin;

impl Plugin for SystemConfigPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_asset::<SystemConfigAsset>()
        .init_asset_loader::<ConfigRonLoader>()
        .add_systems(OnEnter(AppState::Loading), load_system_config)
        .add_systems(Update, 
            promote_system_config_to_resource.run_if(in_state(AppState::Loading))
        );
    }
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct SystemConfigAsset(pub SystemConfig);

#[derive(Default, TypePath)]
pub struct ConfigRonLoader;

#[derive(Resource)]
pub struct SystemConfigHandle(pub Handle<SystemConfigAsset>);

#[derive(Resource, Clone)]
pub struct SystemConfigRes(pub SystemConfig);

fn load_system_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    // assets/system.ron
    let handle: Handle<SystemConfigAsset> = asset_server.load("system.ron");
    commands.insert_resource(SystemConfigHandle(handle));
    commands.insert_resource(LoadingProgress::default());
}

fn promote_system_config_to_resource(
    mut commands: Commands,
    handle: Option<Res<SystemConfigHandle>>,
    assets: Res<Assets<SystemConfigAsset>>,
    mut progress: ResMut<LoadingProgress>,
) {
    let Some(handle) = handle else {
        return;
    };

    if let Some(asset) = assets.get(&handle.0) {
        commands.insert_resource(SystemConfigRes(asset.0.clone()));
        commands.remove_resource::<SystemConfigHandle>();
        progress.config_loaded = true
    }
}

impl AssetLoader for ConfigRonLoader {
    type Asset = SystemConfigAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let s = std::str::from_utf8(&bytes)?;
        let cfg: SystemConfig = ron::from_str(s)?;
        Ok(SystemConfigAsset(cfg))
    }
}