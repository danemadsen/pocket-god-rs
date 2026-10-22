//! Save files in the browser's `localStorage`, base64-encoded.

use bevy::prelude::*;
use pg_engine::prefs::{PreferencesHooks, StorageDir};

use super::{APP_DIRECTORY, directory_name, is_valid_file_name};
use crate::base64;

fn local_storage() -> Option<web_sys::Storage> {
    let window = web_sys::window()?;
    window.local_storage().ok().flatten()
}

/// The `localStorage` key of a file.
fn storage_key(dir: StorageDir, name: &str) -> Option<String> {
    if !is_valid_file_name(name) {
        warn!("refusing the file name {name:?}");
        return None;
    }
    let directory = directory_name(dir);
    Some(format!("{APP_DIRECTORY}/{directory}/{name}"))
}

fn read_file(_world: &mut World, dir: StorageDir, name: &str) -> Option<Vec<u8>> {
    let key = storage_key(dir, name)?;
    let storage = local_storage()?;
    let encoded = storage.get_item(&key).ok().flatten()?;
    base64::decode(&encoded)
}

fn write_file(_world: &mut World, dir: StorageDir, name: &str, data: &[u8]) -> bool {
    let Some(key) = storage_key(dir, name) else {
        return false;
    };
    let Some(storage) = local_storage() else {
        return false;
    };
    let encoded = base64::encode(data);
    match storage.set_item(&key, &encoded) {
        Ok(()) => true,
        Err(error) => {
            warn!("could not save {key}: {error:?}");
            false
        }
    }
}

fn remove_file(_world: &mut World, dir: StorageDir, name: &str) {
    let Some(key) = storage_key(dir, name) else {
        return;
    };
    if let Some(storage) = local_storage() {
        let _ = storage.remove_item(&key);
    }
}

fn file_exists(_world: &mut World, dir: StorageDir, name: &str) -> bool {
    let Some(key) = storage_key(dir, name) else {
        return false;
    };
    let Some(storage) = local_storage() else {
        return false;
    };
    matches!(storage.get_item(&key), Ok(Some(_)))
}

/// The `localStorage` hooks, if the page may use `localStorage`.
pub(super) fn hooks() -> Option<PreferencesHooks> {
    local_storage()?;
    let hooks = PreferencesHooks {
        read_file,
        write_file,
        remove_file,
        file_exists,
        ..PreferencesHooks::default()
    };
    Some(hooks)
}
