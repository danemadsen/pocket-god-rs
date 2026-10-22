//! `KonkeyDongPickLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;

/// `KonkeyDongPickLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct KonkeyDongPickLogic {
    /// `mKonkeyDongPickLogicListener`: `@"<KonkeyDongPickLogicListener>"`
    pub konkey_dong_pick_logic_listener: Option<Entity>,

    /// `mbIsActive`: `B`
    pub is_active: bool,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mLoopLock`: `i`
    pub loop_lock: i32,
}

pub const KONKEY_DONG_PICK_LOGIC: Class = Class {
    name: "KonkeyDongPickLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The pick waiting to be grabbed (`@"KonkeyDongPickInit"`).
const PICK_INIT_BEHAVIOR: &str = "KonkeyDongPickInit";

/// The pick taken out of play (`@"KonkeyDongPickDeactivate"`).
const PICK_DEACTIVATE_BEHAVIOR: &str = "KonkeyDongPickDeactivate";

/// `[[KonkeyDongPickLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&KONKEY_DONG_PICK_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[KonkeyDongPickLogic initWithDisplayObject:]` @ 0x10001a964
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(KonkeyDongPickLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::set_behavior(world, this, Some(PICK_INIT_BEHAVIOR));
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[KonkeyDongPickLogic setKonkeyDongPickLogicListener:]` @ 0x10001a9e4
pub fn set_konkey_dong_pick_logic_listener(world: &mut World, this: Entity, konkey_dong_pick_logic_listener: Entity) {
    world.obj_mut::<KonkeyDongPickLogic>(this).konkey_dong_pick_logic_listener = Some(konkey_dong_pick_logic_listener);
}

/// `-[KonkeyDongPickLogic setLoopLock:]` @ 0x10001aa40
pub fn set_loop_lock(world: &mut World, this: Entity, loop_lock: i32) {
    world.obj_mut::<KonkeyDongPickLogic>(this).loop_lock = loop_lock;
}

/// `-[KonkeyDongPickLogic loopLock]` @ 0x10001aa50
pub fn loop_lock(world: &mut World, this: Entity) -> i32 {
    world.obj::<KonkeyDongPickLogic>(this).loop_lock
}

/// `-[KonkeyDongPickLogic isActive]` @ 0x10001aa60
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<KonkeyDongPickLogic>(this).is_active
}

/// `-[KonkeyDongPickLogic activate:]` @ 0x10001aa70
///
/// The pick is in play only on the loops before its `loopLock` (`activate` is the number of
/// completed level loops).
pub fn activate(world: &mut World, this: Entity, activate: i32) {
    let loop_lock = world.obj::<KonkeyDongPickLogic>(this).loop_lock;
    if activate < loop_lock {
        world.obj_mut::<KonkeyDongPickLogic>(this).is_active = true;
        pg_engine::logic::set_behavior(world, this, Some(PICK_INIT_BEHAVIOR));
        return;
    }

    world.obj_mut::<KonkeyDongPickLogic>(this).is_active = false;
    pg_engine::logic::set_behavior(world, this, Some(PICK_DEACTIVATE_BEHAVIOR));
}

/// `-[KonkeyDongPickLogic deactivate]` @ 0x10001aac8
pub fn deactivate(world: &mut World, this: Entity) {
    world.obj_mut::<KonkeyDongPickLogic>(this).is_active = false;
    pg_engine::logic::set_behavior(world, this, Some(PICK_DEACTIVATE_BEHAVIOR));
}

/// `-[KonkeyDongPickLogic onGameFrame:]` @ 0x10001aaec
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}
