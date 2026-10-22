//! The launch image (`docs/re/managers-platform.md` §2.2, §18 row 6, `docs/re/rendering.md`
//! §12.3): drawn letterboxed over the game viewport until the game hides its loading screen
//! (`PlatformRequests::loading_screen_hidden`, set by `hideLoadingScreen`).
//!
//! The images are the original's `Default@2x.png`, `Default-568h@2x.png` and
//! `Default-Landscape~ipad.png`, converted from Apple's CgBI PNGs and turned to landscape
//! (`assets/launch/`). The original picks one by device; the port takes the one whose aspect is
//! closest to the viewport's.
//!
//! The original runs `loadGame` from a 0.1 s timer so that UIKit draws the launch image first
//! (`docs/re/managers-platform.md` §2.1, §2.4). Bevy compiles render pipelines asynchronously,
//! so the image may take a few frames to show: the game's start-up runs under the
//! [`launch_image_shown`] condition instead.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pg_engine::platform::PlatformRequests;
use pg_render::{SplashPresented, SplashScreen};

use crate::screen::GameScreen;

/// How long start-up waits at most for the launch image to reach the screen (seconds of real
/// time): a renderer that never draws it must not hold the game back.
pub const LAUNCH_IMAGE_WAIT_LIMIT: f32 = 2.0;

const LAUNCH_PHONE: &[u8] = include_bytes!("../assets/launch/launch-3x2.png");
const LAUNCH_WIDE: &[u8] = include_bytes!("../assets/launch/launch-wide.png");
const LAUNCH_TABLET: &[u8] = include_bytes!("../assets/launch/launch-tall.png");

/// The original's launch images.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchImage {
    /// `Default@2x.png`: 3.5-inch iPhones (3:2).
    Phone,

    /// `Default-568h@2x.png`: 4-inch iPhones (568:320).
    Wide,

    /// `Default-Landscape~ipad.png`: iPads (4:3).
    Tablet,
}

impl LaunchImage {
    const ALL: [Self; 3] = [Self::Phone, Self::Wide, Self::Tablet];

    pub fn aspect(self) -> f32 {
        match self {
            Self::Phone => 3.0 / 2.0,
            Self::Wide => 568.0 / 320.0,
            Self::Tablet => 4.0 / 3.0,
        }
    }

    fn png(self) -> &'static [u8] {
        match self {
            Self::Phone => LAUNCH_PHONE,
            Self::Wide => LAUNCH_WIDE,
            Self::Tablet => LAUNCH_TABLET,
        }
    }

    /// The image whose aspect is closest to `viewport_aspect`.
    pub fn for_aspect(viewport_aspect: f32) -> Self {
        let mut best = Self::Phone;
        for candidate in Self::ALL {
            let distance = (candidate.aspect() - viewport_aspect).abs();
            let best_distance = (best.aspect() - viewport_aspect).abs();
            if distance < best_distance {
                best = candidate;
            }
        }
        best
    }
}

/// Which launch image is up, if any.
#[derive(Resource, Default)]
pub struct SplashState {
    shown: Option<LaunchImage>,
    finished: bool,
}

/// The launch image as a texture: texel values unchanged (non-sRGB, like the game's pages),
/// clamped at its edges.
fn launch_texture(image: LaunchImage) -> Option<(Image, UVec2)> {
    let decoded = pg_render::decode_png(image.png())?;
    let size = UVec2::new(decoded.width, decoded.height);
    let extent = Extent3d {
        width: decoded.width,
        height: decoded.height,
        depth_or_array_layers: 1,
    };
    let mut texture = Image::new(
        extent,
        TextureDimension::D2,
        decoded.pixels,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );

    let sampler = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    };
    texture.sampler = ImageSampler::Descriptor(sampler);
    Some((texture, size))
}

/// The launch image suited to the current viewport.
fn wanted_image(screen: &GameScreen) -> LaunchImage {
    let Some(viewport) = screen.viewport else {
        return LaunchImage::Phone;
    };
    let size = viewport.physical_size.as_vec2();
    LaunchImage::for_aspect(size.x / size.y)
}

fn show(image: LaunchImage, splash: &mut SplashScreen, state: &mut SplashState, images: &mut Assets<Image>) {
    let Some((texture, size)) = launch_texture(image) else {
        warn!("the launch image {image:?} does not decode");
        return;
    };
    // The previous image is freed with its last handle.
    splash.image = Some(images.add(texture));
    splash.size = size;
    state.shown = Some(image);
}

/// Puts the launch image up (`showLoadingImage`), after the screen was configured.
pub fn show_splash(
    screen: Res<GameScreen>,
    mut splash: ResMut<SplashScreen>,
    mut state: ResMut<SplashState>,
    mut images: ResMut<Assets<Image>>,
) {
    let image = wanted_image(&screen);
    show(image, &mut splash, &mut state, &mut images);
}

/// Run condition for start-up work that blocks updates (the game's `loadGame`): the launch
/// image has been rendered, there is none (it did not decode, or the loading screen is gone),
/// or [`LAUNCH_IMAGE_WAIT_LIMIT`] passed.
pub fn launch_image_shown(splash: Res<SplashScreen>, presented: Res<SplashPresented>, time: Res<Time<Real>>) -> bool {
    if splash.image.is_none() || presented.get() {
        return true;
    }
    time.elapsed_secs() >= LAUNCH_IMAGE_WAIT_LIMIT
}

/// Takes the launch image down once the game hid its loading screen (`onHideLoadingScreen`),
/// and swaps it when the window's shape asks for another one.
pub fn update_splash(
    requests: Res<PlatformRequests>,
    screen: Res<GameScreen>,
    mut splash: ResMut<SplashScreen>,
    mut state: ResMut<SplashState>,
    mut images: ResMut<Assets<Image>>,
) {
    if state.finished {
        return;
    }

    if requests.loading_screen_hidden {
        splash.image = None;
        state.finished = true;
        state.shown = None;
        return;
    }

    let wanted = wanted_image(&screen);
    if state.shown != Some(wanted) {
        show(wanted, &mut splash, &mut state, &mut images);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_device_shape_gets_its_launch_image() {
        assert_eq!(LaunchImage::for_aspect(1.5), LaunchImage::Phone);
        assert_eq!(LaunchImage::for_aspect(1.6), LaunchImage::Phone);
        assert_eq!(LaunchImage::for_aspect(1.7), LaunchImage::Wide);
        assert_eq!(LaunchImage::for_aspect(1.775), LaunchImage::Wide);
        assert_eq!(LaunchImage::for_aspect(1.4), LaunchImage::Tablet);
        assert_eq!(LaunchImage::for_aspect(4.0 / 3.0), LaunchImage::Tablet);
    }

    fn launch_image_shown_in(world: &mut World) -> bool {
        world.run_system_cached(launch_image_shown).expect("runs")
    }

    #[test]
    fn start_up_waits_for_the_launch_image_to_be_rendered() {
        let mut world = World::new();
        world.insert_resource(Time::<Real>::default());
        world.init_resource::<SplashScreen>();
        let presented = SplashPresented::default();
        world.insert_resource(presented.clone());

        // No launch image (it did not decode, or it is gone): nothing to wait for.
        assert!(launch_image_shown_in(&mut world));

        let mut images = Assets::<Image>::default();
        let (texture, size) = launch_texture(LaunchImage::Phone).expect("embedded PNG");
        let mut splash = world.resource_mut::<SplashScreen>();
        splash.image = Some(images.add(texture));
        splash.size = size;
        assert!(!launch_image_shown_in(&mut world));

        // The render world drew it.
        presented.set(true);
        assert!(launch_image_shown_in(&mut world));
    }

    #[test]
    fn a_launch_image_that_never_renders_holds_start_up_back_only_so_long() {
        let mut world = World::new();
        world.init_resource::<SplashPresented>();
        let mut images = Assets::<Image>::default();
        let (texture, size) = launch_texture(LaunchImage::Wide).expect("embedded PNG");
        let splash = SplashScreen {
            image: Some(images.add(texture)),
            size,
        };
        world.insert_resource(splash);

        // The first update starts the clock.
        let mut time = Time::<Real>::default();
        time.update_with_duration(std::time::Duration::ZERO);
        let just_before = std::time::Duration::from_secs_f32(LAUNCH_IMAGE_WAIT_LIMIT - 0.1);
        time.update_with_duration(just_before);
        world.insert_resource(time);
        assert!(!launch_image_shown_in(&mut world));

        let later = std::time::Duration::from_secs_f32(0.2);
        world.resource_mut::<Time<Real>>().update_with_duration(later);
        assert!(launch_image_shown_in(&mut world));
    }

    #[test]
    fn the_launch_images_are_landscape_at_their_device_aspect() {
        for image in LaunchImage::ALL {
            let (texture, size) = launch_texture(image).expect("embedded PNG");
            let aspect = size.x as f32 / size.y as f32;
            assert!((aspect - image.aspect()).abs() < 0.01, "{image:?} {size}");
            assert_eq!(texture.texture_descriptor.format, TextureFormat::Rgba8Unorm);
        }
    }
}
