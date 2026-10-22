//! Generates the embedded file table for everything under the workspace `assets/` directory.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_hidden = entry.file_name().to_string_lossy().starts_with('.');
        if is_hidden {
            continue;
        }

        if path.is_dir() {
            collect_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
    let assets_dir = manifest_dir.join("../../assets");
    let assets_dir = assets_dir.canonicalize().unwrap_or(assets_dir);
    println!("cargo:rerun-if-changed={}", assets_dir.display());

    let mut files = Vec::new();
    collect_files(&assets_dir, &mut files);

    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&assets_dir).expect("file lies under the assets directory");
        let asset_path = relative.to_string_lossy().replace('\\', "/");
        entries.push((asset_path, file));
    }
    entries.sort();

    let mut source = String::from("pub(crate) static FILES: &[crate::EmbeddedFile] = &[\n");
    for (asset_path, file) in &entries {
        writeln!(
            source,
            "    crate::EmbeddedFile {{ path: {asset_path:?}, bytes: include_bytes!({:?}) }},",
            file.display().to_string()
        )
        .expect("writing to a String cannot fail");
    }
    source.push_str("];\n");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    fs::write(out_dir.join("embedded_files.rs"), source).expect("OUT_DIR is writable");
}
