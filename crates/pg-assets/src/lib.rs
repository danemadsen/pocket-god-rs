//! Every game asset, embedded into the executable at build time.
//!
//! `assets/` at the workspace root holds the converted Pocket God 1.48.2 assets (PNG texture
//! pages, FLAC sounds, the original `Timelines.bin`, story/news XML and the data tables
//! extracted from the original executable). `build.rs` embeds all of it; paths are relative
//! to `assets/` and use `/` separators.
//!
//! [`data`] has the serde types of the JSON tables in `assets/data/` and [`timelines`] parses
//! the original `Timelines.bin`. Both are free of Bevy; only the `bevy` feature adds the
//! asset source.

pub mod data;
pub mod timelines;

#[cfg(feature = "bevy")]
mod bevy_source;

#[cfg(feature = "bevy")]
pub use bevy_source::EmbeddedAssetsPlugin;

include!(concat!(env!("OUT_DIR"), "/embedded_files.rs"));

/// One embedded file.
#[derive(Clone, Copy, Debug)]
pub struct EmbeddedFile {
    pub path: &'static str,
    pub bytes: &'static [u8],
}

/// All embedded files, sorted by path.
pub fn files() -> &'static [EmbeddedFile] {
    FILES
}

/// Returns the bytes of the embedded file at `path` (relative to `assets/`).
pub fn file(path: &str) -> Option<&'static [u8]> {
    let index = FILES.binary_search_by(|entry| entry.path.cmp(path)).ok()?;
    Some(FILES[index].bytes)
}

/// Returns the embedded files directly or indirectly inside `dir` (relative to `assets/`).
pub fn files_in(dir: &str) -> impl Iterator<Item = &'static EmbeddedFile> + '_ {
    let prefix = format!("{}/", dir.trim_end_matches('/'));
    FILES.iter().filter(move |entry| entry.path.starts_with(&prefix))
}
