//! `ApocalypseTimeTempleLeverLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_temples::apocalypse_time_temple_platform_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `ApocalypseTimeTempleLeverLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseTimeTempleLeverLogic {
    /// `mApocalypseTimeTempleLeverLogicListener`: `@"<ApocalypseTimeTempleLeverLogicListener>"`
    pub apocalypse_time_temple_lever_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mFramePercent`: `f`
    pub frame_percent: f32,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mStartDragPos`: `{?="x"f"y"f"z"f"w"f}`
    pub start_drag_pos: Vec4,

    /// `mDragPos`: `{?="x"f"y"f"z"f"w"f}`
    pub drag_pos: Vec4,
}

pub const APOCALYPSE_TIME_TEMPLE_LEVER_LOGIC: Class = Class {
    name: "ApocalypseTimeTempleLeverLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The lever's resting frame (the middle of its animation).
const REST_FRAME_PERCENT: f32 = 0.5;

/// The drag distance that pulls the lever all the way.
const MAX_DRAG_DISTANCE: f32 = 100.0;

/// `[mApocalypseTimeTempleLeverLogicListener apocalypseTimeTemplePlatformLogic:self]` (nil for a
/// nil listener).
fn apocalypse_time_temple_platform_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<ApocalypseTimeTempleLeverLogic>(this).apocalypse_time_temple_lever_logic_listener;
    let listener = listener?;
    crate::controller::apocalypse_time_temple_platform_logic(world, listener, this)
}

/// `[[mDisplayObject displayGroup] viewToDisplayObject:[touch pos]]` (a nil group answers the
/// zero point).
fn touch_pos_in_display_group(world: &World, this: Entity, touch: Entity) -> Vec4 {
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;

    match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    }
}

/// `[[ApocalypseTimeTempleLeverLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_TIME_TEMPLE_LEVER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ApocalypseTimeTempleLeverLogic initWithDisplayObject:]` @ 0x100115514
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseTimeTempleLeverLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[ApocalypseTimeTempleLeverLogic setApocalypseTimeTempleLeverLogicListener:]` @ 0x100115550
pub fn set_apocalypse_time_temple_lever_logic_listener(world: &mut World, this: Entity, apocalypse_time_temple_lever_logic_listener: Entity) {
    world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this).apocalypse_time_temple_lever_logic_listener = Some(apocalypse_time_temple_lever_logic_listener);
}

/// `-[ApocalypseTimeTempleLeverLogic enterIsland:]` @ 0x1001155ac
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let active = world.obj::<ApocalypseTimeTempleLeverLogic>(this).active;
    if active {
        return;
    }

    {
        let mut lever = world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this);
        lever.active = true;
        lever.frame_percent = REST_FRAME_PERCENT;
    }
    logic::set_behavior(world, this, Some("ApocalypseTimeTempleLeverEnterIsland"));
}

/// `-[ApocalypseTimeTempleLeverLogic exitIsland:]` @ 0x1001155f0
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let active = world.obj::<ApocalypseTimeTempleLeverLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this).active = false;
    logic::set_behavior(world, this, Some("ApocalypseTimeTempleLeverExitIsland"));
}

/// `-[ApocalypseTimeTempleLeverLogic exitIslandNow]` @ 0x100115620
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ApocalypseTimeTempleLeverLogic onGameFrame:]` @ 0x100115698
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let lever = world.obj::<ApocalypseTimeTempleLeverLogic>(this);
    let active_touch = lever.active_touch;
    let frame_percent = lever.frame_percent;
    let start_drag_x_pos = lever.start_drag_pos.x;
    let drag_x_pos = lever.drag_pos.x;

    let new_frame_percent = if active_touch.is_none() {
        // Released: ease back to the middle.
        frame_percent + (REST_FRAME_PERCENT - frame_percent) * 0.5
    } else {
        let drag_distance = start_drag_x_pos - drag_x_pos;
        let capped_distance = if drag_distance > MAX_DRAG_DISTANCE { MAX_DRAG_DISTANCE } else { drag_distance };
        let clamped_distance = if capped_distance < -MAX_DRAG_DISTANCE { -MAX_DRAG_DISTANCE } else { capped_distance };
        let move_percent = clamped_distance / MAX_DRAG_DISTANCE;

        let platform = apocalypse_time_temple_platform_logic(world, this);
        if let Some(platform) = platform {
            apocalypse_time_temple_platform_logic::move_platform(world, platform, -move_percent);
        }
        clamped_distance / 200.0 + 0.5
    };

    world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this).frame_percent = new_frame_percent;
    let animation = logic::animation(world, this);
    pg_engine::animation::set_frame_percent(world, animation, new_frame_percent);
}

/// `-[ApocalypseTimeTempleLeverLogic touchBeganInside:allTouches:]` @ 0x1001157cc
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let lever = world.obj::<ApocalypseTimeTempleLeverLogic>(this);
    let listener = lever.apocalypse_time_temple_lever_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let active_touch = world.obj::<ApocalypseTimeTempleLeverLogic>(this).active_touch;
    if story_active || active_touch.is_some() {
        return;
    }

    world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this).active_touch = Some(touch);

    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "PullTheLever");
    }

    let start_drag_pos = touch_pos_in_display_group(world, this, touch);
    let mut lever = world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this);
    lever.start_drag_pos = start_drag_pos;
    lever.drag_pos = start_drag_pos;
}

/// `-[ApocalypseTimeTempleLeverLogic touchBeganOutside:allTouches:]` @ 0x1001158dc
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTimeTempleLeverLogic touchMovedEntered:allTouches:]` @ 0x1001158e0
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTimeTempleLeverLogic touchMovedExited:allTouches:]` @ 0x1001158e4
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTimeTempleLeverLogic touchMovedInside:allTouches:]` @ 0x1001158e8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[ApocalypseTimeTempleLeverLogic touchMovedOutside:allTouches:]` @ 0x1001158f8
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ApocalypseTimeTempleLeverLogic>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    let drag_pos = touch_pos_in_display_group(world, this, touch);
    world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this).drag_pos = drag_pos;
}

/// `-[ApocalypseTimeTempleLeverLogic touchEndedInside:allTouches:]` @ 0x100115998
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<ApocalypseTimeTempleLeverLogic>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    world.obj_mut::<ApocalypseTimeTempleLeverLogic>(this).active_touch = None;

    let platform = apocalypse_time_temple_platform_logic(world, this);
    if let Some(platform) = platform {
        apocalypse_time_temple_platform_logic::stop_moving_platform(world, platform);
    }
}

/// `-[ApocalypseTimeTempleLeverLogic touchEndedOutside:allTouches:]` @ 0x1001159fc
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
