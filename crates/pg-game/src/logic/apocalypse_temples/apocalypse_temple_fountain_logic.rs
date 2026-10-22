//! `ApocalypseTempleFountainLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::logic;

/// `ApocalypseTempleFountainLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseTempleFountainLogic {
    /// `mApocalypseTempleFountainLogicListener`: `@"<ApocalypseTempleFountainLogicListener>"`
    pub apocalypse_temple_fountain_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mPygmy1Logic`: `@"PygmyLogic"`
    pub pygmy1_logic: Option<Entity>,

    /// `mPygmy2Logic`: `@"PygmyLogic"`
    pub pygmy2_logic: Option<Entity>,

    /// `mPygmy3Logic`: `@"PygmyLogic"`
    pub pygmy3_logic: Option<Entity>,
}

pub const APOCALYPSE_TEMPLE_FOUNTAIN_LOGIC: Class = Class {
    name: "ApocalypseTempleFountainLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `ApocalypseTempleFountainLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "setAnimationBasedOnType" => seq_set_animation_based_on_type(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApocalypseTempleFountainLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_TEMPLE_FOUNTAIN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// The pygmy in slot `slot` (1–3; the slot is also the animation channel it is bound to).
fn slot_pygmy(world: &World, this: Entity, slot: usize) -> Option<Entity> {
    let fountain = world.obj::<ApocalypseTempleFountainLogic>(this);
    match slot {
        1 => fountain.pygmy1_logic,
        2 => fountain.pygmy2_logic,
        _ => fountain.pygmy3_logic,
    }
}

/// Stores the pygmy of slot `slot` (1–3).
fn set_slot_pygmy(world: &mut World, this: Entity, slot: usize, pygmy: Option<Entity>) {
    let mut fountain = world.obj_mut::<ApocalypseTempleFountainLogic>(this);
    match slot {
        1 => fountain.pygmy1_logic = pygmy,
        2 => fountain.pygmy2_logic = pygmy,
        _ => fountain.pygmy3_logic = pygmy,
    }
}

/// `fallIntoApocalypseTempleFountainSlot<slot>:`: the pygmy is bound to channel `slot` and
/// falls in.
fn fall_into_slot(world: &mut World, this: Entity, slot: usize, pygmy: Entity) {
    set_slot_pygmy(world, this, slot, Some(pygmy));

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), slot);
    logic::set_behavior(world, this, Some("ApocalypseTempleFountainPygmyFall"));
}

/// `-[ApocalypseTempleFountainLogic initWithDisplayObject:]` @ 0x1001a1604
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseTempleFountainLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().apocalypse_temple_fountain_logic = Some(this);
    logic::stop_game_frame(world, this);
}

/// `-[ApocalypseTempleFountainLogic setApocalypseTempleFountainLogicListener:]` @ 0x1001a1670
pub fn set_apocalypse_temple_fountain_logic_listener(world: &mut World, this: Entity, apocalypse_temple_fountain_logic_listener: Entity) {
    world.obj_mut::<ApocalypseTempleFountainLogic>(this).apocalypse_temple_fountain_logic_listener = Some(apocalypse_temple_fountain_logic_listener);
}

/// `-[ApocalypseTempleFountainLogic setApocalypseDiseaseType:]` @ 0x1001a16cc
pub fn set_apocalypse_disease_type(world: &mut World, this: Entity, apocalypse_disease_type: i32) {
    world.obj_mut::<SpriteLogic>(this).apocalypse_disease_type = apocalypse_disease_type;
}

/// `-[ApocalypseTempleFountainLogic apocalypseDiseaseType]` @ 0x1001a16e0
pub fn apocalypse_disease_type(world: &mut World, this: Entity) -> i32 {
    world.obj::<SpriteLogic>(this).apocalypse_disease_type
}

/// `-[ApocalypseTempleFountainLogic isSlot1Available]` @ 0x1001a16f4
pub fn is_slot1_available(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseTempleFountainLogic>(this).pygmy1_logic.is_none()
}

/// `-[ApocalypseTempleFountainLogic isSlot2Available]` @ 0x1001a170c
pub fn is_slot2_available(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseTempleFountainLogic>(this).pygmy2_logic.is_none()
}

/// `-[ApocalypseTempleFountainLogic isSlot3Available]` @ 0x1001a1724
pub fn is_slot3_available(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseTempleFountainLogic>(this).pygmy3_logic.is_none()
}

/// `-[ApocalypseTempleFountainLogic fallIntoApocalypseTempleFountainSlot1:]` @ 0x1001a173c
pub fn fall_into_apocalypse_temple_fountain_slot1(world: &mut World, this: Entity, pygmy: Entity) {
    fall_into_slot(world, this, 1, pygmy);
}

/// `-[ApocalypseTempleFountainLogic fallIntoApocalypseTempleFountainSlot2:]` @ 0x1001a17bc
pub fn fall_into_apocalypse_temple_fountain_slot2(world: &mut World, this: Entity, pygmy: Entity) {
    fall_into_slot(world, this, 2, pygmy);
}

/// `-[ApocalypseTempleFountainLogic fallIntoApocalypseTempleFountainSlot3:]` @ 0x1001a183c
pub fn fall_into_apocalypse_temple_fountain_slot3(world: &mut World, this: Entity, pygmy: Entity) {
    fall_into_slot(world, this, 3, pygmy);
}

/// `-[ApocalypseTempleFountainLogic ejectFromApocalypseTempleFountain:]` @ 0x1001a18bc
pub fn eject_from_apocalypse_temple_fountain(world: &mut World, this: Entity, pygmy: Entity) {
    for slot in 1..=3 {
        let slot_pygmy = slot_pygmy(world, this, slot);
        if slot_pygmy != Some(pygmy) {
            continue;
        }

        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, slot);
        logic::set_behavior(world, this, Some("ApocalypseTempleFountainEjectPygmy"));
        set_slot_pygmy(world, this, slot, None);
    }
}

/// `-[ApocalypseTempleFountainLogic enterIsland:]` @ 0x1001a19ec
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let active = world.obj::<ApocalypseTempleFountainLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<ApocalypseTempleFountainLogic>(this).active = true;
    logic::set_behavior(world, this, Some("ApocalypseTempleFountainEnterIsland"));
}

/// `-[ApocalypseTempleFountainLogic exitIsland:]` @ 0x1001a1a20
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let active = world.obj::<ApocalypseTempleFountainLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<ApocalypseTempleFountainLogic>(this).active = false;

    for slot in 1..=3 {
        let pygmy = slot_pygmy(world, this, slot);
        let Some(pygmy) = pygmy else {
            continue;
        };

        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, slot);
        crate::pygmy::eject_from_apocalypse_temple_fountain(world, pygmy);
        set_slot_pygmy(world, this, slot, None);
    }
    logic::set_behavior(world, this, Some("ApocalypseTempleFountainExitIsland"));
}

/// `-[ApocalypseTempleFountainLogic exitIslandNow]` @ 0x1001a1b58
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ApocalypseTempleFountainLogic onGameFrame:]` @ 0x1001a1bd0
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ApocalypseTempleFountainLogic setAnimationBasedOnType:::]` @ 0x1001a1bd4
pub fn seq_set_animation_based_on_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let disease_type = world.obj::<SpriteLogic>(this).apocalypse_disease_type;
    let animation_id = match disease_type {
        2 => call.param("purification"),
        1 => call.param("plague"),
        _ => None,
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// `+[ApocalypseTempleFountainLogic instance]` @ 0x1001a15f8
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().apocalypse_temple_fountain_logic
}
