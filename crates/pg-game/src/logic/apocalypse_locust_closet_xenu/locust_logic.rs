//! `LocustLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::math::{deg_to_rad, rad_to_deg, trunc_i32};
use pg_engine::touch::BcTouch;

use crate::controller::god_powers::ControllerGodPowers;
use crate::logic::apocalypse_locust_closet_xenu::locust_model;
use crate::logic::island_core::sprite_logic;
use crate::logic_table;

/// `LocustLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LocustLogic {
    /// `mLocustLogicListener`: `@"<LocustLogicListener>"`
    pub locust_logic_listener: Option<Entity>,

    /// `mbLocustActive`: `B`
    pub locust_active: bool,

    /// `mbLocustDrawActive`: `B`
    pub locust_draw_active: bool,

    /// `mDrawLocustPos`: `{?="x"f"y"f"z"f"w"f}`
    pub draw_locust_pos: Vec4,

    /// `mLastDrawLocustPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_draw_locust_pos: Vec4,

    /// `mpLocustVector`: `^{vector<VECTOR4, std::__1::allocator<VECTOR4> >=^{?}^{?}{__compressed_pair<VECTOR4 *, std::__1::allocator<VECTOR4> >=^{?}}}`
    ///
    /// A pointer to the path the view controller records while the player draws
    /// (`PocketGodViewController::mpLocustVector`), which keeps growing after
    /// `startDrawLocust:startPos:` hands it over. The port keeps the object that owns the
    /// vector and reads its `ControllerGodPowers::locust_vector`; `None` is the NULL pointer
    /// before the first drawing.
    pub locust_vector: Option<Entity>,

    /// `mLocustSwarmMode`: `i`
    pub locust_swarm_mode: i32,

    /// `mLocustFollowCount`: `i`
    pub locust_follow_count: i32,

    /// `mLocustPathLength`: `f`
    pub locust_path_length: f32,

    /// `mExitScreenTimer`: `f`
    pub exit_screen_timer: f32,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mbDeactivateLocusts`: `B`
    pub deactivate_locusts: bool,

    /// `mbHoldingPygmy`: `B`
    pub holding_pygmy: bool,

    /// `mbInteractingWithPygmy`: `B`
    pub interacting_with_pygmy: bool,

    /// `mThrowingPygmyLogic`: `@"PygmyLogic"`
    pub throwing_pygmy_logic: Option<Entity>,

    /// `mThrowTargetPos`: `{?="x"f"y"f"z"f"w"f}`
    pub throw_target_pos: Vec4,

    /// `mStartThrowYPos`: `f`
    pub start_throw_y_pos: f32,

    /// `mGatherSpeedScale`: `f`
    pub gather_speed_scale: f32,
}

/// The number of locust records (`0x3c`).
pub const LOCUST_COUNT: usize = 60;

/// `LocustState` (0xb0 bytes): one locust of the swarm. The record ends with a
/// `BCTextureDef` that nothing uses.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocustState {
    /// `+0x00`: 0 inactive, 1 idle, 2 gather, 3 follow, 4 hover, 5 throw gather.
    pub state: i32,

    /// `+0x04`: the order in which the locust reached the path (`mLocustFollowCount`); −1
    /// before.
    pub follow_index: i32,

    /// `+0x08`: the path point the locust flies to.
    pub path_index: i32,

    /// `+0x0c`
    pub pos: Vec4,

    /// `+0x1c`: heading in degrees.
    pub angle: f32,

    /// `+0x20`: the last random turn rate.
    pub turn_rate: f32,

    /// `+0x24`: turns clockwise towards its target heading.
    pub turn_clockwise: bool,

    /// `+0x28`: `mLocustPathLength` when the locust reached the end of the path.
    pub path_length: f32,
}

/// `LocustLogic`'s file-scope statics (`__bss`, all zero at launch).
#[derive(Resource)]
pub struct LocustLogicStatics {
    /// `DAT_1007e50a8`: the swarm.
    pub locust_states: [LocustState; LOCUST_COUNT],

    /// `DAT_1007e79e8`: the `(x, y)` of the active locusts, packed by `sendLocustPosToModel`.
    pub locust_points: [f32; LOCUST_COUNT * 2],
}

impl Default for LocustLogicStatics {
    fn default() -> Self {
        Self {
            locust_states: [LocustState::default(); LOCUST_COUNT],
            locust_points: [0.0; LOCUST_COUNT * 2],
        }
    }
}

/// `mExitScreenTimer` after any player interaction (`0x41f00000`).
const EXIT_SCREEN_TIME: f32 = 30.0;

/// `DAT_10029d828`: path-following speed by `mLocustSwarmMode == 1`.
const FOLLOW_SPEEDS: [f32; 2] = [1600.0, 600.0];

pub const LOCUST_LOGIC: Class = Class {
    name: "LocustLogic",
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

/// Sequence-item handlers implemented by `LocustLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "throwPygmy" => seq_throw_pygmy(world, this, call),
        "liftPygmy" => seq_lift_pygmy(world, this, call),
        "catchPygmy" => seq_catch_pygmy(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[LocustLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&LOCUST_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `DAT_1007e50a8[index]`, by value.
fn locust_state_at(world: &World, index: usize) -> LocustState {
    match world.get_resource::<LocustLogicStatics>() {
        Some(statics) => statics.locust_states[index],
        None => LocustState::default(),
    }
}

/// The statics for writing (created zeroed on first use).
fn statics_mut(world: &mut World) -> Mut<'_, LocustLogicStatics> {
    world.get_resource_or_insert_with(LocustLogicStatics::default)
}

/// `[[IdolManager instance] triggerEpisodeEvent:event]`.
fn trigger_episode_event(world: &mut World, event: &str) {
    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, event);
    }
}

/// `mpLocustVector->size()`.
fn locust_path_len(world: &World, this: Entity) -> usize {
    let owner = world.obj::<LocustLogic>(this).locust_vector;
    match owner {
        Some(owner) => world.obj::<ControllerGodPowers>(owner).locust_vector.len(),
        None => 0,
    }
}

/// `(*mpLocustVector)[index]`.
// PORT: past the end the original reads the vector's stale storage. It never does in play:
// a locust starts every path at its first point and the path only grows while it is followed.
// The port reads no point there (`None`), and the callers leave the locust as it is.
fn locust_path_point(world: &World, this: Entity, index: i64) -> Option<Vec4> {
    let owner = world.obj::<LocustLogic>(this).locust_vector?;
    let path = &world.obj::<ControllerGodPowers>(owner).locust_vector;
    let index = usize::try_from(index).ok()?;
    path.get(index).copied()
}

/// `*mpLocustVector`, copied.
fn locust_path(world: &World, this: Entity) -> Vec<Vec4> {
    let owner = world.obj::<LocustLogic>(this).locust_vector;
    match owner {
        Some(owner) => world.obj::<ControllerGodPowers>(owner).locust_vector.clone(),
        None => Vec::new(),
    }
}

/// The heading (degrees, `0..360`) of the direction `(dx, dy)`.
fn heading_degrees(dx: f32, dy: f32) -> f32 {
    let mut heading = rad_to_deg(dy.atan2(dx));
    if heading < 0.0 {
        heading = heading + 360.0;
    }
    heading
}

/// The turn from the locust's heading to `target_heading`, going round the locust's way.
fn turn_towards(locust: &LocustState, target_heading: f32) -> f32 {
    let mut turn = target_heading - locust.angle;
    if locust.turn_clockwise {
        if turn > 0.0 {
            turn = turn + -360.0;
        }
    } else if turn < 0.0 {
        turn = turn + 360.0;
    }
    turn
}

/// `(arc4random() % 1501 + 500) / 100`: a turn rate between 5 and 20.
fn random_turn_rate(world: &mut World) -> f32 {
    let random = world.resource_mut::<Random>().arc4random_mod(1501);
    (random + 500) as f32 / 100.0
}

/// A heading after a turn, wrapped once into `0..360`.
fn wrap_heading(heading: f32) -> f32 {
    let mut wrapped = heading;
    if wrapped > 360.0 {
        wrapped = wrapped + -360.0;
    }
    if wrapped < 0.0 {
        wrapped = wrapped + 360.0;
    }
    wrapped
}

/// `[[mDisplayObject displayGroup] viewToDisplayObject:[touch pos]]` (zero for a nil group).
fn touch_point(world: &World, this: Entity, touch: Entity) -> Vec4 {
    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_pos),
        None => Vec4::ZERO,
    }
}

/// `-[LocustLogic initWithDisplayObject:]` @ 0x10008ba54
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(LocustLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, own_display_object);
    object.col_x_min = -100000.0;
    object.col_x_max = 100000.0;
    object.col_y_min = -100000.0;
    object.col_y_max = 100000.0;
}

/// `-[LocustLogic setLocustLogicListener:]` @ 0x10008bb28
pub fn set_locust_logic_listener(world: &mut World, this: Entity, locust_logic_listener: Entity) {
    world.obj_mut::<LocustLogic>(this).locust_logic_listener = Some(locust_logic_listener);
}

/// `-[LocustLogic initLocustState:]` @ 0x10008bbc0
///
/// `locust_state` is the index of the record in `DAT_1007e50a8` the original points to.
pub fn init_locust_state(world: &mut World, _this: Entity, locust_state: usize) {
    let mut statics = statics_mut(world);
    let locust = &mut statics.locust_states[locust_state];
    locust.state = 0;
    locust.follow_index = -1;
}

/// `-[LocustLogic checkForLocustStateInactive:]` @ 0x10008bbcc
pub fn check_for_locust_state_inactive(world: &mut World, this: Entity, locust_state: usize) {
    let exit_screen_timer = world.obj::<LocustLogic>(this).exit_screen_timer;
    if !(exit_screen_timer < 0.0) {
        return;
    }

    let any_active = {
        let mut statics = statics_mut(world);
        statics.locust_states[locust_state].state = 0;
        statics.locust_states.iter().any(|locust| locust.state != 0)
    };
    if any_active {
        return;
    }

    world.obj_mut::<LocustLogic>(this).locust_active = false;
    logic::set_behavior(world, this, Some("LocustStopBuzzing"));
}

/// The idle locust's new heading after leaving the screen at one edge:
/// `(arc4random() % 901 + offset) / 10` degrees.
fn reenter_heading(world: &mut World, this: Entity, locust_state: usize, offset: i32) {
    check_for_locust_state_inactive(world, this, locust_state);

    let random = world.resource_mut::<Random>().arc4random_mod(901);
    let heading_tenths = (random as i32).wrapping_add(offset);
    statics_mut(world).locust_states[locust_state].angle = heading_tenths as f32 / 10.0;
}

/// `-[LocustLogic runLocustStateIdle:deltaTime:]` @ 0x10008bc30
pub fn run_locust_state_idle(world: &mut World, this: Entity, locust_state: usize, delta_time: f32) {
    let mut locust = locust_state_at(world, locust_state);
    locust.state = 1;

    let wander = world.resource_mut::<Random>().arc4random_mod(601);
    let wander_degrees = (wander as i32 - 300) as f32 / 10.0;
    locust.angle = wander_degrees + locust.angle;

    let (sin, cos) = deg_to_rad(locust.angle).sin_cos();
    locust.pos.x = locust.pos.x + cos * 300.0 * delta_time;
    locust.pos.y = locust.pos.y + sin * 300.0 * delta_time;
    statics_mut(world).locust_states[locust_state] = locust;

    let x = locust.pos.x;
    if x < 0.0 {
        reenter_heading(world, this, locust_state, -450);
    }
    if x > 480.0 {
        reenter_heading(world, this, locust_state, 1350);
    }

    let y = locust.pos.y;
    if y < 0.0 {
        reenter_heading(world, this, locust_state, 450);
    }
    if y > 320.0 {
        reenter_heading(world, this, locust_state, 2250);
    }
}

/// `-[LocustLogic runLocustStateGather:deltaTime:]` @ 0x10008be54
pub fn run_locust_state_gather(world: &mut World, this: Entity, locust_state: usize, delta_time: f32) {
    let Some(path_start) = locust_path_point(world, this, 0) else {
        return;
    };

    let mut locust = locust_state_at(world, locust_state);
    let dx = path_start.x - locust.pos.x;
    let dy = path_start.y - locust.pos.y;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance < 40.0 {
        let follow_count = world.obj::<LocustLogic>(this).locust_follow_count;
        locust.state = 3;
        locust.follow_index = follow_count;
        locust.path_length = 0.0;
        world.obj_mut::<LocustLogic>(this).locust_follow_count = follow_count + 1;
    }

    let target_heading = heading_degrees(dx, dy);
    let double_time = delta_time + delta_time;
    let turn = turn_towards(&locust, target_heading);

    let turn_rate = random_turn_rate(world);
    locust.turn_rate = turn_rate;
    let heading = locust.angle + turn * (double_time * turn_rate);
    locust.angle = wrap_heading(heading);

    let (sin, cos) = deg_to_rad(locust.angle).sin_cos();
    locust.pos.x = locust.pos.x + double_time * (cos * 600.0);
    locust.pos.y = locust.pos.y + double_time * (sin * 600.0);
    statics_mut(world).locust_states[locust_state] = locust;
}

/// `-[LocustLogic runLocustStateFollow:deltaTime:]` @ 0x10008c008
pub fn run_locust_state_follow(world: &mut World, this: Entity, locust_state: usize, delta_time: f32) {
    let swarm_mode = world.obj::<LocustLogic>(this).locust_swarm_mode;
    let speed = FOLLOW_SPEEDS[usize::from(swarm_mode == 1)];
    let mut travel = speed * delta_time;

    let mut locust = locust_state_at(world, locust_state);
    let mut path_index = i64::from(locust.path_index);
    let mut from = locust.pos;

    loop {
        let Some(point) = locust_path_point(world, this, path_index) else {
            break;
        };

        let dx = point.x - from.x;
        let dy = point.y - from.y;
        let distance = (dx * dx + dy * dy).sqrt();
        locust.angle = heading_degrees(dx, dy);

        travel = travel - distance;
        if travel < 0.0 {
            let overshoot = -travel / distance;
            locust.pos.x = point.x - overshoot * dx;
            locust.pos.y = point.y - overshoot * dy;
            break;
        }

        locust.pos = point;
        let last_index = locust_path_len(world, this) as i64 - 1;
        if path_index == last_index {
            locust.state = 4;
            if locust.follow_index == 0 {
                world.obj_mut::<LocustLogic>(this).locust_swarm_mode = 1;
            }
            locust.path_length = world.obj::<LocustLogic>(this).locust_path_length;
            break;
        }

        locust.path_index = (path_index as i32).wrapping_add(1);
        path_index = path_index + 1;
        from = point;
    }

    statics_mut(world).locust_states[locust_state] = locust;
}

/// `-[LocustLogic runLocustStateHover:deltaTime:]` @ 0x10008c1a4
pub fn run_locust_state_hover(world: &mut World, this: Entity, locust_state: usize, delta_time: f32) {
    let mut locust = locust_state_at(world, locust_state);
    let path_index = i64::from(locust.path_index);

    let last_index = (locust_path_len(world, this) as u64).wrapping_sub(1);
    if (path_index as u64) < last_index {
        let path_length = world.obj::<LocustLogic>(this).locust_path_length;
        let path_left = path_length - locust.path_length;
        let follow_gap = locust.follow_index as f32 * 10.0;
        if path_left > follow_gap {
            locust.state = 3;
            if locust.follow_index == 0 {
                world.obj_mut::<LocustLogic>(this).locust_swarm_mode = 0;
            }
        }
    }

    let Some(point) = locust_path_point(world, this, path_index) else {
        statics_mut(world).locust_states[locust_state] = locust;
        return;
    };

    let dx = point.x - locust.pos.x;
    let dy = point.y - locust.pos.y;
    let target_heading = heading_degrees(dx, dy);
    let turn = turn_towards(&locust, target_heading);

    let turn_rate = random_turn_rate(world);
    locust.turn_rate = turn_rate;
    let heading = locust.angle + turn * (turn_rate * delta_time);
    locust.angle = wrap_heading(heading);

    let (sin, cos) = deg_to_rad(locust.angle).sin_cos();
    locust.pos.x = locust.pos.x + cos * 600.0 * delta_time;
    locust.pos.y = locust.pos.y + sin * 600.0 * delta_time;
    statics_mut(world).locust_states[locust_state] = locust;
}

/// `-[LocustLogic runLocustStateThrowGather:deltaTime:]` @ 0x10008c374
pub fn run_locust_state_throw_gather(world: &mut World, this: Entity, locust_state: usize, delta_time: f32) {
    let (throw_target_pos, gather_speed_scale) = {
        let locust_logic = world.obj::<LocustLogic>(this);
        (locust_logic.throw_target_pos, locust_logic.gather_speed_scale)
    };

    let mut locust = locust_state_at(world, locust_state);
    let dx = throw_target_pos.x - locust.pos.x;
    let dy = throw_target_pos.y - locust.pos.y;
    let target_heading = heading_degrees(dx, dy);
    let scaled_time = gather_speed_scale * delta_time;
    let turn = turn_towards(&locust, target_heading);

    let turn_rate = random_turn_rate(world);
    locust.turn_rate = turn_rate;
    let heading = locust.angle + turn * (scaled_time * turn_rate);
    locust.angle = wrap_heading(heading);

    let (sin, cos) = deg_to_rad(locust.angle).sin_cos();
    locust.pos.x = locust.pos.x + scaled_time * (cos * 600.0);
    locust.pos.y = locust.pos.y + scaled_time * (sin * 600.0);
    statics_mut(world).locust_states[locust_state] = locust;
}

/// `-[LocustLogic runLocustState:deltaTime:]` @ 0x10008c4e8
pub fn run_locust_state(world: &mut World, this: Entity, locust_state: usize, delta_time: f32) {
    let (throwing_pygmy_logic, locust_draw_active) = {
        let locust_logic = world.obj::<LocustLogic>(this);
        (locust_logic.throwing_pygmy_logic, locust_logic.locust_draw_active)
    };

    let mut state = locust_state_at(world, locust_state).state;
    if throwing_pygmy_logic.is_some() && state != 0 {
        statics_mut(world).locust_states[locust_state].state = 5;
        run_locust_state_throw_gather(world, this, locust_state, delta_time);
        return;
    }

    if locust_draw_active {
        if state == 1 {
            {
                let mut statics = statics_mut(world);
                let locust = &mut statics.locust_states[locust_state];
                locust.state = 2;
                locust.path_index = 0;
            }

            let turn_clockwise = world.resource_mut::<Random>().arc4random() & 1 == 1;
            statics_mut(world).locust_states[locust_state].turn_clockwise = turn_clockwise;
            state = 2;
        }
    } else if state as u32 >= 2 {
        statics_mut(world).locust_states[locust_state].state = 1;
        run_locust_state_idle(world, this, locust_state, delta_time);
        return;
    }

    match state {
        1 => run_locust_state_idle(world, this, locust_state, delta_time),
        2 => run_locust_state_gather(world, this, locust_state, delta_time),
        3 => run_locust_state_follow(world, this, locust_state, delta_time),
        4 => run_locust_state_hover(world, this, locust_state, delta_time),
        5 => run_locust_state_throw_gather(world, this, locust_state, delta_time),
        _ => {}
    }
}

/// `-[LocustLogic sendLocustPosToModel]` @ 0x10008c628
pub fn send_locust_pos_to_model(world: &mut World, this: Entity) {
    let (locust_points, count) = {
        let mut statics = statics_mut(world);
        let mut count = 0;
        for index in 0..LOCUST_COUNT {
            let locust = statics.locust_states[index];
            if locust.state != 0 {
                statics.locust_points[count * 2] = locust.pos.x;
                statics.locust_points[count * 2 + 1] = locust.pos.y;
                count = count + 1;
            }
        }
        (statics.locust_points.to_vec(), count as i32)
    };

    let display_object = logic::display_object(world, this);
    locust_model::set_locust_point_array(world, display_object, locust_points, count);
}

/// `-[LocustLogic throwPygmy:]` @ 0x10008c694
pub fn throw_pygmy_with_arg(world: &mut World, this: Entity, throw_pygmy: Vec4) {
    world.obj_mut::<LocustLogic>(this).throw_target_pos = throw_pygmy;

    let throwing_pygmy_logic = world.obj::<LocustLogic>(this).throwing_pygmy_logic;
    if let Some(pygmy) = throwing_pygmy_logic {
        crate::pygmy::thrown_in_air_by_locusts(world, pygmy, this, throw_pygmy, 0.7);
    }
}

/// The throw range `(min, max)` left of a pygmy at `x`: `x − 300` (at least 100) to `x − 100`.
fn left_throw_range(x: f32) -> (f32, f32) {
    let mut min_x = x + -300.0;
    if min_x < 100.0 {
        min_x = 100.0;
    }
    (min_x, x + -100.0)
}

/// `-[LocustLogic throwPygmy]` @ 0x10008c6d4
pub fn throw_pygmy(world: &mut World, this: Entity) {
    let throwing_pygmy_logic = world.obj::<LocustLogic>(this).throwing_pygmy_logic;

    // PORT: with no thrown pygmy the original reads through a NULL `pos` and crashes.
    let Some(pygmy) = throwing_pygmy_logic else {
        return;
    };
    let pygmy_display_object = logic::display_object(world, pygmy);

    let side_random = world.resource_mut::<Random>().arc4random();
    let x = display::object(world, pygmy_display_object).pos.x;
    let near_left_edge = x + -100.0 < 100.0;

    let throw_right = (u32::from(near_left_edge) | (side_random & 1)) == 1;
    let (min_x, max_x) = if throw_right && !(x + 100.0 > 380.0) {
        let right_max = x + 300.0;
        let max_x = if right_max > 380.0 { 380.0 } else { right_max };
        (x + 100.0, max_x)
    } else {
        left_throw_range(x)
    };

    let min_tenths = trunc_i32(min_x * 10.0);
    let x_random = world.resource_mut::<Random>().arc4random();
    let max_tenths = trunc_i32(max_x * 10.0);

    let range = 1i32.wrapping_sub(min_tenths).wrapping_add(max_tenths) as u32;
    let offset = if range == 0 { x_random } else { x_random % range };
    let target_x_tenths = offset.wrapping_add(min_tenths as u32) as i32;
    world.obj_mut::<LocustLogic>(this).throw_target_pos.x = target_x_tenths as f32 / 10.0;

    let y_random = world.resource_mut::<Random>().arc4random_mod(501);
    world.obj_mut::<LocustLogic>(this).throw_target_pos.y = (y_random + 1500) as f32 / 10.0;

    let throw_target_pos = world.obj::<LocustLogic>(this).throw_target_pos;
    crate::pygmy::thrown_in_air_by_locusts(world, pygmy, this, throw_target_pos, 0.7);
}

/// `-[LocustLogic onGameFrame:]` @ 0x10008c840
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if !world.obj::<LocustLogic>(this).locust_active {
        return;
    }

    let (exit_screen_timer, deactivate_locusts) = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.exit_screen_timer = locust_logic.exit_screen_timer - dt;
        (locust_logic.exit_screen_timer, locust_logic.deactivate_locusts)
    };
    if !(exit_screen_timer > 0.0) && !deactivate_locusts {
        trigger_episode_event(world, "LocustsLeaveBored");
    }

    let (exit_screen_timer, throwing_pygmy_logic) = {
        let locust_logic = world.obj::<LocustLogic>(this);
        (locust_logic.exit_screen_timer, locust_logic.throwing_pygmy_logic)
    };
    if exit_screen_timer < 0.0 {
        if let Some(pygmy) = throwing_pygmy_logic {
            trigger_episode_event(world, "LocustsDropPygmyBored");
            crate::pygmy::dropped_in_air_by_locusts(world, pygmy, this);
            stop_being_thrown_by_locusts(world, this, pygmy);
        }
    }

    for locust_state in 0..LOCUST_COUNT {
        run_locust_state(world, this, locust_state, dt);
    }
    send_locust_pos_to_model(world, this);
}

/// `-[LocustLogic isLocustActive]` @ 0x10008c9b4
pub fn is_locust_active(world: &mut World, this: Entity) -> bool {
    world.obj::<LocustLogic>(this).locust_active
}

/// `-[LocustLogic shouldPygmyPanic]` @ 0x10008c9c4
pub fn should_pygmy_panic(world: &mut World, this: Entity) -> bool {
    if !world.obj::<LocustLogic>(this).locust_active {
        return false;
    }
    let count = active_locust_count(world, this);
    count as f32 > 30.0
}

/// `-[LocustLogic isLocustThrowingPygmy]` @ 0x10008ca0c
pub fn is_locust_throwing_pygmy(world: &mut World, this: Entity) -> bool {
    world.obj::<LocustLogic>(this).throwing_pygmy_logic.is_some()
}

/// `-[LocustLogic deactivateLocusts]` @ 0x10008ca24
pub fn deactivate_locusts(world: &mut World, this: Entity) {
    {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.exit_screen_timer = 0.0;
        locust_logic.deactivate_locusts = true;
    }
    trigger_episode_event(world, "PushLocustsOffScreen");
}

/// `-[LocustLogic canCreateNewLocusts]` @ 0x10008ca7c
pub fn can_create_new_locusts(world: &mut World, _this: Entity) -> bool {
    match world.get_resource::<LocustLogicStatics>() {
        Some(statics) => statics.locust_states.iter().any(|locust| locust.state == 0),
        None => true,
    }
}

/// `-[LocustLogic activeLocustCount]` @ 0x10008cab0
pub fn active_locust_count(world: &mut World, _this: Entity) -> i32 {
    match world.get_resource::<LocustLogicStatics>() {
        Some(statics) => statics.locust_states.iter().filter(|locust| locust.state != 0).count() as i32,
        None => 0,
    }
}

/// `-[LocustLogic addLocust:]` @ 0x10008cadc
pub fn add_locust(world: &mut World, this: Entity, locust: Vec4) {
    world.obj_mut::<LocustLogic>(this).exit_screen_timer = EXIT_SCREEN_TIME;

    if !world.obj::<LocustLogic>(this).locust_active {
        {
            let mut locust_logic = world.obj_mut::<LocustLogic>(this);
            locust_logic.locust_active = true;
            locust_logic.deactivate_locusts = false;
        }

        let listener = world.obj::<LocustLogic>(this).locust_logic_listener;
        let marker = match listener {
            Some(listener) => crate::controller::locust_display_marker(world, listener),
            None => None,
        };
        if let Some(marker) = marker {
            let display_object = logic::display_object(world, this);
            display::insert_after(world, marker, Some(display_object));
        }

        for locust_state in 0..LOCUST_COUNT {
            init_locust_state(world, this, locust_state);
        }
        logic::set_behavior(world, this, Some("LocustStartBuzzing"));
    }

    {
        let mut statics = statics_mut(world);
        let free_record = statics.locust_states.iter_mut().find(|record| record.state == 0);
        if let Some(record) = free_record {
            record.state = 1;
            record.pos = locust;
            record.angle = 135.0;
        }
    }

    if active_locust_count(world, this) != 60 {
        return;
    }
    trigger_episode_event(world, "MaxLocusts");
}

/// `-[LocustLogic startDrawLocust:startPos:]` @ 0x10008cc68
///
/// `draw_locust` stands for the original's pointer to the view controller's path vector: the
/// object that owns it (`ControllerGodPowers::locust_vector`).
pub fn start_draw_locust(world: &mut World, this: Entity, draw_locust: Entity, pos: Vec4) {
    {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.exit_screen_timer = EXIT_SCREEN_TIME;
        locust_logic.locust_swarm_mode = 0;
        locust_logic.locust_follow_count = 0;
        locust_logic.locust_vector = Some(draw_locust);
    }
    trigger_episode_event(world, "GatherLocusts");

    let (throwing_pygmy_logic, holding_pygmy) = {
        let locust_logic = world.obj::<LocustLogic>(this);
        (locust_logic.throwing_pygmy_logic, locust_logic.holding_pygmy)
    };
    if let Some(pygmy) = throwing_pygmy_logic {
        if holding_pygmy {
            logic::set_behavior(world, this, Some("LocustThrowPygmy"));
        } else {
            trigger_episode_event(world, "LocustsDropPygmy");
            crate::pygmy::dropped_in_air_by_locusts(world, pygmy, this);
            stop_being_thrown_by_locusts(world, this, pygmy);
        }
    }

    let mut locust_logic = world.obj_mut::<LocustLogic>(this);
    locust_logic.interacting_with_pygmy = false;
    locust_logic.locust_draw_active = true;
    locust_logic.draw_locust_pos = pos;
    locust_logic.last_draw_locust_pos = pos;
}

/// A pygmy message of the swipe: `cutInAirByLocusts:`, `mouthFilledByLocusts:`, …
type PygmyLocustMessage = fn(&mut World, Entity, Entity) -> bool;

/// `-[LocustLogic drawLocust:]` @ 0x10008cdc4
pub fn draw_locust(world: &mut World, this: Entity, draw_locust: Vec4) {
    {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.exit_screen_timer = EXIT_SCREEN_TIME;
        locust_logic.draw_locust_pos = draw_locust;
        locust_logic.locust_path_length = 0.0;
    }

    // Original quirk, kept: the segment into the newest point is never counted.
    let path = locust_path(world, this);
    let mut path_length = 0.0;
    let counted_points = path.len().saturating_sub(1);
    for index in 1..counted_points {
        let dx = path[index - 1].x - path[index].x;
        let dy = path[index - 1].y - path[index].y;
        path_length = (dx * dx + dy * dy).sqrt() + path_length;
    }
    world.obj_mut::<LocustLogic>(this).locust_path_length = path_length;

    if path_length > 100.0 {
        trigger_episode_event(world, "DrawLocustLine");
    }

    let (throwing_pygmy_logic, interacting_with_pygmy, listener) = {
        let locust_logic = world.obj::<LocustLogic>(this);
        (
            locust_logic.throwing_pygmy_logic,
            locust_logic.interacting_with_pygmy,
            locust_logic.locust_logic_listener,
        )
    };
    if throwing_pygmy_logic.is_none() && !interacting_with_pygmy {
        let pygmies = match listener {
            Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
            None => Vec::new(),
        };

        for pygmy in pygmies {
            let pygmy_display_object = logic::display_object(world, pygmy);
            let pygmy_pos = display::object(world, pygmy_display_object).pos;

            let box_left = pygmy_pos.x + -32.0;
            let box_right = pygmy_pos.x + 32.0;
            if !(draw_locust.x >= box_left) || draw_locust.x > box_right {
                continue;
            }

            let box_bottom = pygmy_pos.y + 0.0;
            let box_top = pygmy_pos.y + 50.0;
            if !(draw_locust.y >= box_bottom) || draw_locust.y > box_top {
                continue;
            }

            if crate::pygmy::cut_in_air_by_locusts(world, pygmy, this) {
                world.obj_mut::<LocustLogic>(this).interacting_with_pygmy = true;
                show_locust_slash(world, this, draw_locust);
                return;
            }

            let last_draw_locust_pos = world.obj::<LocustLogic>(this).last_draw_locust_pos;
            if last_draw_locust_pos.y < box_bottom {
                if !crate::pygmy::start_thrown_in_air_by_locusts(world, pygmy, this) {
                    continue;
                }

                let thrown_pygmy_pos = display::object(world, pygmy_display_object).pos;
                {
                    let mut locust_logic = world.obj_mut::<LocustLogic>(this);
                    locust_logic.interacting_with_pygmy = true;
                    locust_logic.holding_pygmy = true;
                    locust_logic.locust_draw_active = false;
                    locust_logic.throw_target_pos = thrown_pygmy_pos;
                    locust_logic.throw_target_pos.y = locust_logic.throw_target_pos.y + 20.0;
                    locust_logic.throwing_pygmy_logic = Some(pygmy);
                    locust_logic.gather_speed_scale = 2.0;
                }

                let display_object = logic::display_object(world, this);
                display::object_mut(world, display_object).is_selectable = true;
                logic::set_behavior(world, this, Some("LocustStartThrowPygmy"));
                return;
            }

            let pygmy_scale_x = display::object(world, pygmy_display_object).scale.x;
            let pygmy_message: PygmyLocustMessage = if last_draw_locust_pos.y > box_top {
                crate::pygmy::mutated_by_locusts
            } else if last_draw_locust_pos.x < box_left {
                if pygmy_scale_x < 0.0 {
                    crate::pygmy::mouth_filled_by_locusts
                } else {
                    crate::pygmy::strip_flesh_by_locusts
                }
            } else if last_draw_locust_pos.x > box_right {
                if pygmy_scale_x < 0.0 {
                    crate::pygmy::strip_flesh_by_locusts
                } else {
                    crate::pygmy::mouth_filled_by_locusts
                }
            } else {
                continue;
            };

            if pygmy_message(world, pygmy, this) {
                world.obj_mut::<LocustLogic>(this).interacting_with_pygmy = true;
                return;
            }
        }
    }

    world.obj_mut::<LocustLogic>(this).last_draw_locust_pos = draw_locust;
}

/// `drawLocust:`'s cut: the `LocustSlash` line along the swipe and the `LocustSplat` at the
/// finger (logic entries 462 and 463), both after the listener's `locustDisplayMarker`.
fn show_locust_slash(world: &mut World, this: Entity, draw_locust: Vec4) {
    let listener = world.obj::<LocustLogic>(this).locust_logic_listener;
    let marker = match listener {
        Some(listener) => crate::controller::locust_display_marker(world, listener),
        None => None,
    };

    let slash = library::logic(world, logic_table::LOCUST_SLASH);
    let slash_display_object = logic::display_object(world, slash);
    let last_draw_locust_pos = world.obj::<LocustLogic>(this).last_draw_locust_pos;
    let swipe_dx = draw_locust.x - last_draw_locust_pos.x;
    let swipe_dy = draw_locust.y - last_draw_locust_pos.y;
    let slash_angle = rad_to_deg(swipe_dy.atan2(swipe_dx));
    {
        let mut object = display::object_mut(world, slash_display_object);
        object.rot.z = slash_angle;
        object.pos = draw_locust;
        object.pos.z = SPRITE_Z;
    }
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(slash_display_object));
    }
    logic::set_behavior(world, slash, Some("LocustSlash"));

    let splat = library::logic(world, logic_table::LOCUST_SPLAT);
    let splat_display_object = logic::display_object(world, splat);
    {
        let mut object = display::object_mut(world, splat_display_object);
        object.pos = draw_locust;
        object.pos.z = SPRITE_Z;
    }
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(splat_display_object));
    }
    logic::set_behavior(world, splat, Some("LocustSplat"));
}

/// `-[LocustLogic drawLocustComplete]` @ 0x10008d43c
pub fn draw_locust_complete(world: &mut World, this: Entity) {
    world.obj_mut::<LocustLogic>(this).locust_draw_active = false;
}

/// `-[LocustLogic throwPygmy:::]` @ 0x10008d44c
pub fn seq_throw_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let throwing_pygmy_logic = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.exit_screen_timer = EXIT_SCREEN_TIME;
        locust_logic.gather_speed_scale = 2.0;
        if call.first_time {
            locust_logic.start_throw_y_pos = locust_logic.throw_target_pos.y + 0.0;
        }
        locust_logic.throwing_pygmy_logic
    };

    // PORT: with no thrown pygmy the original reads through a NULL `pos` and crashes; the
    // behaviour is always replaced by `LocustIdle` when the pygmy is let go.
    let Some(pygmy) = throwing_pygmy_logic else {
        return ItemResult::Done;
    };

    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = {
        let mut object = display::object_mut(world, pygmy_display_object);
        object.pos.y = object.pos.y + call.delta_time * 400.0;
        object.pos
    };

    let (throw_target_y, start_throw_y_pos) = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.throw_target_pos = pygmy_pos;
        locust_logic.throw_target_pos.y = locust_logic.throw_target_pos.y + 20.0;
        (locust_logic.throw_target_pos.y, locust_logic.start_throw_y_pos)
    };
    if !(throw_target_y > start_throw_y_pos) {
        return ItemResult::Pending;
    }

    let draw_locust_pos = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.holding_pygmy = false;
        locust_logic.draw_locust_pos
    };
    throw_pygmy_with_arg(world, this, draw_locust_pos);
    ItemResult::Done
}

/// `-[LocustLogic liftPygmy:::]` @ 0x10008d588
pub fn seq_lift_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let throwing_pygmy_logic = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.exit_screen_timer = EXIT_SCREEN_TIME;
        locust_logic.gather_speed_scale = 1.0;
        if call.first_time {
            locust_logic.start_throw_y_pos = locust_logic.throw_target_pos.y + 100.0;
        }
        locust_logic.throwing_pygmy_logic
    };

    // PORT: with no thrown pygmy the original reads through a NULL `pos` and crashes; the
    // behaviour is always replaced by `LocustIdle` when the pygmy is let go.
    let Some(pygmy) = throwing_pygmy_logic else {
        return ItemResult::Done;
    };

    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = {
        let mut object = display::object_mut(world, pygmy_display_object);
        object.pos.y = object.pos.y + call.delta_time * 100.0;
        object.pos
    };

    let (throw_target_y, start_throw_y_pos) = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.throw_target_pos = pygmy_pos;
        locust_logic.throw_target_pos.y = locust_logic.throw_target_pos.y + 20.0;
        (locust_logic.throw_target_pos.y, locust_logic.start_throw_y_pos)
    };
    if throw_target_y > start_throw_y_pos {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[LocustLogic catchPygmy:::]` @ 0x10008d694
pub fn seq_catch_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let throwing_pygmy_logic = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.exit_screen_timer = EXIT_SCREEN_TIME;
        locust_logic.gather_speed_scale = 1.0;
        locust_logic.throwing_pygmy_logic
    };

    // PORT: with no thrown pygmy the original reads through a NULL `pos` and crashes; the
    // behaviour is always replaced by `LocustIdle` when the pygmy is let go.
    let Some(pygmy) = throwing_pygmy_logic else {
        return ItemResult::Done;
    };

    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let catch_x = pygmy_pos.x;
    let catch_y = pygmy_pos.y + 20.0;

    let throw_target_pos = world.obj::<LocustLogic>(this).throw_target_pos;
    let dx = throw_target_pos.x - catch_x;
    let dy = throw_target_pos.y - catch_y;
    if !((dx * dx + dy * dy).sqrt() < 10.0) {
        return ItemResult::Pending;
    }

    let y_pos_vel = logic::logic(world, pygmy).pos_vel.y;
    if !(y_pos_vel <= 0.0) {
        return ItemResult::Pending;
    }

    {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.holding_pygmy = true;
        locust_logic.throw_target_pos = Vec4::new(catch_x, catch_y, pygmy_pos.z, pygmy_pos.w);
    }
    crate::pygmy::catch_in_air_by_locusts(world, pygmy, this);
    ItemResult::Done
}

/// `-[LocustLogic stopBeingThrownByLocusts:]` @ 0x10008d7b0
pub fn stop_being_thrown_by_locusts(world: &mut World, this: Entity, _being_thrown_by_locusts: Entity) {
    let active_touch = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.throwing_pygmy_logic = None;
        locust_logic.active_touch
    };
    if active_touch.is_none() {
        let display_object = logic::display_object(world, this);
        display::object_mut(world, display_object).is_selectable = false;
    }
    logic::set_behavior(world, this, Some("LocustIdle"));

    let mut statics = statics_mut(world);
    for locust in statics.locust_states.iter_mut() {
        if locust.state != 0 {
            locust.state = 1;
        }
    }
}

/// `-[LocustLogic touchBeganInside:allTouches:]` @ 0x10008d850
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<LocustLogic>(this).active_touch.is_some() {
        return;
    }
    world.obj_mut::<LocustLogic>(this).active_touch = Some(touch);

    let point = touch_point(world, this, touch);
    let listener = world.obj::<LocustLogic>(this).locust_logic_listener;
    if let Some(listener) = listener {
        crate::controller::start_draw_locust(world, listener, point);
    }
}

/// `-[LocustLogic touchBeganOutside:allTouches:]` @ 0x10008d90c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[LocustLogic touchMovedEntered:allTouches:]` @ 0x10008d910
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[LocustLogic touchMovedExited:allTouches:]` @ 0x10008d914
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[LocustLogic touchMovedInside:allTouches:]` @ 0x10008d918
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[LocustLogic touchMovedOutside:allTouches:]` @ 0x10008d928
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<LocustLogic>(this).active_touch != Some(touch) {
        return;
    }

    let point = touch_point(world, this, touch);
    let listener = world.obj::<LocustLogic>(this).locust_logic_listener;
    if let Some(listener) = listener {
        crate::controller::draw_locust(world, listener, point);
    }
}

/// `-[LocustLogic touchEndedInside:allTouches:]` @ 0x10008d9e8
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<LocustLogic>(this).active_touch != Some(touch) {
        return;
    }

    let throwing_pygmy_logic = {
        let mut locust_logic = world.obj_mut::<LocustLogic>(this);
        locust_logic.active_touch = None;
        locust_logic.throwing_pygmy_logic
    };
    if throwing_pygmy_logic.is_none() {
        let display_object = logic::display_object(world, this);
        display::object_mut(world, display_object).is_selectable = false;
    }

    let point = touch_point(world, this, touch);
    let listener = world.obj::<LocustLogic>(this).locust_logic_listener;
    if let Some(listener) = listener {
        crate::controller::stop_draw_locust(world, listener, point);
    }
}

/// `-[LocustLogic touchEndedOutside:allTouches:]` @ 0x10008dae0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
