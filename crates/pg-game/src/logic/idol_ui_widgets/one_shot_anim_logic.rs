//! `OneShotAnimLogic` (superclass `SpriteLogic`).

use pg_engine::display;
use pg_engine::display::multi_model;
use pg_engine::logic;

use crate::logic::island_core::sprite_logic;
use crate::prelude::*;

/// `OneShotAnimLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct OneShotAnimLogic {
}

pub const ONE_SHOT_ANIM_LOGIC: Class = Class {
    name: "OneShotAnimLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"OneShot"`: the id of the multimodel a one-shot animation draws with.
const ONE_SHOT_MODEL_ID: &str = "OneShot";

/// Sequence-item handlers implemented by `OneShotAnimLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "oneShotComplete" => seq_one_shot_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `-[OneShotAnimLogic initWithSequenceId:]` @ 0x10003794c
///
/// The logic owns itself (`[self retain]`) until its sequence reaches `oneShotComplete:::`.
pub fn init_with_sequence_id(world: &mut World, this: Entity, sequence_id: &str) {
    world.entity_mut(this).insert(OneShotAnimLogic::default());

    let one_shot_model = multi_model::spawn(world, Some(ONE_SHOT_MODEL_ID));
    sprite_logic::init_with_display_object(world, this, one_shot_model);

    logic::stop_game_frame(world, this);
    logic::set_behavior(world, this, Some(sequence_id));
}

/// `-[OneShotAnimLogic dealloc]` @ 0x100037a38
///
/// `BCLogic`'s dealloc releases the `OneShot` multimodel. The creator released its reference
/// right after `initWithSequenceId:`, so once the display group has let go of the model this
/// was its last owner and the model goes too.
pub fn dealloc(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);

    // `[super dealloc]`: `SpriteLogic`'s only releases its cloud style string.
    logic::dealloc(world, this);

    let out_of_display_tree = match world.get::<DisplayObject>(display_object) {
        Some(object) => object.display_group.is_none(),
        None => false,
    };
    if out_of_display_tree {
        world.despawn(display_object);
    }
}

/// `-[OneShotAnimLogic oneShotComplete:::]` @ 0x100037a74
pub fn seq_one_shot_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    logic::shutdown(world, this);

    // `[self release]` drops the reference `initWithSequenceId:` took, the last one left.
    dealloc(world, this);
    ItemResult::Done
}
