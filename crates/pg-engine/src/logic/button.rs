//! `BCButtonLogic` (`docs/re/logic.md` §9.5).
//!
//! The armed flag is per button, not per finger: a second finger beginning elsewhere disarms
//! it, sliding off and back on cancels the click, and every button in the active tree hears
//! `onButtonReleaseOutside:` for each finger that lifts anywhere else.

use bevy_ecs::prelude::*;

use crate::logic::{BC_LOGIC, listener_class};
use crate::object::{Class, Isa, ObjectWorldExt};

/// `BCButtonLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BcButtonLogic {
    /// `id<BCButtonLogicListener>`; callbacks go through the listener's [`Class`].
    pub listener: Option<Entity>,

    pub touch_began_inside: bool,
}

/// `touchMovedEntered:`, `touchMovedInside:` and `touchMovedOutside:` are empty overrides
/// (0x100091d84, 0x100091d98, 0x100091d9c), like `BCLogic`'s.
pub const BC_BUTTON_LOGIC: Class = Class {
    name: "BCButtonLogic",
    superclass: Some(&BC_LOGIC),
    touch_began_inside,
    touch_began_outside,
    touch_moved_exited,
    touch_ended_inside,
    touch_ended_outside,
    ..BC_LOGIC
};

/// `-[BCButtonLogic initWithDisplayObject:]` @ 0x100091c20.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let entity = world.spawn(Isa(&BC_BUTTON_LOGIC)).id();
    crate::logic::init_with_display_object(world, entity, display_object);
    world.entity_mut(entity).insert(BcButtonLogic::default());
    entity
}

/// `-setButtonLogicListener:` @ 0x100091c78.
pub fn set_button_logic_listener(world: &mut World, this: Entity, listener: Option<Entity>) {
    world.obj_mut::<BcButtonLogic>(this).listener = listener;
}

/// `-[BCButtonLogic touchBeganInside:allTouches:]` @ 0x100091d38: arms and reports the press.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = {
        let mut button = world.obj_mut::<BcButtonLogic>(this);
        button.touch_began_inside = true;
        button.listener
    };
    if let Some((listener, class)) = listener_class(world, listener) {
        (class.on_button_press)(world, listener, this, touch);
    }
}

/// `-[BCButtonLogic touchBeganOutside:allTouches:]` @ 0x100091d74.
pub fn touch_began_outside(world: &mut World, this: Entity, _touch: Entity) {
    world.obj_mut::<BcButtonLogic>(this).touch_began_inside = false;
}

/// `-[BCButtonLogic touchMovedExited:allTouches:]` @ 0x100091d88.
pub fn touch_moved_exited(world: &mut World, this: Entity, _touch: Entity) {
    world.obj_mut::<BcButtonLogic>(this).touch_began_inside = false;
}

/// `-[BCButtonLogic touchEndedInside:allTouches:]` @ 0x100091da0: a release when armed, a
/// release-outside otherwise; disarms after the callback.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let (listener, armed) = {
        let button = world.obj::<BcButtonLogic>(this);
        (button.listener, button.touch_began_inside)
    };
    if let Some((listener, class)) = listener_class(world, listener) {
        let callback = if armed { class.on_button_release } else { class.on_button_release_outside };
        callback(world, listener, this, touch);
    }
    world.obj_mut::<BcButtonLogic>(this).touch_began_inside = false;
}

/// `-[BCButtonLogic touchEndedOutside:allTouches:]` @ 0x100091e04.
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<BcButtonLogic>(this).listener;
    if let Some((listener, class)) = listener_class(world, listener) {
        (class.on_button_release_outside)(world, listener, this, touch);
    }
    world.obj_mut::<BcButtonLogic>(this).touch_began_inside = false;
}
