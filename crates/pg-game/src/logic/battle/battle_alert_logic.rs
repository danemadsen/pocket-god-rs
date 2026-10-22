//! `BattleAlertLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::managers::match_and_message::{self, MATCH_AND_MESSAGE, MatchAndMessage};
use pg_engine::display;
use pg_engine::display::text;
use pg_engine::library::{self, Library};
use pg_engine::logic;

/// `BattleAlertLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BattleAlertLogic {
    /// `mBattleAlertLogicListener`: `@"<BattleAlertLogicListener>"`
    pub battle_alert_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mMatchArray`: `@"NSMutableArray"`
    pub match_array: Vec<Entity>,
}

pub const BATTLE_ALERT_LOGIC: Class = Class {
    name: "BattleAlertLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
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

/// Sequence-item handlers implemented by `BattleAlertLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onBattleAlertClose" => seq_on_battle_alert_close(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[BattleAlertLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BATTLE_ALERT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[BattleAlertLogic initWithDisplayObject:]` @ 0x100027bfc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BattleAlertLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, own_display_object);
        display.col_x_min = 0.0;
        display.col_x_max = 480.0;
        display.col_y_min = 0.0;
        display.col_y_max = 42.0;
        display.is_selectable = true;
    }

    world.obj_mut::<BattleAlertLogic>(this).match_array = Vec::with_capacity(5);
}

/// `-[BattleAlertLogic setBattleAlertLogicListener:]` @ 0x100027d10
pub fn set_battle_alert_logic_listener(world: &mut World, this: Entity, battle_alert_logic_listener: Entity) {
    world.obj_mut::<BattleAlertLogic>(this).battle_alert_logic_listener = Some(battle_alert_logic_listener);
}

/// `-[BattleAlertLogic activateNextMatch]` @ 0x100027d6c
///
/// Shows the message of the first queued match.
pub fn activate_next_match(world: &mut World, this: Entity) {
    let match_count = world.obj::<BattleAlertLogic>(this).match_array.len();
    if match_count == 0 {
        deactivate(world, this);
    }

    // Original bug, kept: with an empty queue `objectAtIndex:0` raises NSRangeException. Both
    // callers only get here with a match queued.
    let first_match = world.obj::<BattleAlertLogic>(this).match_array.first().copied();
    let first_match = first_match.expect("BattleAlertLogic: activateNextMatch with no queued match");

    let tap_here_text = world.resource::<Library>().dynamic_text("BattleAlertMessageTapHere");
    if let Some(tap_here_text) = tap_here_text {
        text::set_text(world, tap_here_text, Some(""));
    }

    // `[match message]` returns the ivar.
    let message_text = world.resource::<Library>().dynamic_text("BattleAlertMessage");
    let message = world.obj::<MatchAndMessage>(first_match).message.clone();
    if let Some(message_text) = message_text {
        text::set_text(world, message_text, message.as_deref());
    }

    logic::set_behavior(world, this, Some("BattleAlertActivate"));
}

/// `-[BattleAlertLogic activateMatchAlert:message:]` @ 0x100027e98
///
/// Queues the match with its message; the first alert also shows the banner.
pub fn activate_match_alert(world: &mut World, this: Entity, match_: Entity, message: &str) {
    // `[[MatchAndMessage alloc] initWithMatch:message:]`; the queue holds the only reference.
    let entry = world.spawn(Isa(&MATCH_AND_MESSAGE)).id();
    match_and_message::init_with_match(world, entry, match_, message);
    world.obj_mut::<BattleAlertLogic>(this).match_array.push(entry);

    let active = world.obj::<BattleAlertLogic>(this).active;
    if active {
        return;
    }
    world.obj_mut::<BattleAlertLogic>(this).active = true;

    let pocket_god_group = library::display_group(world, "PocketGod");
    let display_object = logic::display_object(world, this);
    display::move_to_display_group(world, display_object, pocket_god_group);

    activate_next_match(world, this);
}

/// `-[BattleAlertLogic deactivate]` @ 0x100027fb8
pub fn deactivate(world: &mut World, this: Entity) {
    let active = world.obj::<BattleAlertLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<BattleAlertLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[BattleAlertLogic onBattleAlertClose:::]` @ 0x10002806c
///
/// Drops the shown match, then shows the next one or hides the banner.
pub fn seq_on_battle_alert_close(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    // `removeObjectAtIndex:0` releases the queue's only reference: the entry dies.
    let shown_match = world.obj_mut::<BattleAlertLogic>(this).match_array.remove(0);
    world.despawn(shown_match);

    let match_count = world.obj::<BattleAlertLogic>(this).match_array.len();
    if match_count == 0 {
        deactivate(world, this);
    } else {
        activate_next_match(world, this);
    }
    ItemResult::Pending
}

/// `-[BattleAlertLogic touchBeganInside:allTouches:]` @ 0x1000280e4
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let listener = world.obj::<BattleAlertLogic>(this).battle_alert_logic_listener;
    let Some(listener) = listener else {
        return;
    };

    let story_active = crate::controller::is_story_active(world, listener);
    if story_active {
        return;
    }

    let screen_open = crate::controller::is_screen_open(world, listener);
    if screen_open {
        return;
    }

    crate::controller::on_battle_alert_select(world, listener);
}

/// `-[BattleAlertLogic touchBeganOutside:allTouches:]` @ 0x10002817c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BattleAlertLogic touchMovedEntered:allTouches:]` @ 0x100028180
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BattleAlertLogic touchMovedExited:allTouches:]` @ 0x100028184
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BattleAlertLogic touchMovedInside:allTouches:]` @ 0x100028188
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[BattleAlertLogic touchMovedOutside:allTouches:]` @ 0x100028198
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BattleAlertLogic touchEndedInside:allTouches:]` @ 0x10002819c
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BattleAlertLogic touchEndedOutside:allTouches:]` @ 0x1000281a0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
