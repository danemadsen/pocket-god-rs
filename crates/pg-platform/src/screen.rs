//! The game viewport inside the window (`docs/re/rendering.md` §1.4–§1.7).
//!
//! The original only ran on 3:2 iPhones, 568:320 iPhones and 4:3 iPads, so the window aspect
//! is clamped to [4:3, 568:320] and the rest of the window stays black. The viewport goes to
//! the engine (`BCView`'s renderbuffer in pixels and its view in points, from which it derives
//! the logical screen and the wide / tall flags) and, as the same pixel rect, to the
//! renderer's [`LogicalScreen`], so drawing, touches and letterboxing always agree.
//!
//! Point sprites (locusts) are the one thing the original sized in renderbuffer pixels, and its
//! renderbuffer was in points at scale 1 (`docs/re/rendering.md` §1.2, §1.5, §8.3): 16 pixels
//! were 16 game units on every iPhone and 7.5 on the iPad. The engine's point-sprite scale is
//! therefore set from the renderbuffer of the iOS device of the viewport's shape
//! ([`device_renderbuffer_width`]), not from the window's pixels.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use pg_engine::library::Library;
use pg_engine::texture::{self, Textures};
use pg_engine::view::{self, BcView};
use pg_render::LogicalScreen;

/// The narrowest aspect the original supports: the iPad's 4:3.
pub const MIN_ASPECT: f64 = 4.0 / 3.0;

/// The widest: the 4-inch iPhone's 568 × 320 points.
pub const MAX_ASPECT: f64 = 568.0 / 320.0;

/// The slot `loadGame` fills with the overlay bands of wide and tall screens, and its image.
const OVERLAY_ATLAS: &str = "widescreenOverlayAtlas0";
const OVERLAY_IMAGE: &str = "WidescreenOverlayAtlas0";

/// The 3.5-inch iPhone's aspect: the narrowest screen of the iPhone class.
const IPHONE_ASPECT: f32 = 1.5;

/// Every iPhone's renderbuffer height (pixels at scale 1).
const IPHONE_RENDERBUFFER_HEIGHT: f32 = 320.0;

/// The iPad's aspect and its renderbuffer width (1024 × 768 pixels at scale 1).
const IPAD_ASPECT: f32 = 4.0 / 3.0;
const IPAD_RENDERBUFFER_WIDTH: f32 = 1024.0;

/// The width of the original's renderbuffer on an iOS device of `aspect`: 320 pixels high on
/// every iPhone (480 wide at 3:2, 568 at 568:320), 1024 wide on the iPad (4:3). The port's
/// windows between 4:3 and 3:2 have no device; their width blends linearly from the iPad's to
/// the 3.5-inch iPhone's, so point sprites grow smoothly while a window is resized.
pub fn device_renderbuffer_width(aspect: f32) -> f32 {
    if aspect >= IPHONE_ASPECT {
        return IPHONE_RENDERBUFFER_HEIGHT * aspect;
    }

    let iphone_width = IPHONE_RENDERBUFFER_HEIGHT * IPHONE_ASPECT;
    let toward_iphone = (aspect - IPAD_ASPECT) / (IPHONE_ASPECT - IPAD_ASPECT);
    let blend = toward_iphone.clamp(0.0, 1.0);
    IPAD_RENDERBUFFER_WIDTH + (iphone_width - IPAD_RENDERBUFFER_WIDTH) * blend
}

/// Game units per pixel of point sprites on a logical screen of `screen_width ×
/// screen_height` (`W_L × H_L`): 1 on every iPhone shape, 480 / 1024 on the iPad's.
pub fn point_sprite_units_per_pixel(screen_width: f32, screen_height: f32) -> f32 {
    let aspect = screen_width / screen_height;
    let renderbuffer_width = device_renderbuffer_width(aspect);
    screen_width / renderbuffer_width
}

/// The rect of the window the game draws into and takes touches from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameViewport {
    /// Top-left corner and size in physical pixels.
    pub physical_origin: UVec2,
    pub physical_size: UVec2,

    /// The same rect in logical pixels (the window's points).
    pub logical_origin: Vec2,
    pub logical_size: Vec2,
}

impl GameViewport {
    /// The largest rect of the window's aspect clamped to [`MIN_ASPECT`, `MAX_ASPECT`], centred
    /// in a window of `window_size` physical pixels; `None` for an empty window.
    pub fn fit(window_size: UVec2, scale_factor: f32) -> Option<Self> {
        // NaN fails the comparison too.
        let valid_scale = scale_factor > 0.0;
        if window_size.x == 0 || window_size.y == 0 || !valid_scale {
            return None;
        }

        let window_width = f64::from(window_size.x);
        let window_height = f64::from(window_size.y);
        let aspect = window_width / window_height;

        let mut size = window_size;
        if aspect > MAX_ASPECT {
            let width = (window_height * MAX_ASPECT).round() as u32;
            size.x = width.clamp(1, window_size.x);
        } else if aspect < MIN_ASPECT {
            let height = (window_width / MIN_ASPECT).round() as u32;
            size.y = height.clamp(1, window_size.y);
        }

        let physical_origin = (window_size - size) / 2;
        let logical_origin = physical_origin.as_vec2() / scale_factor;
        let logical_size = size.as_vec2() / scale_factor;

        let viewport = Self {
            physical_origin,
            physical_size: size,
            logical_origin,
            logical_size,
        };
        Some(viewport)
    }

    /// The rect in physical pixels.
    pub fn physical_rect(&self) -> URect {
        let max = self.physical_origin + self.physical_size;
        URect::from_corners(self.physical_origin, max)
    }

    /// A window position (logical pixels) in view points: origin at the viewport's top-left,
    /// y down, the viewport `logical_size` points large.
    pub fn view_point(&self, window_position: Vec2) -> Vec2 {
        window_position - self.logical_origin
    }

    /// Whether a view point lies on the game.
    pub fn contains_view_point(&self, point: Vec2) -> bool {
        let inside_x = point.x >= 0.0 && point.x < self.logical_size.x;
        let inside_y = point.y >= 0.0 && point.y < self.logical_size.y;
        inside_x && inside_y
    }
}

/// The viewport the engine and the renderer are configured with.
#[derive(Resource, Default)]
pub struct GameScreen {
    pub viewport: Option<GameViewport>,

    /// The screen changed since the frame driver last looked: the engine's draw list was built
    /// for the old one.
    redraw: bool,
}

impl GameScreen {
    /// Notes that the engine's draw list was built for an old screen.
    pub(crate) fn request_redraw(&mut self) {
        self.redraw = true;
    }

    /// Whether the screen changed since the last call.
    pub(crate) fn take_redraw(&mut self) -> bool {
        std::mem::take(&mut self.redraw)
    }
}

/// The primary window's viewport, if there is a window with a size.
fn window_viewport(world: &mut World) -> Option<GameViewport> {
    let mut windows = world.query_filtered::<&Window, With<PrimaryWindow>>();
    let window = windows.single(world).ok()?;
    let physical_size = window.physical_size();
    let scale_factor = window.scale_factor();
    GameViewport::fit(physical_size, scale_factor)
}

/// The overlay atlas for a screen that became wide or tall after `loadGame` (a resized
/// window; the original's screen never changes): `loadGame` only fills the slot when the
/// screen is wide or tall at load time, and the overlay groups draw from it.
fn load_overlay_atlas_if_needed(world: &mut World) {
    let letterboxed = {
        let view = world.resource::<BcView>();
        view.wide_screen || view.tall_screen
    };
    if !letterboxed {
        return;
    }

    let atlas = world.resource::<Library>().atlas_id(OVERLAY_ATLAS);
    let Some(atlas) = atlas else {
        // The game data is not loaded yet; `loadGame` will look at the flags itself.
        return;
    };
    let loaded = world.resource::<Textures>().atlas(atlas).image.is_some();
    if !loaded {
        texture::load_atlas(world, atlas, OVERLAY_IMAGE);
    }
}

/// `BCView::setFrustumHorizontalShift:verticalShift:` for the viewport, the point-sprite scale
/// of the matching iOS device, and the same rect for the renderer.
fn apply_viewport(world: &mut World, viewport: GameViewport) {
    let view_width = viewport.physical_size.x as i32;
    let view_height = viewport.physical_size.y as i32;
    let touch_size = viewport.logical_size;
    view::configure_screen(world, view_width, view_height, touch_size.x, touch_size.y);

    let (screen_width, screen_height) = {
        let view = world.resource::<BcView>();
        (view.screen_width, view.screen_height)
    };

    let units_per_pixel = point_sprite_units_per_pixel(screen_width, screen_height);
    let mut view = world.resource_mut::<BcView>();
    if let Some(gl) = view.gl.as_mut() {
        gl.units_per_pixel = units_per_pixel;
    }

    let logical_screen = LogicalScreen {
        width: screen_width,
        height: screen_height,
        viewport: Some(viewport.physical_rect()),
    };
    world.insert_resource(logical_screen);
    world.resource_mut::<GameScreen>().viewport = Some(viewport);

    load_overlay_atlas_if_needed(world);
}

/// Configures the screen from the window before any game data loads (`PreStartup`).
pub fn configure_screen_at_startup(world: &mut World) {
    let Some(viewport) = window_viewport(world) else {
        warn!("no primary window with a size at start-up; the engine keeps its 480x320 screen");
        return;
    };
    apply_viewport(world, viewport);
}

/// Reconfigures the screen whenever the window's viewport changes, and has the game drawn for
/// the new screen.
pub fn update_screen(world: &mut World) {
    let Some(viewport) = window_viewport(world) else {
        return;
    };
    let current = world.resource::<GameScreen>().viewport;
    if current == Some(viewport) {
        return;
    }
    apply_viewport(world, viewport);
    world.resource_mut::<GameScreen>().request_redraw();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fit(width: u32, height: u32, scale: f32) -> GameViewport {
        let size = UVec2::new(width, height);
        GameViewport::fit(size, scale).expect("a window with a size")
    }

    #[test]
    fn supported_aspects_fill_the_window() {
        let iphone = fit(960, 640, 2.0);
        assert_eq!(iphone.physical_origin, UVec2::ZERO);
        assert_eq!(iphone.physical_size, UVec2::new(960, 640));
        assert_eq!(iphone.logical_size, Vec2::new(480.0, 320.0));

        let iphone5 = fit(1136, 640, 2.0);
        assert_eq!(iphone5.physical_size, UVec2::new(1136, 640));

        let ipad = fit(2048, 1536, 2.0);
        assert_eq!(ipad.physical_size, UVec2::new(2048, 1536));
        assert_eq!(ipad.logical_size, Vec2::new(1024.0, 768.0));
    }

    #[test]
    fn wider_windows_are_pillarboxed_at_568_by_320() {
        let tv = fit(1920, 1080, 1.0);
        assert_eq!(tv.physical_size, UVec2::new(1917, 1080));
        assert_eq!(tv.physical_origin, UVec2::new(1, 0));

        let ultrawide = fit(2000, 1000, 2.0);
        assert_eq!(ultrawide.physical_size, UVec2::new(1775, 1000));
        assert_eq!(ultrawide.physical_origin, UVec2::new(112, 0));
        assert_eq!(ultrawide.logical_origin, Vec2::new(56.0, 0.0));
        assert_eq!(ultrawide.logical_size, Vec2::new(887.5, 500.0));
    }

    #[test]
    fn taller_windows_are_letterboxed_at_4_by_3() {
        let square = fit(1000, 1000, 1.0);
        assert_eq!(square.physical_size, UVec2::new(1000, 750));
        assert_eq!(square.physical_origin, UVec2::new(0, 125));

        let portrait = fit(1080, 2400, 3.0);
        assert_eq!(portrait.physical_size, UVec2::new(1080, 810));
        assert_eq!(portrait.physical_origin, UVec2::new(0, 795));
        assert_eq!(portrait.logical_size, Vec2::new(360.0, 270.0));
    }

    #[test]
    fn empty_windows_have_no_viewport() {
        assert_eq!(GameViewport::fit(UVec2::new(0, 600), 1.0), None);
        assert_eq!(GameViewport::fit(UVec2::new(800, 0), 1.0), None);
        assert_eq!(GameViewport::fit(UVec2::new(800, 600), 0.0), None);
    }

    #[test]
    fn view_points_start_at_the_viewport_corner() {
        let viewport = fit(2000, 1000, 2.0);
        let corner = viewport.view_point(Vec2::new(56.0, 0.0));
        assert_eq!(corner, Vec2::ZERO);

        let inside = viewport.view_point(Vec2::new(500.0, 250.0));
        assert_eq!(inside, Vec2::new(444.0, 250.0));
        assert!(viewport.contains_view_point(inside));

        let left_bar = viewport.view_point(Vec2::new(20.0, 250.0));
        assert!(!viewport.contains_view_point(left_bar));
        let right_edge = viewport.view_point(Vec2::new(56.0 + 887.5, 10.0));
        assert!(!viewport.contains_view_point(right_edge));
    }

    #[test]
    fn the_engine_screen_follows_the_clamped_viewport() {
        let mut world = World::new();
        world.init_resource::<BcView>();
        world.init_resource::<Library>();
        world.init_resource::<Textures>();
        world.init_resource::<GameScreen>();

        let wide = fit(2000, 1000, 2.0);
        apply_viewport(&mut world, wide);
        let view = world.resource::<BcView>();
        assert!(view.wide_screen && !view.tall_screen);
        assert_eq!(view.screen_height, 320.0);
        assert!((view.screen_width - 568.0).abs() < 0.2);
        assert_eq!(view.touch_width, 887.5);
        assert_eq!(point_sprite_scale(&world), 1.0);

        let screen = *world.resource::<LogicalScreen>();
        assert_eq!(screen.viewport, Some(URect::new(112, 0, 1887, 1000)));
        assert_eq!(screen.width, view.screen_width);

        let tall = fit(1000, 1000, 1.0);
        apply_viewport(&mut world, tall);
        let view = world.resource::<BcView>();
        assert!(view.tall_screen && !view.wide_screen);
        assert_eq!(view.screen_width, 480.0);
        assert!((view.screen_height - 360.0).abs() < 0.01);
        assert!((point_sprite_scale(&world) - 480.0 / 1024.0).abs() < 1e-6);

        // A Retina iPhone-shaped window: 16 sprite pixels are 16 game units, not 8.
        let retina = fit(1920, 1280, 2.0);
        apply_viewport(&mut world, retina);
        assert_eq!(point_sprite_scale(&world), 1.0);
    }

    fn point_sprite_scale(world: &World) -> f32 {
        let view = world.resource::<BcView>();
        let gl = view.gl.as_ref().expect("the GL context is idle");
        gl.units_per_pixel
    }

    #[test]
    fn point_sprites_keep_the_size_of_the_matching_device() {
        assert_eq!(device_renderbuffer_width(1.5), 480.0);
        assert_eq!(device_renderbuffer_width(568.0 / 320.0), 568.0);
        assert_eq!(device_renderbuffer_width(4.0 / 3.0), 1024.0);

        // 16 pixels: 16 game units on the iPhones, 7.5 on the iPad.
        assert_eq!(16.0 * point_sprite_units_per_pixel(480.0, 320.0), 16.0);
        assert_eq!(16.0 * point_sprite_units_per_pixel(568.0, 320.0), 16.0);
        let ipad = 16.0 * point_sprite_units_per_pixel(480.0, 360.0);
        assert!((ipad - 7.5).abs() < 1e-4, "{ipad}");

        // In between, the size grows steadily toward the iPhone's.
        let mut previous = ipad;
        for step in 1..=10 {
            let aspect = 4.0 / 3.0 + (1.5 - 4.0 / 3.0) * step as f32 / 10.0;
            let size = 16.0 * point_sprite_units_per_pixel(480.0, 480.0 / aspect);
            assert!(size > previous, "{aspect}: {size} after {previous}");
            previous = size;
        }
        assert!((previous - 16.0).abs() < 1e-3);
    }
}
