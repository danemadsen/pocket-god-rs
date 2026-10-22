//! `XenuLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::logic;

use crate::logic::island_core::sprite_logic;

/// `XenuLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct XenuLogic {
    /// `mXenuLogicListener`: `@"<XenuLogicListener>"`
    pub xenu_logic_listener: Option<Entity>,

    /// `mbAlive`: `B`
    pub alive: bool,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbSpawnXenuComplete`: `B`
    pub spawn_xenu_complete: bool,

    /// `mPygmy1Logic`: `@"PygmyLogic"`
    pub pygmy1_logic: Option<Entity>,

    /// `mPygmy2Logic`: `@"PygmyLogic"`
    pub pygmy2_logic: Option<Entity>,
}

pub const XENU_LOGIC: Class = Class {
    name: "XenuLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Animation channel of the pygmy in slot 1.
const SLOT1_CHANNEL: usize = 2;

/// Animation channel of the pygmy in slot 2.
const SLOT2_CHANNEL: usize = 3;

/// Sequence-item handlers implemented by `XenuLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "spawnXenuComplete" => seq_spawn_xenu_complete(world, this, call),
        "eatPygmy1Complete" => seq_eat_pygmy1_complete(world, this, call),
        "eatPygmy2Complete" => seq_eat_pygmy2_complete(world, this, call),
        "puppetPygmyComplete" => seq_puppet_pygmy_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[XenuLogic alloc] initWithDisplayObject:]`.
pub fn spawn_object(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&XENU_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mXenuLogicListener startApocalypse]`.
fn start_apocalypse(world: &mut World, this: Entity) {
    let listener = world.obj::<XenuLogic>(this).xenu_logic_listener;
    if let Some(listener) = listener {
        crate::controller::start_apocalypse(world, listener);
    }
}

/// Unbinds the pygmy in slot 1 (`mPygmy1Logic`) from its channel and sends it
/// `ejectFromXenu`.
fn release_pygmy1(world: &mut World, this: Entity) {
    let pygmy = world.obj::<XenuLogic>(this).pygmy1_logic;
    let Some(pygmy) = pygmy else {
        return;
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, SLOT1_CHANNEL);
    crate::pygmy::eject_from_xenu(world, pygmy);
    world.obj_mut::<XenuLogic>(this).pygmy1_logic = None;
}

/// Unbinds the pygmy in slot 2 (`mPygmy2Logic`) from its channel and sends it
/// `ejectFromXenu`.
fn release_pygmy2(world: &mut World, this: Entity) {
    let pygmy = world.obj::<XenuLogic>(this).pygmy2_logic;
    let Some(pygmy) = pygmy else {
        return;
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, SLOT2_CHANNEL);
    crate::pygmy::eject_from_xenu(world, pygmy);
    world.obj_mut::<XenuLogic>(this).pygmy2_logic = None;
}

/// `-[XenuLogic initWithDisplayObject:]` @ 0x100124748
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(XenuLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().xenu_logic = Some(this);
    logic::stop_game_frame(world, this);
}

/// `-[XenuLogic setXenuLogicListener:]` @ 0x1001247b4
pub fn set_xenu_logic_listener(world: &mut World, this: Entity, xenu_logic_listener: Entity) {
    world.obj_mut::<XenuLogic>(this).xenu_logic_listener = Some(xenu_logic_listener);
}

/// `-[XenuLogic spawn]` @ 0x100124810
pub fn spawn(world: &mut World, this: Entity) {
    world.obj_mut::<XenuLogic>(this).alive = true;
    logic::set_behavior(world, this, Some("XenuSpawn"));
}

/// `-[XenuLogic isAlive]` @ 0x100124838
pub fn is_alive(world: &mut World, this: Entity) -> bool {
    world.obj::<XenuLogic>(this).alive
}

/// `-[XenuLogic banish]` @ 0x100124848
pub fn banish(world: &mut World, this: Entity) {
    world.obj_mut::<XenuLogic>(this).alive = false;
}

/// `-[XenuLogic isSlot1Available:]` @ 0x100124858
///
/// Free while Xenu is alive and the slot is empty; with slot 2 taken, only a cut-in-half
/// pygmy can join another cut-in-half pygmy.
pub fn is_slot1_available(world: &mut World, this: Entity, slot1_available: Entity) -> bool {
    let (alive, pygmy1, pygmy2) = {
        let xenu = world.obj::<XenuLogic>(this);
        (xenu.alive, xenu.pygmy1_logic, xenu.pygmy2_logic)
    };
    if !alive || pygmy1.is_some() {
        return false;
    }

    let Some(pygmy2) = pygmy2 else {
        return true;
    };
    if !crate::pygmy::is_cut_in_half(world, pygmy2) {
        return false;
    }
    crate::pygmy::is_cut_in_half(world, slot1_available)
}

/// `-[XenuLogic isSlot2Available:]` @ 0x1001248d8
pub fn is_slot2_available(world: &mut World, this: Entity, slot2_available: Entity) -> bool {
    let (alive, pygmy1, pygmy2) = {
        let xenu = world.obj::<XenuLogic>(this);
        (xenu.alive, xenu.pygmy1_logic, xenu.pygmy2_logic)
    };
    if !alive || pygmy2.is_some() {
        return false;
    }

    let Some(pygmy1) = pygmy1 else {
        return true;
    };
    if !crate::pygmy::is_cut_in_half(world, pygmy1) {
        return false;
    }
    crate::pygmy::is_cut_in_half(world, slot2_available)
}

/// `-[XenuLogic fallIntoXenuSlot1:]` @ 0x100124958
pub fn fall_into_xenu_slot1(world: &mut World, this: Entity, fall_into_xenu_slot1: Entity) {
    world.obj_mut::<XenuLogic>(this).pygmy1_logic = Some(fall_into_xenu_slot1);

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, fall_into_xenu_slot1);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), SLOT1_CHANNEL);

    let behavior = if !crate::pygmy::is_cut_in_half(world, fall_into_xenu_slot1) {
        "XenuEatPygmy1"
    } else if world.obj::<XenuLogic>(this).pygmy2_logic.is_none() {
        "XenuPuppetPygmy1"
    } else {
        "XenuPuppetPygmy1And2"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[XenuLogic fallIntoXenuSlot2:]` @ 0x100124a2c
pub fn fall_into_xenu_slot2(world: &mut World, this: Entity, fall_into_xenu_slot2: Entity) {
    world.obj_mut::<XenuLogic>(this).pygmy2_logic = Some(fall_into_xenu_slot2);

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, fall_into_xenu_slot2);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), SLOT2_CHANNEL);

    let behavior = if !crate::pygmy::is_cut_in_half(world, fall_into_xenu_slot2) {
        "XenuEatPygmy2"
    } else if world.obj::<XenuLogic>(this).pygmy1_logic.is_none() {
        "XenuPuppetPygmy2"
    } else {
        "XenuPuppetPygmy1And2"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[XenuLogic ejectFromXenu:]` @ 0x100124b00
pub fn eject_from_xenu(world: &mut World, this: Entity, eject_from_xenu: Entity) {
    if world.obj::<XenuLogic>(this).pygmy1_logic == Some(eject_from_xenu) {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, SLOT1_CHANNEL);
        logic::set_behavior(world, this, Some("XenuEjectPygmy"));
        world.obj_mut::<XenuLogic>(this).pygmy1_logic = None;
    }

    if world.obj::<XenuLogic>(this).pygmy2_logic == Some(eject_from_xenu) {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, SLOT2_CHANNEL);
        logic::set_behavior(world, this, Some("XenuEjectPygmy"));
        world.obj_mut::<XenuLogic>(this).pygmy2_logic = None;
    }
}

/// `-[XenuLogic spawnXenuComplete:::]` @ 0x100124bd8
pub fn seq_spawn_xenu_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<XenuLogic>(this).spawn_xenu_complete = true;
    start_apocalypse(world, this);
    ItemResult::Done
}

/// `-[XenuLogic eatPygmy1Complete:::]` @ 0x100124c18
pub fn seq_eat_pygmy1_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    release_pygmy1(world, this);
    ItemResult::Done
}

/// `-[XenuLogic eatPygmy2Complete:::]` @ 0x100124c84
pub fn seq_eat_pygmy2_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    release_pygmy2(world, this);
    ItemResult::Done
}

/// `-[XenuLogic puppetPygmyComplete:::]` @ 0x100124cf0
pub fn seq_puppet_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    release_pygmy1(world, this);
    release_pygmy2(world, this);
    ItemResult::Done
}

/// `-[XenuLogic enterIsland:]` @ 0x100124da8
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<XenuLogic>(this).active {
        return;
    }
    world.obj_mut::<XenuLogic>(this).active = true;

    let behavior = if world.obj::<XenuLogic>(this).alive {
        "XenuEnterIslandAlive"
    } else {
        "XenuEnterIsland"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[XenuLogic exitIsland:]` @ 0x100124df8
///
/// Leaving the room before Xenu's spawn sequence ends still starts the apocalypse.
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    if !world.obj::<XenuLogic>(this).active {
        return;
    }
    world.obj_mut::<XenuLogic>(this).active = false;

    let (alive, spawn_xenu_complete) = {
        let xenu = world.obj::<XenuLogic>(this);
        (xenu.alive, xenu.spawn_xenu_complete)
    };
    if alive && !spawn_xenu_complete {
        world.obj_mut::<XenuLogic>(this).spawn_xenu_complete = true;
        start_apocalypse(world, this);
    }

    release_pygmy1(world, this);
    release_pygmy2(world, this);
    logic::set_behavior(world, this, Some("XenuExitIsland"));
}

/// `-[XenuLogic exitIslandNow]` @ 0x100124f28
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[XenuLogic onGameFrame:]` @ 0x100124fa0
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `+[XenuLogic instance]` @ 0x10012473c
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().xenu_logic
}
