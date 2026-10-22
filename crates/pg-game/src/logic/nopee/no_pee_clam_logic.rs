//! `NoPeeClamLogic` (superclass `NoPeeObjectLogic`).

use crate::prelude::*;

use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_object_logic::{self, NoPeeObjectLogic, NO_PEE_Z_POS};
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeClamLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeClamLogic {
    /// `mNoPeeClamLogicListener`: `@"<NoPeeClamLogicListener>"`
    pub no_pee_clam_logic_listener: Option<Entity>,

    /// `mNoPeeClamType`: `i`
    pub no_pee_clam_type: i32,
}

pub const NO_PEE_CLAM_LOGIC: Class = Class {
    name: "NoPeeClamLogic",
    superclass: Some(&crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC),
    on_game_frame,
    ..crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC
};

/// Clams sit on the sea floor.
const FLOOR_Y_POS: f32 = 70.0;

/// The clam snaps at the pygmy within this distance.
const EAT_RADIUS: f32 = 50.0;

/// `[[NoPeeClamLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_CLAM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeClamLogic initWithDisplayObject:]` @ 0x1000c06c0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeClamLogic::default());
    no_pee_object_logic::init_with_display_object(world, this, display_object);

    logic::set_behavior(world, this, Some("NoPeeClamInit"));
}

/// `-[NoPeeClamLogic randomSpawnPosition]` @ 0x1000c072c
pub fn random_spawn_position(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);

    let x_random = world.resource_mut::<Random>().arc4random_mod(30001);
    let x_pos = x_random as i32 as f32 / 10.0;
    display::object_mut(world, display_object).pos.x = x_pos;

    // The y draw of `NoPeeObjectLogic` is kept but its value is not used.
    world.resource_mut::<Random>().arc4random();

    let mut display = display::object_mut(world, display_object);
    display.pos.y = FLOOR_Y_POS;
    display.pos.z = NO_PEE_Z_POS;
}

/// `-[NoPeeClamLogic activate]` @ 0x1000c07b8
pub fn activate(world: &mut World, this: Entity) {
    no_pee_object_logic::activate(world, this);
    no_pee_object_logic::random_spawn_position_no_overlap_with_min_y_pos(world, this, FLOOR_Y_POS, FLOOR_Y_POS);

    // Original quirk: a mirrored clam is never turned back.
    let random = world.resource_mut::<Random>().arc4random();
    if random & 1 != 0 {
        let display_object = logic::display_object(world, this);
        display::object_mut(world, display_object).scale.x = -1.0;
    }
}

/// `-[NoPeeClamLogic setNoPeeClamLogicListener:]` @ 0x1000c0850
pub fn set_no_pee_clam_logic_listener(world: &mut World, this: Entity, no_pee_clam_logic_listener: Entity) {
    world.obj_mut::<NoPeeClamLogic>(this).no_pee_clam_logic_listener = Some(no_pee_clam_logic_listener);
    no_pee_object_logic::set_no_pee_object_logic_listener(world, this, no_pee_clam_logic_listener);
}

/// `-[NoPeeClamLogic setNoPeeClamType:]` @ 0x1000c08c0
pub fn set_no_pee_clam_type(world: &mut World, this: Entity, no_pee_clam_type: i32) {
    world.obj_mut::<NoPeeClamLogic>(this).no_pee_clam_type = no_pee_clam_type;
}

/// `-[NoPeeClamLogic onGameFrame:]` @ 0x1000c08d0
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let object_listener = world.obj::<NoPeeObjectLogic>(this).no_pee_object_logic_listener;
    let pygmy = match object_listener {
        Some(object_listener) => no_pee_logic::no_pee_pygmy_logic(world, object_listener),
        None => None,
    };

    if let Some(pygmy) = pygmy {
        let caught = no_pee_object_logic::check_collision(world, this, pygmy, EAT_RADIUS);
        if caught {
            crate::pygmy::hit_by_no_pee_clam(world, pygmy, this);
            logic::set_behavior(world, this, Some("NoPeeClamEatPygmy"));
        }
    }

    no_pee_object_logic::on_game_frame(world, this, dt);
}
