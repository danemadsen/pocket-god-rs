//! `ClosetTempleClosetDoorLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::display;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

use crate::logic::apocalypse_locust_closet_xenu::closet_temple_thetan_meter_logic;
use crate::logic::island_core::sprite_logic;

/// `ClosetTempleClosetDoorLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ClosetTempleClosetDoorLogic {
    /// `mClosetTempleClosetDoorLogicListener`: `@"<ClosetTempleClosetDoorLogicListener>"`
    pub closet_temple_closet_door_logic_listener: Option<Entity>,

    /// `mbInit`: `B`
    pub init: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchIslandPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_island_pos: Vec4,

    /// `mStartTouchIslandXPos`: `f`
    pub start_touch_island_x_pos: f32,

    /// `mStartDoorXPos`: `f`
    pub start_door_x_pos: f32,

    /// `mClosedDoorXPos`: `f`
    pub closed_door_x_pos: f32,

    /// `mbIsDoorOpen`: `B`
    pub is_door_open: bool,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const CLOSET_TEMPLE_CLOSET_DOOR_LOGIC: Class = Class {
    name: "ClosetTempleClosetDoorLogic",
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

/// How far the door slides open from its closed x.
const DOOR_TRAVEL: f32 = 220.0;

/// `[[ClosetTempleClosetDoorLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&CLOSET_TEMPLE_CLOSET_DOOR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[[IdolManager instance] triggerEpisodeEvent:event]`.
fn trigger_episode_event(world: &mut World, event: &str) {
    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, event);
    }
}

/// `[[mDisplayObject displayGroup] viewToDisplayObject:[touch pos]]` (zero for a nil group).
fn touch_island_pos(world: &World, this: Entity, touch: Entity) -> Vec4 {
    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    }
}

/// `[mClosetTempleClosetDoorLogicListener closetTempleThetanMeterLogic:self]`.
fn thetan_meter_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<ClosetTempleClosetDoorLogic>(this).closet_temple_closet_door_logic_listener;
    match listener {
        Some(listener) => crate::controller::closet_temple_thetan_meter_logic(world, listener, this),
        None => None,
    }
}

/// `[meter areThetansActive]` (NO for nil).
fn are_thetans_active(world: &mut World, meter: Option<Entity>) -> bool {
    match meter {
        Some(meter) => closet_temple_thetan_meter_logic::are_thetans_active(world, meter),
        None => false,
    }
}

/// `-[ClosetTempleClosetDoorLogic initWithDisplayObject:]` @ 0x10000b0b0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ClosetTempleClosetDoorLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().closet_temple_closet_door_logic = Some(this);
    logic::stop_game_frame(world, this);
}

/// `-[ClosetTempleClosetDoorLogic setClosetTempleClosetDoorLogicListener:]` @ 0x10000b11c
pub fn set_closet_temple_closet_door_logic_listener(world: &mut World, this: Entity, closet_temple_closet_door_logic_listener: Entity) {
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).closet_temple_closet_door_logic_listener = Some(closet_temple_closet_door_logic_listener);
}

/// `-[ClosetTempleClosetDoorLogic isAvailable]` @ 0x10000b178
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let door = world.obj::<ClosetTempleClosetDoorLogic>(this);
    if door.pygmy_logic.is_some() {
        return false;
    }
    door.is_door_open
}

/// `-[ClosetTempleClosetDoorLogic slamDoor]` @ 0x10000b1a8
pub fn slam_door(world: &mut World, this: Entity) {
    let (active_touch, listener) = {
        let mut door = world.obj_mut::<ClosetTempleClosetDoorLogic>(this);
        door.is_door_open = false;
        (door.active_touch, door.closet_temple_closet_door_logic_listener)
    };
    if active_touch.is_none() {
        return;
    }

    if let Some(listener) = listener {
        crate::controller::on_closet_temple_closet_door_drag_complete(world, listener, this);
    }
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).active_touch = None;
}

/// `-[ClosetTempleClosetDoorLogic fallIntoClosetTempleClosetDoor:]` @ 0x10000b204
pub fn fall_into_closet_temple_closet_door(world: &mut World, this: Entity, fall_into_closet_temple_closet_door: Entity) {
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).pygmy_logic = Some(fall_into_closet_temple_closet_door);
}

/// `-[ClosetTempleClosetDoorLogic ejectFromClosetTempleClosetDoor:]` @ 0x10000b214
pub fn eject_from_closet_temple_closet_door(world: &mut World, this: Entity, eject_from_closet_temple_closet_door: Entity) {
    let mut door = world.obj_mut::<ClosetTempleClosetDoorLogic>(this);
    if door.pygmy_logic == Some(eject_from_closet_temple_closet_door) {
        door.pygmy_logic = None;
    }
}

/// `-[ClosetTempleClosetDoorLogic isDoorOpen]` @ 0x10000b230
pub fn is_door_open(world: &mut World, this: Entity) -> bool {
    world.obj::<ClosetTempleClosetDoorLogic>(this).is_door_open
}

/// `-[ClosetTempleClosetDoorLogic minClosetXPos]` @ 0x10000b240
pub fn min_closet_x_pos(world: &mut World, this: Entity) -> f32 {
    world.obj::<ClosetTempleClosetDoorLogic>(this).closed_door_x_pos + 25.0
}

/// `-[ClosetTempleClosetDoorLogic maxClosetXPos]` @ 0x10000b258
pub fn max_closet_x_pos(world: &mut World, this: Entity) -> f32 {
    world.obj::<ClosetTempleClosetDoorLogic>(this).closed_door_x_pos + DOOR_TRAVEL + -25.0
}

/// `-[ClosetTempleClosetDoorLogic closetYPos]` @ 0x10000b27c
pub fn closet_y_pos(world: &mut World, this: Entity) -> f32 {
    let display_object = logic::display_object(world, this);
    display::object(world, display_object).pos.y + 15.0
}

/// `-[ClosetTempleClosetDoorLogic grabNewTom]` @ 0x10000b2b8
pub fn grab_new_tom(world: &mut World, this: Entity) {
    let listener = world.obj::<ClosetTempleClosetDoorLogic>(this).closet_temple_closet_door_logic_listener;
    let tom = match listener {
        Some(listener) => crate::controller::create_tom_pygmy(world, listener),
        None => None,
    };

    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).pygmy_logic = tom;
    if let Some(tom) = tom {
        crate::pygmy::start_in_closet_temple_closet_door(world, tom);
    }
}

/// `-[ClosetTempleClosetDoorLogic enterIsland:]` @ 0x10000b308
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<ClosetTempleClosetDoorLogic>(this).active {
        return;
    }
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).active = true;

    let display_object = logic::display_object(world, this);
    if !world.obj::<ClosetTempleClosetDoorLogic>(this).init {
        let door_x = display::object(world, display_object).pos.x;
        let mut door = world.obj_mut::<ClosetTempleClosetDoorLogic>(this);
        door.init = true;
        door.closed_door_x_pos = door_x;
    }

    let listener = world.obj::<ClosetTempleClosetDoorLogic>(this).closet_temple_closet_door_logic_listener;
    let tom_alive = match listener {
        Some(listener) => crate::controller::is_tom_pygmy_alive(world, listener),
        None => false,
    };
    if !tom_alive {
        grab_new_tom(world, this);
    }

    let closed_door_x_pos = {
        let mut door = world.obj_mut::<ClosetTempleClosetDoorLogic>(this);
        door.is_door_open = false;
        door.closed_door_x_pos
    };
    display::object_mut(world, display_object).pos.x = closed_door_x_pos;
    logic::set_behavior(world, this, Some("ClosetTempleClosetDoorEnterIsland"));
}

/// `-[ClosetTempleClosetDoorLogic exitIsland:]` @ 0x10000b404
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    if !world.obj::<ClosetTempleClosetDoorLogic>(this).active {
        return;
    }
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).active = false;

    // Tom (personality 2) stays in the closet.
    let pygmy = world.obj::<ClosetTempleClosetDoorLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        if crate::pygmy::persona_personality(world, pygmy) != 2 {
            crate::pygmy::eject_from_closet_temple_closet_door(world, pygmy);
            world.obj_mut::<ClosetTempleClosetDoorLogic>(this).pygmy_logic = None;
        }
    }
    logic::set_behavior(world, this, Some("ClosetTempleClosetDoorExitIsland"));
}

/// `-[ClosetTempleClosetDoorLogic exitIslandNow]` @ 0x10000b498
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ClosetTempleClosetDoorLogic onGameFrame:]` @ 0x10000b510
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let door = world.obj::<ClosetTempleClosetDoorLogic>(this).clone();

    if door.active_touch.is_none() {
        if !door.is_door_open {
            let mut object = display::object_mut(world, display_object);
            object.pos.x = object.pos.x + (door.closed_door_x_pos - object.pos.x) * 0.5;
        }
        return;
    }

    let mut door_x = (door.start_door_x_pos + door.touch_island_pos.x) - door.start_touch_island_x_pos;
    if door_x < door.closed_door_x_pos {
        door_x = door.closed_door_x_pos;
    }
    let open_door_x = door.closed_door_x_pos + DOOR_TRAVEL;
    if door_x > open_door_x {
        door_x = open_door_x;
    }
    display::object_mut(world, display_object).pos.x = door_x;

    if door_x == door.closed_door_x_pos + DOOR_TRAVEL {
        if door.is_door_open {
            return;
        }
        world.obj_mut::<ClosetTempleClosetDoorLogic>(this).is_door_open = true;
        trigger_episode_event(world, "OpenClosetDoor");

        let meter = thetan_meter_logic(world, this);
        if are_thetans_active(world, meter) {
            let held_open = match meter {
                Some(meter) => closet_temple_thetan_meter_logic::hold_closet_door(world, meter),
                None => false,
            };
            if held_open {
                return;
            }
        }

        let pygmy = world.obj::<ClosetTempleClosetDoorLogic>(this).pygmy_logic;
        let Some(pygmy) = pygmy else {
            return;
        };
        if !crate::pygmy::can_slam_closet_door(world, pygmy) {
            return;
        }

        let pygmy = world.obj::<ClosetTempleClosetDoorLogic>(this).pygmy_logic;
        if let Some(pygmy) = pygmy {
            crate::pygmy::slam_closet_door(world, pygmy);
        }
        return;
    }

    if !door.is_door_open {
        return;
    }
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).is_door_open = false;

    let meter = thetan_meter_logic(world, this);
    if are_thetans_active(world, meter) {
        if let Some(meter) = meter {
            closet_temple_thetan_meter_logic::stop_holding_closet_door(world, meter);
        }
    }
}

/// `-[ClosetTempleClosetDoorLogic touchBeganInside:allTouches:]` @ 0x10000b760
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<ClosetTempleClosetDoorLogic>(this).active_touch.is_some() {
        return;
    }
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).active_touch = Some(touch);

    let display_object = logic::display_object(world, this);
    let door_x = display::object(world, display_object).pos.x;
    let touch_island_pos = touch_island_pos(world, this, touch);
    let listener = {
        let mut door = world.obj_mut::<ClosetTempleClosetDoorLogic>(this);
        door.touch_island_pos = touch_island_pos;
        door.start_touch_island_x_pos = touch_island_pos.x;
        door.start_door_x_pos = door_x;
        door.closet_temple_closet_door_logic_listener
    };

    if let Some(listener) = listener {
        crate::controller::on_closet_temple_closet_door_drag(world, listener, this);
    }
}

/// `-[ClosetTempleClosetDoorLogic touchBeganOutside:allTouches:]` @ 0x10000b878
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ClosetTempleClosetDoorLogic touchMovedEntered:allTouches:]` @ 0x10000b87c
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<ClosetTempleClosetDoorLogic>(this).active_touch != Some(touch) {
        return;
    }

    let touch_island_pos = touch_island_pos(world, this, touch);
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).touch_island_pos = touch_island_pos;
}

/// `-[ClosetTempleClosetDoorLogic touchMovedExited:allTouches:]` @ 0x10000b91c
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<ClosetTempleClosetDoorLogic>(this).active_touch != Some(touch) {
        return;
    }

    let touch_island_pos = touch_island_pos(world, this, touch);
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).touch_island_pos = touch_island_pos;
}

/// `-[ClosetTempleClosetDoorLogic touchMovedInside:allTouches:]` @ 0x10000b9bc
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[ClosetTempleClosetDoorLogic touchMovedOutside:allTouches:]` @ 0x10000b9cc
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<ClosetTempleClosetDoorLogic>(this).active_touch != Some(touch) {
        return;
    }
    trigger_episode_event(world, "SlideClosetDoor");

    let touch_island_pos = touch_island_pos(world, this, touch);
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).touch_island_pos = touch_island_pos;
}

/// `-[ClosetTempleClosetDoorLogic touchEndedInside:allTouches:]` @ 0x10000baa8
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let (active_touch, listener) = {
        let door = world.obj::<ClosetTempleClosetDoorLogic>(this);
        (door.active_touch, door.closet_temple_closet_door_logic_listener)
    };
    if active_touch != Some(touch) {
        return;
    }

    if let Some(listener) = listener {
        crate::controller::on_closet_temple_closet_door_drag_complete(world, listener, this);
    }
    world.obj_mut::<ClosetTempleClosetDoorLogic>(this).active_touch = None;
}

/// `-[ClosetTempleClosetDoorLogic touchEndedOutside:allTouches:]` @ 0x10000bb00
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// `+[ClosetTempleClosetDoorLogic instance]` @ 0x10000b0a4
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().closet_temple_closet_door_logic
}
