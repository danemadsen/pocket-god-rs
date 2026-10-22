//! Serves the embedded files as Bevy's default asset source, so `AssetServer::load("…")`
//! reads from the executable instead of the file system.

use std::path::{Path, PathBuf};

use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSourceBuilder, AssetSourceId, PathStream, Reader, SliceReader,
};
use bevy::prelude::*;
use bevy::tasks::futures_lite::stream;

use crate::FILES;

/// Registers the embedded files as the default asset source. Add it before `DefaultPlugins`.
pub struct EmbeddedAssetsPlugin;

impl Plugin for EmbeddedAssetsPlugin {
    fn build(&self, app: &mut App) {
        let source = AssetSourceBuilder::new(|| Box::new(EmbeddedReader));
        app.register_asset_source(AssetSourceId::Default, source);
    }
}

struct EmbeddedReader;

fn asset_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl AssetReader for EmbeddedReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let key = asset_key(path);
        let bytes = crate::file(&key).ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))?;
        Ok(SliceReader::new(bytes))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        // Embedded assets never carry .meta files; Bevy falls back to default settings.
        Err::<SliceReader<'static>, _>(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_directory<'a>(&'a self, path: &'a Path) -> Result<Box<PathStream>, AssetReaderError> {
        let key = asset_key(path);
        let prefix = if key.is_empty() { String::new() } else { format!("{key}/") };

        let mut children: Vec<PathBuf> = Vec::new();
        for entry in FILES.iter() {
            let Some(rest) = entry.path.strip_prefix(&prefix) else {
                continue;
            };
            let child_name = rest.split('/').next().unwrap_or(rest);
            let child = PathBuf::from(format!("{prefix}{child_name}"));
            if !children.contains(&child) {
                children.push(child);
            }
        }

        if children.is_empty() {
            return Err(AssetReaderError::NotFound(path.to_path_buf()));
        }
        Ok(Box::new(stream::iter(children)))
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        let key = asset_key(path);
        let prefix = format!("{key}/");
        let is_directory = FILES.iter().any(|entry| entry.path.starts_with(&prefix));
        Ok(is_directory)
    }
}
