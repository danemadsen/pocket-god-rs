//! Android entry point. `GameActivity` loads this library (`android.app.lib_name` in
//! `android/app/src/main/AndroidManifest.xml`) and calls the `android_main` that
//! `#[bevy_main]` generates around `main`.

use bevy::prelude::*;

#[bevy_main]
fn main() {
    let mut app = pocket_god::app();
    app.run();
}
