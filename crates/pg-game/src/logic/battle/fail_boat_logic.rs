//! `FailBoatLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `FailBoatLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct FailBoatLogic {
    /// `mFailBoatLogicListener`: `@"<FailBoatLogicListener>"`
    pub fail_boat_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mPygmyLogicArray`: `@"NSMutableArray"`
    pub pygmy_logic_array: Vec<Entity>,
}

pub const FAIL_BOAT_LOGIC: Class = Class {
    name: "FailBoatLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The rocking amplitude of `onGameFrame:` (degrees).
const ROCK_AMPLITUDE: f32 = 3.0;

/// The phase step of `onGameFrame:` per frame (a double constant).
const ROCK_PHASE_STEP: f64 = 0.01;

/// `[[FailBoatLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&FAIL_BOAT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[FailBoatLogic initWithDisplayObject:]` @ 0x1000885b0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(FailBoatLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().fail_boat_logic = Some(this);

    // The original stores an autoreleased `arrayWithCapacity:5` without retaining it; nothing in
    // 1.48.2 sends `fallIntoFailBoat:` or `ejectFromFailBoat:`, so the stale array is never used.
    world.obj_mut::<FailBoatLogic>(this).pygmy_logic_array = Vec::with_capacity(5);
}

/// `-[FailBoatLogic setFailBoatLogicListener:]` @ 0x100088630
pub fn set_fail_boat_logic_listener(world: &mut World, this: Entity, fail_boat_logic_listener: Entity) {
    world.obj_mut::<FailBoatLogic>(this).fail_boat_logic_listener = Some(fail_boat_logic_listener);
}

/// `-[FailBoatLogic fallIntoFailBoat:]` @ 0x10008868c
pub fn fall_into_fail_boat(world: &mut World, this: Entity, pygmy_logic: Entity) {
    world.obj_mut::<FailBoatLogic>(this).pygmy_logic_array.push(pygmy_logic);
}

/// `-[FailBoatLogic ejectFromFailBoat:]` @ 0x1000886a8
pub fn eject_from_fail_boat(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let mut boat = world.obj_mut::<FailBoatLogic>(this);
    let on_board = boat.pygmy_logic_array.contains(&pygmy_logic);
    if !on_board {
        return;
    }

    // `removeObject:` drops every occurrence.
    boat.pygmy_logic_array.retain(|&pygmy| pygmy != pygmy_logic);
}

/// `-[FailBoatLogic enterIsland:]` @ 0x100088714
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let mut boat = world.obj_mut::<FailBoatLogic>(this);
    if !boat.active {
        boat.active = true;
    }
}

/// `-[FailBoatLogic exitIsland:]` @ 0x100088730
pub fn exit_island(world: &mut World, this: Entity, _now: bool) {
    let mut boat = world.obj_mut::<FailBoatLogic>(this);
    if boat.active {
        boat.active = false;
    }
}

/// `-[FailBoatLogic exitIslandNow]` @ 0x100088748
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[FailBoatLogic onGameFrame:]` @ 0x1000887d8
///
/// The boat rocks: `rot.z = sin(rotVel.z) * 3`, with the rotation velocity's z used as the phase.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;
    let phase = state.rot_vel.z;

    let rock = phase.sin() * ROCK_AMPLITUDE;
    display::object_mut(world, display_object).rot.z = rock;

    // The phase advances in double precision: `(float)((double)rotVel.z + 0.01)`.
    let next_phase = (f64::from(phase) + ROCK_PHASE_STEP) as f32;
    logic::logic_mut(world, this).rot_vel.z = next_phase;
}

/// `+[FailBoatLogic instance]` @ 0x1000885a4
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().fail_boat_logic
}
