//! `RainbowLogic` (superclass `SpriteLogic`).
//!
//! The display object is always the `RainbowModel` built in `loadGame`. `[mDisplayObject
//! pathList]` hands out the model's point list, which this class clears and appends to in
//! place: the port edits [`RainbowModel::path_list`] directly.

use std::f32::consts::PI;

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::sky_powers::rainbow_model::{self, RainbowModel};
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::logic::multi_model;

/// `RainbowLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct RainbowLogic {
    /// `mRainbowLogicListener`: `@"<RainbowLogicListener>"`
    pub rainbow_logic_listener: Option<Entity>,

    /// `mpRainbowListCopy`: `^{list<VECTOR4, std::__1::allocator<VECTOR4> >={__list_node_base<VECTOR4, void *>=^{__list_node<VECTOR4, void *>}^{__list_node<VECTOR4, void *>}}{__compressed_pair<unsigned long, std::__1::allocator<std::__1::__list_node<VECTOR4, void *> > >=Q}}`
    pub rainbow_list_copy: Vec<Vec4>,

    /// `mLastRainbowPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_rainbow_pos: Vec4,

    /// `mLastRainbowAngle`: `f`
    pub last_rainbow_angle: f32,

    /// `mRainbowGlowLogicArray`: `@"NSMutableArray"`
    pub rainbow_glow_logic_array: Vec<Entity>,

    /// `mGushingPygmyLogic`: `@"PygmyLogic"`
    pub gushing_pygmy_logic: Option<Entity>,

    /// `mbRainbowActive`: `B`
    pub rainbow_active: bool,

    /// `mbRainbowDrawing`: `B`
    pub rainbow_drawing: bool,

    /// `mRainAlpha`: `f`
    pub rain_alpha: f32,

    /// `mRainArray`: `@"NSMutableArray"`
    pub rain_array: Vec<Entity>,

    /// `mDrawRainbowIndex`: `i`
    pub draw_rainbow_index: i32,

    /// `mDrawRainbowAngle`: `f`
    pub draw_rainbow_angle: f32,

    /// `mNextGlowTime`: `f`
    pub next_glow_time: f32,
}

pub const RAINBOW_LOGIC: Class = Class {
    name: "RainbowLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    on_multi_model_event,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Texture of the rain drops.
const RAINDROP_TEXTURE_ID: &str = "Raindrop";

/// Model id of the rain drops.
const RAIN_MODEL_ID: &str = "Rain";

/// Rain drops falling while a storm meets the rainbow.
const RAIN_DROP_COUNT: usize = 10;

/// Rain drops fall slanted by this many degrees.
const RAIN_Z_ROT: f32 = 30.0;

/// Per-frame rain movement.
const RAIN_X_DRIFT: f32 = 5.0;
const RAIN_Y_FALL: f32 = -30.0;

/// A drop below this height starts again at the top of the screen.
const RAIN_BOTTOM_Y_POS: f32 = -30.0;
const RAIN_TOP_Y_POS: f32 = 320.0;

/// Share of the way to the target alpha the rain covers each frame (applied in double).
const RAIN_ALPHA_EASE: f64 = 0.2;

/// The rainbow goes right after this marker.
const FIREWORKS_DISPLAY_MARKER_ID: &str = "FireworksDisplayMarker";

/// The rain goes right after this marker.
const CLOUD_FRONT_DISPLAY_MARKER_ID: &str = "CloudFrontDisplayMarker";

/// The sparkles' display group.
const ISLAND_DISPLAY_GROUP_ID: &str = "IslandDisplayGroup";

/// Model id and behaviour of the sparkles that trail the finger.
const GLOW_MODEL_ID: &str = "RainbowGlow";
const GLOW_BEHAVIOR: &str = "RainbowGlow";

/// The event a sparkle sends when its animation ends.
const GLOW_COMPLETE_EVENT_ID: &str = "RainbowGlowComplete";

/// Seconds between two sparkles.
const GLOW_INTERVAL: f32 = 0.07;

/// New sparkles are only created while fewer than this many exist.
const MAX_GLOW_LOGIC_COUNT: usize = 201;

/// A replayed rainbow of the player needs at least this many points.
const MIN_MY_RAINBOW_POINT_COUNT: usize = 3;

/// The stock story-mode arc: an ellipse from angle π down to 0 in 5° steps.
const STORY_ARC_CENTER_X_POS: f32 = 240.0;
const STORY_ARC_CENTER_Y_POS: f32 = 110.0;
const STORY_ARC_X_RADIUS: f32 = 200.0;
const STORY_ARC_Y_RADIUS: f32 = 150.0;
const STORY_ARC_ANGLE_STEP: f32 = -0.08726647;

/// Sequence-item handlers implemented by `RainbowLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "storyModeDrawMyRainbow" => seq_story_mode_draw_my_rainbow(world, this, call),
        "storyModeDrawRainbow" => seq_story_mode_draw_rainbow(world, this, call),
        "storyModeComplete" => seq_story_mode_complete(world, this, call),
        "waitForGushingPygmy" => seq_wait_for_gushing_pygmy(world, this, call),
        "lookForGushingPygmy" => seq_look_for_gushing_pygmy(world, this, call),
        "rainbowStrikeComplete" => seq_rainbow_strike_complete(world, this, call),
        "stareAtRainbowComplete" => seq_stare_at_rainbow_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[RainbowLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&RAINBOW_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// A rain drop's x when it (re)appears: `(rand % 5301 − 500) / 10`.
fn random_rain_x_pos(world: &mut World) -> f32 {
    let roll = world.resource_mut::<Random>().arc4random_mod(5301);
    (roll as i32 - 500) as f32 / 10.0
}

/// `[mRainbowLogicListener pygmyLogicArray:self]` (the listener is the view controller).
fn listener_pygmy_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<RainbowLogic>(this).rainbow_logic_listener;
    match listener {
        Some(listener) => crate::controller::world_registry::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

/// `-[RainbowLogic initWithDisplayObject:]` @ 0x10001fd70
///
/// Also creates the ten rain drops (not yet in the display tree).
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(RainbowLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut rainbow = world.obj_mut::<RainbowLogic>(this);
        rainbow.rainbow_glow_logic_array = Vec::with_capacity(5);
        rainbow.rainbow_list_copy = Vec::new();
        rainbow.rain_array = Vec::with_capacity(5);
    }

    let raindrop_texture = library::texture_def_id(world, RAINDROP_TEXTURE_ID);
    for _ in 0..RAIN_DROP_COUNT {
        let rain = display::model::spawn(world, Some(RAIN_MODEL_ID), raindrop_texture);
        let x_pos = random_rain_x_pos(world);
        let y_roll = world.resource_mut::<Random>().arc4random_mod(5001);
        let y_pos = (y_roll + 3200) as f32 / 10.0;
        {
            let mut rain_display = display::object_mut(world, rain);
            rain_display.pos.x = x_pos;
            rain_display.pos.y = y_pos;
            rain_display.pos.z = SPRITE_Z;
        }
        world.obj_mut::<RainbowLogic>(this).rain_array.push(rain);
    }
}

/// `-[RainbowLogic setRainbowLogicListener:]` @ 0x10001ffe8
pub fn set_rainbow_logic_listener(world: &mut World, this: Entity, rainbow_logic_listener: Entity) {
    world.obj_mut::<RainbowLogic>(this).rainbow_logic_listener = Some(rainbow_logic_listener);
}

/// `-[RainbowLogic onGameFrame:]` @ 0x100020044
///
/// No shutdown check and no physics: counts down to the next sparkle and lets the rain fall,
/// its alpha easing toward the rainbow's while a storm is on (else toward 0).
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let listener = {
        let mut rainbow = world.obj_mut::<RainbowLogic>(this);
        rainbow.next_glow_time = rainbow.next_glow_time - dt;
        rainbow.rainbow_logic_listener
    };

    let storm_active = match listener {
        Some(listener) => crate::controller::sky_weather::is_storm_active(world, listener),
        None => false,
    };
    let target_alpha = if storm_active {
        let display_object = logic::display_object(world, this);
        display::object(world, display_object).alpha
    } else {
        0.0
    };

    let rain_alpha = {
        let mut rainbow = world.obj_mut::<RainbowLogic>(this);
        let alpha_step = (target_alpha - rainbow.rain_alpha) as f64 * RAIN_ALPHA_EASE;
        rainbow.rain_alpha = (rainbow.rain_alpha as f64 + alpha_step) as f32;
        rainbow.rain_alpha
    };

    let rain_array = world.obj::<RainbowLogic>(this).rain_array.clone();
    for rain in rain_array {
        let fallen_y_pos = {
            let mut rain_display = display::object_mut(world, rain);
            rain_display.set_alpha(rain_alpha);
            rain_display.rot.z = RAIN_Z_ROT;
            rain_display.pos.y = rain_display.pos.y + RAIN_Y_FALL;
            rain_display.pos.x = rain_display.pos.x + RAIN_X_DRIFT;
            rain_display.pos.y
        };

        if fallen_y_pos < RAIN_BOTTOM_Y_POS {
            let x_pos = random_rain_x_pos(world);
            let mut rain_display = display::object_mut(world, rain);
            rain_display.pos.x = x_pos;
            rain_display.pos.y = RAIN_TOP_Y_POS;
        }
    }
}

/// `-[RainbowLogic isRainbowActive]` @ 0x1000202ac
pub fn is_rainbow_active(world: &mut World, this: Entity) -> bool {
    world.obj::<RainbowLogic>(this).rainbow_active
}

/// `-[RainbowLogic clearRainbowGlowLogics]` @ 0x1000202bc
pub fn clear_rainbow_glow_logics(world: &mut World, this: Entity) {
    let glow_logics = world.obj::<RainbowLogic>(this).rainbow_glow_logic_array.clone();
    for glow_logic in glow_logics {
        let glow = logic::display_object(world, glow_logic);
        let in_display_tree = display::object(world, glow).display_group.is_some();
        if in_display_tree {
            display::remove_from_display_group(world, glow);
        }
    }
}

/// `-[RainbowLogic storyModeActivateRainbow]` @ 0x10002040c
///
/// Story mode draws the stock arc: an empty rainbow, no sparkles, the rainbow back after
/// `FireworksDisplayMarker`.
pub fn story_mode_activate_rainbow(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    world.obj_mut::<RainbowModel>(display_object).path_list.clear();
    clear_rainbow_glow_logics(world, this);

    let fireworks_marker = library::display_marker(world, FIREWORKS_DISPLAY_MARKER_ID);
    if let Some(fireworks_marker) = fireworks_marker {
        display::insert_after(world, fireworks_marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("RainbowStoryModeActivate"));
}

/// `-[RainbowLogic storyModeActivateMyRainbow]` @ 0x1000204bc
///
/// As `storyModeActivateRainbow`, but replays the player's last rainbow when it has at least
/// three points.
pub fn story_mode_activate_my_rainbow(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    world.obj_mut::<RainbowModel>(display_object).path_list.clear();
    clear_rainbow_glow_logics(world, this);

    let fireworks_marker = library::display_marker(world, FIREWORKS_DISPLAY_MARKER_ID);
    if let Some(fireworks_marker) = fireworks_marker {
        display::insert_after(world, fireworks_marker, Some(display_object));
    }

    let copy_point_count = world.obj::<RainbowLogic>(this).rainbow_list_copy.len();
    let behavior = if copy_point_count < MIN_MY_RAINBOW_POINT_COUNT {
        "RainbowStoryModeActivate"
    } else {
        "RainbowStoryModeActivateMine"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[RainbowLogic storyModeDeactivateRainbow]` @ 0x100020590
pub fn story_mode_deactivate_rainbow(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("RainbowStoryModeDeactivate"));
}

/// `-[RainbowLogic storyModeClearRainbow]` @ 0x1000205a8
pub fn story_mode_clear_rainbow(world: &mut World, this: Entity) {
    let rain_array = world.obj::<RainbowLogic>(this).rain_array.clone();
    for rain in rain_array {
        display::remove_from_display_group(world, rain);
    }

    let display_object = logic::display_object(world, this);
    world.obj_mut::<RainbowModel>(display_object).path_list.clear();
    logic::set_behavior(world, this, Some("RainbowStoryModeClear"));
}

/// `-[RainbowLogic showNextRainbowGlowLogic]` @ 0x100020700
///
/// Shows a sparkle at the last rainbow point, reusing one that has left the display tree or
/// creating a new one while fewer than 201 exist.
pub fn show_next_rainbow_glow_logic(world: &mut World, this: Entity) {
    let glow_logics = world.obj::<RainbowLogic>(this).rainbow_glow_logic_array.clone();

    let mut free_glow_logic = None;
    for &glow_logic in &glow_logics {
        let glow = logic::display_object(world, glow_logic);
        let in_display_tree = display::object(world, glow).display_group.is_some();
        if !in_display_tree {
            free_glow_logic = Some(glow_logic);
            break;
        }
    }

    let glow_logic = match free_glow_logic {
        Some(glow_logic) => glow_logic,
        None => {
            if glow_logics.len() >= MAX_GLOW_LOGIC_COUNT {
                return;
            }
            let glow = display::multi_model::spawn(world, Some(GLOW_MODEL_ID));
            let glow_logic = multi_model::spawn(world, glow);
            multi_model::set_multi_model_logic_listener(world, glow_logic, Some(this));
            world.obj_mut::<RainbowLogic>(this).rainbow_glow_logic_array.push(glow_logic);
            glow_logic
        }
    };

    let glow = logic::display_object(world, glow_logic);
    let rainbow = world.obj::<RainbowLogic>(this);
    let last_rainbow_pos = rainbow.last_rainbow_pos;
    let last_rainbow_angle = rainbow.last_rainbow_angle;
    {
        let mut glow_display = display::object_mut(world, glow);
        glow_display.pos.x = last_rainbow_pos.x;
        glow_display.pos.y = last_rainbow_pos.y;
        glow_display.pos.z = SPRITE_Z;
        glow_display.rot.z = last_rainbow_angle;
    }

    let island_display_group = library::display_group(world, ISLAND_DISPLAY_GROUP_ID);
    display::move_to_display_group(world, glow, island_display_group);
    logic::set_behavior(world, glow_logic, Some(GLOW_BEHAVIOR));
}

/// `-[RainbowLogic onMultiModelEvent:eventId:]` @ 0x1000209e8
///
/// A sparkle whose animation ended leaves the display tree (and becomes reusable).
pub fn on_multi_model_event(world: &mut World, _this: Entity, multi_model_logic: Entity, event_id: &str) {
    if event_id == GLOW_COMPLETE_EVENT_ID {
        let glow = logic::display_object(world, multi_model_logic);
        display::remove_from_display_group(world, glow);
    }
}

/// `-[RainbowLogic findGushingPygmyLogic]` @ 0x100020a50
///
/// Picks one of the pygmies staring at the rainbow uniformly and makes it gush over it.
pub fn find_gushing_pygmy_logic(world: &mut World, this: Entity) {
    let pygmy_logics = listener_pygmy_logic_array(world, this);

    let mut staring_count: u32 = 0;
    for &pygmy_logic in &pygmy_logics {
        let staring = crate::pygmy::is_staring_at_rainbow(world, pygmy_logic);
        staring_count = staring_count.wrapping_add(staring as u32);
    }
    if (staring_count as i32) <= 0 {
        return;
    }

    let roll = world.resource_mut::<Random>().arc4random_mod(staring_count);
    let mut remaining = roll as i32 + 1;
    for &pygmy_logic in &pygmy_logics {
        let staring = crate::pygmy::is_staring_at_rainbow(world, pygmy_logic);
        if !staring {
            continue;
        }
        remaining -= 1;
        if remaining == 0 {
            world.obj_mut::<RainbowLogic>(this).gushing_pygmy_logic = Some(pygmy_logic);
            crate::pygmy::gush_over_rainbow(world, pygmy_logic);
            return;
        }
    }
}

/// `-[RainbowLogic gushOverRainbowComplete:]` @ 0x100020c64
///
/// The gushing pygmy is done; while the player is still drawing another one gets a turn.
pub fn gush_over_rainbow_complete(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let gushing_pygmy_logic = world.obj::<RainbowLogic>(this).gushing_pygmy_logic;
    if gushing_pygmy_logic != Some(pygmy_logic) {
        return;
    }

    let drawing = {
        let mut rainbow = world.obj_mut::<RainbowLogic>(this);
        rainbow.gushing_pygmy_logic = None;
        rainbow.rainbow_drawing
    };
    if drawing {
        find_gushing_pygmy_logic(world, this);
    }
}

/// `-[RainbowLogic startDrawRainbow]` @ 0x100020ca0
///
/// The first stroke puts the rainbow and its rain into the display tree and sets every pygmy
/// staring; each stroke clears the sparkles and plays `RainbowStart`.
pub fn start_draw_rainbow(world: &mut World, this: Entity) {
    let active = world.obj::<RainbowLogic>(this).rainbow_active;
    if !active {
        world.obj_mut::<RainbowLogic>(this).rainbow_active = true;

        let display_object = logic::display_object(world, this);
        let fireworks_marker = library::display_marker(world, FIREWORKS_DISPLAY_MARKER_ID);
        if let Some(fireworks_marker) = fireworks_marker {
            display::insert_after(world, fireworks_marker, Some(display_object));
        }

        let cloud_front_marker = library::display_marker(world, CLOUD_FRONT_DISPLAY_MARKER_ID);
        let rain_array = world.obj::<RainbowLogic>(this).rain_array.clone();
        if let Some(cloud_front_marker) = cloud_front_marker {
            for rain in rain_array {
                display::insert_after(world, cloud_front_marker, Some(rain));
            }
        }

        let pygmy_logics = listener_pygmy_logic_array(world, this);
        for pygmy_logic in pygmy_logics {
            crate::pygmy::start_staring_at_rainbow(world, pygmy_logic);
        }
        find_gushing_pygmy_logic(world, this);
    }

    {
        let mut rainbow = world.obj_mut::<RainbowLogic>(this);
        rainbow.rainbow_drawing = true;
        rainbow.last_rainbow_pos.x = 0.0;
        rainbow.last_rainbow_pos.y = 0.0;
    }
    clear_rainbow_glow_logics(world, this);
    logic::set_behavior(world, this, Some("RainbowStart"));
}

/// `-[RainbowLogic drawRainbow:]` @ 0x100020f64
///
/// `pos` is the new point (the `VECTOR4 *` argument): the sparkle angle follows the stroke,
/// and every 0.07 s the point becomes the last rainbow point and gets a sparkle.
pub fn draw_rainbow(world: &mut World, this: Entity, pos: Vec4) {
    let next_glow_time = {
        let mut rainbow = world.obj_mut::<RainbowLogic>(this);
        let last_rainbow_pos = rainbow.last_rainbow_pos;
        let angle = (pos.y - last_rainbow_pos.y).atan2(pos.x - last_rainbow_pos.x);
        rainbow.last_rainbow_angle = (angle / PI) * 180.0;
        rainbow.next_glow_time
    };

    if next_glow_time < 0.0 {
        {
            let mut rainbow = world.obj_mut::<RainbowLogic>(this);
            rainbow.next_glow_time = GLOW_INTERVAL;
            rainbow.last_rainbow_pos = pos;
        }
        show_next_rainbow_glow_logic(world, this);
    }
}

/// `-[RainbowLogic drawRainbowComplete]` @ 0x100021018
///
/// Keeps a copy of the finished rainbow (for story mode) and strikes.
pub fn draw_rainbow_complete(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let points = rainbow_model::path_list(world, display_object);
    {
        let mut rainbow = world.obj_mut::<RainbowLogic>(this);
        rainbow.rainbow_list_copy = points;
        rainbow.rainbow_drawing = false;
    }
    logic::set_behavior(world, this, Some("RainbowStrike"));
}

/// `-[RainbowLogic storyModeDrawMyRainbow:::]` @ 0x10002126c
///
/// Redraws the player's saved rainbow one point per frame.
pub fn seq_story_mode_draw_my_rainbow(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<RainbowLogic>(this).draw_rainbow_index = 0;
    }

    let rainbow = world.obj::<RainbowLogic>(this);
    let draw_rainbow_index = rainbow.draw_rainbow_index;
    let point = rainbow.rainbow_list_copy.get(draw_rainbow_index as usize).copied();

    // PORT: past the end of the copy (only when a new stroke replaced it with a shorter one
    // mid-replay) the original reads the list's sentinel node as a point; nothing is drawn.
    if let Some(point) = point {
        draw_rainbow(world, this, point);
        let display_object = logic::display_object(world, this);
        world.obj_mut::<RainbowModel>(display_object).path_list.push(point);
    }

    let next_index = draw_rainbow_index + 1;
    let mut rainbow = world.obj_mut::<RainbowLogic>(this);
    rainbow.draw_rainbow_index = next_index;
    if rainbow.rainbow_list_copy.len() <= next_index as usize {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[RainbowLogic storyModeDrawRainbow:::]` @ 0x100021384
///
/// Draws the stock arc, one 5° step per frame.
pub fn seq_story_mode_draw_rainbow(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<RainbowLogic>(this).draw_rainbow_angle = PI;
    }

    let angle = world.obj::<RainbowLogic>(this).draw_rainbow_angle;
    let x_pos = angle.cos() * STORY_ARC_X_RADIUS + STORY_ARC_CENTER_X_POS;
    let y_pos = angle.sin() * STORY_ARC_Y_RADIUS + STORY_ARC_CENTER_Y_POS;

    // The original leaves z and w of this stack point uninitialised; nothing reads them.
    let point = Vec4::new(x_pos, y_pos, 0.0, 0.0);
    draw_rainbow(world, this, point);

    let display_object = logic::display_object(world, this);
    world.obj_mut::<RainbowModel>(display_object).path_list.push(point);

    let next_angle = angle + STORY_ARC_ANGLE_STEP;
    world.obj_mut::<RainbowLogic>(this).draw_rainbow_angle = next_angle;
    if next_angle < 0.0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[RainbowLogic storyModeComplete:::]` @ 0x10002149c
pub fn seq_story_mode_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    clear_rainbow_glow_logics(world, this);

    let display_object = logic::display_object(world, this);
    world.obj_mut::<RainbowModel>(display_object).path_list.clear();
    world.obj_mut::<RainbowLogic>(this).rainbow_active = false;
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[RainbowLogic waitForGushingPygmy:::]` @ 0x100021510
pub fn seq_wait_for_gushing_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let gushing_pygmy_logic = world.obj::<RainbowLogic>(this).gushing_pygmy_logic;
    if gushing_pygmy_logic.is_none() {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[RainbowLogic lookForGushingPygmy:::]` @ 0x10002152c
///
/// Never completes: keeps looking for a pygmy to gush while the behaviour lasts.
pub fn seq_look_for_gushing_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let gushing_pygmy_logic = world.obj::<RainbowLogic>(this).gushing_pygmy_logic;
    if gushing_pygmy_logic.is_none() {
        find_gushing_pygmy_logic(world, this);
    }
    ItemResult::Pending
}

/// `-[RainbowLogic rainbowStrikeComplete:::]` @ 0x100021560
pub fn seq_rainbow_strike_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    clear_rainbow_glow_logics(world, this);

    let display_object = logic::display_object(world, this);
    world.obj_mut::<RainbowModel>(display_object).path_list.clear();
    ItemResult::Done
}

/// `-[RainbowLogic stareAtRainbowComplete:::]` @ 0x1000215b4
///
/// The rainbow and its rain leave the display tree and every pygmy stops staring.
pub fn seq_stare_at_rainbow_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<RainbowLogic>(this).rainbow_active = false;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let rain_array = world.obj::<RainbowLogic>(this).rain_array.clone();
    for rain in rain_array {
        display::remove_from_display_group(world, rain);
    }

    let pygmy_logics = listener_pygmy_logic_array(world, this);
    for pygmy_logic in pygmy_logics {
        crate::pygmy::stop_staring_at_rainbow(world, pygmy_logic);
    }
    ItemResult::Done
}
