//! Desktop and web entry point. Android starts from `crates/pocket-god-android` instead.

fn main() {
    let mut app = pocket_god::app();
    app.run();
}
