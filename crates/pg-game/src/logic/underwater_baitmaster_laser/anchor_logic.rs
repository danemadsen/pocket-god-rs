//! `AnchorLogic` (superclass `SpriteLogic`).
//!
//! The anchor of the underwater ruins. Dropping it onto the sea floor starts Bait Master, during
//! which it sends in piranhas. Its listener is the view controller.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::underwater_baitmaster_laser::piranha_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `AnchorLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct AnchorLogic {
    /// `mAnchorLogicListener`: `@"<AnchorLogicListener>"`
    pub anchor_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,

    /// `mbBaitMasterActive`: `B`
    pub bait_master_active: bool,

    /// `mNextPiranhaTime`: `f`
    pub next_piranha_time: f32,

    /// `mTimeBetweenPiranha`: `f`
    pub time_between_piranha: f32,

    /// `mArrowLogic`: `@"BCLogic"`
    pub arrow_logic: Option<Entity>,

    /// `mArrowDisplayObject`: `@"BCDisplayObject"`
    pub arrow_display_object: Option<Entity>,
}

pub const ANCHOR_LOGIC: Class = Class {
    name: "AnchorLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
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

/// The sea floor the anchor rests on.
const FLOOR_Y_POS: f32 = 57.0;

/// Water gravity, set again every frame.
const GRAVITY_FORCE: Vec4 = Vec4::new(0.0, -60.0, 0.0, 1.0);

/// Piranhas enter from just off the left or the right edge (`DAT_10029d608`).
const PIRANHA_START_X_POSITIONS: [f32; 2] = [-120.0, 600.0];

/// A new piranha comes only while fewer than this many swim.
const MAX_PIRANHA_COUNT: usize = 5;

/// Sequence-item handlers implemented by `AnchorLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        "drag" => seq_drag(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[AnchorLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ANCHOR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[AnchorLogic initWithDisplayObject:]` @ 0x10005403c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(AnchorLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[AnchorLogic setAnchorLogicListener:]` @ 0x1000540bc
pub fn set_anchor_logic_listener(world: &mut World, this: Entity, anchor_logic_listener: Entity) {
    world.obj_mut::<AnchorLogic>(this).anchor_logic_listener = Some(anchor_logic_listener);
}

/// `-[AnchorLogic pygmyLogicArray:]` @ 0x100054118
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<AnchorLogic>(this).anchor_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[AnchorLogic piranhaLogicArray:]` @ 0x10005413c
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn piranha_logic_array(world: &mut World, this: Entity, _piranha_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<AnchorLogic>(this).anchor_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::piranha_logic_array(world, listener, this)
}

/// `-[AnchorLogic removeDisplayObjects]` @ 0x100054160
///
/// Drops the warning arrow and takes every piranha off the screen; the listener then releases
/// them.
pub fn remove_display_objects(world: &mut World, this: Entity) {
    let (arrow_display_object, arrow_logic, listener) = {
        let mut anchor = world.obj_mut::<AnchorLogic>(this);
        let arrow_display_object = anchor.arrow_display_object.take();
        let arrow_logic = anchor.arrow_logic.take();
        (arrow_display_object, arrow_logic, anchor.anchor_logic_listener)
    };

    if let Some(arrow_display_object) = arrow_display_object {
        display::remove_from_display_group(world, arrow_display_object);
    }
    if let Some(arrow_logic) = arrow_logic {
        logic::shutdown(world, arrow_logic);
        logic::dealloc(world, arrow_logic);
    }

    // The anchor and the arrow's logic were the arrow model's only owners.
    if let Some(arrow_display_object) = arrow_display_object {
        world.despawn(arrow_display_object);
    }

    let piranhas = match listener {
        Some(listener) => crate::controller::piranha_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for piranha in piranhas {
        let piranha_display_object = logic::display_object(world, piranha);
        display::remove_from_display_group(world, piranha_display_object);
        logic::shutdown(world, piranha);
    }

    if let Some(listener) = listener {
        crate::controller::on_remove_all_piranha_logics(world, listener, this);
    }
}

/// `-[AnchorLogic onGameFrame:]` @ 0x100054348
///
/// Sinks under water gravity (pulled by the finger while dragged), stays on screen and rests on
/// the sea floor; landing there unheld starts Bait Master. During Bait Master it sends in a
/// piranha whenever the countdown runs out and there is room.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;
    let rot_vel = state.rot_vel;

    let previous_y_pos = display::object(world, display_object).pos.y;

    let ignore_gravity = {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.gravity_force = GRAVITY_FORCE;
        sprite.ignore_gravity
    };
    let (apply_spring, spring_force) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.apply_spring, anchor.spring_force)
    };

    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !ignore_gravity {
        pos_force.x = GRAVITY_FORCE.x + pos_force.x;
        pos_force.y = GRAVITY_FORCE.y + pos_force.y;
        pos_force.z = GRAVITY_FORCE.z + pos_force.z;
    }
    if apply_spring {
        pos_force.x = spring_force.x + pos_force.x;
        pos_force.y = spring_force.y + pos_force.y;
        pos_force.z = spring_force.z + pos_force.z;
    }

    let pos_vel = {
        let mut state = logic::logic_mut(world, this);
        state.pos_force = pos_force;
        state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
        state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
        state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
        state.pos_vel
    };

    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
        object.rot.x = object.rot.x + rot_vel.x * dt;
        object.rot.y = object.rot.y + rot_vel.y * dt;
        object.rot.z = object.rot.z + rot_vel.z * dt;
    }

    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    if min_pos.x < 0.0 {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = (object.pos.x - min_pos.x) + 0.0;
    }
    if max_pos.x > 480.0 {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = 480.0 - (max_pos.x - object.pos.x);
    }

    let y_pos = display::object(world, display_object).pos.y;
    if y_pos < FLOOR_Y_POS {
        let (active_touch, bait_master_active, active, listener) = {
            let anchor = world.obj::<AnchorLogic>(this);
            (anchor.active_touch, anchor.bait_master_active, anchor.active, anchor.anchor_logic_listener)
        };

        let landed = active_touch.is_none() && previous_y_pos > FLOOR_Y_POS;
        if landed && !bait_master_active && active {
            {
                let mut anchor = world.obj_mut::<AnchorLogic>(this);
                anchor.bait_master_active = true;
                anchor.time_between_piranha = 8.0;
                anchor.next_piranha_time = 0.0;
            }
            logic::set_behavior(world, this, Some("AnchorBounce"));
            if let Some(listener) = listener {
                crate::controller::on_bait_master_start(world, listener, this);
            }
        }

        logic::logic_mut(world, this).pos_vel.y = 0.0;
        display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;
    }

    let bait_master_active = world.obj::<AnchorLogic>(this).bait_master_active;
    if !bait_master_active {
        return;
    }

    let next_piranha_time = world.obj::<AnchorLogic>(this).next_piranha_time;
    if next_piranha_time <= 0.0 {
        let listener = world.obj::<AnchorLogic>(this).anchor_logic_listener;
        let piranha_count = match listener {
            Some(listener) => crate::controller::piranha_logic_array(world, listener, this).len(),
            None => 0,
        };
        if piranha_count < MAX_PIRANHA_COUNT {
            send_in_piranha(world, this);
        }
    }

    let mut anchor = world.obj_mut::<AnchorLogic>(this);
    anchor.next_piranha_time = anchor.next_piranha_time - dt;
}

/// The spawning half of `onGameFrame:`: schedules the next piranha (random within a second of
/// `mTimeBetweenPiranha`, which shrinks toward 2 s), then sends one in from a random side at a
/// random height, with the warning arrow on that side.
fn send_in_piranha(world: &mut World, this: Entity) {
    let time_between_piranha = world.obj::<AnchorLogic>(this).time_between_piranha;
    let delay_random = world.resource_mut::<Random>().arc4random();
    let next_piranha_time = logic::random_scaled_time(time_between_piranha - 1.0, time_between_piranha + 1.0, 10.0, delay_random);
    world.obj_mut::<AnchorLogic>(this).next_piranha_time = next_piranha_time;

    let shortened_time = time_between_piranha - 0.15;
    let new_time_between_piranha = if shortened_time < 2.0 { 2.0 } else { shortened_time };
    world.obj_mut::<AnchorLogic>(this).time_between_piranha = new_time_between_piranha;

    let piranha_model = display::multi_model::spawn(world, Some("Piranha"));
    let side_random = world.resource_mut::<Random>().arc4random();
    let from_right = side_random & 1 != 0;
    let start_x_pos = PIRANHA_START_X_POSITIONS[(side_random & 1) as usize];
    let height_random = world.resource_mut::<Random>().arc4random();
    let start_y_pos = ((height_random % 1801 + 700) as i32) as f32 / 10.0;

    {
        let mut object = display::object_mut(world, piranha_model);
        object.is_selectable = true;
        object.origin_pos.x = -20.0;
        object.pos.x = start_x_pos;
        object.pos.y = start_y_pos;
        object.pos.z = SPRITE_Z;
        object.col_x_min = -30.0;
        object.col_x_max = 30.0;
        object.col_y_min = -25.0;
        object.col_y_max = 25.0;
    }

    let marker = pg_engine::library::display_marker(world, "UnderwaterPiranhaDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(piranha_model));
    }

    let piranha = piranha_logic::spawn(world, piranha_model);
    piranha_logic::set_piranha_logic_listener(world, piranha, this);
    let listener = world.obj::<AnchorLogic>(this).anchor_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_new_piranha_logic(world, listener, piranha);
    }

    let (arrow_display_object, arrow_logic) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.arrow_display_object, anchor.arrow_logic)
    };
    let (arrow_x_pos, arrow_x_scale) = if from_right { (450.0, 1.0) } else { (30.0, -1.0) };
    if let Some(arrow_display_object) = arrow_display_object {
        let mut object = display::object_mut(world, arrow_display_object);
        object.pos.x = arrow_x_pos;
        object.scale.x = arrow_x_scale;
        object.pos.y = start_y_pos;
    }
    if let Some(marker) = marker {
        display::insert_after(world, marker, arrow_display_object);
    }
    if let Some(arrow_logic) = arrow_logic {
        logic::set_behavior(world, arrow_logic, Some("PiranhaWarningArrow"));
    }
}

/// `-[AnchorLogic onPiranhaOffScreenAtePygmy:]` @ 0x100054a94
pub fn on_piranha_off_screen_ate_pygmy(world: &mut World, this: Entity, piranha_off_screen_ate_pygmy: Entity) {
    let piranha = piranha_off_screen_ate_pygmy;
    let piranha_display_object = logic::display_object(world, piranha);
    display::remove_from_display_group(world, piranha_display_object);
    logic::shutdown(world, piranha);

    let listener = world.obj::<AnchorLogic>(this).anchor_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_remove_piranha_logic(world, listener, piranha);
        crate::controller::on_piranha_eat_pygmy(world, listener, this);
    }
}

/// `-[AnchorLogic onPiranhaDie:]` @ 0x100054b28
pub fn on_piranha_die(world: &mut World, this: Entity, piranha_die: Entity) {
    let piranha = piranha_die;
    let piranha_display_object = logic::display_object(world, piranha);
    display::remove_from_display_group(world, piranha_display_object);
    logic::shutdown(world, piranha);

    let listener = world.obj::<AnchorLogic>(this).anchor_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_remove_piranha_logic(world, listener, piranha);
    }
}

/// `-[AnchorLogic stopBaitMaster]` @ 0x100054b9c
pub fn stop_bait_master(world: &mut World, this: Entity) {
    let (bait_master_active, listener) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.bait_master_active, anchor.anchor_logic_listener)
    };
    if !bait_master_active {
        return;
    }
    world.obj_mut::<AnchorLogic>(this).bait_master_active = false;

    let piranhas = match listener {
        Some(listener) => crate::controller::piranha_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for piranha in piranhas {
        crate::send::stop_bait_master(world, Some(piranha));
    }

    if let Some(listener) = listener {
        crate::controller::on_bait_master_stop(world, listener, this);
    }
}

/// `-[AnchorLogic pickedUpByWhirlPool:]` @ 0x100054cf4
///
/// The whirlpool never takes the anchor, but its pull ends Bait Master.
pub fn picked_up_by_whirl_pool(world: &mut World, this: Entity, _picked_up_by_whirl_pool: Entity) -> bool {
    stop_bait_master(world, this);
    false
}

/// `-[AnchorLogic onPiranhaFight:]` @ 0x100054d18
pub fn on_piranha_fight(world: &mut World, this: Entity, _piranha_fight: Entity) {
    let bait_master_active = world.obj::<AnchorLogic>(this).bait_master_active;
    if bait_master_active {
        stop_bait_master(world, this);
    }
}

/// `-[AnchorLogic activate]` @ 0x100054d3c
///
/// Creates the piranha warning arrow and brings the anchor in.
pub fn activate(world: &mut World, this: Entity) {
    let arrow_display_object = display::multi_model::spawn(world, Some("PiranhaArrow"));
    world.obj_mut::<AnchorLogic>(this).arrow_display_object = Some(arrow_display_object);
    let arrow_logic = pg_engine::logic::multi_model::spawn(world, arrow_display_object);
    world.obj_mut::<AnchorLogic>(this).arrow_logic = Some(arrow_logic);
    display::object_mut(world, arrow_display_object).pos.z = SPRITE_Z;

    let display_object = logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    logic::start_game_frame(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("AnchorTransitionIn"));
}

/// `-[AnchorLogic deactivate]` @ 0x100054e80
pub fn deactivate(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("AnchorTransitionOut"));
    remove_display_objects(world, this);

    let (bait_master_active, listener) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.bait_master_active, anchor.anchor_logic_listener)
    };
    if bait_master_active {
        world.obj_mut::<AnchorLogic>(this).bait_master_active = false;
        if let Some(listener) = listener {
            crate::controller::on_bait_master_stop(world, listener, this);
        }
    }

    let active_touch = world.obj::<AnchorLogic>(this).active_touch;
    if active_touch.is_some() {
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        {
            let mut anchor = world.obj_mut::<AnchorLogic>(this);
            anchor.active_touch = None;
            anchor.apply_spring = false;
        }
        if let Some(listener) = listener {
            crate::controller::on_anchor_drag_complete(world, listener, this);
        }
    }
}

/// `-[AnchorLogic enterIsland:]` @ 0x100054f80
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let (active, transition_in) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.active, anchor.transition_in)
    };
    if active {
        return;
    }
    world.obj_mut::<AnchorLogic>(this).active = true;
    if transition_in {
        activate(world, this);
    }
}

/// `-[AnchorLogic exitIsland]` @ 0x100054fbc
pub fn exit_island(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.active, anchor.transition_in)
    };
    if !active {
        return;
    }
    world.obj_mut::<AnchorLogic>(this).active = false;
    if transition_in {
        deactivate(world, this);
    }
}

/// `-[AnchorLogic transitionIn]` @ 0x100054ff4
pub fn transition_in(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.active, anchor.transition_in)
    };
    if transition_in {
        return;
    }
    world.obj_mut::<AnchorLogic>(this).transition_in = true;
    if active {
        activate(world, this);
    }
}

/// `-[AnchorLogic transitionOut]` @ 0x100055030
pub fn transition_out(world: &mut World, this: Entity) {
    let (active, transition_in) = {
        let anchor = world.obj::<AnchorLogic>(this);
        (anchor.active, anchor.transition_in)
    };
    if !transition_in {
        return;
    }
    world.obj_mut::<AnchorLogic>(this).transition_in = false;
    if active {
        deactivate(world, this);
    }
}

/// `-[AnchorLogic die:::]` @ 0x100055068
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[AnchorLogic drag:::]` @ 0x10005509c
///
/// A damped spring toward the finger (stiffness 50, damping 6), applied by `onGameFrame:`.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        logic::logic_mut(world, this).rot_vel.z = 0.0;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = logic::display_object(world, this);
    let (pos, display_group) = {
        let object = display::object(world, display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<AnchorLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    let offset = Vec4::new(pos.x - touch_pos.x, pos.y - touch_pos.y, pos.z - touch_pos.z, pos.w);
    let distance = length3(offset);
    if distance != 0.0 {
        let pos_vel = logic::logic(world, this).pos_vel;
        let stiffness = (distance * -50.0) / distance;
        let spring_force = Vec4::new(
            -6.0 * pos_vel.x + stiffness * offset.x,
            -6.0 * pos_vel.y + stiffness * offset.y,
            -6.0 * pos_vel.z + stiffness * offset.z,
            offset.w,
        );
        world.obj_mut::<AnchorLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[AnchorLogic touchBeganInside:allTouches:]` @ 0x10005526c
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let listener = {
        let mut anchor = world.obj_mut::<AnchorLogic>(this);
        anchor.touch_screen_pos = touch_pos;
        anchor.apply_spring = true;
        anchor.active_touch = Some(touch);
        anchor.anchor_logic_listener
    };
    if let Some(listener) = listener {
        crate::controller::on_anchor_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("AnchorDrag"));
}

/// `-[AnchorLogic touchBeganOutside:allTouches:]` @ 0x100055320
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[AnchorLogic touchMovedEntered:allTouches:]` @ 0x100055324
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[AnchorLogic touchMovedExited:allTouches:]` @ 0x100055328
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[AnchorLogic touchMovedInside:allTouches:]` @ 0x10005532c
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[AnchorLogic touchMovedOutside:allTouches:]` @ 0x10005533c
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<AnchorLogic>(this).active_touch;
    let touch_is_ours = active_touch.is_none() || active_touch == Some(touch);
    if !touch_is_ours {
        return;
    }
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<AnchorLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[AnchorLogic touchEndedInside:allTouches:]` @ 0x100055394
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<AnchorLogic>(this).active_touch;
    if active_touch.is_none() || active_touch != Some(touch) {
        return;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let listener = {
        let mut anchor = world.obj_mut::<AnchorLogic>(this);
        anchor.active_touch = None;
        anchor.apply_spring = false;
        anchor.anchor_logic_listener
    };
    if let Some(listener) = listener {
        crate::controller::on_anchor_drag_complete(world, listener, this);
    }
    logic::set_behavior(world, this, Some("AnchorFall"));
}

/// `-[AnchorLogic touchEndedOutside:allTouches:]` @ 0x100055430
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
