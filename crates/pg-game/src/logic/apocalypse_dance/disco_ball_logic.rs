//! `DiscoBallLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_dance::dance_dance_execution_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic_table;
use pg_engine::library;
use pg_engine::logic;

/// `DiscoBallLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DiscoBallLogic {
    /// `mDiscoBallLogicListener`: `@"<DiscoBallLogicListener>"`
    pub disco_ball_logic_listener: Option<Entity>,
}

pub const DISCO_BALL_LOGIC: Class = Class {
    name: "DiscoBallLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `DiscoBallLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "stopBackgroundLoop" => seq_stop_background_loop(world, this, call),
        "playBackgroundLoop" => seq_play_background_loop(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `DiscoBallLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animSmashPygmy" => anim_smash_pygmy(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[DiscoBallLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DISCO_BALL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DiscoBallLogic initWithDisplayObject:]` @ 0x10011ba4c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DiscoBallLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
}

/// `-[DiscoBallLogic setDiscoBallLogicListener:]` @ 0x10011bab0
pub fn set_disco_ball_logic_listener(world: &mut World, this: Entity, disco_ball_logic_listener: Entity) {
    world.obj_mut::<DiscoBallLogic>(this).disco_ball_logic_listener = Some(disco_ball_logic_listener);
}

/// `-[DiscoBallLogic enterIsland:]` @ 0x10011bb0c
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    logic::set_behavior(world, this, Some("DiscoBallEnterIsland"));
}

/// `-[DiscoBallLogic exitIsland:]` @ 0x10011bb24
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    logic::set_behavior(world, this, Some("DiscoBallExitIsland"));
}

/// `-[DiscoBallLogic smashPygmy]` @ 0x10011bb3c
pub fn smash_pygmy(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("DiscoBallSmashPygmy"));
}

/// `-[DiscoBallLogic animSmashPygmy]` @ 0x10011bb54
///
/// The falling ball reaches the stage: entry 500 crushes its pygmy.
pub fn anim_smash_pygmy(world: &mut World, _this: Entity) {
    let dance_dance_execution = library::logic(world, logic_table::DANCE_DANCE_EXECUTION);
    dance_dance_execution_logic::smash_pygmy(world, dance_dance_execution);
}

/// `-[DiscoBallLogic stopBackgroundLoop:::]` @ 0x10011bb9c
pub fn seq_stop_background_loop(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<DiscoBallLogic>(this).disco_ball_logic_listener;
    if let Some(listener) = listener {
        crate::controller::stop_background_loop(world, listener);
    }
    ItemResult::Done
}

/// `-[DiscoBallLogic playBackgroundLoop:::]` @ 0x10011bbcc
pub fn seq_play_background_loop(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<DiscoBallLogic>(this).disco_ball_logic_listener;
    if let Some(listener) = listener {
        crate::controller::play_background_loop(world, listener, None);
    }
    ItemResult::Done
}
