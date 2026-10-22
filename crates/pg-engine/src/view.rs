//! `BCView`: screen geometry, the frame loop, game-frame listeners, overlays and the
//! accelerometer (`docs/re/rendering.md` §1–§2, §10).

use std::f32::consts::PI;

use bevy_ecs::prelude::*;
use bevy_math::{Mat4, Vec3, Vec4};

use crate::display;
use crate::gl::{GlContext, NdcRect};
use crate::math::{VEC4_ORIGIN, length3};
use crate::object::try_isa;
use crate::texture::AtlasId;
use crate::touch::TouchEventRing;

/// Raw accelerometer input from the platform, in the iOS convention (device portrait axes, g
/// units, gravity included; `(0, 0, 0)` when there is no sensor).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MotionInput {
    pub acceleration: Vec3,
}

/// `BCView` ivars.
#[derive(Resource)]
pub struct BcView {
    pub time: f32,
    pub game_frame_count: i32,
    pub active_display_group: Option<Entity>,
    pub wide_screen_overlay: Option<Entity>,
    pub tall_screen_overlay: Option<Entity>,

    /// `mGameFrameListenerArray` (insertion order; sorted stably by order every frame).
    pub game_frame_listeners: Vec<Entity>,

    /// `mUITouchArray`.
    pub ui_touches: Vec<Entity>,

    /// `mGameTouchArray`: the fingers currently down.
    pub game_touches: Vec<Entity>,
    pub touch_active: bool,

    /// The 1000-slot touch event ring (`DAT_1008162d0` with its read and write indices).
    pub touch_events: TouchEventRing,

    /// Renderbuffer size in pixels (`mViewWidth`, `mViewHeight`).
    pub view_width: i32,
    pub view_height: i32,

    /// Logical screen (`mScreenWidth × mScreenHeight`, e.g. 568 × 320).
    pub screen_width: f32,
    pub screen_height: f32,

    /// `mScreenZPos`: computed by `setFrustumHorizontalShift:verticalShift:`, never read.
    pub screen_z_pos: f32,

    /// View size in points (`mTouchWidth × mTouchHeight`).
    pub touch_width: f32,
    pub touch_height: f32,
    pub frustum_horizontal_shift: f32,
    pub frustum_vertical_shift: f32,
    pub wide_screen: bool,
    pub tall_screen: bool,
    pub scissor_x_pos: f32,
    pub scissor_y_pos: f32,
    pub scissor_width: f32,
    pub scissor_height: f32,
    pub clear_color: [f32; 4],

    pub total_accel_force: f32,
    pub xy_accel_force: f32,
    pub xz_accel_force: f32,
    pub yz_accel_force: f32,
    pub accel_force: Vec4,
    pub accel_rot_radians: Vec4,
    pub accel_rot_degrees: Vec4,
    pub total_raw_accel_force: f32,
    pub xy_raw_accel_force: f32,
    pub xz_raw_accel_force: f32,
    pub yz_raw_accel_force: f32,
    pub raw_accel_force: Vec4,
    pub raw_accel_rot_radians: Vec4,
    pub raw_accel_rot_degrees: Vec4,

    /// `activateTextureAtlasNextFrame:` queue (max 10, consumed LIFO).
    pub pending_atlas_loads: Vec<(AtlasId, String)>,

    /// The GL state machine; its colour / blend / texture state persists across frames.
    pub gl: Option<GlContext>,

    /// `mbRenderLoopActive`: the platform's display link calls [`game_loop`] only while set.
    pub render_loop_active: bool,

    /// `mDisplayLinkCount`: the display link's frame interval (1 = every 60 Hz frame, 2 = 30 Hz).
    pub display_link_count: i32,

    /// Set by [`start_render_loop`] (`mLastTime = -1/15`): the next display-link tick has a dt of
    /// exactly 1/15. The platform's frame driver clears it.
    pub render_loop_restarted: bool,

    /// `startAccelerometerUpdates` / `stopAccelerometerUpdates` of the motion manager.
    pub accelerometer_enabled: bool,
}

impl Default for BcView {
    fn default() -> Self {
        Self {
            time: 0.0,
            game_frame_count: 0,
            active_display_group: None,
            wide_screen_overlay: None,
            tall_screen_overlay: None,
            game_frame_listeners: Vec::new(),
            ui_touches: Vec::new(),
            game_touches: Vec::new(),
            touch_active: false,
            touch_events: TouchEventRing::default(),
            view_width: 480,
            view_height: 320,
            screen_width: 480.0,
            screen_height: 320.0,
            screen_z_pos: screen_z_pos(480.0),
            touch_width: 480.0,
            touch_height: 320.0,
            frustum_horizontal_shift: 0.5,
            frustum_vertical_shift: 0.5,
            wide_screen: false,
            tall_screen: false,
            scissor_x_pos: 0.0,
            scissor_y_pos: 0.0,
            scissor_width: 480.0,
            scissor_height: 320.0,
            clear_color: [1.0, 0.0, 0.0, 1.0],
            total_accel_force: 0.0,
            xy_accel_force: 0.0,
            xz_accel_force: 0.0,
            yz_accel_force: 0.0,
            accel_force: VEC4_ORIGIN,
            accel_rot_radians: VEC4_ORIGIN,
            accel_rot_degrees: VEC4_ORIGIN,
            total_raw_accel_force: 0.0,
            xy_raw_accel_force: 0.0,
            xz_raw_accel_force: 0.0,
            yz_raw_accel_force: 0.0,
            raw_accel_force: VEC4_ORIGIN,
            raw_accel_rot_radians: VEC4_ORIGIN,
            raw_accel_rot_degrees: VEC4_ORIGIN,
            pending_atlas_loads: Vec::new(),
            gl: Some(GlContext::default()),
            render_loop_active: false,
            display_link_count: 1,
            render_loop_restarted: false,
            accelerometer_enabled: false,
        }
    }
}

/// `BCView::startRenderLoop` @ 0x100177af8: from now on the display link (the platform's frame
/// driver) runs [`game_loop`]. Game time restarts (`mStartTime = now`; the original's `mTime` is
/// the time since this call) and, as `mLastTime = -1/15`, the first tick's dt is exactly 1/15.
pub fn start_render_loop(world: &mut World) {
    let mut view = world.resource_mut::<BcView>();
    if view.render_loop_active {
        return;
    }
    view.render_loop_active = true;
    view.accelerometer_enabled = true;
    view.time = 0.0;
    view.render_loop_restarted = true;
}

/// `BCView::stopRenderLoop` @ 0x100177c28.
pub fn stop_render_loop(world: &mut World) {
    let mut view = world.resource_mut::<BcView>();
    if !view.render_loop_active {
        return;
    }
    view.render_loop_active = false;
    view.accelerometer_enabled = false;
}

/// `BCView::isRenderLoopActive` @ 0x100177720.
pub fn is_render_loop_active(world: &World) -> bool {
    world.resource::<BcView>().render_loop_active
}

/// `BCView::setDisplayLinkCount:` @ 0x100177ad8 (the frame interval the display link uses).
pub fn set_display_link_count(world: &mut World, count: i32) {
    world.resource_mut::<BcView>().display_link_count = count;
}

/// `BCView::addGameFrameListener:` @ 0x100177528 (appends).
pub fn add_game_frame_listener(world: &mut World, listener: Entity) {
    world.resource_mut::<BcView>().game_frame_listeners.push(listener);
}

/// `BCView::removeGameFrameListener:` @ 0x100177544 (removes every occurrence).
pub fn remove_game_frame_listener(world: &mut World, listener: Entity) {
    let mut view = world.resource_mut::<BcView>();
    view.game_frame_listeners.retain(|entry| *entry != listener);
}

/// `BCView::setActiveDisplayGroup:` @ 0x100177570.
pub fn set_active_display_group(world: &mut World, group: Option<Entity>) {
    let old = world.resource::<BcView>().active_display_group;
    if old == group {
        return;
    }
    if let Some(old) = old {
        display::hibernate(world, old);
    }
    world.resource_mut::<BcView>().active_display_group = group;
    if let Some(group) = group {
        display::unhibernate(world, group);
    }
}

/// `BCView::setWideScreenOverlayDisplayGroup:` @ 0x1001775e8.
pub fn set_wide_screen_overlay_display_group(world: &mut World, group: Option<Entity>) {
    let old = world.resource::<BcView>().wide_screen_overlay;
    if old == group {
        return;
    }
    if let Some(old) = old {
        display::hibernate(world, old);
    }
    world.resource_mut::<BcView>().wide_screen_overlay = group;
    if let Some(group) = group {
        display::unhibernate(world, group);
    }
}

/// `BCView::setTallScreenOverlayDisplayGroup:` @ 0x100177660.
pub fn set_tall_screen_overlay_display_group(world: &mut World, group: Option<Entity>) {
    let old = world.resource::<BcView>().tall_screen_overlay;
    if old == group {
        return;
    }
    if let Some(old) = old {
        display::hibernate(world, old);
    }
    world.resource_mut::<BcView>().tall_screen_overlay = group;
    if let Some(group) = group {
        display::unhibernate(world, group);
    }
}

/// `BCView::frustumCenterXPos` @ 0x100175e60.
pub fn frustum_center_x_pos(view: &BcView) -> f32 {
    view.frustum_horizontal_shift * view.screen_width
}

/// `BCView::frustumCenterYPos` @ 0x100175e80.
pub fn frustum_center_y_pos(view: &BcView) -> f32 {
    view.frustum_vertical_shift * view.screen_height
}

/// `mScreenZPos = W / (2·tanf(π/6))` (`0x3f060a92`), the distance of a 60° camera.
fn screen_z_pos(screen_width: f32) -> f32 {
    let half_angle = f32::from_bits(0x3f06_0a92);
    let tangent = half_angle.tan();
    screen_width / (tangent + tangent)
}

/// `BCView::setFrustumHorizontalShift:verticalShift:` @ 0x100176618 for a renderbuffer of
/// `view_width × view_height` pixels and a view of `touch_width × touch_height` points.
/// Like `initWithFrame:` and `createFramebuffer`, it first takes the touch size from the view
/// and the logical screen from the renderbuffer, then derives the logical screen, the
/// wide/tall flags and the scissor with the only shifts the game uses (0.5, 0.5).
pub fn configure_screen(world: &mut World, view_width: i32, view_height: i32, touch_width: f32, touch_height: f32) {
    let mut view = world.resource_mut::<BcView>();
    view.touch_width = touch_width;
    view.touch_height = touch_height;
    view.view_width = view_width;
    view.view_height = view_height;
    view.screen_width = view_width as f32;
    view.screen_height = view_height as f32;

    set_frustum_horizontal_shift_vertical_shift(&mut view, 0.5, 0.5);

    let units_per_pixel = view.screen_width / view_width as f32;
    if let Some(gl) = view.gl.as_mut() {
        gl.units_per_pixel = units_per_pixel;
    }
}

/// `BCView::setFrustumHorizontalShift:verticalShift:` @ 0x100176618. The projection itself is
/// [`projection`] (which assumes the shifts of 0.5).
fn set_frustum_horizontal_shift_vertical_shift(view: &mut BcView, horizontal_shift: f32, vertical_shift: f32) {
    view.frustum_horizontal_shift = horizontal_shift;
    view.frustum_vertical_shift = vertical_shift;
    view.tall_screen = false;
    view.wide_screen = false;

    let aspect = view.screen_width / view.screen_height;
    let view_width = view.view_width as f32;
    let view_height = view.view_height as f32;
    if aspect < 1.5 {
        view.tall_screen = true;
        view.screen_width = 480.0;
        view.screen_height = 480.0 / aspect;
        view.scissor_width = view_width;
        view.scissor_height = view_width / 1.5;
    } else {
        // NaN takes this branch too, without setting the wide flag.
        if aspect > 1.5 {
            view.wide_screen = true;
        }
        view.screen_height = 320.0;
        view.screen_width = aspect * 320.0;
        view.scissor_height = view_height;
        view.scissor_width = view_height * 1.5;
    }
    view.scissor_x_pos = (view_width - view.scissor_width) * 0.5;
    view.scissor_y_pos = (view_height - view.scissor_height) * 0.5;

    view.screen_z_pos = screen_z_pos(view.screen_width);
}

/// The projection for the current logical screen.
pub fn projection(view: &BcView) -> Mat4 {
    GlContext::view_projection(view.screen_width, view.screen_height)
}

/// The scissor of the central 3:2 area in NDC (only when wide or tall), rounded to whole
/// renderbuffer pixels like `FUN_10013fc94`: `x0 = floor(x)`, `x1 = (int)(floor(x) + ceil(w))`,
/// width clamped at 0 (same for y).
pub fn game_area_scissor(view: &BcView) -> Option<NdcRect> {
    if !view.wide_screen && !view.tall_screen {
        return None;
    }

    let x_floor = view.scissor_x_pos.floor();
    let y_floor = view.scissor_y_pos.floor();
    let width_ceil = view.scissor_width.ceil();
    let height_ceil = view.scissor_height.ceil();

    let x0 = x_floor as i32;
    let y0 = y_floor as i32;
    let x1 = (x_floor + width_ceil) as i32;
    let y1 = (y_floor + height_ceil) as i32;
    let width = x1.wrapping_sub(x0).max(0);
    let height = y1.wrapping_sub(y0).max(0);

    let pixels_wide = view.view_width as f32;
    let pixels_high = view.view_height as f32;
    let rect = NdcRect {
        min_x: x0 as f32 / pixels_wide * 2.0 - 1.0,
        min_y: y0 as f32 / pixels_high * 2.0 - 1.0,
        max_x: x0.wrapping_add(width) as f32 / pixels_wide * 2.0 - 1.0,
        max_y: y0.wrapping_add(height) as f32 / pixels_high * 2.0 - 1.0,
    };
    Some(rect)
}

/// `BCView::activateTextureAtlasNextFrame:fileName:ext:` @ 0x1001776d8.
pub fn activate_texture_atlas_next_frame(world: &mut World, atlas: AtlasId, file_name: &str) {
    let mut view = world.resource_mut::<BcView>();
    if view.pending_atlas_loads.len() < 10 {
        view.pending_atlas_loads.push((atlas, file_name.to_owned()));
    }
}

/// `0.7f` widened to double: the raw filter, and the smoothed filter when the force falls.
const ACCEL_FILTER: f64 = 0.7_f32 as f64;

/// `0.1f` widened to double: the smoothed filter when the x force rises.
const ACCEL_FILTER_RISING: f64 = 0.1_f32 as f64;

/// `(float)(current + (target - current) * factor)`, computed in double.
fn filter_toward(current: f32, target: f64, factor: f64) -> f32 {
    let current = f64::from(current);
    let step = (target - current) * factor;
    (current + step) as f32
}

/// The magnitudes and angles derived from one force vector: total, xy, xz, yz, then the
/// radians (`atan2f(z, y)`, `atan2f(z, x)`, `atan2f(y, x)`) and degrees (`rad / π · 180`).
struct AccelDerived {
    total: f32,
    xy: f32,
    xz: f32,
    yz: f32,
    radians: [f32; 3],
    degrees: [f32; 3],
}

fn derive_accel(force: Vec4) -> AccelDerived {
    let radians = [force.z.atan2(force.y), force.z.atan2(force.x), force.y.atan2(force.x)];
    let degrees = radians.map(|angle| angle / PI * 180.0);
    AccelDerived {
        total: length3(force),
        xy: (force.x * force.x + force.y * force.y).sqrt(),
        xz: (force.x * force.x + force.z * force.z).sqrt(),
        yz: (force.y * force.y + force.z * force.z).sqrt(),
        radians,
        degrees,
    }
}

/// `BCView::readAccelerometer` @ 0x100175f3c: the raw force follows the input at 0.7; the
/// smoothed force at 0.1 while x rises and 0.7 otherwise (decided on x only).
pub fn read_accelerometer(world: &mut World) {
    // `[nil acceleration]` is (0, 0, 0) when there is no sensor data.
    let acceleration = match world.get_resource::<MotionInput>() {
        Some(motion) => motion.acceleration,
        None => Vec3::ZERO,
    };
    let input_x = f64::from(acceleration.x);
    let input_y = f64::from(acceleration.y);
    let input_z = f64::from(acceleration.z);

    let mut view = world.resource_mut::<BcView>();

    let mut raw = view.raw_accel_force;
    raw.x = filter_toward(raw.x, input_x, ACCEL_FILTER);
    raw.y = filter_toward(raw.y, input_y, ACCEL_FILTER);
    raw.z = filter_toward(raw.z, input_z, ACCEL_FILTER);
    view.raw_accel_force = raw;

    let raw_derived = derive_accel(raw);
    view.total_raw_accel_force = raw_derived.total;
    view.xy_raw_accel_force = raw_derived.xy;
    view.xz_raw_accel_force = raw_derived.xz;
    view.yz_raw_accel_force = raw_derived.yz;
    let [raw_rad_x, raw_rad_y, raw_rad_z] = raw_derived.radians;
    view.raw_accel_rot_radians.x = raw_rad_x;
    view.raw_accel_rot_radians.y = raw_rad_y;
    view.raw_accel_rot_radians.z = raw_rad_z;
    let [raw_deg_x, raw_deg_y, raw_deg_z] = raw_derived.degrees;
    view.raw_accel_rot_degrees.x = raw_deg_x;
    view.raw_accel_rot_degrees.y = raw_deg_y;
    view.raw_accel_rot_degrees.z = raw_deg_z;

    let mut accel = view.accel_force;
    let rising = input_x - f64::from(accel.x) > 0.0;
    let factor = if rising { ACCEL_FILTER_RISING } else { ACCEL_FILTER };
    accel.x = filter_toward(accel.x, input_x, factor);
    accel.y = filter_toward(accel.y, input_y, factor);
    accel.z = filter_toward(accel.z, input_z, factor);
    view.accel_force = accel;

    let derived = derive_accel(accel);
    view.total_accel_force = derived.total;
    view.xy_accel_force = derived.xy;
    view.xz_accel_force = derived.xz;
    view.yz_accel_force = derived.yz;
    let [rad_x, rad_y, rad_z] = derived.radians;
    view.accel_rot_radians.x = rad_x;
    view.accel_rot_radians.y = rad_y;
    view.accel_rot_radians.z = rad_z;
    let [deg_x, deg_y, deg_z] = derived.degrees;
    view.accel_rot_degrees.x = deg_x;
    view.accel_rot_degrees.y = deg_y;
    view.accel_rot_degrees.z = deg_z;
}

/// `BCView::gameLoop` @ 0x100177730 for one tick of `dt` seconds (already clamped by the
/// caller): platform touches into the ring, atlas loads, accelerometer, touches, game-frame
/// listeners, then the draw. Direct calls before the first [`start_render_loop`] (the three
/// warm-up ticks of `loadGame`) pass 1/15, 0 and 0: with `mStartTime` and `mLastTime` still zero
/// the first clamps to 1/15, and the next two land within the same f32 timestamp (~8e8 s).
pub fn game_loop(world: &mut World, dt: f32) {
    // The UIKit callbacks of the original queue touches between two ticks.
    crate::touch::create_touch_events(world);

    let clear_color = {
        let mut view = world.resource_mut::<BcView>();
        view.game_frame_count += 1;
        // `glClear` runs here, before the update: a colour set during this tick shows next tick.
        let clear_color = view.clear_color;
        view.time += dt;
        clear_color
    };

    let pending = std::mem::take(&mut world.resource_mut::<BcView>().pending_atlas_loads);
    for (atlas, file_name) in pending.into_iter().rev() {
        crate::texture::load_atlas(world, atlas, &file_name);
    }

    read_accelerometer(world);
    let now = world.resource::<BcView>().time;
    crate::touch::process_touch_events(world, now);

    run_game_frame_listeners(world, dt);
    draw_frame(world, clear_color);
}

/// Sorts listeners stably by `gameFrameOrder`, snapshots them and calls `onGameFrame:`.
pub fn run_game_frame_listeners(world: &mut World, dt: f32) {
    let mut listeners = world.resource::<BcView>().game_frame_listeners.clone();
    listeners.sort_by_key(|listener| match try_isa(world, *listener) {
        Some(class) => class.game_frame_order,
        None => i32::MAX,
    });
    world.resource_mut::<BcView>().game_frame_listeners = listeners.clone();

    for listener in listeners {
        let Some(class) = try_isa(world, listener) else {
            continue;
        };
        (class.on_game_frame)(world, listener, dt);
    }
}

/// The draw half of `gameLoop`: active group inside the 3:2 scissor, then the overlays,
/// cleared with the current clear colour.
pub fn draw(world: &mut World) {
    let clear_color = world.resource::<BcView>().clear_color;
    draw_frame(world, clear_color);
}

/// The draw of `BCView::gameLoop` @ 0x100177730 (from the scissor push to the overlays).
fn draw_frame(world: &mut World, clear_color: [f32; 4]) {
    let (projection, scissor, active_display_group) = {
        let view = world.resource::<BcView>();
        (projection(view), game_area_scissor(view), view.active_display_group)
    };
    let Some(mut gl) = world.resource_mut::<BcView>().gl.take() else {
        panic!("BCView's GL context is already in use");
    };
    gl.begin_frame(projection, clear_color);

    if scissor.is_some() {
        gl.set_scissor(scissor);
    }
    if let Some(group) = active_display_group {
        display::draw(world, group, &mut gl);
    }

    let (letterboxed, wide_screen, tall_screen, wide_screen_overlay, tall_screen_overlay) = {
        let view = world.resource::<BcView>();
        let letterboxed = view.wide_screen || view.tall_screen;
        (letterboxed, view.wide_screen, view.tall_screen, view.wide_screen_overlay, view.tall_screen_overlay)
    };
    if letterboxed {
        gl.set_scissor(None);
        if let Some(overlay) = wide_screen_overlay
            && wide_screen
        {
            display::draw(world, overlay, &mut gl);
        }
        if let Some(overlay) = tall_screen_overlay
            && tall_screen
        {
            display::draw(world, overlay, &mut gl);
        }
    }

    world.resource_mut::<BcView>().gl = Some(gl);
}
