//! `MessageLogic`: timed message overlays (`docs/re/logic.md` §11.1).

use bevy_ecs::prelude::*;

use crate::display;
use crate::logic::{self, BC_LOGIC, listener_class};
use crate::math::trunc_i32;
use crate::object::{Class, Isa, ObjectWorldExt};
use crate::sequence::{ItemResult, SequenceCall};

/// The group a shown message is moved into (literal 0x10038a270).
const MESSAGE_DISPLAY_GROUP_ID: &str = "PocketGod";

/// `countDownTimer` writes the remaining seconds into object `Timer` of group `ChallengeTimer`.
const TIMER_DISPLAY_GROUP_ID: &str = "ChallengeTimer";
const TIMER_DISPLAY_OBJECT_ID: &str = "Timer";

/// `MessageLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MessageLogic {
    /// `id<MessageLogicListener>`.
    pub listener: Option<Entity>,

    pub active: bool,
}

pub const MESSAGE_LOGIC: Class = Class {
    name: "MessageLogic",
    superclass: Some(&BC_LOGIC),
    sequence_item,
    ..BC_LOGIC
};

/// `-[MessageLogic initWithDisplayObject:]` @ 0x10019ee38: the message stays hidden (out of
/// its group) until `showMessage:`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    // `alloc` zero-fills the MessageLogic ivars before any initialiser runs.
    let entity = world.spawn((Isa(&MESSAGE_LOGIC), MessageLogic::default())).id();
    logic::init_with_display_object(world, entity, display_object);
    logic::stop_game_frame(world, entity);
    let bound_object = logic::display_object(world, entity);
    display::remove_from_display_group(world, bound_object);
    entity
}

/// `-setMessageLogicListener:` @ 0x10019ef5c.
pub fn set_message_logic_listener(world: &mut World, this: Entity, listener: Option<Entity>) {
    world.obj_mut::<MessageLogic>(this).listener = listener;
}

/// `-[MessageLogic showMessage:]` @ 0x10019eebc: shows the message in group `PocketGod` (if not
/// already shown) and runs `behavior`.
pub fn show_message(world: &mut World, this: Entity, behavior: &str) {
    let active = world.obj::<MessageLogic>(this).active;
    if !active {
        world.obj_mut::<MessageLogic>(this).active = true;
        let message_group = crate::library::display_group(world, MESSAGE_DISPLAY_GROUP_ID);
        let display_object = logic::display_object(world, this);
        display::move_to_display_group(world, display_object, message_group);
    }
    logic::set_behavior(world, this, Some(behavior));
}

/// `MessageLogic` handlers: `countDownTimer`, `onMessageComplete`.
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "countDownTimer" => seq_count_down_timer(world, this, call),
        "onMessageComplete" => seq_on_message_complete(world, this, call),
        _ => return crate::logic::sequence_item(world, this, call),
    };
    Some(result)
}

/// `-[MessageLogic countDownTimer:::]` @ 0x10019efb8: counts `[0] time` seconds down, showing
/// the whole seconds left (truncated, before this frame's decrement) in `ChallengeTimer/Timer`.
pub fn seq_count_down_timer(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let time = call.float("time");
        logic::logic_mut(world, this).stall_time = time;
    }

    let timer_group = crate::library::display_group(world, TIMER_DISPLAY_GROUP_ID);
    let timer = match timer_group {
        Some(timer_group) => display::group::get_display_object(world, timer_group, TIMER_DISPLAY_OBJECT_ID),
        None => None,
    };
    let seconds_left = trunc_i32(logic::logic(world, this).stall_time);
    let timer_text = format!("{seconds_left}");
    if let Some(timer) = timer {
        display::text::set_text(world, timer, Some(&timer_text));
    }

    let mut state = logic::logic_mut(world, this);
    if state.stall_time <= 0.0 {
        return ItemResult::Done;
    }
    state.stall_time -= call.delta_time;
    ItemResult::Pending
}

/// `-[MessageLogic onMessageComplete:::]` @ 0x10019f0e8: hides the message and tells the listener.
pub fn seq_on_message_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<MessageLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let listener = world.obj::<MessageLogic>(this).listener;
    if let Some((listener, class)) = listener_class(world, listener) {
        (class.on_message_complete)(world, listener, this);
    }
    ItemResult::Done
}
