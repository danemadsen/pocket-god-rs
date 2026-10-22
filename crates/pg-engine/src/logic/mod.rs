//! `BCLogic`, the base of every logic object (`docs/re/logic.md`).
//!
//! A logic object is an entity with [`BcLogic`] (plus one component per subclass) bound to a
//! display object. It owns a `BCAnimation` player (its listener is the logic) and, lazily, a
//! behaviour `BCSequence` (its delegate is the logic).

pub mod button;
pub mod message;
pub mod multi_model;
pub mod screen;

use bevy_ecs::prelude::*;
use bevy_math::Vec4;

use crate::animation::FrameRef;
use crate::display;
use crate::math::{VEC4_ORIGIN, trunc_i32};
use crate::object::{Class, Isa, ObjectWorldExt, is_kind_of, isa, try_isa};
use crate::random::Random;
use crate::sequence::{self, ItemResult, SequenceCall};
use crate::view;

/// `BCLogic` ivars (defaults from `initWithDisplayObject:` @ 0x10013841c).
#[derive(Component, Clone, Debug)]
pub struct BcLogic {
    pub shutdown: bool,
    pub display_object: Entity,
    pub game_frame_running: bool,

    /// The owned `BCAnimation` player ([`NO_ANIMATION`] until the initialiser creates it).
    pub animation: Entity,

    /// The behaviour sequence, created by the first `setBehavior:`.
    pub sequence: Option<Entity>,

    pub hibernate: bool,
    pub hibernate_game_frame_running: bool,
    pub pos_vel: Vec4,
    pub pos_force: Vec4,
    pub rot_vel: Vec4,
    pub rot_force: Vec4,

    /// Scratch state shared by `stall`, `tweenAlpha`, `countDownTimer` and subclass timers.
    pub stall_time: f32,

    pub delta_value: f32,
    pub last_move_x_pos: f32,
    pub last_move_y_pos: f32,
}

/// `mAnimation` while it is still nil: `initWithDisplayObject:` sends `setLogic:` (which may
/// hibernate the logic) before it creates the player.
pub const NO_ANIMATION: Entity = Entity::PLACEHOLDER;

/// `stall` draws `minTime`/`maxTime` durations in whole milliseconds.
const STALL_TIME_SCALE: f32 = 1000.0;

pub const BC_LOGIC: Class = Class {
    name: "BCLogic",
    superclass: None,
    perform,
    sequence_item,
    game_frame_order: 40,
    shutdown: shutdown_base,
    set_behavior: set_behavior_base,
    apply_move_pos: apply_move_pos_base,
    id: id_base,
    ..Class::NS_OBJECT
};

/// `-initWithDisplayObject:` @ 0x10013841c on an entity that already carries its `Isa`.
/// Subclass initialisers call this first (it is their `[super initWithDisplayObject:]`).
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    // `alloc`: every ivar is zero and `mAnimation` / `mSequence` are nil. The live-logic
    // counter `DAT_1007d1f14` only feeds the debug log `+traceGlobalStatus`.
    let allocated = BcLogic {
        shutdown: false,
        display_object,
        game_frame_running: false,
        animation: NO_ANIMATION,
        sequence: None,
        hibernate: false,
        hibernate_game_frame_running: false,
        pos_vel: Vec4::ZERO,
        pos_force: Vec4::ZERO,
        rot_vel: Vec4::ZERO,
        rot_force: Vec4::ZERO,
        stall_time: 0.0,
        delta_value: 0.0,
        last_move_x_pos: 0.0,
        last_move_y_pos: 0.0,
    };
    world.entity_mut(this).insert(allocated);

    // Hibernates or wakes the logic at once, while it has no animation yet.
    display::set_logic(world, display_object, Some(this));

    {
        let mut state = logic_mut(world, this);
        state.pos_vel = VEC4_ORIGIN;
        state.pos_force = VEC4_ORIGIN;
        state.rot_vel = VEC4_ORIGIN;
        state.rot_force = VEC4_ORIGIN;
        state.game_frame_running = false;
    }

    let player = crate::animation::spawn(world);
    logic_mut(world, this).animation = player;
    crate::animation::set_animation_listener(world, player, Some(this));
    let hibernating = logic(world, this).hibernate;
    if hibernating {
        crate::animation::hibernate(world, player);
    }

    let mut state = logic_mut(world, this);
    state.sequence = None;
    state.stall_time = 0.0;
    state.delta_value = 0.0;
}

/// Allocates a logic object of `class` and runs `BCLogic`'s initialiser.
pub fn spawn(world: &mut World, class: &'static Class, display_object: Entity) -> Entity {
    let entity = world.spawn(Isa(class)).id();
    init_with_display_object(world, entity, display_object);
    entity
}

pub fn logic(world: &World, this: Entity) -> &BcLogic {
    world.obj::<BcLogic>(this)
}

pub fn logic_mut(world: &mut World, this: Entity) -> Mut<'_, BcLogic> {
    world.obj_mut::<BcLogic>(this)
}

/// `mAnimation`, or `None` while it is nil (during `initWithDisplayObject:`).
fn created_animation(world: &World, this: Entity) -> Option<Entity> {
    let player = logic(world, this).animation;
    if player == NO_ANIMATION {
        return None;
    }
    Some(player)
}

/// The listener and its class, to send it a protocol method; `None` for a nil listener, whose
/// messages are dropped as in Objective-C.
pub fn listener_class(world: &World, listener: Option<Entity>) -> Option<(Entity, &'static Class)> {
    let listener = listener?;
    let class = try_isa(world, listener)?;
    Some((listener, class))
}

/// `-displayObject`.
pub fn display_object(world: &World, this: Entity) -> Entity {
    logic(world, this).display_object
}

/// `-animation`.
pub fn animation(world: &World, this: Entity) -> Entity {
    logic(world, this).animation
}

/// `-setBehavior:` (dispatched; `OutHouseLogic` overrides).
pub fn set_behavior(world: &mut World, this: Entity, behavior: Option<&str>) {
    let class = isa(world, this);
    (class.set_behavior)(world, this, behavior);
}

/// `BCLogic::setBehavior:` @ 0x1001385f8 (including the hibernation quirk of logic.md §3.4).
pub fn set_behavior_base(world: &mut World, this: Entity, behavior: Option<&str>) {
    // `getSequenceDefById:` looks a nil id up as "", which names no sequence (NULL).
    let behavior_id = behavior.unwrap_or("");
    let def = crate::library::sequence_def(world, behavior_id);

    let existing = logic(world, this).sequence;
    if let Some(running) = existing {
        sequence::set_sequence_def(world, running, def);
        return;
    }

    let created = sequence::spawn(world, def, Some(this));
    let hibernating = {
        let mut state = logic_mut(world, this);
        state.sequence = Some(created);
        state.hibernate
    };
    if hibernating {
        // Original quirk: this sets the logic's own deferred game-frame flag, so the logic
        // starts its game frame at the next unhibernate even if it had stopped it.
        logic_mut(world, this).hibernate_game_frame_running = true;
        sequence::hibernate(world, created);
    }
}

/// `-playAnimationId:` @ 0x100138738.
pub fn play_animation_id(world: &mut World, this: Entity, id: Option<&str>) {
    let animation = animation(world, this);
    crate::animation::play_animation_id(world, animation, id);
}

/// `-startGameFrame` @ 0x100138754.
pub fn start_game_frame(world: &mut World, this: Entity) {
    {
        let mut state = logic_mut(world, this);
        if state.hibernate {
            state.hibernate_game_frame_running = true;
            return;
        }
        if state.game_frame_running {
            return;
        }
        state.game_frame_running = true;
    }
    view::add_game_frame_listener(world, this);
}

/// `-stopGameFrame` @ 0x1001387e0.
pub fn stop_game_frame(world: &mut World, this: Entity) {
    {
        let mut state = logic_mut(world, this);
        if state.hibernate {
            state.hibernate_game_frame_running = false;
            return;
        }
        if !state.game_frame_running {
            return;
        }
        state.game_frame_running = false;
    }
    view::remove_game_frame_listener(world, this);
}

/// `-isHibernating` @ 0x100138864.
pub fn is_hibernating(world: &World, this: Entity) -> bool {
    logic(world, this).hibernate
}

/// `-hibernate` @ 0x100138874.
pub fn hibernate(world: &mut World, this: Entity) {
    let was_running = {
        let mut state = logic_mut(world, this);
        if state.hibernate {
            return;
        }
        state.hibernate = true;
        let was_running = state.game_frame_running;
        state.hibernate_game_frame_running = was_running;
        state.game_frame_running = false;
        was_running
    };
    if was_running {
        view::remove_game_frame_listener(world, this);
    }

    if let Some(player) = created_animation(world, this) {
        crate::animation::hibernate(world, player);
    }
    let behavior = logic(world, this).sequence;
    if let Some(behavior) = behavior {
        sequence::hibernate(world, behavior);
    }
}

/// `-unhibernate` @ 0x100138940 (keeps `mbHibernateGameFrameRunning`).
pub fn unhibernate(world: &mut World, this: Entity) {
    let resume_game_frame = {
        let mut state = logic_mut(world, this);
        if !state.hibernate {
            return;
        }
        state.hibernate = false;
        state.hibernate_game_frame_running
    };
    if resume_game_frame {
        start_game_frame(world, this);
    }

    if let Some(player) = created_animation(world, this) {
        crate::animation::unhibernate(world, player);
    }
    let behavior = logic(world, this).sequence;
    if let Some(behavior) = behavior {
        sequence::unhibernate(world, behavior);
    }
}

/// `-shutdown` (dispatched; `PiranhaLogic` overrides).
pub fn shutdown(world: &mut World, this: Entity) {
    let class = isa(world, this);
    (class.shutdown)(world, this);
}

/// `BCLogic::shutdown` @ 0x1001389e8.
pub fn shutdown_base(world: &mut World, this: Entity) {
    logic_mut(world, this).shutdown = true;
    stop_game_frame(world, this);

    let behavior = logic(world, this).sequence;
    if let Some(behavior) = behavior {
        sequence::stop(world, behavior);
    }

    // Breaks the logic ↔ animation retain cycle: the player no longer calls back.
    if let Some(player) = created_animation(world, this) {
        crate::animation::stop(world, player);
        crate::animation::set_animation_listener(world, player, None);
    }
}

/// `-[BCLogic dealloc]` @ 0x100138a74, once the last reference to the logic is gone (the
/// self-destroying `die` / `oneShotComplete` idioms, logic.md §3.5).
///
/// The behaviour sequence is stopped but kept: its extra retain leaks it in the original, and
/// it may be the very sequence running the item that released the logic. The animation player
/// is destroyed. The display object only loses this owner and stays for its others (its group,
/// its creator). The framework subclasses' deallocs only release their listener (or, for
/// `SpriteLogic`, the cloud style string), which needs no port.
pub fn dealloc(world: &mut World, this: Entity) {
    stop_game_frame(world, this);

    let behavior = logic(world, this).sequence;
    if let Some(behavior) = behavior {
        sequence::stop(world, behavior);
    }

    if let Some(player) = created_animation(world, this) {
        crate::animation::stop(world, player);
        crate::animation::dealloc(world, player);
    }

    world.despawn(this);
}

/// `-id` (dispatched): `BCLogic` returns its display object's id.
pub fn id(world: &World, this: Entity) -> Option<String> {
    let class = isa(world, this);
    (class.id)(world, this)
}

/// `-[BCLogic id]` @ 0x1001385ac.
fn id_base(world: &World, this: Entity) -> Option<String> {
    let display_object = display_object(world, this);
    crate::display::id(world, display_object)
}

/// `-isSoundDisabled` (dispatched; `PygmyLogic` overrides).
pub fn is_sound_disabled(world: &World, this: Entity) -> bool {
    let class = isa(world, this);
    (class.is_sound_disabled)(world, this)
}

/// `-applyMoveXPos:moveYPos:` (dispatched; `PygmyLogic` overrides).
pub fn apply_move_pos(world: &mut World, this: Entity, move_x: f32, move_y: f32) {
    let class = isa(world, this);
    (class.apply_move_pos)(world, this, move_x, move_y);
}

/// `BCLogic::applyMoveXPos:moveYPos:` @ 0x100138bf8 (x term in f64, logic.md §6).
pub fn apply_move_pos_base(world: &mut World, this: Entity, move_x: f32, move_y: f32) {
    let (display_object, last_move_x, last_move_y) = {
        let state = logic(world, this);
        (state.display_object, state.last_move_x_pos, state.last_move_y_pos)
    };

    {
        let mut object = display::object_mut(world, display_object);
        let scale_x = object.scale.x;

        // x moves by the sign of scale.x only; the division and the sum are in double
        // precision (`fcvt` to d registers around `fabs`), rounded back to float.
        let scaled_move_x = scale_x * (move_x - last_move_x);
        let step_x = f64::from(scaled_move_x) / f64::from(scale_x.abs());
        let moved_x = step_x + f64::from(object.pos.x);
        object.pos.x = moved_x as f32;

        let scaled_move_y = object.scale.y * (move_y - last_move_y);
        object.pos.y += scaled_move_y;
    }

    let mut state = logic_mut(world, this);
    state.last_move_x_pos = move_x;
    state.last_move_y_pos = move_y;
}

/// `-onAnimationMoveEnabled` @ 0x100138cc8.
pub fn on_animation_move_enabled(world: &mut World, this: Entity) {
    let mut state = logic_mut(world, this);
    state.last_move_x_pos = 0.0;
    state.last_move_y_pos = 0.0;
}

/// `-parentTweenable` @ 0x1001385e8.
pub fn parent_tweenable(world: &World, this: Entity) -> Entity {
    display_object(world, this)
}

/// `-setBinaryFrameDef:tweenableChannelArray:` @ 0x100138b50: forwarded only when the display
/// object `isKindOfClass:BCMultiModel` (which includes `BCTileModel`); ignored otherwise.
pub fn set_binary_frame_def(world: &mut World, this: Entity, frame: Option<FrameRef>, channels_of: Option<Entity>) {
    let display_object = display_object(world, this);
    let is_multi_model = is_kind_of(world, Some(display_object), "BCMultiModel");
    if !is_multi_model {
        return;
    }
    display::multi_model::set_binary_frame_def(world, display_object, frame, channels_of);
}

/// `-doesPointIntersect:` @ 0x10013984c (`checkSelect` = YES).
pub fn does_point_intersect(world: &World, this: Entity, view_point: Vec4) -> bool {
    let display_object = display_object(world, this);
    crate::display::does_point_intersect(world, display_object, view_point, true)
}

/// Velocity accessors (`posVel`, `setXPosVel:` …) are field accesses on [`BcLogic`].
///
/// `BCLogic` responds to the frame actions `animXFlip` / `animYFlip`.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animXFlip" => anim_x_flip(world, this),
        "animYFlip" => anim_y_flip(world, this),
        _ => return false,
    }
    true
}

/// `-animXFlip` @ 0x100138ce4.
pub fn anim_x_flip(world: &mut World, this: Entity) {
    let display_object = display_object(world, this);
    let mut object = crate::display::object_mut(world, display_object);
    object.scale.x = -object.scale.x;
}

/// `-animYFlip` @ 0x100138d30.
pub fn anim_y_flip(world: &mut World, this: Entity) {
    let display_object = display_object(world, this);
    let mut object = crate::display::object_mut(world, display_object);
    object.scale.y = -object.scale.y;
}

/// The random duration of `stall`-style items (`docs/re/sequences.md` §3.13), from one
/// `arc4random()` draw: `lo = (int)(min·scale)`, `hi = (int)(max·scale)` (single-precision
/// products, truncated), then `(int)(r % (1 − lo + hi) + lo) / scale` in the original's u32
/// arithmetic, where a zero range keeps `r` unreduced.
pub fn random_scaled_time(min: f32, max: f32, scale: f32, random: u32) -> f32 {
    let low = trunc_i32(min * scale);
    let high = trunc_i32(max * scale);
    let range = 1u32.wrapping_sub(low as u32).wrapping_add(high as u32);
    let offset = if range == 0 { random } else { random % range };
    let value = offset.wrapping_add(low as u32) as i32;
    value as f32 / scale
}

/// `[param floatValue]` of a parameter that a handler only applies when it is present.
fn present_float(call: &SequenceCall, name: &str) -> Option<f32> {
    let value = call.param(name)?;
    Some(sequence::ns_float_value(value))
}

/// `BCLogic`'s sequence-item handlers (logic.md §8). Subclass handlers call this for
/// selectors they do not implement.
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "removeFromDisplayGroup" => seq_remove_from_display_group(world, this, call),
        "setAnimation" => seq_set_animation(world, this, call),
        "chooseAnimation" => seq_choose_animation(world, this, call),
        "setTextureDef" => seq_set_texture_def(world, this, call),
        "waitForAnimation" => seq_wait_for_animation(world, this, call),
        "setOrigin" => seq_set_origin(world, this, call),
        "setVel" => seq_set_vel(world, this, call),
        "setScale" => seq_set_scale(world, this, call),
        "setRot" => seq_set_rot(world, this, call),
        "setPos" => seq_set_pos(world, this, call),
        "offsetPos" => seq_offset_pos(world, this, call),
        "setPivot" => seq_set_pivot(world, this, call),
        "rotate" => seq_rotate(world, this, call),
        "stall" => seq_stall(world, this, call),
        "tweenAlpha" => seq_tween_alpha(world, this, call),
        "setAlpha" => seq_set_alpha(world, this, call),
        _ => return None,
    };
    Some(result)
}

/// `-[BCLogic removeFromDisplayGroup:::]` @ 0x100138d7c.
pub fn seq_remove_from_display_group(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[BCLogic setAnimation:::]` @ 0x100138dac: plays `[1] animation`, then scrubs to
/// `[4] percent` when present; the other params are ignored.
pub fn seq_set_animation(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let player = animation(world, this);
    let animation_id = call.param("animation");
    crate::animation::play_animation_id(world, player, animation_id);

    if let Some(percent) = present_float(call, "percent") {
        crate::animation::set_frame_percent(world, player, percent);
    }
    ItemResult::Done
}

/// `-[BCLogic chooseAnimation:::]` @ 0x100138e30: plays `list[arc4random() % count]` of the
/// comma-separated `[0] list`.
pub fn seq_choose_animation(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    // A nil list splits into nil: `arc4random()` is still drawn and nil is played.
    let choices = call.list("list");
    let choice_count = choices.len() as u32;
    let choice_index = world.resource_mut::<Random>().arc4random_mod(choice_count);
    let chosen = choices.get(choice_index as usize).copied();

    let player = animation(world, this);
    crate::animation::play_animation_id(world, player, chosen);
    ItemResult::Done
}

/// `-[BCLogic setTextureDef:::]` @ 0x100138edc: with `[2] id` the item acts on that other object
/// through `BCLibrary`; otherwise it stops the animation and shows `[0] texture`.
pub fn seq_set_texture_def(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.has("id") {
        return crate::library::handlers::seq_set_texture_def(world, call);
    }

    let player = animation(world, this);
    crate::animation::stop(world, player);

    let display_object = display_object(world, this);
    let texture = match call.param("texture") {
        Some(texture_id) => crate::library::texture_def_id(world, texture_id),
        None => None,
    };
    display::set_texture_def(world, display_object, texture);
    ItemResult::Done
}

/// `-[BCLogic waitForAnimation:::]` @ 0x100138fcc.
pub fn seq_wait_for_animation(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let player = animation(world, this);
    if crate::animation::is_animation_complete(world, player) {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[BCLogic setOrigin:::]` @ 0x100139004: `[0] zPos`, `[1] xPos`, `[2] yPos` (absent → 0),
/// applied in place when `[3] inPlace` is YES.
pub fn seq_set_origin(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let x = call.float("xPos");
    let y = call.float("yPos");
    let z = call.float("zPos");
    let in_place = call.bool("inPlace");
    let display_object = display_object(world, this);

    if in_place {
        display::set_origin_x_pos_in_place(world, display_object, x);
        display::set_origin_y_pos_in_place(world, display_object, y);
        display::set_origin_z_pos_in_place(world, display_object, z);
    } else {
        let mut object = display::object_mut(world, display_object);
        object.origin_pos.x = x;
        object.origin_pos.y = y;
        object.origin_pos.z = z;
    }
    ItemResult::Done
}

/// `-[BCLogic setVel:::]` @ 0x100139140: `[0] zVel`, `[1] yVel`, `[2] xVel`, each if present.
pub fn seq_set_vel(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let mut state = logic_mut(world, this);
    if let Some(x) = present_float(call, "xVel") {
        state.pos_vel.x = x;
    }
    if let Some(y) = present_float(call, "yVel") {
        state.pos_vel.y = y;
    }
    if let Some(z) = present_float(call, "zVel") {
        state.pos_vel.z = z;
    }
    ItemResult::Done
}

/// `-[BCLogic setScale:::]` @ 0x1001391e4: `[0] xScale`, `[1] zScale`, `[2] yScale`, each if present.
pub fn seq_set_scale(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    if let Some(x) = present_float(call, "xScale") {
        object.scale.x = x;
    }
    if let Some(y) = present_float(call, "yScale") {
        object.scale.y = y;
    }
    if let Some(z) = present_float(call, "zScale") {
        object.scale.z = z;
    }
    ItemResult::Done
}

/// `-[BCLogic setRot:::]` @ 0x100139284: `[0] zRot`, `[1] xRot`, `[2] yRot` (degrees), each if present.
pub fn seq_set_rot(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    if let Some(x) = present_float(call, "xRot") {
        object.rot.x = x;
    }
    if let Some(y) = present_float(call, "yRot") {
        object.rot.y = y;
    }
    if let Some(z) = present_float(call, "zRot") {
        object.rot.z = z;
    }
    ItemResult::Done
}

/// `-[BCLogic setPos:::]` @ 0x100139324: `[0] xPos`, `[1] zPos`, `[2] yPos`, each if present;
/// `[3] id` and `[4] displayGroupId` are ignored.
pub fn seq_set_pos(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    if let Some(x) = present_float(call, "xPos") {
        object.pos.x = x;
    }
    if let Some(y) = present_float(call, "yPos") {
        object.pos.y = y;
    }
    if let Some(z) = present_float(call, "zPos") {
        object.pos.z = z;
    }
    ItemResult::Done
}

/// `-[BCLogic offsetPos:::]` @ 0x1001393c4: adds `[1] xPos`, `[2] yPos`, `[0] zPos`, each if present.
pub fn seq_offset_pos(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    if let Some(x) = present_float(call, "xPos") {
        object.pos.x += x;
    }
    if let Some(y) = present_float(call, "yPos") {
        object.pos.y += y;
    }
    if let Some(z) = present_float(call, "zPos") {
        object.pos.z += z;
    }
    ItemResult::Done
}

/// `-[BCLogic setPivot:::]` @ 0x10013947c: `[0] zPos`, `[1] xPos`, `[2] yPos` (absent → 0).
pub fn seq_set_pivot(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let x = call.float("xPos");
    let y = call.float("yPos");
    let z = call.float("zPos");
    let display_object = display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.pivot_pos.x = x;
    object.pivot_pos.y = y;
    object.pivot_pos.z = z;
    ItemResult::Done
}

/// `-[BCLogic rotate:::]` @ 0x100139550: `zRot += 2` per call (frame-rate dependent); past 360
/// it resets to 0 and completes. The select is `fcsel …, gt`, so a NaN angle stays NaN and the
/// item keeps running.
pub fn seq_rotate(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    let rotated = object.rot.z + 2.0;
    let wrapped = rotated > 360.0;
    object.rot.z = if wrapped { 0.0 } else { rotated };
    if wrapped {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[BCLogic stall:::]` @ 0x1001395cc: `[0] time`, or a random duration between
/// `[1] minTime` and `[2] maxTime`. Checks before it subtracts, so it completes on the first
/// call that finds the time already used up.
pub fn seq_stall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let stall_time = if call.has("minTime") {
            let min_time = call.float("minTime");
            let max_time = call.float("maxTime");
            let random = world.resource_mut::<Random>().arc4random();
            random_scaled_time(min_time, max_time, STALL_TIME_SCALE, random)
        } else {
            call.float("time")
        };
        logic_mut(world, this).stall_time = stall_time;
    }

    let mut state = logic_mut(world, this);
    if state.stall_time <= 0.0 {
        return ItemResult::Done;
    }
    state.stall_time -= call.delta_time;
    ItemResult::Pending
}

/// `-[BCLogic tweenAlpha:::]` @ 0x1001396dc: fades the own display object to `[2] target` over
/// `[1] time` seconds (`[0] displayGroupId` and `[3] id` are ignored).
pub fn seq_tween_alpha(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = display_object(world, this);
    let alpha = display::object(world, display_object).alpha;
    let target = call.float("target");
    if call.first_time {
        let time = call.float("time");
        let mut state = logic_mut(world, this);
        state.stall_time = time;
        state.delta_value = (target - alpha) / time;
    }

    // `fcsel` / `csel …, hi`: a NaN time counts as not used up and keeps tweening.
    let (tweened_alpha, finished) = {
        let mut state = logic_mut(world, this);
        let remaining = state.stall_time;
        let finished = remaining <= 0.0;
        let tweened_alpha = if finished { target } else { target - remaining * state.delta_value };
        state.stall_time = remaining - call.delta_time;
        (tweened_alpha, finished)
    };
    display::object_mut(world, display_object).set_alpha(tweened_alpha);

    if finished {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[BCLogic setAlpha:::]` @ 0x1001397f8: `[0] alpha` on the own display object.
pub fn seq_set_alpha(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let alpha = call.float("alpha");
    let display_object = display_object(world, this);
    display::object_mut(world, display_object).set_alpha(alpha);
    ItemResult::Done
}
