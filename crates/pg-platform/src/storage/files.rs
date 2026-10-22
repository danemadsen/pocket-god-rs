//! Save files in a directory (desktop and Android), written atomically.

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use pg_engine::prefs::{PreferencesHooks, StorageDir};

use super::{directory_name, is_valid_file_name};

/// Environment variable naming the save directory itself (tests, portable installs).
#[cfg(not(target_os = "android"))]
pub const DATA_DIR_ENV: &str = "PG_DATA_DIR";

/// The directory holding `Documents/` and `Caches/`.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct StorageRoot(pub PathBuf);

impl StorageRoot {
    /// Where `name` lives in `dir`, or `None` for a name that could leave the directory.
    pub fn path(&self, dir: StorageDir, name: &str) -> Option<PathBuf> {
        if !is_valid_file_name(name) {
            warn!("refusing the file name {name:?}");
            return None;
        }
        let directory = self.0.join(directory_name(dir));
        Some(directory.join(name))
    }
}

/// The per-user data directory of the desktop OS.
#[cfg(not(target_os = "android"))]
fn platform_data_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);

    if cfg!(target_os = "macos") {
        let home = home?;
        return Some(home.join("Library").join("Application Support"));
    }
    if cfg!(windows) {
        let app_data = std::env::var_os("APPDATA")?;
        return Some(PathBuf::from(app_data));
    }
    let xdg = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    if let Some(xdg) = xdg.filter(|path| path.is_absolute()) {
        return Some(xdg);
    }
    let home = home?;
    Some(home.join(".local").join("share"))
}

/// The save directory: `PG_DATA_DIR`, else `<data dir>/PocketGod`.
#[cfg(not(target_os = "android"))]
fn storage_root() -> Option<PathBuf> {
    if let Some(overridden) = std::env::var_os(DATA_DIR_ENV) {
        return Some(PathBuf::from(overridden));
    }
    let data_dir = platform_data_dir()?;
    Some(data_dir.join(super::APP_DIRECTORY))
}

/// The save directory: the app's internal storage.
#[cfg(target_os = "android")]
fn storage_root() -> Option<PathBuf> {
    crate::android::internal_data_path()
}

/// Creates `path` with `data` and flushes it to disk.
fn write_and_sync(path: &Path, data: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    file.sync_all()
}

/// Replaces `path` with `data` atomically: a temporary file in the same directory, flushed to
/// disk, then renamed over the target.
pub fn write_atomically(path: &Path, data: &[u8]) -> io::Result<()> {
    let Some(directory) = path.parent() else {
        return Err(io::Error::other("a file needs a directory"));
    };
    let Some(name) = path.file_name() else {
        return Err(io::Error::other("a file needs a name"));
    };
    fs::create_dir_all(directory)?;

    let mut temporary_name = std::ffi::OsString::from(".");
    temporary_name.push(name);
    temporary_name.push(".tmp");
    let temporary = directory.join(temporary_name);

    let written = write_and_sync(&temporary, data);
    if let Err(error) = written {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    fs::rename(&temporary, path)
}

fn read_file(world: &mut World, dir: StorageDir, name: &str) -> Option<Vec<u8>> {
    let root = world.get_resource::<StorageRoot>()?;
    let path = root.path(dir, name)?;
    fs::read(path).ok()
}

fn write_file(world: &mut World, dir: StorageDir, name: &str, data: &[u8]) -> bool {
    let Some(root) = world.get_resource::<StorageRoot>() else {
        return false;
    };
    let Some(path) = root.path(dir, name) else {
        return false;
    };
    match write_atomically(&path, data) {
        Ok(()) => true,
        Err(error) => {
            warn!("could not save {}: {error}", path.display());
            false
        }
    }
}

fn remove_file(world: &mut World, dir: StorageDir, name: &str) {
    let Some(root) = world.get_resource::<StorageRoot>() else {
        return;
    };
    let Some(path) = root.path(dir, name) else {
        return;
    };
    let _ = fs::remove_file(path);
}

fn file_exists(world: &mut World, dir: StorageDir, name: &str) -> bool {
    let Some(root) = world.get_resource::<StorageRoot>() else {
        return false;
    };
    match root.path(dir, name) {
        Some(path) => path.is_file(),
        None => false,
    }
}

/// The file hooks over the platform's save directory (inserted as [`StorageRoot`]).
pub(super) fn install(app: &mut App) -> Option<PreferencesHooks> {
    let root = storage_root()?;
    info!("saved games live in {}", root.display());
    app.insert_resource(StorageRoot(root));

    let hooks = PreferencesHooks {
        read_file,
        write_file,
        remove_file,
        file_exists,
        ..PreferencesHooks::default()
    };
    Some(hooks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_root(test: &str) -> PathBuf {
        let unique = format!("pg-platform-{test}-{}", std::process::id());
        let root = std::env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn world_with_root(root: &Path) -> World {
        let mut world = World::new();
        world.insert_resource(StorageRoot(root.to_path_buf()));
        world
    }

    #[test]
    fn files_round_trip_per_directory() {
        let root = scratch_root("round-trip");
        let mut world = world_with_root(&root);

        assert!(!file_exists(&mut world, StorageDir::Documents, "Options.json"));
        assert!(write_file(&mut world, StorageDir::Documents, "Options.json", b"{\"a\":1}"));
        assert!(write_file(&mut world, StorageDir::Caches, "news.xml", b"<news/>"));

        assert!(file_exists(&mut world, StorageDir::Documents, "Options.json"));
        assert!(!file_exists(&mut world, StorageDir::Caches, "Options.json"));
        let options = read_file(&mut world, StorageDir::Documents, "Options.json");
        assert_eq!(options.as_deref(), Some(&b"{\"a\":1}"[..]));
        assert!(root.join("Caches").join("news.xml").is_file());

        assert!(write_file(&mut world, StorageDir::Documents, "Options.json", b"2"));
        let replaced = read_file(&mut world, StorageDir::Documents, "Options.json");
        assert_eq!(replaced.as_deref(), Some(&b"2"[..]));
        let leftovers: Vec<_> = fs::read_dir(root.join("Documents")).expect("exists").collect();
        assert_eq!(leftovers.len(), 1, "no temporary file stays behind");

        remove_file(&mut world, StorageDir::Documents, "Options.json");
        assert!(!file_exists(&mut world, StorageDir::Documents, "Options.json"));
        assert_eq!(read_file(&mut world, StorageDir::Documents, "Options.json"), None);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn names_that_leave_the_directory_are_refused() {
        let root = scratch_root("escape");
        let mut world = world_with_root(&root);
        assert!(!write_file(&mut world, StorageDir::Documents, "../outside", b"x"));
        assert!(!root.join("outside").exists());
        assert_eq!(read_file(&mut world, StorageDir::Documents, "../outside"), None);
        let _ = fs::remove_dir_all(&root);
    }
}
