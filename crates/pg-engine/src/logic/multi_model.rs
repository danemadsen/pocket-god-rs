//! `MultiModelLogic`: an animation-only logic with a listener (`docs/re/logic.md` §1, §8).

use bevy_ecs::prelude::*;

use crate::display;
use crate::logic::{self, BC_LOGIC, listener_class};
use crate::object::{Class, Isa, ObjectWorldExt};
use crate::sequence::{ItemResult, SequenceCall};

/// `MultiModelLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MultiModelLogic {
    /// `id<MultiModelLogicListener>`.
    pub listener: Option<Entity>,
}

pub const MULTI_MODEL_LOGIC: Class = Class {
    name: "MultiModelLogic",
    superclass: Some(&BC_LOGIC),
    sequence_item,
    ..BC_LOGIC
};

/// `-[MultiModelLogic initWithDisplayObject:]` @ 0x10000bb14 (stops its game frame).
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    crate::logic::init_with_display_object(world, this, display_object);
    world.entity_mut(this).insert(MultiModelLogic::default());
    crate::logic::stop_game_frame(world, this);
}

/// Allocates a plain `MultiModelLogic`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let entity = world.spawn(Isa(&MULTI_MODEL_LOGIC)).id();
    init_with_display_object(world, entity, display_object);
    entity
}

/// `-setMultiModelLogicListener:` @ 0x10000bbdc.
pub fn set_multi_model_logic_listener(world: &mut World, this: Entity, listener: Option<Entity>) {
    world.obj_mut::<MultiModelLogic>(this).listener = listener;
}

/// `MultiModelLogic` handlers: `die`, `multiModelEvent`, `triggerEpisodeEvent`.
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        "multiModelEvent" => seq_multi_model_event(world, this, call),
        "triggerEpisodeEvent" => seq_trigger_episode_event(world, this, call),
        _ => return crate::logic::sequence_item(world, this, call),
    };
    Some(result)
}

/// `-[MultiModelLogic die:::]` @ 0x10000bc38: leaves the display tree, shuts down and releases
/// itself.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    logic::shutdown(world, this);

    // `[self release]`: the creator handed its reference to this item and `shutdown` made the
    // animation drop its own, so this was the last one and the logic is deallocated.
    logic::dealloc(world, this);
    ItemResult::Done
}

/// `-[MultiModelLogic multiModelEvent:::]` @ 0x10000bca0: `[0] eventId` to the listener.
pub fn seq_multi_model_event(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<MultiModelLogic>(this).listener;

    // Every shipped item names its event; a nil id would compare unequal to all ids, as "" does.
    let event_id = call.param("eventId").unwrap_or("");
    if let Some((listener, class)) = listener_class(world, listener) {
        (class.on_multi_model_event)(world, listener, this, event_id);
    }
    ItemResult::Done
}

/// `-[MultiModelLogic triggerEpisodeEvent:::]` @ 0x10000bce0: `[[IdolManager instance]
/// triggerEpisodeEvent:[0] event]` through [`crate::hooks::EngineHooks::trigger_episode_event`].
pub fn seq_trigger_episode_event(world: &mut World, _this: Entity, call: &mut SequenceCall) -> ItemResult {
    // Every shipped item names its event; "" stands in for a nil one (the hook takes a string).
    let event = call.param("event").unwrap_or("");
    let hooks = crate::hooks::hooks(world);
    (hooks.trigger_episode_event)(world, event);
    ItemResult::Done
}
