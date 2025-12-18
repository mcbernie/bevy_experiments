use bevy::prelude::*;
use bevy::asset::{AssetLoader, io::Reader, LoadContext};
use crate::app_state::AppState;

use super::atlas::BlocksConfig;

pub struct AtlasConfigPlugin;

impl Plugin for AtlasConfigPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_asset::<BlocksConfigAsset>()
        .init_asset_loader::<BlocksRonLoader>()
        .add_systems(OnEnter(AppState::Loading), load_blocks_config)
        .add_systems(Update, 
            promote_blocks_config_to_resource.run_if(in_state(AppState::Loading))
        );
    }
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct BlocksConfigAsset(pub BlocksConfig);

#[derive(Default)]
pub struct BlocksRonLoader;

#[derive(Resource)]
pub struct BlocksConfigHandle(pub Handle<BlocksConfigAsset>);

#[derive(Resource, Clone)]
pub struct BlocksConfigRes(pub BlocksConfig);

fn load_blocks_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    // assets/blocks.ron
    let handle: Handle<BlocksConfigAsset> = asset_server.load("blocks.ron");
    commands.insert_resource(BlocksConfigHandle(handle));
}

fn promote_blocks_config_to_resource(
    mut commands: Commands,
    handle: Res<BlocksConfigHandle>,
    assets: Res<Assets<BlocksConfigAsset>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(asset) = assets.get(&handle.0) {
        commands.insert_resource(BlocksConfigRes(asset.0.clone()));
        commands.remove_resource::<BlocksConfigHandle>();
        next_state.set(AppState::InGame);
    }
}

impl AssetLoader for BlocksRonLoader {
    type Asset = BlocksConfigAsset;
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
        let cfg: BlocksConfig = ron::from_str(s)?;
        Ok(BlocksConfigAsset(cfg))
    }
}