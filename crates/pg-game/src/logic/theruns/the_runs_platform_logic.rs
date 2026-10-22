//! `TheRunsPlatformLogic` (superclass `SpriteLogic`).
//!
//! One of the four stone platforms of The Runs (`TheRunsDisplayGroup/Platform1…4`). The crack
//! hole (its listener) keeps moving the platforms that scrolled off screen to the end of the
//! course and gives each the event (`TheRunsPlatformEvent`) it belongs to; the pygmy lands on
//! them, runs into their sides, and sinks with the crumbling one.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::theruns::crack_hole_logic::{self, CrackHoleLogic};
use pg_engine::display;

/// `TheRunsPlatformLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TheRunsPlatformLogic {
    /// `mTheRunsPlatformLogicListener`: `@"<TheRunsPlatformLogicListener>"`
    pub the_runs_platform_logic_listener: Option<Entity>,

    /// `mBurningPygmyLogic`: `@"PygmyLogic"`
    pub burning_pygmy_logic: Option<Entity>,

    /// `mbCheckSideCollision`: `B`
    pub check_side_collision: bool,

    /// `mbSinkingStarted`: `B`
    pub sinking_started: bool,

    /// `mpTheRunsPlatformEvent`: `^{TheRunsPlatformEvent=iiiiiiiff}`
    pub the_runs_platform_event: Option<usize>,
}

pub const THE_RUNS_PLATFORM_LOGIC: Class = Class {
    name: "TheRunsPlatformLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"TheRunsPlatformInit"`
const INIT_BEHAVIOR: &str = "TheRunsPlatformInit";

/// `@"TheRunsPlatformCrumbleInit"`: the look of the sinking platform.
const CRUMBLE_INIT_BEHAVIOR: &str = "TheRunsPlatformCrumbleInit";

/// `@"TheRunsPlatformBurnPygmyInit"`
const BURN_PYGMY_INIT_BEHAVIOR: &str = "TheRunsPlatformBurnPygmyInit";

/// `@"TheRunsPlatformStumble"`
const STUMBLE_BEHAVIOR: &str = "TheRunsPlatformStumble";

/// `@"TheRunsPlatformSinking"`
const SINKING_BEHAVIOR: &str = "TheRunsPlatformSinking";

/// The animation channel that carries the burning pygmy.
const BURN_PYGMY_CHANNEL: usize = 1;

/// `mTheRunsPlatformEventType` of the sinking (crumbling) platform's event.
const SINKING_EVENT_TYPE: i32 = 4;

/// How far the platform's top is above its `pos.y`.
const TOP_HEIGHT: f32 = 227.0;

/// The platform's width from its `pos.x`.
const WIDTH: f32 = 205.0;

/// Sequence-item handlers implemented by `TheRunsPlatformLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onBurnPygmyComplete" => seq_on_burn_pygmy_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[TheRunsPlatformLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&THE_RUNS_PLATFORM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TheRunsPlatformLogic initWithDisplayObject:]` @ 0x1000d5b5c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TheRunsPlatformLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[TheRunsPlatformLogic setTheRunsPlatformLogicListener:]` @ 0x1000d5bdc
pub fn set_the_runs_platform_logic_listener(world: &mut World, this: Entity, the_runs_platform_logic_listener: Entity) {
    world.obj_mut::<TheRunsPlatformLogic>(this).the_runs_platform_logic_listener = Some(the_runs_platform_logic_listener);
}

/// `-[TheRunsPlatformLogic activate]` @ 0x1000d5c38
pub fn activate(_world: &mut World, _this: Entity) {}

/// `-[TheRunsPlatformLogic deactivate]` @ 0x1000d5c3c
pub fn deactivate(_world: &mut World, _this: Entity) {}

/// `mpTheRunsPlatformEvent->mTheRunsPlatformEventType`.
///
/// `mpTheRunsPlatformEvent` points into the `mTheRunsPlatformEventArray` of the crack hole,
/// which is this platform's listener and the only caller of `setTheRunsPlatformEvent:`; the
/// port keeps the index into that array. Before the crack hole has given the platform an event
/// (the original would read through NULL; nothing asks then) the type reads as 0.
fn event_type(world: &World, this: Entity) -> i32 {
    let platform = world.obj::<TheRunsPlatformLogic>(this);
    let (Some(crack_hole), Some(event_index)) = (platform.the_runs_platform_logic_listener, platform.the_runs_platform_event) else {
        return 0;
    };

    let events = &world.obj::<CrackHoleLogic>(crack_hole).the_runs_platform_event_array;
    events[event_index].the_runs_platform_event_type
}

/// `-[TheRunsPlatformLogic setTheRunsPlatformEvent:]` @ 0x1000d5c40
///
/// `the_runs_platform_event` is the index of the event in the crack hole's
/// `mTheRunsPlatformEventArray`.
pub fn set_the_runs_platform_event(world: &mut World, this: Entity, the_runs_platform_event: Option<usize>) {
    {
        let mut platform = world.obj_mut::<TheRunsPlatformLogic>(this);
        platform.sinking_started = false;
        platform.the_runs_platform_event = the_runs_platform_event;
    }

    let event_type = event_type(world, this);
    let behavior = if event_type == SINKING_EVENT_TYPE { CRUMBLE_INIT_BEHAVIOR } else { INIT_BEHAVIOR };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[TheRunsPlatformLogic theRunsPlatformEvent]` @ 0x1000d5c88
///
/// The index of the platform's event in the crack hole's `mTheRunsPlatformEventArray`.
pub fn the_runs_platform_event(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<TheRunsPlatformLogic>(this).the_runs_platform_event
}

/// `-[TheRunsPlatformLogic setCheckSideCollision:]` @ 0x1000d5c98
pub fn set_check_side_collision(world: &mut World, this: Entity, check_side_collision: bool) {
    world.obj_mut::<TheRunsPlatformLogic>(this).check_side_collision = check_side_collision;
}

/// `-[TheRunsPlatformLogic burnPygmy:]` @ 0x1000d5ca8
///
/// The pygmy's first landing in the cave: the platform's animation takes the pygmy on channel 1
/// and burns its feet until `onBurnPygmyComplete:::`.
pub fn burn_pygmy(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<TheRunsPlatformLogic>(this).burning_pygmy_logic = Some(pygmy_logic);

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), BURN_PYGMY_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some(BURN_PYGMY_INIT_BEHAVIOR));
}

/// `-[TheRunsPlatformLogic onBurnPygmyComplete:::]` @ 0x1000d5d28
pub fn seq_on_burn_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, BURN_PYGMY_CHANNEL);

    let burning_pygmy_logic = world.obj::<TheRunsPlatformLogic>(this).burning_pygmy_logic;
    if let Some(burning_pygmy_logic) = burning_pygmy_logic {
        crate::pygmy::the_runs_burn_pygmy_complete(world, burning_pygmy_logic);
    }
    world.obj_mut::<TheRunsPlatformLogic>(this).burning_pygmy_logic = None;

    let listener = world.obj::<TheRunsPlatformLogic>(this).the_runs_platform_logic_listener;
    if let Some(listener) = listener {
        crack_hole_logic::on_pygmy_burn_pygmy_complete(world, listener);
    }
    ItemResult::Done
}

/// `-[TheRunsPlatformLogic checkCollision:lastPos:]` @ 0x1000d5da8
///
/// The pygmy (`pygmy_logic`, at `last_pos` on the previous frame) against this platform, whose
/// top is 227 above its `pos.y` and which spans 205 from its `pos.x`. Returns 0 (not over the
/// platform), 1 (on the top: it lands or stands there), 2 (ran into the side: stopped at it),
/// 3 (above the top), 4 (the sinking platform went too deep: the pygmy falls off) or 5 (ran
/// into the side just under the top: put on the top, and the platform stumbles). Every
/// comparison is false for NaN, like the original's branches, except that a NaN height counts
/// as above the top.
pub fn check_collision(world: &mut World, this: Entity, pygmy_logic: Entity, last_pos: Vec4) -> i32 {
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy_logic);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let top_y_pos = pos.y + TOP_HEIGHT;

    // The left side, the first platform of an event only: below the top, at or past the side,
    // and on the left of it a frame ago.
    let check_side_collision = world.obj::<TheRunsPlatformLogic>(this).check_side_collision;
    let hit_side = check_side_collision && pygmy_pos.y < top_y_pos && pygmy_pos.x >= pos.x && last_pos.x < pos.x;
    if hit_side {
        if pygmy_pos.y <= top_y_pos - 15.0 {
            display::object_mut(world, pygmy_display_object).pos.x = pos.x;
            return 2;
        }

        let mut platform_y_pos = pos.y;
        if event_type(world, this) != SINKING_EVENT_TYPE {
            pg_engine::logic::set_behavior(world, this, Some(STUMBLE_BEHAVIOR));
            platform_y_pos = display::object(world, display_object).pos.y;
        }
        display::object_mut(world, pygmy_display_object).pos.y = platform_y_pos + TOP_HEIGHT;
        return 5;
    }

    let over_platform = pygmy_pos.x > pos.x && pygmy_pos.x < pos.x + WIDTH;
    if !over_platform {
        return 0;
    }
    // `b.ls`: a NaN height is above the top.
    let on_or_below_top = pygmy_pos.y <= top_y_pos;
    if !on_or_below_top {
        return 3;
    }

    let listener = world.obj::<TheRunsPlatformLogic>(this).the_runs_platform_logic_listener;
    if pos.y < -193.0 {
        if let Some(listener) = listener {
            crack_hole_logic::on_pygmy_fall_from_sinking_platform(world, listener);
        }
        return 4;
    }

    // The pygmy being burnt stays where the animation puts it.
    let burning_pygmy_logic = world.obj::<TheRunsPlatformLogic>(this).burning_pygmy_logic;
    if burning_pygmy_logic != Some(pygmy_logic) {
        display::object_mut(world, pygmy_display_object).pos.y = top_y_pos;
    }

    if event_type(world, this) == SINKING_EVENT_TYPE {
        let sinking_started = world.obj::<TheRunsPlatformLogic>(this).sinking_started;
        if !sinking_started {
            world.obj_mut::<TheRunsPlatformLogic>(this).sinking_started = true;
            pg_engine::logic::set_behavior(world, this, Some(SINKING_BEHAVIOR));
            if let Some(listener) = listener {
                crack_hole_logic::on_pygmy_land_on_sinking_platform(world, listener, this);
            }
        }
    }
    1
}

/// `-[TheRunsPlatformLogic onGameFrame:]` @ 0x1000d5fd0
///
/// Empty: the platform has none of `SpriteLogic`'s physics.
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}
