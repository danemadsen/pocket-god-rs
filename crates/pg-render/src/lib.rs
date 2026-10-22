//! Renders [`pg_engine::draw::DrawList`]s with Bevy.
//!
//! Each frame, after the engine ticked, the bridge applies the engine's texture requests
//! ([`pg_engine::texture::Textures::requests`]) to the [`TexturePages`] and, whenever `BCView`
//! changed (a tick, or the game drawn again for a new screen), copies the draw list of its GL
//! context into [`FrameDrawList`] (when there is no engine, whoever fills [`FrameDrawList`]
//! drives the renderer). The frame is drawn into the letterboxed game viewport
//! ([`LogicalScreen`]): the clear colour, the draw list's batches in order, then the
//! [`SplashScreen`] when one is up.

mod batches;
mod material;
mod pages;
mod presentation;

use bevy::asset::{AssetEventSystems, RenderAssetUsages, embedded_asset};
use bevy::camera::{CameraUpdateSystems, CompositingSpace, Hdr, RenderTarget, Viewport};
use bevy::prelude::*;
use bevy::render::render_resource::{DownlevelFlags, Extent3d, TextureDimension, TextureFormat};
use bevy::render::renderer::RenderAdapter;
use bevy::render::view::Tonemapping;
use bevy::sprite_render::Material2dPlugin;
use bevy::transform::TransformSystems;
use bevy::window::{PrimaryWindow, WindowRef};
use pg_engine::draw::DrawList;
use pg_engine::view::BcView;

pub use material::{ATTRIBUTE_CLIP_POSITION, BcMaterial};
pub use presentation::SplashPresented;
pub use pages::{
    DecodedImage, RectCopy, SourceImages, TexturePages, build_mip_chain, copy_rect, decode_png, decode_texture_image,
    fit_to_slot, mip_level_count, page_image,
};

pub struct PgRenderPlugin;

/// The renderer's systems in `PostUpdate`, after the engine ticked in `Update` and before
/// Bevy turns their changes into asset events, global transforms, visibility and camera data,
/// so a frame reaches the GPU whole in that frame: a batch mesh edited after the asset events
/// would be drawn a frame late, with its new material and depth.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderSystems {
    /// Texture requests and the draw-list copy from the engine.
    EngineBridge,

    /// Batches, splash and camera viewport.
    Draw,
}

impl Plugin for PgRenderPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "bc.wgsl");

        app.add_plugins(Material2dPlugin::<BcMaterial>::default());
        app.init_resource::<FrameDrawList>();
        app.init_resource::<TexturePages>();
        app.init_resource::<SourceImages>();
        app.init_resource::<LogicalScreen>();
        app.init_resource::<SplashScreen>();
        app.init_resource::<batches::BatchPool>();
        presentation::build(app);

        let sets = (RenderSystems::EngineBridge, RenderSystems::Draw).chain();
        app.configure_sets(PostUpdate, sets);
        let before_bevy = RenderSystems::Draw
            .before(AssetEventSystems)
            .before(TransformSystems::Propagate)
            .before(CameraUpdateSystems);
        app.configure_sets(PostUpdate, before_bevy);
        app.add_systems(Startup, setup_render);

        let bridge = (pages::apply_texture_requests, copy_engine_draw_list).chain();
        app.add_systems(PostUpdate, bridge.in_set(RenderSystems::EngineBridge));

        let draw = (batches::sync_batches, fit_viewport).chain();
        app.add_systems(PostUpdate, draw.in_set(RenderSystems::Draw));
    }
}

/// The draw list for the current frame (a copy of the engine's latest tick).
#[derive(Resource, Default)]
pub struct FrameDrawList(pub DrawList);

/// The game screen inside the window.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct LogicalScreen {
    /// The logical screen in game units (`BCView`'s `mScreenWidth × mScreenHeight`, e.g. 480 × 320).
    pub width: f32,
    pub height: f32,

    /// The game viewport in physical window pixels, as the platform layer letterboxes it.
    /// `None` centres the largest rect of `width : height` in the window.
    pub viewport: Option<URect>,
}

impl Default for LogicalScreen {
    fn default() -> Self {
        Self {
            width: 480.0,
            height: 320.0,
            viewport: None,
        }
    }
}

/// An image drawn over the whole game viewport, letterboxed in black where its aspect ratio
/// differs (the launch image until the game hides its loading screen). [`SplashPresented`]
/// tells when it reached the screen.
#[derive(Resource, Clone, Debug, Default)]
pub struct SplashScreen {
    pub image: Option<Handle<Image>>,

    /// The image's size in pixels.
    pub size: UVec2,
}

#[derive(Component)]
struct GameCamera;

/// Whether the GPU can create textures with extra view formats. Bevy 0.20 gives the
/// `Rgba8Unorm` main texture of a [`CompositingSpace::Srgb`] camera an sRGB view format, which
/// WebGL2 (and GLES without texture views) cannot create.
fn supports_view_formats(adapter: Option<Res<RenderAdapter>>) -> bool {
    let Some(adapter) = adapter else {
        return true;
    };

    let downlevel = adapter.get_downlevel_capabilities();
    downlevel.flags.contains(DownlevelFlags::VIEW_FORMATS)
}

fn setup_render(
    mut commands: Commands,
    adapter: Option<Res<RenderAdapter>>,
    mut images: ResMut<Assets<Image>>,
    mut pages: ResMut<TexturePages>,
) {
    // Untextured draws (`glDisable(GL_TEXTURE_2D)`) sample this white texel.
    let pixel_size = Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    let white_texel = vec![255, 255, 255, 255];
    let white_pixel = Image::new(
        pixel_size,
        TextureDimension::D2,
        white_texel,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    pages.white = images.add(white_pixel);

    // The camera clears the whole window: black letterbox bars around the game viewport.
    let camera = Camera {
        clear_color: ClearColorConfig::Custom(Color::BLACK),
        ..default()
    };
    commands.insert_resource(ClearColor(Color::BLACK));
    let mut game_camera = commands.spawn((Camera2d, camera, CompositingSpace::Srgb, GameCamera));

    // Without view formats, an HDR (`Rgba16Float`) main texture holds the same gamma-space
    // values; `Tonemapping::None` keeps them unchanged on the way to the screen.
    if !supports_view_formats(adapter) {
        game_camera.insert((Hdr, Tonemapping::None));
    }
}

/// Copies the engine's latest draw list whenever `BCView` changed: every tick draws, and so does
/// the platform when the screen changes between ticks.
fn copy_engine_draw_list(view: Option<Res<BcView>>, mut frame: ResMut<FrameDrawList>) {
    let Some(view) = view else {
        return;
    };
    if !view.is_changed() {
        return;
    }
    let Some(gl) = view.gl.as_ref() else {
        return;
    };

    frame.0.clone_from(&gl.list);
}

/// The largest rect of the logical screen's aspect, centred in the window.
fn centred_viewport(window_size: UVec2, screen: &LogicalScreen) -> URect {
    let window = window_size.as_vec2();
    let scale = (window.x / screen.width).min(window.y / screen.height);
    let scaled = Vec2::new(screen.width, screen.height) * scale;
    let largest = window_size.max(UVec2::ONE);
    let rounded = scaled.round().as_uvec2();
    let size = rounded.clamp(UVec2::ONE, largest);

    let min = window_size.saturating_sub(size) / 2;
    URect::from_corners(min, min + size)
}

/// The camera viewport: the platform's letterboxed rect, kept inside the window.
fn viewport_rect(window_size: UVec2, screen: &LogicalScreen) -> URect {
    let requested = match screen.viewport {
        Some(rect) => rect,
        None => centred_viewport(window_size, screen),
    };

    let last_pixel = window_size.saturating_sub(UVec2::ONE);
    let min = requested.min.min(last_pixel);
    let inside = requested.max.min(window_size);
    let max = inside.max(min + UVec2::ONE);
    URect::from_corners(min, max)
}

/// The cameras other than the game camera (the egui overlay's), with what they draw into.
type OtherCameras<'world, 'state> = Query<
    'world,
    'state,
    (&'static RenderTarget, &'static mut Projection),
    (With<Camera>, Without<GameCamera>),
>;

/// Whether a camera with `target` draws into the window `window`, the primary one.
fn renders_to_window(target: &RenderTarget, window: Entity) -> bool {
    match target {
        RenderTarget::Window(WindowRef::Primary) => true,
        RenderTarget::Window(WindowRef::Entity(target_window)) => *target_window == window,
        _ => false,
    }
}

/// Letterboxes the camera to the game viewport.
///
/// A new viewport size makes Bevy recompute the camera's render-target size from the window
/// at once, while the window's other cameras (the egui overlay) wait for the window's resize
/// message, which a window resized by the app itself only sends a frame later. The 2D cameras
/// of a window share one depth texture, so a frame in which their target sizes differ fails
/// validation (and Bevy quits): the window's other cameras are updated in the same frame.
fn fit_viewport(
    screen: Res<LogicalScreen>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<GameCamera>>,
    mut other_cameras: OtherCameras,
) {
    let Ok((window_entity, window)) = windows.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    let window_size = window.physical_size();
    if window_size.x == 0 || window_size.y == 0 {
        return;
    }

    let rect = viewport_rect(window_size, &screen);
    let physical_position = rect.min;
    let physical_size = rect.size();
    if let Some(current) = camera.viewport.as_ref() {
        let unchanged = current.physical_position == physical_position && current.physical_size == physical_size;
        if unchanged {
            return;
        }
    }

    let viewport = Viewport {
        physical_position,
        physical_size,
        ..default()
    };
    camera.viewport = Some(viewport);

    // A changed projection makes Bevy recompute the camera's target size.
    for (target, mut projection) in &mut other_cameras {
        if renders_to_window(target, window_entity) {
            projection.set_changed();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use bevy::render::camera::camera_system;
    use bevy::render::texture::ManualTextureViews;
    use bevy::window::{WindowCreated, WindowResized, WindowResolution, WindowScaleFactorChanged};

    use super::*;

    #[test]
    fn platform_viewports_are_used_as_given_inside_the_window() {
        let screen = LogicalScreen {
            width: 568.0,
            height: 320.0,
            viewport: Some(URect::new(1, 0, 1918, 1080)),
        };
        let rect = viewport_rect(UVec2::new(1920, 1080), &screen);
        assert_eq!(rect, URect::new(1, 0, 1918, 1080));

        let shrunk = viewport_rect(UVec2::new(800, 600), &screen);
        assert_eq!(shrunk, URect::new(1, 0, 800, 600));
    }

    fn set_engine_clear_color(world: &mut World, color: [f32; 4]) {
        let mut view = world.resource_mut::<BcView>();
        let gl = view.gl.as_mut().expect("the GL context is idle");
        gl.list.clear_color = color;
    }

    fn frame_clear_color(world: &World) -> [f32; 4] {
        world.resource::<FrameDrawList>().0.clear_color
    }

    #[test]
    fn the_draw_list_is_copied_whenever_the_view_changed() {
        let mut world = World::new();
        world.init_resource::<FrameDrawList>();
        world.insert_resource(BcView::default());

        set_engine_clear_color(&mut world, [1.0, 0.0, 0.0, 1.0]);
        world.run_system_cached(copy_engine_draw_list).expect("runs");
        assert_eq!(frame_clear_color(&world), [1.0, 0.0, 0.0, 1.0]);

        // An untouched view is not copied again.
        world.resource_mut::<FrameDrawList>().0.clear_color = [0.0; 4];
        world.run_system_cached(copy_engine_draw_list).expect("runs");
        assert_eq!(frame_clear_color(&world), [0.0; 4]);

        // A redraw without a tick (same frame count) is copied.
        set_engine_clear_color(&mut world, [0.0, 1.0, 0.0, 1.0]);
        world.run_system_cached(copy_engine_draw_list).expect("runs");
        assert_eq!(frame_clear_color(&world), [0.0, 1.0, 0.0, 1.0]);
    }

    /// pg-render's viewport, then Bevy's camera update, as in `PostUpdate`.
    fn update_cameras(world: &mut World) {
        world.run_system_cached(fit_viewport).expect("runs");
        let updated: Result<(), BevyError> = world.run_system_cached(camera_system).expect("runs");
        updated.expect("every camera target resolves");
    }

    fn target_size(world: &World, camera: Entity) -> Option<UVec2> {
        let camera = world.get::<Camera>(camera)?;
        camera.physical_target_size()
    }

    #[test]
    fn every_camera_of_the_window_takes_a_new_window_size_in_the_same_frame() {
        let mut world = World::new();
        world.init_resource::<Messages<WindowCreated>>();
        world.init_resource::<Messages<WindowResized>>();
        world.init_resource::<Messages<WindowScaleFactorChanged>>();
        world.init_resource::<Messages<AssetEvent<Image>>>();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<ManualTextureViews>();
        world.init_resource::<LogicalScreen>();

        let window = Window {
            resolution: WindowResolution::new(960, 640),
            ..default()
        };
        world.spawn((window, PrimaryWindow));
        let game_camera = world.spawn((Camera2d, GameCamera)).id();
        let overlay_camera = world.spawn(Camera2d).id();
        update_cameras(&mut world);
        assert_eq!(target_size(&world, game_camera), Some(UVec2::new(960, 640)));
        assert_eq!(target_size(&world, overlay_camera), Some(UVec2::new(960, 640)));

        // Resized by the app: the window has its new size a frame before its resize message.
        let mut windows = world.query::<&mut Window>();
        let mut window = windows.single_mut(&mut world).expect("one window");
        window.resolution.set(800.0, 800.0);
        update_cameras(&mut world);

        assert_eq!(target_size(&world, game_camera), Some(UVec2::new(800, 800)));
        assert_eq!(target_size(&world, overlay_camera), Some(UVec2::new(800, 800)));
    }

    #[test]
    fn without_a_platform_viewport_the_screen_is_centred() {
        let screen = LogicalScreen::default();
        let rect = viewport_rect(UVec2::new(1000, 1000), &screen);
        assert_eq!(rect.size(), UVec2::new(1000, 667));
        assert_eq!(rect.min, UVec2::new(0, 166));
    }
}
