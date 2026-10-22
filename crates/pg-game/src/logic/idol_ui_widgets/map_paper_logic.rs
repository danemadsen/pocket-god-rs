//! `MapPaperLogic` (superclass `BCLogic`).

use pg_engine::logic;

use crate::prelude::*;
use crate::send;

/// `MapPaperLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MapPaperLogic {
    /// `mMapPaperLogicListener`: `@"<MapPaperLogicListener>"`
    pub map_paper_logic_listener: Option<Entity>,
}

pub const MAP_PAPER_LOGIC: Class = Class {
    name: "MapPaperLogic",
    superclass: Some(&pg_engine::logic::BC_LOGIC),
    sequence_item,
    perform,
    id,
    ..pg_engine::logic::BC_LOGIC
};

/// `@"MapPaperEnter"`: unrolls the map paper.
const MAP_PAPER_ENTER: &str = "MapPaperEnter";

/// `@"MapPaperExit"`: rolls the map paper up.
const MAP_PAPER_EXIT: &str = "MapPaperExit";

/// Sequence-item handlers implemented by `MapPaperLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onMapPaperAnimComplete" => seq_on_map_paper_anim_complete(world, this, call),
        "onMapPaperEnterComplete" => seq_on_map_paper_enter_complete(world, this, call),
        "onMapPaperHideContentsComplete" => seq_on_map_paper_hide_contents_complete(world, this, call),
        "onMapPaperExitComplete" => seq_on_map_paper_exit_complete(world, this, call),
        _ => return (pg_engine::logic::BC_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `MapPaperLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "onHideMap" => on_hide_map(world, this),
        "onShowMap" => on_show_map(world, this),
        _ => return (pg_engine::logic::BC_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `-[MapPaperLogic init]` @ 0x10017a08c
///
/// Nothing sends it: `setLogic:::` creates the map paper with `BCLogic`'s
/// `initWithDisplayObject:`, which does not run this method.
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(MapPaperLogic::default());

    // `[super init]` is `NSObject`'s, so the `BCLogic` ivars are still zero and
    // `stopGameFrame` finds the game frame already stopped.
    if world.get::<BcLogic>(this).is_some() {
        logic::stop_game_frame(world, this);
    }
}

/// `-[MapPaperLogic id]` @ 0x10017a0f0
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(String::from("MapPaperLogic"))
}

/// `-[MapPaperLogic setMapPaperLogicListener:]` @ 0x10017a0fc
pub fn set_map_paper_logic_listener(world: &mut World, this: Entity, map_paper_logic_listener: Entity) {
    world.obj_mut::<MapPaperLogic>(this).map_paper_logic_listener = Some(map_paper_logic_listener);
}

/// `-[MapPaperLogic activate]` @ 0x10017a1bc
pub fn activate(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some(MAP_PAPER_ENTER));
}

/// `-[MapPaperLogic deactivate]` @ 0x10017a1d4
pub fn deactivate(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some(MAP_PAPER_EXIT));
}

/// `-[MapPaperLogic onHideMap]` @ 0x10017a1ec
pub fn on_hide_map(world: &mut World, this: Entity) {
    let listener = world.obj::<MapPaperLogic>(this).map_paper_logic_listener;
    send::on_map_paper_logic_hide_map(world, listener);
}

/// `-[MapPaperLogic onShowMap]` @ 0x10017a208
pub fn on_show_map(_world: &mut World, _this: Entity) {}

/// `-[MapPaperLogic onMapPaperAnimComplete:::]` @ 0x10017a20c
pub fn seq_on_map_paper_anim_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<MapPaperLogic>(this).map_paper_logic_listener;
    send::on_map_paper_logic_anim_complete(world, listener);
    ItemResult::Done
}

/// `-[MapPaperLogic onMapPaperEnterComplete:::]` @ 0x10017a23c
pub fn seq_on_map_paper_enter_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<MapPaperLogic>(this).map_paper_logic_listener;
    send::on_map_paper_logic_show_map(world, listener);
    ItemResult::Done
}

/// `-[MapPaperLogic onMapPaperHideContentsComplete:::]` @ 0x10017a26c
pub fn seq_on_map_paper_hide_contents_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<MapPaperLogic>(this).map_paper_logic_listener;
    send::on_map_paper_logic_hide_map(world, listener);
    ItemResult::Done
}

/// `-[MapPaperLogic onMapPaperExitComplete:::]` @ 0x10017a29c
pub fn seq_on_map_paper_exit_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<MapPaperLogic>(this).map_paper_logic_listener;
    send::on_map_paper_logic_anim_complete(world, listener);
    ItemResult::Done
}
