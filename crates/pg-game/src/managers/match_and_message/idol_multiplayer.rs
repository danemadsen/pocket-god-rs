//! `MatchAndMessage`: methods in this part of the class.

use super::*;

/// `-[MatchAndMessage initWithMatch:message:]` @ 0x100027ac4
///
/// The object starts as `world.spawn(Isa(&MATCH_AND_MESSAGE))`; both ivars are retained (the
/// retains vanish).
pub fn init_with_match(world: &mut World, this: Entity, match_: Entity, message: &str) {
    world.entity_mut(this).insert(MatchAndMessage::default());

    let mut ivars = world.obj_mut::<MatchAndMessage>(this);
    ivars.match_ = Some(match_);
    ivars.message = Some(message.to_owned());
}

/// `-[MatchAndMessage match]` @ 0x100027b60
pub fn match_(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<MatchAndMessage>(this).match_
}

/// `-[MatchAndMessage message]` @ 0x100027b70
pub fn message(world: &mut World, this: Entity) -> Option<String> {
    world.obj::<MatchAndMessage>(this).message.clone()
}

/// `-[MatchAndMessage dealloc]` @ 0x100027b80
///
/// Releases both ivars: the match belongs to the match service and outlives this object.
pub fn dealloc(world: &mut World, this: Entity) {
    world.despawn(this);
}
