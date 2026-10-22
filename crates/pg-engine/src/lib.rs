//! Port of Bolt Creative's "BC" engine (BoltLib), the engine Pocket God is built on.
//!
//! The engine is platform independent. Each frame it produces a [`draw::DrawList`] (already
//! projected into clip space, exactly like the original OpenGL ES 1.1 pipeline), texture
//! requests ([`texture::Textures`]) and sound commands ([`sound::Sounds`]); the Bevy
//! integration crates consume them. See `docs/ARCHITECTURE.md` for the object model.

pub mod animation;
pub mod display;
pub mod draw;
pub mod gl;
pub mod hooks;
pub mod library;
pub mod logic;
pub mod math;
pub mod object;
pub mod platform;
pub mod prefs;
pub mod prelude;
pub mod random;
pub mod sequence;
pub mod sound;
pub mod texture;
pub mod touch;
pub mod view;

use bevy_app::{App, Plugin};

/// Registers the engine resources.
pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<library::Library>();
        app.init_resource::<texture::Textures>();
        app.init_resource::<animation::Timelines>();
        app.init_resource::<sound::Sounds>();
        app.init_resource::<view::BcView>();
        app.init_resource::<view::MotionInput>();
        app.init_resource::<touch::PlatformTouches>();
        app.init_resource::<hooks::EngineHooks>();
        app.init_resource::<random::Random>();
        app.init_resource::<platform::PlatformRequests>();
        app.init_resource::<platform::Locale>();
        app.init_resource::<platform::TextField>();
    }
}
