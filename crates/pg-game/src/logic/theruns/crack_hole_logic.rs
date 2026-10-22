//! `CrackHoleLogic` (superclass `SpriteLogic`).
//!
//! The crack hole on the main island and host of The Runs. Dragging a finger along the island
//! opens a chain of cracks (`Crack` multimodels) that a pygmy standing over them falls into. A
//! pygmy that falls in starts The Runs: an endless course of platforms in the lava cave of
//! `TheRunsDisplayGroup`, which this class lays out event by event (`TheRunsPlatformEvent`),
//! recycles as the camera scrolls, and decorates with turds, the lava rock, a sinking platform,
//! falling stalactites and the parallax scenery.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::theruns::{the_runs_platform_logic, the_runs_rock_logic, the_runs_stalactite_logic, the_runs_turd_logic};
use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::model::SPRITE_Z;

/// `CrackHoleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct CrackHoleLogic {
    /// `mCrackHoleLogicListener`: `@"<CrackHoleLogicListener>"`
    pub crack_hole_logic_listener: Option<Entity>,

    /// `mCrackLogicArray`: `@"NSMutableArray"`
    pub crack_logic_array: Vec<Entity>,

    /// `mNextTurdIndex`: `i`
    pub next_turd_index: i32,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbCrackOpening`: `B`
    pub crack_opening: bool,

    /// `mDragXPos`: `f`
    pub drag_x_pos: f32,

    /// `mStartCrackXPos`: `f`
    pub start_crack_x_pos: f32,

    /// `mNextCrackXPos`: `f`
    pub next_crack_x_pos: f32,

    /// `mDragDirection`: `f`
    pub drag_direction: f32,

    /// `mbTheRuns`: `B`
    pub the_runs: bool,

    /// `mbRockActive`: `B`
    pub rock_active: bool,

    /// `mbStalactitesActive`: `B`
    pub stalactites_active: bool,

    /// `mbStalactitesStarted`: `B`
    pub stalactites_started: bool,

    /// `mbWaitForStalactitesToBeOffScreen`: `B`
    pub wait_for_stalactites_to_be_off_screen: bool,

    /// `mStalactitesYPos`: `f`
    pub stalactites_y_pos: f32,

    /// `mbSinkingActive`: `B`
    pub sinking_active: bool,

    /// `mbSinkingStarted`: `B`
    pub sinking_started: bool,

    /// `mSinkingPlatformYPos`: `f`
    pub sinking_platform_y_pos: f32,

    /// `mbDemonOnScreen`: `B`
    pub demon_on_screen: bool,

    /// `mbBatsOnScreen`: `B`
    pub bats_on_screen: bool,

    /// `mTheRunsPlatformEventArray`: `[5{TheRunsPlatformEvent="mTheRunsPlatformEventType"i"mPlatformCount"i"mPlacedPlatformCount"i"mOffScreenPlatformCount"i"mStalactiteCount"i"mPlacedStalactiteCount"i"mTurdCount"i"mHeight"f"mGapWidth"f}]`
    pub the_runs_platform_event_array: [TheRunsPlatformEvent; 5],

    /// `mLeadTheRunsPlatformEventIndex`: `i`
    pub lead_the_runs_platform_event_index: i32,

    /// `mDifficulty`: `f`
    pub difficulty: f32,

    /// `mNextPlatformXPos`: `f`
    pub next_platform_x_pos: f32,

    /// `mLastPlatformHeight`: `f`
    pub last_platform_height: f32,

    /// `mPlatformSinkingVel`: `f`
    pub platform_sinking_vel: f32,

    /// `mpLastTheRunsPlatformEvent`: `^{TheRunsPlatformEvent=iiiiiiiff}`
    pub last_the_runs_platform_event: Option<usize>,

    /// `mTheRunsDisplayGroup`: `@"BCDisplayGroup"`
    pub the_runs_display_group: Option<Entity>,

    /// `mTheRunsDemonDisplayGroup`: `@"BCDisplayGroup"`
    pub the_runs_demon_display_group: Option<Entity>,

    /// `mCrackHolePygmyLogic`: `@"PygmyLogic"`
    pub crack_hole_pygmy_logic: Option<Entity>,

    /// `mTheRunsPygmyLogic`: `@"PygmyLogic"`
    pub the_runs_pygmy_logic: Option<Entity>,

    /// `mTheRunsDemonLogic`: `@"MultiModelLogic"`
    pub the_runs_demon_logic: Option<Entity>,

    /// `mTheRunsBatsLogic`: `@"MultiModelLogic"`
    pub the_runs_bats_logic: Option<Entity>,

    /// `mTheRunsBackgroundLogic`: `@"MultiModelLogic"`
    pub the_runs_background_logic: Option<Entity>,

    /// `mTheRunsLavaBackgroundLogic`: `@"MultiModelLogic"`
    pub the_runs_lava_background_logic: Option<Entity>,

    /// `mTheRunsStalactiteBackgroundLogic`: `@"MultiModelLogic"`
    pub the_runs_stalactite_background_logic: Option<Entity>,

    /// `mTheRunsLavaForeground1Logic`: `@"MultiModelLogic"`
    pub the_runs_lava_foreground1_logic: Option<Entity>,

    /// `mTheRunsLavaForeground2Logic`: `@"MultiModelLogic"`
    pub the_runs_lava_foreground2_logic: Option<Entity>,

    /// `mTheRunsRockLogic`: `@"TheRunsRockLogic"`
    pub the_runs_rock_logic: Option<Entity>,

    /// `mTheRunsTurdLogicArray`: `@"NSMutableArray"`
    pub the_runs_turd_logic_array: Vec<Entity>,

    /// `mTheRunsPlatformLogicArray`: `@"NSMutableArray"`
    pub the_runs_platform_logic_array: Vec<Entity>,

    /// `mTheRunsStalactiteLogicArray`: `@"NSMutableArray"`
    pub the_runs_stalactite_logic_array: Vec<Entity>,
}

pub const CRACK_HOLE_LOGIC: Class = Class {
    name: "CrackHoleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_multi_model_event,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"TheRunsDisplayGroup"`: the course, which the view controller's camera scrolls left.
const THE_RUNS_DISPLAY_GROUP: &str = "TheRunsDisplayGroup";

/// `@"TheRunsDemonDisplayGroup"`: the parallax layer of the demon and the bats.
const THE_RUNS_DEMON_DISPLAY_GROUP: &str = "TheRunsDemonDisplayGroup";

/// `@"StartCoconutDisplayMarker"`: the cracks go into the island just before it.
const CRACK_DISPLAY_MARKER: &str = "StartCoconutDisplayMarker";

/// `@"Crack"`
const CRACK_MODEL: &str = "Crack";

/// `@"Turd"`
const TURD_MODEL: &str = "Turd";

/// `@"LavaRock"`
const ROCK_MODEL: &str = "LavaRock";

/// The turds the init creates.
const TURD_COUNT: usize = 4;

/// `@"Platform1"` … `@"Platform4"` in `TheRunsDisplayGroup`.
const PLATFORM_IDS: [&str; 4] = ["Platform1", "Platform2", "Platform3", "Platform4"];

/// `@"Stalactite1"` … `@"Stalactite4"` in `TheRunsDisplayGroup`.
const STALACTITE_IDS: [&str; 4] = ["Stalactite1", "Stalactite2", "Stalactite3", "Stalactite4"];

/// `@"CrackOpen"`: the behaviour of a new crack and the event at its end.
const CRACK_OPEN: &str = "CrackOpen";

/// `@"CrackClose"`: the behaviour of a closing crack and the event at its end.
const CRACK_CLOSE: &str = "CrackClose";

/// `@"BlankAnim"`
const BLANK_BEHAVIOR: &str = "BlankAnim";

/// `@"TheRunsDemonInit"`
const DEMON_INIT_BEHAVIOR: &str = "TheRunsDemonInit";

/// The width of a crack; the chain of cracks is laid on a grid of this size.
const CRACK_WIDTH: f32 = 67.0;

/// How far inside the open chain (from both of its ends) a pygmy falls in.
const HALF_CRACK_WIDTH: f32 = 33.5;

/// The x step from one platform of an event to the next.
const PLATFORM_SPACING: f32 = 199.0;

/// The width of a platform from its `pos.x`.
const PLATFORM_WIDTH: f32 = 205.0;

/// How far a platform's `pos.y` is below its top (the event's height).
const PLATFORM_TOP_HEIGHT: f32 = 227.0;

/// The width of a stalactite from its `pos.x`.
const STALACTITE_WIDTH: f32 = 233.0;

/// The width of the screen.
const SCREEN_WIDTH: f32 = 480.0;

/// `mTheRunsPlatformEventType` of a stretch with a turd on its first platform.
const TURD_EVENT_TYPE: i32 = 2;

/// `mTheRunsPlatformEventType` of a stretch with the lava rock.
const ROCK_EVENT_TYPE: i32 = 3;

/// `mTheRunsPlatformEventType` of a stretch of sinking (crumbling) platforms.
const SINKING_EVENT_TYPE: i32 = 4;

/// `mTheRunsPlatformEventType` of a stretch under falling stalactites.
const STALACTITE_EVENT_TYPE: i32 = 5;

/// `0x3d088889` (0.033333335): the 30 Hz step of `getJumpDistance:…`.
const JUMP_STEP_TIME: f32 = f32::from_bits(0x3d08_8889);

/// `0xc2c80001` (-100.00001): the change of the vertical speed per jump step (3000 gravity).
const JUMP_STEP_Y_VEL_CHANGE: f32 = f32::from_bits(0xc2c8_0001);

/// `[[CrackHoleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&CRACK_HOLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[[displayGroup getDisplayObject:id] logic]`: nil when the group or the object is missing.
fn display_group_object_logic(world: &World, display_group: Option<Entity>, id: &str) -> Option<Entity> {
    let display_group = display_group?;
    let display_object = group::get_display_object(world, display_group, id)?;
    display::object(world, display_object).logic
}

/// `[logic displayObject]`: nil for a nil logic.
fn display_object_of(world: &World, logic: Option<Entity>) -> Option<Entity> {
    let logic = logic?;
    let display_object = pg_engine::logic::display_object(world, logic);
    Some(display_object)
}

/// `[[logic displayObject] pos]`.
fn logic_pos(world: &World, logic: Entity) -> Vec4 {
    let display_object = pg_engine::logic::display_object(world, logic);
    display::object(world, display_object).pos
}

/// `[logic setBehavior:behavior]`: nothing for a nil logic.
fn set_logic_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    if let Some(logic) = logic {
        pg_engine::logic::set_behavior(world, logic, Some(behavior));
    }
}

/// `display_object.pos.x = x_pos`. The original writes through `[display_object pos]`; its
/// objects are never nil.
fn set_x_pos(world: &mut World, display_object: Option<Entity>, x_pos: f32) {
    if let Some(display_object) = display_object {
        display::object_mut(world, display_object).pos.x = x_pos;
    }
}

/// `[group pos].x`; 0 for a nil group (the original reads through `[nil pos]`; its groups exist).
fn display_group_x_pos(world: &World, display_group: Option<Entity>) -> f32 {
    match display_group {
        Some(display_group) => display::object(world, display_group).pos.x,
        None => 0.0,
    }
}

/// `[mTheRunsDisplayGroup pos].x`: how far the camera has scrolled the course (negative), so
/// its negation is the course x at the left edge of the screen.
fn the_runs_scroll_x_pos(world: &World, this: Entity) -> f32 {
    let the_runs_display_group = world.obj::<CrackHoleLogic>(this).the_runs_display_group;
    display_group_x_pos(world, the_runs_display_group)
}

/// `(arc4random() % range + offset) / 10.0`, in the original's integer arithmetic: a random
/// distance or speed in tenths.
fn random_tenths(world: &mut World, range: u32, offset: i32) -> f32 {
    let random = world.resource_mut::<Random>().arc4random_mod(range);
    let tenths = random.wrapping_add_signed(offset) as i32;
    tenths as f32 / 10.0
}

/// `mCrackHoleLogicListener` (the view controller).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<CrackHoleLogic>(this).crack_hole_logic_listener
}

/// `[mCrackHoleLogicListener onTheRunsShakeScreenStart]`.
fn shake_screen_start(world: &mut World, this: Entity) {
    if let Some(listener) = listener_of(world, this) {
        crate::controller::on_the_runs_shake_screen_start(world, listener);
    }
}

/// `[mCrackHoleLogicListener onTheRunsShakeScreenStop]`.
fn shake_screen_stop(world: &mut World, this: Entity) {
    if let Some(listener) = listener_of(world, this) {
        crate::controller::on_the_runs_shake_screen_stop(world, listener);
    }
}

/// `[mCrackHoleLogicListener islandMinXPos]` (0 for a nil listener).
fn island_min_x_pos(world: &mut World, this: Entity) -> f32 {
    match listener_of(world, this) {
        Some(listener) => crate::controller::island_min_x_pos(world, listener),
        None => 0.0,
    }
}

/// `[mCrackHoleLogicListener islandMaxXPos]` (0 for a nil listener).
fn island_max_x_pos(world: &mut World, this: Entity) -> f32 {
    match listener_of(world, this) {
        Some(listener) => crate::controller::island_max_x_pos(world, listener),
        None => 0.0,
    }
}

/// `mTheRunsPlatformEventArray[event_index]`.
fn platform_event(world: &World, this: Entity, event_index: usize) -> TheRunsPlatformEvent {
    world.obj::<CrackHoleLogic>(this).the_runs_platform_event_array[event_index]
}

/// The event slot after `event_index`, wrapping after the fifth.
fn next_event_index(event_index: i32) -> i32 {
    let next = event_index + 1;
    if next == 5 { 0 } else { next }
}

/// `-[CrackHoleLogic initWithDisplayObject:]` @ 0x10012566c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(CrackHoleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.crack_logic_array = Vec::with_capacity(5);
        crack_hole.the_runs_turd_logic_array = Vec::with_capacity(5);
    }
    for _ in 0..TURD_COUNT {
        let turd_model = display::multi_model::spawn(world, Some(TURD_MODEL));
        let turd = the_runs_turd_logic::spawn(world, turd_model);
        world.obj_mut::<CrackHoleLogic>(this).the_runs_turd_logic_array.push(turd);
    }

    let rock_model = display::multi_model::spawn(world, Some(ROCK_MODEL));
    let rock = the_runs_rock_logic::spawn(world, rock_model);
    world.obj_mut::<CrackHoleLogic>(this).the_runs_rock_logic = Some(rock);

    let the_runs_display_group = pg_engine::library::display_group(world, THE_RUNS_DISPLAY_GROUP);
    let demon_display_group = pg_engine::library::display_group(world, THE_RUNS_DEMON_DISPLAY_GROUP);
    let background_logic = display_group_object_logic(world, the_runs_display_group, "Background");
    let lava_background_logic = display_group_object_logic(world, the_runs_display_group, "LavaBackground");
    let stalactite_background_logic = display_group_object_logic(world, the_runs_display_group, "StalactiteBackground");
    let lava_foreground1_logic = display_group_object_logic(world, the_runs_display_group, "LavaForeground1");
    let lava_foreground2_logic = display_group_object_logic(world, the_runs_display_group, "LavaForeground2");
    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.the_runs_display_group = the_runs_display_group;
        crack_hole.the_runs_demon_display_group = demon_display_group;
        crack_hole.the_runs_background_logic = background_logic;
        crack_hole.the_runs_lava_background_logic = lava_background_logic;
        crack_hole.the_runs_stalactite_background_logic = stalactite_background_logic;
        crack_hole.the_runs_lava_foreground1_logic = lava_foreground1_logic;
        crack_hole.the_runs_lava_foreground2_logic = lava_foreground2_logic;
    }

    // The demon and the bats scroll with their own parallax, in the demon group.
    let demon_logic = display_group_object_logic(world, the_runs_display_group, "Demon");
    world.obj_mut::<CrackHoleLogic>(this).the_runs_demon_logic = demon_logic;
    let demon_display_object = display_object_of(world, demon_logic);
    if let Some(demon_display_object) = demon_display_object {
        display::move_to_display_group(world, demon_display_object, demon_display_group);
    }

    let bats_logic = display_group_object_logic(world, the_runs_display_group, "Bats");
    world.obj_mut::<CrackHoleLogic>(this).the_runs_bats_logic = bats_logic;
    let bats_display_object = display_object_of(world, bats_logic);
    if let Some(bats_display_object) = bats_display_object {
        display::remove_from_display_group(world, bats_display_object);
        display::move_to_display_group(world, bats_display_object, demon_display_group);
    }

    // `addObject:` of a nil logic would throw; the course objects get their logics first.
    world.obj_mut::<CrackHoleLogic>(this).the_runs_platform_logic_array = Vec::with_capacity(5);
    for platform_id in PLATFORM_IDS {
        let platform = display_group_object_logic(world, the_runs_display_group, platform_id);
        if let Some(platform) = platform {
            the_runs_platform_logic::set_the_runs_platform_logic_listener(world, platform, this);
            world.obj_mut::<CrackHoleLogic>(this).the_runs_platform_logic_array.push(platform);
        }
    }

    world.obj_mut::<CrackHoleLogic>(this).the_runs_stalactite_logic_array = Vec::with_capacity(5);
    for stalactite_id in STALACTITE_IDS {
        let stalactite = display_group_object_logic(world, the_runs_display_group, stalactite_id);
        if let Some(stalactite) = stalactite {
            the_runs_stalactite_logic::set_the_runs_stalactite_logic_listener(world, stalactite, this);
            the_runs_stalactite_logic::deactivate(world, stalactite);
            world.obj_mut::<CrackHoleLogic>(this).the_runs_stalactite_logic_array.push(stalactite);
        }
    }

    pg_engine::logic::stop_game_frame(world, this);

    let own_display_object = pg_engine::logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[CrackHoleLogic setCrackHoleLogicListener:]` @ 0x100125cec
pub fn set_crack_hole_logic_listener(world: &mut World, this: Entity, crack_hole_logic_listener: Entity) {
    world.obj_mut::<CrackHoleLogic>(this).crack_hole_logic_listener = Some(crack_hole_logic_listener);
}

/// `-[CrackHoleLogic theRunsRockLogic]` @ 0x100125e14
pub fn the_runs_rock_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<CrackHoleLogic>(this).the_runs_rock_logic
}

/// `-[CrackHoleLogic theRunsPlatformLogicArray]` @ 0x100125e24
pub fn the_runs_platform_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<CrackHoleLogic>(this).the_runs_platform_logic_array.clone()
}

/// `-[CrackHoleLogic theRunsStalactiteLogicArray]` @ 0x100125e34
pub fn the_runs_stalactite_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<CrackHoleLogic>(this).the_runs_stalactite_logic_array.clone()
}

/// `-[CrackHoleLogic theRunsTurdLogicArray]` @ 0x100125e44
pub fn the_runs_turd_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<CrackHoleLogic>(this).the_runs_turd_logic_array.clone()
}

/// `-[CrackHoleLogic createCrack:]` @ 0x100125e54
///
/// Opens a crack at `x_pos` on the island, facing the drag direction.
pub fn create_crack(world: &mut World, this: Entity, x_pos: f32) {
    let crack_model = display::multi_model::spawn(world, Some(CRACK_MODEL));
    let marker = pg_engine::library::display_marker(world, CRACK_DISPLAY_MARKER);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(crack_model));
    }

    let drag_direction = world.obj::<CrackHoleLogic>(this).drag_direction;
    {
        let mut object = display::object_mut(world, crack_model);
        object.pos.x = x_pos;
        object.pos.y = 90.0;
        object.pos.z = SPRITE_Z;
        object.scale.x = drag_direction;
    }

    let crack = pg_engine::logic::multi_model::spawn(world, crack_model);
    pg_engine::logic::multi_model::set_multi_model_logic_listener(world, crack, Some(this));
    pg_engine::logic::set_behavior(world, crack, Some(CRACK_OPEN));
    world.obj_mut::<CrackHoleLogic>(this).crack_logic_array.push(crack);
}

/// `-[CrackHoleLogic onPygmyBurnPygmyComplete]` @ 0x100125fe4
pub fn on_pygmy_burn_pygmy_complete(world: &mut World, this: Entity) {
    if let Some(listener) = listener_of(world, this) {
        crate::controller::on_the_runs_start_moving_camera(world, listener);
    }
}

/// `-[CrackHoleLogic onPygmyLandOnSinkingPlatform:]` @ 0x100126000
///
/// The first landing on the sinking platform (`platform`) starts the sinking at its height.
pub fn on_pygmy_land_on_sinking_platform(world: &mut World, this: Entity, platform: Entity) {
    let sinking_started = world.obj::<CrackHoleLogic>(this).sinking_started;
    if sinking_started {
        return;
    }

    world.obj_mut::<CrackHoleLogic>(this).sinking_started = true;
    shake_screen_start(world, this);

    let platform_y_pos = logic_pos(world, platform).y;
    world.obj_mut::<CrackHoleLogic>(this).sinking_platform_y_pos = platform_y_pos;
}

/// `-[CrackHoleLogic onPygmyFallFromSinkingPlatform]` @ 0x100126088
pub fn on_pygmy_fall_from_sinking_platform(world: &mut World, this: Entity) {
    shake_screen_stop(world, this);
}

/// `-[CrackHoleLogic getJumpDistance:heightDelta:jumpYVel:jumpTime:]` @ 0x1001260a4
///
/// How far a jump at `x_vel` carries the pygmy: it rises at `jump_y_vel` for `jump_time`,
/// then falls in 30 Hz steps until it is 100 below the take-off or, on the way down, below
/// `height_delta`.
pub fn get_jump_distance(_world: &mut World, _this: Entity, x_vel: f32, height_delta: f32, jump_y_vel: f32, jump_time: f32) -> f32 {
    let mut distance = x_vel * jump_time;
    let mut height = jump_y_vel * jump_time;
    let mut y_vel = jump_y_vel;
    let step_distance = x_vel * JUMP_STEP_TIME;
    loop {
        y_vel += JUMP_STEP_Y_VEL_CHANGE;
        distance = step_distance + distance;
        height += y_vel * JUMP_STEP_TIME;
        if height < -100.0 {
            return distance;
        }

        let landed = y_vel < 0.0 && height < height_delta;
        if landed {
            return distance;
        }
    }
}

/// `-[CrackHoleLogic chooseTheRunsPlatformEvent:]` @ 0x1001260f4
///
/// Fills the event in slot `choose_the_runs_platform_event` (an index into
/// `mTheRunsPlatformEventArray`) with the next stretch of the course: a height within 50 of the
/// last one (and between 60 and 170), a gap between the slowest and the fastest jump to it,
/// and what the stretch holds. The difficulty rises by 0.01 per event, up to 1.
pub fn choose_the_runs_platform_event(world: &mut World, this: Entity, choose_the_runs_platform_event: Option<usize>) {
    let Some(event_index) = choose_the_runs_platform_event else {
        return;
    };

    // The height, in tenths, drawn from [last - 50, last + 50] within [60, 170].
    let last_platform_height = world.obj::<CrackHoleLogic>(this).last_platform_height;
    let lowest_height = last_platform_height - 50.0;
    let highest_height = last_platform_height + 50.0;
    let min_tenths = if lowest_height < 60.0 { 600 } else { (lowest_height * 10.0) as i32 };
    let max_tenths = if highest_height > 170.0 { 1701 } else { ((highest_height * 10.0) as i32).wrapping_add(1) };
    let range = max_tenths.wrapping_sub(min_tenths) as u32;
    let random_tenths = world.resource_mut::<Random>().arc4random_mod(range);
    let height_tenths = min_tenths.wrapping_add(random_tenths as i32);
    let height = height_tenths as f32 / 10.0;

    // The jumps of the slowest run (300) and of the fastest (600).
    let height_delta = height - last_platform_height;
    let slowest_jump_distance = get_jump_distance(world, this, 300.0, height_delta, 300.0, 0.15);
    let fastest_jump_distance = get_jump_distance(world, this, 600.0, height_delta, 400.0, 0.2);

    let (last_event_type, difficulty, sinking_active) = {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        let event = &mut crack_hole.the_runs_platform_event_array[event_index];
        event.turd_count = 0;
        event.placed_platform_count = 0;
        event.off_screen_platform_count = 0;
        event.stalactite_count = 0;
        event.placed_stalactite_count = 0;
        event.height = height;

        // `mpLastTheRunsPlatformEvent` is set by `startTheRuns:` before any event is chosen.
        let last_event = crack_hole.last_the_runs_platform_event;
        let last_event_type = match last_event {
            Some(last_event) => crack_hole.the_runs_platform_event_array[last_event].the_runs_platform_event_type,
            None => 0,
        };
        (last_event_type, crack_hole.difficulty, crack_hole.sinking_active)
    };

    // A random part of the gap (-0.2 up to 0.2 + 0.4 × difficulty) after a turd or the rock at
    // low difficulty, during a sinking platform, at the very start and 3 times in 4; otherwise
    // a fixed part growing with the difficulty. `b.le`: a NaN difficulty counts as the start.
    let after_hazard_while_easy = (last_event_type & !1) == TURD_EVENT_TYPE && difficulty < 0.2;
    let random_gap = after_hazard_while_easy
        || sinking_active
        || !(difficulty > 0.05)
        || world.resource_mut::<Random>().arc4random() & 3 != 0;
    let gap_fraction = if random_gap {
        let range = ((difficulty * 0.4 * 100.0) as i32).wrapping_add(21) as u32;
        let random_hundredths = world.resource_mut::<Random>().arc4random_mod(range);
        let hundredths = random_hundredths.wrapping_sub(20) as i32;
        hundredths as f32 / 100.0
    } else {
        difficulty * 0.2 + 0.5
    };
    let gap_width = slowest_jump_distance + (fastest_jump_distance - slowest_jump_distance) * gap_fraction;
    world.obj_mut::<CrackHoleLogic>(this).the_runs_platform_event_array[event_index].gap_width = gap_width;

    let next_last_platform_height = choose_platform_event_contents(world, this, event_index, height);

    let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
    let raised_difficulty = crack_hole.difficulty + 0.01;
    crack_hole.difficulty = if raised_difficulty > 1.0 { 1.0 } else { raised_difficulty };
    crack_hole.last_platform_height = next_last_platform_height;
    crack_hole.last_the_runs_platform_event = Some(event_index);
}

/// The second half of `chooseTheRunsPlatformEvent:`: the type, the platform count and the
/// hazards of the event in slot `event_index`, by the difficulty. Returns the height the next
/// event is drawn around: the event's own, or 50 after a sinking platform (which ends low).
fn choose_platform_event_contents(world: &mut World, this: Entity, event_index: usize, height: f32) -> f32 {
    // Plain platforms at the very start, and half the time after that. `b.le`: a NaN
    // difficulty counts as the start.
    let difficulty = world.obj::<CrackHoleLogic>(this).difficulty;
    let plain = !(difficulty > 0.03) || world.resource_mut::<Random>().arc4random() & 1 != 0;
    if plain {
        let random = world.resource_mut::<Random>().arc4random();
        let platform_count = if difficulty < 0.6 { (random & 3) + 2 } else { (random & 3) + 1 };

        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        let event = &mut crack_hole.the_runs_platform_event_array[event_index];
        event.the_runs_platform_event_type = 1;
        event.platform_count = platform_count as i32;
        return height;
    }

    let random = world.resource_mut::<Random>().arc4random();
    if random & 1 == 0 {
        let crack_hole = world.obj::<CrackHoleLogic>(this);
        let stalactites_active = crack_hole.stalactites_active;
        let sinking_active = crack_hole.sinking_active;
        if !stalactites_active && !sinking_active && height > 140.0 {
            // Six platforms under the stalactites, which fall at the speed of a run across 1230.
            let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
            crack_hole.stalactites_active = true;
            let event = &mut crack_hole.the_runs_platform_event_array[event_index];
            event.the_runs_platform_event_type = STALACTITE_EVENT_TYPE;
            event.platform_count = 6;
            event.stalactite_count = 6;

            let run_speed = (crack_hole.difficulty * 0.2 + 0.5) * 300.0 + 300.0;
            crack_hole.platform_sinking_vel = 80.0 / (1230.0 / run_speed);
            return height;
        }
        if !stalactites_active && !sinking_active && height > 100.0 {
            // Four to six sinking platforms, which sink the whole height while the pygmy runs
            // across them.
            {
                let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
                crack_hole.sinking_active = true;
                crack_hole.the_runs_platform_event_array[event_index].the_runs_platform_event_type = SINKING_EVENT_TYPE;
            }

            let platform_count = world.resource_mut::<Random>().arc4random_mod(3) | 4;
            let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
            crack_hole.the_runs_platform_event_array[event_index].platform_count = platform_count as i32;

            let run_speed = crack_hole.difficulty * 0.2 * 300.0 + 300.0;
            let run_time = (platform_count as i32 as f32 * PLATFORM_WIDTH) / run_speed;
            crack_hole.platform_sinking_vel = height / run_time;
            return 50.0;
        }

        let rock_active = world.obj::<CrackHoleLogic>(this).rock_active;
        if !rock_active {
            let difficulty = {
                let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
                crack_hole.rock_active = true;
                crack_hole.the_runs_platform_event_array[event_index].the_runs_platform_event_type = ROCK_EVENT_TYPE;
                crack_hole.difficulty
            };

            let platform_count = hazard_platform_count(world, difficulty, true);
            world.obj_mut::<CrackHoleLogic>(this).the_runs_platform_event_array[event_index].platform_count = platform_count as i32;
            return height;
        }
    }

    // A turd on the first platform.
    let difficulty = {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        let event = &mut crack_hole.the_runs_platform_event_array[event_index];
        event.the_runs_platform_event_type = TURD_EVENT_TYPE;
        event.turd_count = 1;
        crack_hole.difficulty
    };

    let platform_count = hazard_platform_count(world, difficulty, false);
    world.obj_mut::<CrackHoleLogic>(this).the_runs_platform_event_array[event_index].platform_count = platform_count as i32;
    height
}

/// The platform count of a rock (`rock`) or turd stretch: from 5 or 6 down to 3 as the
/// difficulty rises. NaN draws nothing and gives 3, like the original's `b.pl` chain.
fn hazard_platform_count(world: &mut World, difficulty: f32, rock: bool) -> u32 {
    if difficulty < 0.2 {
        let random = world.resource_mut::<Random>().arc4random();
        return (random & 1) + 5;
    }
    if difficulty < 0.4 {
        if rock {
            let random = world.resource_mut::<Random>().arc4random_mod(3);
            return random | 4;
        }
        let random = world.resource_mut::<Random>().arc4random();
        return (random & 1) | 4;
    }
    if difficulty < 0.6 {
        let random = world.resource_mut::<Random>().arc4random_mod(3);
        return random + 3;
    }
    if difficulty < 0.8 {
        let random = world.resource_mut::<Random>().arc4random();
        return (random & 1) + 3;
    }
    3
}

/// `-[CrackHoleLogic placeTheRunsPlatformLogic:]` @ 0x1001265e4
///
/// Puts `platform` at the end of the course as the next platform of the first event, from the
/// lead one on, that still has platforms to place. An event's first platform comes after the
/// event's gap and carries its turd and the lava rock; a stalactite event hangs a stalactite
/// over each of its platforms.
pub fn place_the_runs_platform_logic(world: &mut World, this: Entity, platform: Entity) {
    let platform_display_object = pg_engine::logic::display_object(world, platform);

    // There are five events and only four platforms, and an event is chosen anew once its
    // last platform has scrolled away, so one of them always has a platform left to place.
    let mut event_index = world.obj::<CrackHoleLogic>(this).lead_the_runs_platform_event_index;
    loop {
        let event = platform_event(world, this, event_index as usize);
        if event.placed_platform_count != event.platform_count {
            break;
        }
        event_index = next_event_index(event_index);
    }
    let event_slot = event_index as usize;

    let event = platform_event(world, this, event_slot);
    if event.placed_platform_count == 0 {
        {
            let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
            crack_hole.next_platform_x_pos = event.gap_width + crack_hole.next_platform_x_pos;
        }
        if event.turd_count > 0 {
            place_next_turd(world, this, event);
        }
        if event.the_runs_platform_event_type == ROCK_EVENT_TYPE {
            place_rock(world, this, event);
        }
    }

    let next_platform_x_pos = world.obj::<CrackHoleLogic>(this).next_platform_x_pos;
    {
        let mut object = display::object_mut(world, platform_display_object);
        object.pos.x = next_platform_x_pos;
        object.pos.y = event.height - PLATFORM_TOP_HEIGHT;
    }
    the_runs_platform_logic::set_the_runs_platform_event(world, platform, Some(event_slot));
    the_runs_platform_logic::set_check_side_collision(world, platform, event.placed_platform_count == 0);
    world.obj_mut::<CrackHoleLogic>(this).the_runs_platform_event_array[event_slot].placed_platform_count += 1;

    if event.the_runs_platform_event_type == STALACTITE_EVENT_TYPE {
        place_next_stalactite(world, this, event_slot);
    }

    world.obj_mut::<CrackHoleLogic>(this).next_platform_x_pos += PLATFORM_SPACING;
}

/// The turd of a turd event, `mNextTurdIndex` of the pool, somewhere over the middle 30–70 %
/// of the event's platforms (none when the pool is used up).
fn place_next_turd(world: &mut World, this: Entity, event: TheRunsPlatformEvent) {
    let (next_turd_index, turds) = {
        let crack_hole = world.obj::<CrackHoleLogic>(this);
        (crack_hole.next_turd_index, crack_hole.the_runs_turd_logic_array.clone())
    };
    // `(NSUInteger)(long)mNextTurdIndex < count`: a negative index is out of range too.
    let turd_available = (next_turd_index as i64 as u64) < turds.len() as u64;
    if !turd_available {
        return;
    }

    let turd = turds[next_turd_index as usize];
    let events_span = event.platform_count as f32 * PLATFORM_SPACING;
    let next_platform_x_pos = world.obj::<CrackHoleLogic>(this).next_platform_x_pos;
    let min_tenths = (events_span * 0.3 * 10.0) as i32;
    let max_tenths = ((events_span * 0.7 * 10.0) as i32).wrapping_add(1);
    let range = max_tenths.wrapping_sub(min_tenths) as u32;
    let random_tenths = world.resource_mut::<Random>().arc4random_mod(range);
    let offset_tenths = min_tenths.wrapping_add(random_tenths as i32);
    let turd_x_pos = next_platform_x_pos + offset_tenths as f32 / 10.0;
    the_runs_turd_logic::activate_at_x_pos(world, turd, turd_x_pos, event.height);

    world.obj_mut::<CrackHoleLogic>(this).next_turd_index += 1;
}

/// The lava rock of a rock event, somewhere over the middle 40–60 % of the event's platforms.
fn place_rock(world: &mut World, this: Entity, event: TheRunsPlatformEvent) {
    let events_span = event.platform_count as f32 * PLATFORM_SPACING;
    let (rock, next_platform_x_pos) = {
        let crack_hole = world.obj::<CrackHoleLogic>(this);
        (crack_hole.the_runs_rock_logic, crack_hole.next_platform_x_pos)
    };
    let min_tenths = (events_span * 0.4 * 10.0) as i32;
    let max_tenths = ((events_span * 0.6 * 10.0) as i32).wrapping_add(1);
    let range = max_tenths.wrapping_sub(min_tenths) as u32;
    let random_tenths = world.resource_mut::<Random>().arc4random_mod(range);
    let offset_tenths = (random_tenths as i32).wrapping_add(min_tenths);
    let rock_x_pos = next_platform_x_pos + offset_tenths as f32 / 10.0;
    if let Some(rock) = rock {
        the_runs_rock_logic::activate_at_x_pos(world, rock, rock_x_pos, event.height);
    }
}

/// Hangs the next stalactite (round-robin by the event's placed count) over the platform being
/// placed.
fn place_next_stalactite(world: &mut World, this: Entity, event_slot: usize) {
    let (placed_stalactite_count, stalactites, next_platform_x_pos) = {
        let crack_hole = world.obj::<CrackHoleLogic>(this);
        let placed_stalactite_count = crack_hole.the_runs_platform_event_array[event_slot].placed_stalactite_count;
        (placed_stalactite_count, crack_hole.the_runs_stalactite_logic_array.clone(), crack_hole.next_platform_x_pos)
    };

    // `objectAtIndex:` of an empty array would throw; the four stalactites always exist.
    if !stalactites.is_empty() {
        let stalactite_index = (placed_stalactite_count as i64 as u64 % stalactites.len() as u64) as usize;
        let stalactite = stalactites[stalactite_index];
        the_runs_stalactite_logic::activate_at_x_pos(world, stalactite, next_platform_x_pos - 16.0, 260.0);
    }

    world.obj_mut::<CrackHoleLogic>(this).the_runs_platform_event_array[event_slot].placed_stalactite_count += 1;
}

/// `-[CrackHoleLogic areCracksActive]` @ 0x1001269ac
pub fn are_cracks_active(world: &mut World, this: Entity) -> bool {
    !world.obj::<CrackHoleLogic>(this).crack_logic_array.is_empty()
}

/// `-[CrackHoleLogic startTheRuns:]` @ 0x1001269e0
///
/// The pygmy (`the_runs`) that fell into the crack starts The Runs: the course is reset to the
/// starting event (seven plain platforms at height 133) and four chosen ones, the platforms
/// are laid out, and the scenery starts.
pub fn start_the_runs(world: &mut World, this: Entity, the_runs: Entity) {
    pg_engine::view::add_game_frame_listener(world, this);
    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.the_runs = true;
        crack_hole.difficulty = 0.0;
        crack_hole.the_runs_pygmy_logic = Some(the_runs);
    }

    crate::pygmy::start_the_runs(world, the_runs);
    if let Some(listener) = listener_of(world, this) {
        crate::controller::on_the_runs_start(world, listener, this, the_runs);
    }

    let starting_event = TheRunsPlatformEvent {
        the_runs_platform_event_type: 0,
        platform_count: 7,
        height: 133.0,
        ..TheRunsPlatformEvent::default()
    };
    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.lead_the_runs_platform_event_index = 0;
        crack_hole.the_runs_platform_event_array[0] = starting_event;
        crack_hole.last_platform_height = 133.0;
        crack_hole.last_the_runs_platform_event = Some(0);
    }
    for event_index in 1..5 {
        choose_the_runs_platform_event(world, this, Some(event_index));
    }
    world.obj_mut::<CrackHoleLogic>(this).next_platform_x_pos = -14.0;

    let platforms = world.obj::<CrackHoleLogic>(this).the_runs_platform_logic_array.clone();
    for platform in platforms {
        the_runs_platform_logic::activate(world, platform);
        place_the_runs_platform_logic(world, this, platform);
    }

    let crack_hole = world.obj::<CrackHoleLogic>(this);
    let bats_logic = crack_hole.the_runs_bats_logic;
    let demon_logic = crack_hole.the_runs_demon_logic;
    let background_logic = crack_hole.the_runs_background_logic;
    let lava_background_logic = crack_hole.the_runs_lava_background_logic;
    let stalactite_background_logic = crack_hole.the_runs_stalactite_background_logic;
    let lava_foreground1_logic = crack_hole.the_runs_lava_foreground1_logic;
    let lava_foreground2_logic = crack_hole.the_runs_lava_foreground2_logic;
    set_logic_behavior(world, bats_logic, "TheRunsBatsInit");
    set_logic_behavior(world, background_logic, "TheRunsBackgroundInit");
    set_logic_behavior(world, lava_background_logic, "TheRunsLavaBackgroundInit");
    set_logic_behavior(world, stalactite_background_logic, "TheRunsStalactiteBackgroundInit");
    set_logic_behavior(world, lava_foreground1_logic, "TheRunsLavaForegroundInit");
    set_logic_behavior(world, lava_foreground2_logic, "TheRunsLavaForegroundInit");

    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.demon_on_screen = false;
        crack_hole.bats_on_screen = false;
    }

    // The demon and the bats first come 240 to 720 into the course.
    let demon_display_object = display_object_of(world, demon_logic);
    let demon_x_pos = random_tenths(world, 4801, 2400);
    set_x_pos(world, demon_display_object, demon_x_pos);

    let bats_display_object = display_object_of(world, bats_logic);
    let bats_x_pos = random_tenths(world, 4801, 2400);
    set_x_pos(world, bats_display_object, bats_x_pos);
}

/// `-[CrackHoleLogic stopTheRuns]` @ 0x100126db8
///
/// Ends The Runs (the pygmy died): every hazard and platform is taken down, the shaking stops
/// and the scenery goes blank.
pub fn stop_the_runs(world: &mut World, this: Entity) {
    let the_runs = world.obj::<CrackHoleLogic>(this).the_runs;
    if !the_runs {
        return;
    }

    world.obj_mut::<CrackHoleLogic>(this).the_runs = false;
    pg_engine::view::remove_game_frame_listener(world, this);
    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.the_runs_pygmy_logic = None;
        crack_hole.crack_hole_pygmy_logic = None;
    }

    let turds = world.obj::<CrackHoleLogic>(this).the_runs_turd_logic_array.clone();
    for turd in turds {
        the_runs_turd_logic::deactivate(world, turd);
    }

    let rock = {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.next_turd_index = 0;
        crack_hole.rock_active = false;
        crack_hole.the_runs_rock_logic
    };
    if let Some(rock) = rock {
        the_runs_rock_logic::deactivate(world, rock);
    }

    let sinking_started = {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.sinking_active = false;
        crack_hole.sinking_started
    };
    if sinking_started {
        world.obj_mut::<CrackHoleLogic>(this).sinking_started = false;
        shake_screen_stop(world, this);
    }

    let platforms = world.obj::<CrackHoleLogic>(this).the_runs_platform_logic_array.clone();
    for platform in platforms {
        the_runs_platform_logic::deactivate(world, platform);
    }

    let stalactites_started = world.obj::<CrackHoleLogic>(this).stalactites_started;
    if stalactites_started {
        world.obj_mut::<CrackHoleLogic>(this).stalactites_started = false;
        shake_screen_stop(world, this);
    }
    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.stalactites_active = false;
        crack_hole.wait_for_stalactites_to_be_off_screen = false;
    }

    let stalactites = world.obj::<CrackHoleLogic>(this).the_runs_stalactite_logic_array.clone();
    for stalactite in stalactites {
        the_runs_stalactite_logic::deactivate(world, stalactite);
    }

    if let Some(listener) = listener_of(world, this) {
        crate::controller::on_the_runs_stop(world, listener, this);
    }

    let crack_hole = world.obj::<CrackHoleLogic>(this);
    let scenery_logics = [
        crack_hole.the_runs_demon_logic,
        crack_hole.the_runs_bats_logic,
        crack_hole.the_runs_background_logic,
        crack_hole.the_runs_lava_background_logic,
        crack_hole.the_runs_stalactite_background_logic,
        crack_hole.the_runs_lava_foreground1_logic,
        crack_hole.the_runs_lava_foreground2_logic,
    ];
    for scenery_logic in scenery_logics {
        set_logic_behavior(world, scenery_logic, BLANK_BEHAVIOR);
    }
}

/// `-[CrackHoleLogic startDragging:]` @ 0x1001271f8
///
/// A finger starts dragging at `pos` (island space); the cracks start there once it has moved
/// far enough (`drag:`).
pub fn start_dragging(world: &mut World, this: Entity, pos: Vec4) {
    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.crack_opening = false;
        crack_hole.drag_x_pos = pos.x;
        crack_hole.start_crack_x_pos = pos.x;
        crack_hole.next_crack_x_pos = pos.x;
        crack_hole.crack_hole_pygmy_logic = None;
    }
    pg_engine::view::add_game_frame_listener(world, this);
}

/// `[[mCrackLogicArray objectAtIndex:0] setBehavior:@"CrackClose"]` when there is a crack: the
/// cracks close one after the other, oldest first.
fn close_first_crack(world: &mut World, this: Entity) {
    let first_crack = world.obj::<CrackHoleLogic>(this).crack_logic_array.first().copied();
    if let Some(first_crack) = first_crack {
        pg_engine::logic::set_behavior(world, first_crack, Some(CRACK_CLOSE));
    }
}

/// `-[CrackHoleLogic stopDragging]` @ 0x100127284
pub fn stop_dragging(world: &mut World, this: Entity) {
    world.obj_mut::<CrackHoleLogic>(this).dragging = false;

    let the_runs = world.obj::<CrackHoleLogic>(this).the_runs;
    if !the_runs {
        pg_engine::view::remove_game_frame_listener(world, this);
    }

    close_first_crack(world, this);
}

/// The common tail of `drag:` and of the `CrackOpen` event: once the finger has passed the
/// start of the next crack and no crack is still opening, that crack opens, provided the one
/// after it would still start on the island.
fn open_next_crack(world: &mut World, this: Entity) {
    let island_min_x_pos = island_min_x_pos(world, this);
    let island_max_x_pos = island_max_x_pos(world, this);

    let crack_hole = world.obj::<CrackHoleLogic>(this);
    let next_crack_x_pos = crack_hole.next_crack_x_pos;
    let drag_direction = crack_hole.drag_direction;
    let drag_delta = crack_hole.drag_x_pos - next_crack_x_pos;
    let crack_opening = crack_hole.crack_opening;
    let following_crack_x_pos = next_crack_x_pos + drag_direction * CRACK_WIDTH;

    let passed_next_crack = drag_delta * drag_direction > 0.0 && !crack_opening;
    let following_on_island = following_crack_x_pos >= island_min_x_pos && following_crack_x_pos <= island_max_x_pos;
    if !passed_next_crack || !following_on_island {
        return;
    }

    world.obj_mut::<CrackHoleLogic>(this).crack_opening = true;
    create_crack(world, this, next_crack_x_pos);
    world.obj_mut::<CrackHoleLogic>(this).next_crack_x_pos = following_crack_x_pos;
}

/// `-[CrackHoleLogic drag:]` @ 0x100127340
///
/// The finger moved to `pos` (island space). After 25 points of drag the chain's direction is
/// fixed and its first crack is snapped to the 67-point grid that ends at the island's edge
/// ahead; from then on the cracks open one by one as the finger passes them.
pub fn drag(world: &mut World, this: Entity, pos: Vec4) {
    let (dragging, next_crack_x_pos) = {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.drag_x_pos = pos.x;
        (crack_hole.dragging, crack_hole.next_crack_x_pos)
    };

    if !dragging {
        // `b.le`: a NaN distance does not start the chain either.
        let drag_delta = pos.x - next_crack_x_pos;
        let far_enough = drag_delta.abs() > 25.0;
        if !far_enough {
            return;
        }

        let drag_direction = drag_delta.abs() / drag_delta;
        {
            let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
            crack_hole.dragging = true;
            crack_hole.drag_direction = drag_direction;
        }

        let first_crack_x_pos = if drag_direction < 0.0 {
            let island_min_x_pos = island_min_x_pos(world, this);
            let drag_x_pos = world.obj::<CrackHoleLogic>(this).drag_x_pos;
            let grid_steps = ((drag_x_pos - island_min_x_pos) / CRACK_WIDTH).ceil();
            island_min_x_pos + grid_steps * CRACK_WIDTH
        } else {
            let island_max_x_pos = island_max_x_pos(world, this);
            let drag_x_pos = world.obj::<CrackHoleLogic>(this).drag_x_pos;
            let grid_steps = ((island_max_x_pos - drag_x_pos) / CRACK_WIDTH).ceil();
            island_max_x_pos - grid_steps * CRACK_WIDTH
        };
        world.obj_mut::<CrackHoleLogic>(this).next_crack_x_pos = first_crack_x_pos;
    }

    let dragging = world.obj::<CrackHoleLogic>(this).dragging;
    if dragging {
        open_next_crack(world, this);
    }
}

/// `[mCrackLogicArray removeObject:crack]`: the array held the crack's last reference, and the
/// crack the last one of its model, which has left the island.
fn release_crack(world: &mut World, crack: Entity, crack_display_object: Entity) {
    pg_engine::logic::dealloc(world, crack);
    world.despawn(crack_display_object);
}

/// `-[CrackHoleLogic onMultiModelEvent:eventId:]` @ 0x100127504
///
/// A crack (`logic`) finished opening (the next one may open) or closing (it goes away and the
/// next one closes).
pub fn on_multi_model_event(world: &mut World, this: Entity, logic: Entity, event_id: &str) {
    if event_id == CRACK_OPEN {
        world.obj_mut::<CrackHoleLogic>(this).crack_opening = false;
        let dragging = world.obj::<CrackHoleLogic>(this).dragging;
        if dragging {
            open_next_crack(world, this);
        }
        return;
    }

    if event_id == CRACK_CLOSE {
        let crack_display_object = pg_engine::logic::display_object(world, logic);
        display::remove_from_display_group(world, crack_display_object);
        pg_engine::logic::shutdown(world, logic);

        world.obj_mut::<CrackHoleLogic>(this).crack_logic_array.retain(|crack| *crack != logic);
        release_crack(world, logic, crack_display_object);

        close_first_crack(world, this);
    }
}

/// `[[mTheRunsTurdLogicArray objectAtIndex:0] …]`: the oldest turd is taken down and goes to
/// the back of the pool.
fn recycle_oldest_turd(world: &mut World, this: Entity) {
    let oldest_turd = world.obj::<CrackHoleLogic>(this).the_runs_turd_logic_array[0];
    the_runs_turd_logic::deactivate(world, oldest_turd);

    let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
    crack_hole.the_runs_turd_logic_array.remove(0);
    crack_hole.the_runs_turd_logic_array.push(oldest_turd);
    crack_hole.next_turd_index -= 1;
}

/// `-[CrackHoleLogic processOffScreenPlatforms]` @ 0x1001276e8
///
/// Moves each platform that scrolled off the left of the screen to the end of the course.
/// When the last platform of an event has gone, the event ends (its turd is recycled, the
/// rock taken down, the sinking stopped, or the stalactites left to scroll away) and a new
/// one is chosen in its slot. Then the stalactites that left the screen are taken down, and
/// once all have, the stalactite event is over.
pub fn process_off_screen_platforms(world: &mut World, this: Entity) {
    let platforms = world.obj::<CrackHoleLogic>(this).the_runs_platform_logic_array.clone();
    for platform in platforms {
        let platform_x_pos = logic_pos(world, platform).x;
        let screen_left_x_pos = -the_runs_scroll_x_pos(world, this);
        let off_screen = platform_x_pos + PLATFORM_WIDTH < screen_left_x_pos;
        if !off_screen {
            continue;
        }

        // The event the platform had; placing it gives it its next one.
        let event_index = the_runs_platform_logic::the_runs_platform_event(world, platform);
        place_the_runs_platform_logic(world, this, platform);
        let Some(event_index) = event_index else {
            continue;
        };

        let event = {
            let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
            let event = &mut crack_hole.the_runs_platform_event_array[event_index];
            event.off_screen_platform_count += 1;
            *event
        };
        if event.off_screen_platform_count != event.platform_count {
            continue;
        }

        match event.the_runs_platform_event_type {
            TURD_EVENT_TYPE => {
                if event.turd_count > 0 {
                    recycle_oldest_turd(world, this);
                }
            }
            ROCK_EVENT_TYPE => {
                let rock = {
                    let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
                    crack_hole.rock_active = false;
                    crack_hole.the_runs_rock_logic
                };
                if let Some(rock) = rock {
                    the_runs_rock_logic::deactivate(world, rock);
                }
            }
            SINKING_EVENT_TYPE => {
                {
                    let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
                    crack_hole.sinking_active = false;
                    crack_hole.sinking_started = false;
                }
                shake_screen_stop(world, this);
            }
            STALACTITE_EVENT_TYPE => {
                world.obj_mut::<CrackHoleLogic>(this).wait_for_stalactites_to_be_off_screen = true;
            }
            _ => {}
        }

        choose_the_runs_platform_event(world, this, Some(event_index));
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        crack_hole.lead_the_runs_platform_event_index = next_event_index(crack_hole.lead_the_runs_platform_event_index);
    }

    let stalactites_active = world.obj::<CrackHoleLogic>(this).stalactites_active;
    if !stalactites_active {
        return;
    }

    let mut stalactite_on_screen = false;
    let stalactites = world.obj::<CrackHoleLogic>(this).the_runs_stalactite_logic_array.clone();
    for stalactite in stalactites {
        if !the_runs_stalactite_logic::is_active(world, stalactite) {
            continue;
        }

        // `b.pl`: a NaN position counts as on screen.
        let stalactite_x_pos = logic_pos(world, stalactite).x;
        let screen_left_x_pos = -the_runs_scroll_x_pos(world, this);
        let off_screen = stalactite_x_pos + STALACTITE_WIDTH < screen_left_x_pos;
        if off_screen {
            the_runs_stalactite_logic::deactivate(world, stalactite);
        } else {
            stalactite_on_screen = true;
        }
    }

    let waiting = world.obj::<CrackHoleLogic>(this).wait_for_stalactites_to_be_off_screen;
    if waiting && !stalactite_on_screen {
        {
            let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
            crack_hole.stalactites_active = false;
            crack_hole.stalactites_started = false;
            crack_hole.wait_for_stalactites_to_be_off_screen = false;
        }
        shake_screen_stop(world, this);
    }
}

/// `-[CrackHoleLogic processPlatformEvents:]` @ 0x100127c50
///
/// The sinking platforms sink together once the pygmy has landed on one; the stalactites start
/// to fall (and the screen to shake) once the first of them is less than a screen ahead, and
/// fall together at the same speed down to 180.
pub fn process_platform_events(world: &mut World, this: Entity, process_platform_events: f32) {
    let dt = process_platform_events;

    let platforms = world.obj::<CrackHoleLogic>(this).the_runs_platform_logic_array.clone();
    for platform in platforms {
        let platform_display_object = pg_engine::logic::display_object(world, platform);
        let event_index = the_runs_platform_logic::the_runs_platform_event(world, platform);

        let crack_hole = world.obj::<CrackHoleLogic>(this);
        let event_type = match event_index {
            Some(event_index) => crack_hole.the_runs_platform_event_array[event_index].the_runs_platform_event_type,
            None => 0,
        };
        let sinking_started = crack_hole.sinking_started;
        let sinking_platform_y_pos = crack_hole.sinking_platform_y_pos;
        if event_type == SINKING_EVENT_TYPE && sinking_started {
            display::object_mut(world, platform_display_object).pos.y = sinking_platform_y_pos;
        }
    }

    {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        if crack_hole.sinking_started {
            let sink_distance = crack_hole.platform_sinking_vel * dt;
            crack_hole.sinking_platform_y_pos -= sink_distance;
        }
    }

    let stalactites_active = world.obj::<CrackHoleLogic>(this).stalactites_active;
    if !stalactites_active {
        return;
    }

    let stalactites = world.obj::<CrackHoleLogic>(this).the_runs_stalactite_logic_array.clone();
    for stalactite in stalactites {
        if !the_runs_stalactite_logic::is_active(world, stalactite) {
            continue;
        }

        let stalactite_display_object = pg_engine::logic::display_object(world, stalactite);
        let (stalactites_started, stalactites_y_pos) = {
            let crack_hole = world.obj::<CrackHoleLogic>(this);
            (crack_hole.stalactites_started, crack_hole.stalactites_y_pos)
        };
        if stalactites_started {
            display::object_mut(world, stalactite_display_object).pos.y = stalactites_y_pos;
            continue;
        }

        let stalactite_pos = display::object(world, stalactite_display_object).pos;
        let screen_left_x_pos = -the_runs_scroll_x_pos(world, this);
        let within_a_screen = stalactite_pos.x - SCREEN_WIDTH < screen_left_x_pos;
        if within_a_screen {
            {
                let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
                crack_hole.stalactites_started = true;
                crack_hole.stalactites_y_pos = stalactite_pos.y;
            }
            shake_screen_start(world, this);
        }
    }

    let stalactites_started = world.obj::<CrackHoleLogic>(this).stalactites_started;
    if !stalactites_started {
        return;
    }

    let stalactites_y_pos = {
        let mut crack_hole = world.obj_mut::<CrackHoleLogic>(this);
        let fall_distance = crack_hole.platform_sinking_vel * dt;
        crack_hole.stalactites_y_pos -= fall_distance;
        crack_hole.stalactites_y_pos
    };
    // Down: they stay at 180, and the shaking is stopped again every frame.
    if stalactites_y_pos < 180.0 {
        world.obj_mut::<CrackHoleLogic>(this).stalactites_y_pos = 180.0;
        shake_screen_stop(world, this);
    }
}

/// While a finger drags the cracks open, the first pygmy (of the view controller's
/// `pygmyLogicArray:`) that can fall and stands at least half a crack inside both ends of the
/// open chain falls into the crack hole.
fn drop_pygmy_into_crack(world: &mut World, this: Entity) {
    let Some(listener) = listener_of(world, this) else {
        return;
    };

    let pygmies = crate::controller::pygmy_logic_array(world, listener, this);
    for pygmy in pygmies {
        let can_fall = crate::pygmy::can_fall_into_crack_hole(world, pygmy, this);
        if !can_fall {
            continue;
        }

        let pygmy_x_pos = logic_pos(world, pygmy).x;
        let crack_hole = world.obj::<CrackHoleLogic>(this);
        let drag_direction = crack_hole.drag_direction;
        let next_crack_x_pos = crack_hole.next_crack_x_pos;
        let start_crack_x_pos = crack_hole.start_crack_x_pos;
        let inside_rightward_chain = drag_direction == 1.0
            && pygmy_x_pos <= next_crack_x_pos - HALF_CRACK_WIDTH
            && pygmy_x_pos >= start_crack_x_pos + HALF_CRACK_WIDTH;
        let inside_leftward_chain = drag_direction == -1.0
            && pygmy_x_pos >= next_crack_x_pos + HALF_CRACK_WIDTH
            && pygmy_x_pos <= start_crack_x_pos - HALF_CRACK_WIDTH;
        if inside_rightward_chain || inside_leftward_chain {
            world.obj_mut::<CrackHoleLogic>(this).crack_hole_pygmy_logic = Some(pygmy);
            crate::pygmy::fall_into_crack_hole(world, pygmy, this);
            return;
        }
    }
}

/// The scenery of The Runs follows the course's scroll: the score (a point per 3.2 of course)
/// goes to the view controller, and each layer moves back by its share of the scroll.
fn scroll_the_runs_scenery(world: &mut World, this: Entity) {
    let scroll_x_pos = the_runs_scroll_x_pos(world, this);
    let score = (scroll_x_pos * -0.3125) as i32;
    if let Some(listener) = listener_of(world, this) {
        crate::controller::on_the_runs_score(world, listener, score);
    }

    let crack_hole = world.obj::<CrackHoleLogic>(this);
    let demon_display_group = crack_hole.the_runs_demon_display_group;
    let background_display_object = display_object_of(world, crack_hole.the_runs_background_logic);
    let lava_background_display_object = display_object_of(world, crack_hole.the_runs_lava_background_logic);
    let stalactite_background_display_object = display_object_of(world, crack_hole.the_runs_stalactite_background_logic);
    let lava_foreground1_display_object = display_object_of(world, crack_hole.the_runs_lava_foreground1_logic);
    let lava_foreground2_display_object = display_object_of(world, crack_hole.the_runs_lava_foreground2_logic);

    let scroll_x_pos = the_runs_scroll_x_pos(world, this);
    set_x_pos(world, background_display_object, scroll_x_pos * -0.4);
    set_x_pos(world, demon_display_group, scroll_x_pos * -0.35);
    set_x_pos(world, stalactite_background_display_object, scroll_x_pos * -0.3);
    set_x_pos(world, lava_background_display_object, scroll_x_pos * -0.1);
    set_x_pos(world, lava_foreground1_display_object, scroll_x_pos * 0.1);
    set_x_pos(world, lava_foreground2_display_object, scroll_x_pos * 0.2);
}

/// The demon starts its walk when it comes within 80 of the right edge of the screen, and once
/// it is 80 past the left edge it moves on 1440 to 1920 for its next appearance.
fn move_demon(world: &mut World, this: Entity) {
    let crack_hole = world.obj::<CrackHoleLogic>(this);
    let demon_logic = crack_hole.the_runs_demon_logic;
    let demon_on_screen = crack_hole.demon_on_screen;
    let demon_display_group = crack_hole.the_runs_demon_display_group;
    let Some(demon_logic) = demon_logic else {
        return;
    };

    let demon_display_object = pg_engine::logic::display_object(world, demon_logic);
    let demon_x_pos = display::object(world, demon_display_object).pos.x;
    let demon_group_x_pos = display_group_x_pos(world, demon_display_group);
    let scroll_x_pos = the_runs_scroll_x_pos(world, this);
    let demon_screen_x_pos = demon_x_pos + demon_group_x_pos + scroll_x_pos;

    if demon_on_screen {
        if demon_screen_x_pos + 80.0 < 0.0 {
            world.obj_mut::<CrackHoleLogic>(this).demon_on_screen = false;
            let advance = random_tenths(world, 4801, 14400);
            let mut demon = display::object_mut(world, demon_display_object);
            demon.pos.x = advance + demon.pos.x;
        }
    } else if demon_screen_x_pos - 80.0 < SCREEN_WIDTH {
        world.obj_mut::<CrackHoleLogic>(this).demon_on_screen = true;
        pg_engine::logic::set_behavior(world, demon_logic, Some(DEMON_INIT_BEHAVIOR));
    }
}

/// The bats fly across with a random height and velocity from when they come within 40 of the
/// right edge of the screen; once they are 40 past the left edge they move on 720 to 1440.
fn move_bats(world: &mut World, this: Entity, dt: f32) {
    let crack_hole = world.obj::<CrackHoleLogic>(this);
    let bats_logic = crack_hole.the_runs_bats_logic;
    let bats_on_screen = crack_hole.bats_on_screen;
    let demon_display_group = crack_hole.the_runs_demon_display_group;
    let Some(bats_logic) = bats_logic else {
        return;
    };

    let bats_display_object = pg_engine::logic::display_object(world, bats_logic);
    let bats_pos = display::object(world, bats_display_object).pos;
    let bats_pos_vel = pg_engine::logic::logic(world, bats_logic).pos_vel;
    let demon_group_x_pos = display_group_x_pos(world, demon_display_group);
    let scroll_x_pos = the_runs_scroll_x_pos(world, this);
    let bats_screen_x_pos = bats_pos.x + demon_group_x_pos + scroll_x_pos;

    if bats_on_screen {
        {
            let mut bats = display::object_mut(world, bats_display_object);
            bats.pos.x = bats_pos.x + bats_pos_vel.x * dt;
            bats.pos.y += bats_pos_vel.y * dt;
        }
        if bats_screen_x_pos + 40.0 < 0.0 {
            world.obj_mut::<CrackHoleLogic>(this).bats_on_screen = false;
            let advance = random_tenths(world, 7201, 7200);
            let mut bats = display::object_mut(world, bats_display_object);
            bats.pos.x = advance + bats.pos.x;
        }
    } else if bats_screen_x_pos - 40.0 < SCREEN_WIDTH {
        world.obj_mut::<CrackHoleLogic>(this).bats_on_screen = true;

        let y_pos = random_tenths(world, 701, 1400);
        display::object_mut(world, bats_display_object).pos.y = y_pos;
        let x_vel = random_tenths(world, 2001, -1000);
        pg_engine::logic::logic_mut(world, bats_logic).pos_vel.x = x_vel;
        let y_vel = random_tenths(world, 801, -400);
        pg_engine::logic::logic_mut(world, bats_logic).pos_vel.y = y_vel;
    }
}

/// `-[CrackHoleLogic onGameFrame:]` @ 0x10012800c
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = pg_engine::logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let (dragging, crack_hole_pygmy_logic) = {
        let crack_hole = world.obj::<CrackHoleLogic>(this);
        (crack_hole.dragging, crack_hole.crack_hole_pygmy_logic)
    };
    if dragging && crack_hole_pygmy_logic.is_none() {
        drop_pygmy_into_crack(world, this);
    }

    let the_runs = world.obj::<CrackHoleLogic>(this).the_runs;
    if !the_runs {
        return;
    }

    scroll_the_runs_scenery(world, this);
    move_demon(world, this);
    move_bats(world, this, dt);
    process_platform_events(world, this, dt);
    process_off_screen_platforms(world, this);
}
