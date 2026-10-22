//! `ThetanLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::{deg_to_rad, rad_to_deg};
use pg_engine::touch::BcTouch;

use crate::logic::apocalypse_locust_closet_xenu::closet_temple_thetan_meter_logic;
use crate::logic::island_core::sprite_logic;

/// `ThetanLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ThetanLogic {
    /// `mThetanLogicListener`: `@"<ThetanLogicListener>"`
    pub thetan_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchTargetPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_target_pos: Vec4,

    /// `mHoldClosetOpen`: `B`
    pub hold_closet_open: bool,

    /// `mClosetPos`: `{?="x"f"y"f"z"f"w"f}`
    pub closet_pos: Vec4,

    /// `mMoveToTarget`: `B`
    pub move_to_target: bool,

    /// `mMoveToTargetTime`: `f`
    pub move_to_target_time: f32,

    /// `mMoveRot`: `f`
    pub move_rot: f32,

    /// `mMoveTarget`: `f`
    pub move_target: f32,

    /// `mMoveDirection`: `f`
    pub move_direction: f32,

    /// `mRotateTime`: `f`
    pub rotate_time: f32,

    /// `mStayAliveTime`: `f`
    pub stay_alive_time: f32,

    /// `mbApocalypseMode`: `B`
    pub apocalypse_mode: bool,

    /// `mbPossessionActive`: `B`
    pub possession_active: bool,

    /// `mPossessedPygmyLogic`: `@"PygmyLogic"`
    pub possessed_pygmy_logic: Option<Entity>,
}

pub const THETAN_LOGIC: Class = Class {
    name: "ThetanLogic",
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

/// `[[ThetanLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&THETAN_LOGIC)).id();
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

/// `arc4random() % 3601 / 10`: a random heading in degrees.
fn random_move_rot(world: &mut World) -> f32 {
    let random = world.resource_mut::<Random>().arc4random_mod(3601);
    random as f32 / 10.0
}

/// The new heading after leaving the screen at one edge: `(arc4random() % 901 + offset) / 10`
/// degrees, with the turn rate reset.
fn reenter_heading(world: &mut World, this: Entity, offset: i32) {
    world.obj_mut::<ThetanLogic>(this).move_target = 0.0;

    let random = world.resource_mut::<Random>().arc4random_mod(901);
    let heading_tenths = (random as i32).wrapping_add(offset);
    world.obj_mut::<ThetanLogic>(this).move_rot = heading_tenths as f32 / 10.0;
}

/// Puts the thetan back where its possessed pygmy stands (`(px, py + 20)`), selectable.
fn appear_above(world: &mut World, this: Entity, pygmy: Entity) {
    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.pos.x = pygmy_pos.x + 0.0;
    object.pos.y = pygmy_pos.y + 20.0;
    object.is_selectable = true;
}

/// `-[ThetanLogic initWithDisplayObject:]` @ 0x10019b9c0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ThetanLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, own_display_object);
        object.is_selectable = true;
        object.col_x_min = -20.0;
        object.col_x_max = 20.0;
        object.col_y_min = -20.0;
        object.col_y_max = 20.0;
    }

    {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.move_target = 1.0;
        thetan.move_direction = 1.0;
    }
    let move_rot = random_move_rot(world);

    let mut thetan = world.obj_mut::<ThetanLogic>(this);
    thetan.move_rot = move_rot;
    thetan.stay_alive_time = 5.0;
}

/// `-[ThetanLogic setThetanLogicListener:]` @ 0x10019bb08
pub fn set_thetan_logic_listener(world: &mut World, this: Entity, thetan_logic_listener: Entity) {
    world.obj_mut::<ThetanLogic>(this).thetan_logic_listener = Some(thetan_logic_listener);
}

/// `-[ThetanLogic canHoldCloset]` @ 0x10019bbc8
pub fn can_hold_closet(world: &mut World, this: Entity) -> bool {
    let thetan = world.obj::<ThetanLogic>(this);
    if thetan.possessed_pygmy_logic.is_some() {
        return false;
    }
    thetan.active_touch.is_none()
}

/// `-[ThetanLogic setApocalypseMode:]` @ 0x10019bbf8
pub fn set_apocalypse_mode(world: &mut World, this: Entity, apocalypse_mode: bool) {
    world.obj_mut::<ThetanLogic>(this).apocalypse_mode = apocalypse_mode;

    let possessed_pygmy_logic = world.obj::<ThetanLogic>(this).possessed_pygmy_logic;
    if let Some(pygmy) = possessed_pygmy_logic {
        appear_above(world, this, pygmy);
        crate::pygmy::thetan_stop_possessing_pygmy(world, pygmy, this);

        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.possessed_pygmy_logic = None;
        thetan.possession_active = false;
    }

    let behavior = if world.obj::<ThetanLogic>(this).apocalypse_mode {
        "ThetanInitApocalypse"
    } else {
        "ThetanInit"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ThetanLogic exitScreen]` @ 0x10019bd0c
pub fn exit_screen(world: &mut World, this: Entity) {
    let mut thetan = world.obj_mut::<ThetanLogic>(this);
    if thetan.possessed_pygmy_logic.is_none() {
        thetan.apocalypse_mode = false;
        thetan.stay_alive_time = 0.0;
    }
}

/// `-[ThetanLogic exitScreenNow]` @ 0x10019bd38
///
/// Ends with the listener's `onThetanDie:`, which deallocates this thetan.
pub fn exit_screen_now(world: &mut World, this: Entity) {
    let (possessed_pygmy_logic, apocalypse_mode, listener) = {
        let thetan = world.obj::<ThetanLogic>(this);
        (thetan.possessed_pygmy_logic, thetan.apocalypse_mode, thetan.thetan_logic_listener)
    };

    if let Some(pygmy) = possessed_pygmy_logic {
        if apocalypse_mode {
            return;
        }
        crate::pygmy::thetan_stop_possessing_pygmy(world, pygmy, this);
    }

    if let Some(listener) = listener {
        closet_temple_thetan_meter_logic::on_thetan_die(world, listener, this);
    }
}

/// `-[ThetanLogic refreshDisplayMarker]` @ 0x10019bdb0
pub fn refresh_display_marker(world: &mut World, this: Entity) {
    stop_holding_closet_door(world, this);

    let listener = world.obj::<ThetanLogic>(this).thetan_logic_listener;
    let marker = match listener {
        Some(listener) => closet_temple_thetan_meter_logic::locust_display_marker(world, listener),
        None => None,
    };
    if let Some(marker) = marker {
        let display_object = logic::display_object(world, this);
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[ThetanLogic holdClosetOpenAtXPos:yPos:]` @ 0x10019be14
pub fn hold_closet_open_at_x_pos(world: &mut World, this: Entity, hold_closet_open_at_x_pos: f32, y_pos: f32) {
    {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        if thetan.possessed_pygmy_logic.is_some() {
            return;
        }
        thetan.hold_closet_open = true;
        thetan.closet_pos.x = hold_closet_open_at_x_pos;
        thetan.closet_pos.y = y_pos;
    }
    trigger_episode_event(world, "ThetansHoldClosetDoorOpen");
}

/// `-[ThetanLogic stopHoldingClosetDoor]` @ 0x10019be8c
pub fn stop_holding_closet_door(world: &mut World, this: Entity) {
    {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        if thetan.possessed_pygmy_logic.is_some() || !thetan.hold_closet_open {
            return;
        }
        thetan.hold_closet_open = false;
    }

    let move_rot = random_move_rot(world);
    world.obj_mut::<ThetanLogic>(this).move_rot = move_rot;
}

/// `-[ThetanLogic onGameFrame:]` @ 0x10019bf04
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }
    if world.obj::<ThetanLogic>(this).possession_active {
        return;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.move_to_target_time = thetan.move_to_target_time - dt;
        thetan.stay_alive_time = thetan.stay_alive_time - dt;
        thetan.rotate_time = thetan.rotate_time - dt;
        if !(thetan.rotate_time > 0.0) {
            thetan.rotate_time = 2.0;
            thetan.move_direction = -thetan.move_direction;
        }
    }

    let (active_touch, hold_closet_open, move_to_target, move_to_target_time) = {
        let thetan = world.obj::<ThetanLogic>(this);
        (thetan.active_touch, thetan.hold_closet_open, thetan.move_to_target, thetan.move_to_target_time)
    };
    if active_touch.is_some() {
        return;
    }

    let target = if hold_closet_open {
        world.obj::<ThetanLogic>(this).closet_pos
    } else if move_to_target {
        if !(move_to_target_time > 0.0) {
            world.obj_mut::<ThetanLogic>(this).move_to_target = false;

            let possessed_pygmy_logic = world.obj::<ThetanLogic>(this).possessed_pygmy_logic;
            if possessed_pygmy_logic.is_some() {
                world.obj_mut::<ThetanLogic>(this).possession_active = true;
                logic::set_behavior(world, this, Some("ThetanPossessPygmy"));

                let possessed_pygmy_logic = world.obj::<ThetanLogic>(this).possessed_pygmy_logic;
                if let Some(pygmy) = possessed_pygmy_logic {
                    crate::pygmy::possessed_by_thetan(world, pygmy, this);
                }
            }
        }
        world.obj::<ThetanLogic>(this).touch_target_pos
    } else {
        fly_free(world, this, display_object, dt);
        return;
    };

    let mut object = display::object_mut(world, display_object);
    object.pos.x = object.pos.x + (target.x - object.pos.x) * 0.1;
    object.pos.y = object.pos.y + (target.y - object.pos.y) * 0.1;
}

/// `onGameFrame:`'s free flight: a wobbling heading at 150 per second, re-aimed at the screen
/// edges, until the stay-alive time is over (outside the apocalypse) and the thetan flies off
/// the screen and dies.
fn fly_free(world: &mut World, this: Entity, display_object: Entity, dt: f32) {
    let move_target = {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.move_target = thetan.move_target + (thetan.move_direction - thetan.move_target) * 0.05;
        thetan.move_target
    };

    let wobble = world.resource_mut::<Random>().arc4random_mod(51);
    let move_rot = {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.move_rot = move_target * (wobble as f32 / 10.0) + thetan.move_rot;
        thetan.move_rot
    };

    let (sin, cos) = deg_to_rad(move_rot).sin_cos();
    let (x, y) = {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + cos * 150.0 * dt;
        object.pos.y = object.pos.y + sin * 150.0 * dt;
        (object.pos.x, object.pos.y)
    };

    let (stay_alive_time, apocalypse_mode, listener) = {
        let thetan = world.obj::<ThetanLogic>(this);
        (thetan.stay_alive_time, thetan.apocalypse_mode, thetan.thetan_logic_listener)
    };
    if !(stay_alive_time > 0.0) && !apocalypse_mode {
        let off_screen = x < -20.0 || x > 500.0 || y < -20.0 || y > 340.0;
        if !off_screen {
            return;
        }

        trigger_episode_event(world, "ThetansFlyOffScreen");
        if let Some(listener) = listener {
            closet_temple_thetan_meter_logic::on_thetan_die(world, listener, this);
        }
        return;
    }

    if x < 0.0 {
        reenter_heading(world, this, -450);
    }
    if x > 480.0 {
        reenter_heading(world, this, 1350);
    }
    if y < 0.0 {
        reenter_heading(world, this, 450);
    }
    if y > 320.0 {
        reenter_heading(world, this, 2250);
    }
}

/// `-[ThetanLogic touchBeganInside:allTouches:]` @ 0x10019c34c
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<ThetanLogic>(this).thetan_logic_listener;
    let story_active = match listener {
        Some(listener) => closet_temple_thetan_meter_logic::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        if thetan.hold_closet_open || thetan.active_touch.is_some() {
            return;
        }
        thetan.active_touch = Some(touch);
    }

    if let Some(listener) = listener {
        closet_temple_thetan_meter_logic::on_thetan_drag(world, listener, this);
    }
}

/// `-[ThetanLogic touchBeganOutside:allTouches:]` @ 0x10019c3dc
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ThetanLogic touchMovedEntered:allTouches:]` @ 0x10019c3e0
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ThetanLogic touchMovedExited:allTouches:]` @ 0x10019c3e4
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ThetanLogic touchMovedInside:allTouches:]` @ 0x10019c3e8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[ThetanLogic touchMovedOutside:allTouches:]` @ 0x10019c3f8
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ThetanLogic touchEndedInside:allTouches:]` @ 0x10019c3fc
///
/// The thetan flies to the release point and possesses the first pygmy there.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let (active_touch, apocalypse_mode, listener) = {
        let thetan = world.obj::<ThetanLogic>(this);
        (thetan.active_touch, thetan.apocalypse_mode, thetan.thetan_logic_listener)
    };
    if active_touch != Some(touch) {
        return;
    }

    let drag_event = if apocalypse_mode {
        "DragApocalypseThetans"
    } else {
        "DragThetan"
    };
    trigger_episode_event(world, drag_event);
    if let Some(listener) = listener {
        closet_temple_thetan_meter_logic::on_thetan_drag_complete(world, listener, this);
    }

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let touch_target_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    };
    {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.touch_target_pos = touch_target_pos;
        thetan.move_to_target = true;
        thetan.move_target = 0.0;
    }

    let pos = display::object(world, display_object).pos;
    let heading = (-(pos.y - touch_target_pos.y)).atan2(-(pos.x - touch_target_pos.x));
    {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.move_rot = rad_to_deg(heading);
        thetan.move_to_target_time = 0.3;
        thetan.active_touch = None;
    }

    let pygmies = match listener {
        Some(listener) => closet_temple_thetan_meter_logic::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        if !crate::pygmy::can_be_possessed_by_thetan(world, pygmy) {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let touch_target_pos = world.obj::<ThetanLogic>(this).touch_target_pos;
        let dx = touch_target_pos.x - (pygmy_pos.x + 0.0);
        let dy = touch_target_pos.y - (pygmy_pos.y + 20.0);
        if !(dx * dx + dy * dy < 1600.0) {
            continue;
        }

        display::object_mut(world, display_object).is_selectable = false;
        world.obj_mut::<ThetanLogic>(this).possessed_pygmy_logic = Some(pygmy);
        crate::pygmy::start_possessed_by_thetan(world, pygmy, this);
        return;
    }
}

/// `-[ThetanLogic touchEndedOutside:allTouches:]` @ 0x10019c74c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// `-[ThetanLogic stopPossessingPygmy:]` @ 0x10019c75c
pub fn stop_possessing_pygmy(world: &mut World, this: Entity, possessing_pygmy: Entity) {
    if world.obj::<ThetanLogic>(this).possessed_pygmy_logic != Some(possessing_pygmy) {
        return;
    }

    appear_above(world, this, possessing_pygmy);
    let apocalypse_mode = {
        let mut thetan = world.obj_mut::<ThetanLogic>(this);
        thetan.possessed_pygmy_logic = None;
        thetan.possession_active = false;
        thetan.apocalypse_mode
    };

    let behavior = if apocalypse_mode {
        "ThetanStopPossessionApocalypse"
    } else {
        "ThetanStopPossession"
    };
    logic::set_behavior(world, this, Some(behavior));
}
