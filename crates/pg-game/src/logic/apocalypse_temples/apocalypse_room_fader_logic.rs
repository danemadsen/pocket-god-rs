//! `ApocalypseRoomFaderLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::display;
use pg_engine::logic;

/// `ApocalypseRoomFaderLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseRoomFaderLogic {
    /// `mApocalypseRoomFaderLogicListener`: `@"<ApocalypseRoomFaderLogicListener>"`
    pub apocalypse_room_fader_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mIslandLocationState`: `i`
    pub island_location_state: i32,
}

pub const APOCALYPSE_ROOM_FADER_LOGIC: Class = Class {
    name: "ApocalypseRoomFaderLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `ApocalypseRoomFaderLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "apocalypseFaderComplete" => seq_apocalypse_fader_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `ApocalypseRoomFaderLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animChangeLocations" => anim_change_locations(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[ApocalypseRoomFaderLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_ROOM_FADER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ApocalypseRoomFaderLogic initWithDisplayObject:]` @ 0x1001782e4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseRoomFaderLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[ApocalypseRoomFaderLogic setApocalypseRoomFaderLogicListener:]` @ 0x100178320
pub fn set_apocalypse_room_fader_logic_listener(world: &mut World, this: Entity, apocalypse_room_fader_logic_listener: Entity) {
    world.obj_mut::<ApocalypseRoomFaderLogic>(this).apocalypse_room_fader_logic_listener = Some(apocalypse_room_fader_logic_listener);
}

/// `-[ApocalypseRoomFaderLogic isActive]` @ 0x1001783e0
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseRoomFaderLogic>(this).active
}

/// `-[ApocalypseRoomFaderLogic onGameFrame:]` @ 0x1001783f0
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ApocalypseRoomFaderLogic activate:]` @ 0x1001783f4
pub fn activate(world: &mut World, this: Entity, activate: i32) {
    {
        let mut fader = world.obj_mut::<ApocalypseRoomFaderLogic>(this);
        fader.active = true;
        fader.island_location_state = activate;
    }

    let map_display_marker = pg_engine::library::display_marker(world, "MapDisplayMarker");
    if let Some(map_display_marker) = map_display_marker {
        let display_object = logic::display_object(world, this);
        display::insert_after(world, map_display_marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("ApocalypseRoomFaderActivate"));
}

/// `-[ApocalypseRoomFaderLogic animChangeLocations]` @ 0x100178494
pub fn anim_change_locations(world: &mut World, this: Entity) {
    let fader = world.obj::<ApocalypseRoomFaderLogic>(this);
    let listener = fader.apocalypse_room_fader_logic_listener;
    let island_location_state = fader.island_location_state;

    if let Some(listener) = listener {
        crate::controller::on_apocalypse_room_fader_change_island_location_state(world, listener, island_location_state);
    }
}

/// `-[ApocalypseRoomFaderLogic apocalypseFaderComplete:::]` @ 0x1001784c0
pub fn seq_apocalypse_fader_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApocalypseRoomFaderLogic>(this).active = false;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}
