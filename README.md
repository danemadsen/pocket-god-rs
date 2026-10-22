# Pocket God

A Rust and Bevy port of Bolt Creative's Pocket God 1.48.2 for iOS. See
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Building

Every build embeds everything under `assets/` in the program. The scripts work from any
directory; `cargo run -p pocket-god` is the quickest way to run the desktop version.

| Platform | Command | Output |
|---|---|---|
| Desktop | `scripts/build-desktop.sh` (`--run` to start it) | `target/release/pocket-god` |
| Web | `scripts/build-web.sh` (`--serve` to serve it at http://localhost:8080) | `web/`, with the generated `web/pkg/` |
| Android | `scripts/build-android.sh` (`--release` for a signed release APK, `--install` to install and launch it with adb) | `android/app/build/outputs/apk/{debug,release}/app-*.apk` |

Besides the toolchain from `rust-toolchain.toml`:

- **Web:** `rustup target add wasm32-unknown-unknown` and
  `cargo install wasm-bindgen-cli --version <wasm-bindgen version in Cargo.lock>`. Any static
  file server works; browsers need WebGL2.
- **Android:** `rustup target add aarch64-linux-android` (and `x86_64-linux-android` for
  `ANDROID_ABIS="arm64-v8a x86_64"`), `cargo install cargo-ndk`, a JDK 17+ and
  `ANDROID_HOME` pointing at an SDK with platform 36 and an NDK. Gradle comes from the
  wrapper in `android/`. Release builds are signed with the key named in
  `android/keystore.properties`, which stays out of git: copy
  `android/keystore.properties.example` and point it at your keystore and its passwords file.

## Playing

The game keeps the original's landscape screen. Windows wider than 568:320 (the 4-inch
iPhone) or taller than 4:3 (the iPad) are letterboxed in black; in between, the game uses the
wide and tall layouts of the original.

- **Touch:** every finger is a finger of the original. With a mouse, the left button is one
  finger.
- **Tilt and shake:** phones and tablets use their accelerometer (Android, and mobile browsers;
  iOS Safari asks for permission on the first touch). On desktop, and in browsers without
  motion sensors, the keyboard tilts an imaginary phone held upright in landscape:

  | Key | Motion |
  |---|---|
  | ← / → | tilt it like a steering wheel by 30° (left / right edge down) |
  | ↑ / ↓ | lean its top edge away from / toward you by 30° |
  | Shift + arrow | tilt by 70° instead (things slide off) |
  | F | turn it upside down |
  | S | shake it (earthquake) |

  The phone turns at 240°/s while a key is held and straightens up when it is released. Keys
  typed into the game's text fields do not tilt it.
- **Pausing:** like the original, the game pauses while its window is unfocused or hidden, and
  while the app is in the background. It also falls silent while the window is hidden
  (minimised, or a hidden browser tab) and while the app is in the background; a window that
  is only unfocused keeps its sound, as an inactive iPhone app does.

Saved games (the original's `Documents` and `Caches` folders) live in
`~/Library/Application Support/PocketGod` (macOS), `%APPDATA%\PocketGod` (Windows),
`$XDG_DATA_HOME/PocketGod` or `~/.local/share/PocketGod` (Linux), the app's internal storage
(Android), or the browser's `localStorage` (web).

Desktop environment variables:

| Variable | Effect |
|---|---|
| `PG_SCREENSHOT=<file.png>` | saves a screenshot after about a second and exits; the window stays on top and never pauses |
| `PG_SCREENSHOT_FRAME=<n>` | with `PG_SCREENSHOT`: the frame to capture (default 60) |
| `PG_TOUCHES="tap <frame> <x> <y>; drag <frame> <x1> <y1> <x2> <y2> <frames>"` | scripted touches in the original's screen points (480 × 320 on a 3:2 window) |
| `PG_WINDOW_SIZE=<width>x<height>` | initial window size in points, e.g. `1136x640` or `1024x768` |
| `PG_DATA_DIR=<dir>` | keeps the saved games in `<dir>` instead |
