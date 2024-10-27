use std::mem::size_of;
use std::ops::Deref;

use bevy::app::{self, App};
use bevy::asset::io::AssetReaderError;
use bevy::asset::{
    self, Asset, AssetApp, AssetLoadError, AssetLoader, AssetServer, Assets, AsyncReadExt, Handle,
};
use bevy::ecs::system::{Res, ResMut, Resource, SystemParam};
use bevy::ecs::world::{FromWorld, World};
use bevy::reflect::TypePath;
use bevy::utils::HashMap;
use serde::Deserialize;
use traffloat_view::translation;

use crate::options::Options;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cache>();
        app.init_asset::<AssetWrapper>();
        app.init_asset_loader::<JsonAssetLoader>();

        app.add_systems(app::FixedUpdate, |mut loader: ResMut<Cache>, server: Res<AssetServer>| {
            loader.maintain(&server);
        });
    }
}

#[derive(Resource)]
struct Cache {
    locales: Vec<String>,
    handles: HashMap<translation::GlossarySha, FallbackState>,
}

impl FromWorld for Cache {
    fn from_world(world: &mut World) -> Self {
        let locales = world.resource::<Options>().locales.clone();
        assert!(!locales.is_empty(), "locale list must not be empty");

        Self { locales, handles: HashMap::new() }
    }
}

fn asset_path(sha: translation::GlossarySha, locale: &str) -> String {
    let mut path_buf = vec![0u8; size_of::<translation::GlossarySha>() * 2 + 1 + locale.len() + 7];
    hex::encode_to_slice(sha.0, &mut path_buf[..sha.0.len() * 2]).unwrap();
    path_buf[sha.0.len() * 2] = b'/';
    path_buf[sha.0.len() * 2 + 1..][..locale.len()].copy_from_slice(locale.as_bytes());
    let extension_offset = path_buf.len() - 7;
    path_buf[extension_offset..].copy_from_slice(b".tfglos");
    String::from_utf8(path_buf).unwrap()
}

impl Cache {
    fn maintain(&mut self, server: &AssetServer) {
        for (&sha, state) in &mut self.handles {
            let Some(ref handle) = state.handle else { continue };

            if let asset::LoadState::Failed(err) =
                server.get_load_state(handle).expect("active handle must have load state")
            {
                if !matches!(
                    *err,
                    AssetLoadError::AssetReaderError(
                        AssetReaderError::NotFound(_) | AssetReaderError::HttpError(404)
                    )
                ) {
                    bevy::log::warn!("Failed loading translation file: {err}");
                }

                state.step += 1;
                match self.locales.get(state.step) {
                    Some(locale) => state.handle = Some(server.load(asset_path(sha, locale))),
                    None => state.handle = None,
                };
            }
        }
    }

    fn get<'assets>(
        &mut self,
        sha: translation::GlossarySha,
        server: &AssetServer,
        assets: &'assets Assets<AssetWrapper>,
    ) -> Option<&'assets translation::Glossary> {
        if let Some(state) = self.handles.get(&sha) {
            let Some(ref handle) = state.handle else {
                return None; // all locales are unavailable
            };

            assets.get(handle).map(|asset| &asset.0)
        } else {
            let locale = self.locales.first().expect("checked emptiness");

            // if there is a race condition in another thread,
            // AssetServer will handle the duplicate load.
            let handle = server.load(asset_path(sha, locale));
            self.handles.insert(sha, FallbackState { handle: Some(handle), step: 0 });
            None
        }
    }
}

#[derive(SystemParam)]
pub struct Provider<'w> {
    loader: ResMut<'w, Cache>,
    server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<AssetWrapper>>,
}

impl<'w> translation::Provider for Provider<'w> {
    fn get(
        &mut self,
        sha: translation::GlossarySha,
    ) -> Option<impl Deref<Target = translation::Glossary> + '_> {
        self.loader.get(sha, &self.server, &self.assets)
    }
}

struct FallbackState {
    handle: Option<Handle<AssetWrapper>>,
    step:   usize,
}

#[derive(Asset, TypePath, Debug, Deserialize)]
struct AssetWrapper(translation::Glossary);

#[derive(Default)]
struct JsonAssetLoader;

impl AssetLoader for JsonAssetLoader {
    type Asset = AssetWrapper;
    type Settings = ();
    type Error = LoadError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut asset::io::Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await.map_err(LoadError::Read)?;
        let glossary =
            serde_json::from_slice::<translation::Glossary>(&bytes).map_err(LoadError::Decode)?;
        Ok(AssetWrapper(glossary))
    }
}

#[derive(Debug, thiserror::Error)]
enum LoadError {
    #[error("io error: {0}")]
    Read(std::io::Error),
    #[error("decode input: {0}")]
    Decode(serde_json::Error),
}
