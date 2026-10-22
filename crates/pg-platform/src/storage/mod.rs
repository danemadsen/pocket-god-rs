//! Save files behind [`PreferencesHooks`] (`docs/re/audio-prefs-text.md` §2, `pg_engine::prefs`):
//! the original's `Documents/` and `Library/Caches/` directories.
//!
//! - Desktop: `<data dir>/PocketGod/Documents` and `…/Caches`, where the data dir is
//!   `~/Library/Application Support` (macOS), `%APPDATA%` (Windows) or `$XDG_DATA_HOME` /
//!   `~/.local/share` (Linux and other Unixes); `PG_DATA_DIR` overrides the whole
//!   `PocketGod` directory.
//! - Android: the app's internal storage.
//! - Web: `localStorage`, one base64 entry per file under `PocketGod/<dir>/<name>`.
//!
//! Writes are atomic on desktop and Android (temporary file, then rename). File names are the
//! game's own (`Persona.json`, `StoryMap`, `story-…`, icon names from the news feed); a name
//! that could leave its directory is refused. There is no network: downloads fail at once.

#[cfg(not(target_arch = "wasm32"))]
mod files;

#[cfg(target_arch = "wasm32")]
mod web;

use bevy::prelude::*;
use pg_engine::prefs::{PreferencesHooks, StorageDir};

/// The name the save directory is created under.
pub const APP_DIRECTORY: &str = "PocketGod";

/// Whether `name` is a plain file name inside its directory.
pub fn is_valid_file_name(name: &str) -> bool {
    let reserved = name.is_empty() || name == "." || name == "..";
    let has_separator = name.contains(['/', '\\', '\0']);
    !reserved && !has_separator
}

/// The directory name of a storage directory, as in the original's sandbox.
pub fn directory_name(dir: StorageDir) -> &'static str {
    match dir {
        StorageDir::Documents => "Documents",
        StorageDir::Caches => "Caches",
    }
}

/// `NSURLConnection` cannot be created: the port has no network (`managers-platform.md` §18).
fn no_download(_world: &mut World, _prefs: Entity, url: &str) -> bool {
    info!("no network in the port; not downloading {url}");
    false
}

/// Installs the platform's file storage for the engine's preference documents.
pub(crate) fn install(app: &mut App) {
    #[cfg(not(target_arch = "wasm32"))]
    let hooks = files::install(app);

    #[cfg(target_arch = "wasm32")]
    let hooks = web::hooks();

    let Some(file_hooks) = hooks else {
        warn!("no save directory; saved games last until the app quits");
        return;
    };
    let hooks = PreferencesHooks {
        start_download: no_download,
        ..file_hooks
    };
    app.insert_resource(hooks);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_names_are_files() {
        assert!(is_valid_file_name("Persona.json"));
        assert!(is_valid_file_name("story-20240101120000"));
        assert!(is_valid_file_name(".hidden"));
        assert!(!is_valid_file_name(""));
        assert!(!is_valid_file_name(".."));
        assert!(!is_valid_file_name("../Options.json"));
        assert!(!is_valid_file_name("a/b"));
        assert!(!is_valid_file_name("a\\b"));
        assert!(!is_valid_file_name("nul\0"));
    }
}
