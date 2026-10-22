//! `NoPeeMineLogic` (superclass `NoPeeObjectLogic`).

use crate::prelude::*;

use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_object_logic::{self, NoPeeObjectLogic};
use pg_engine::logic;

/// `NoPeeMineLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeMineLogic {
    /// `mNoPeeMineLogicListener`: `@"<NoPeeMineLogicListener>"`
    pub no_pee_mine_logic_listener: Option<Entity>,

    /// `mNoPeeMineType`: `i`
    pub no_pee_mine_type: i32,

    /// `mbMineHit`: `B`
    pub mine_hit: bool,
}

pub const NO_PEE_MINE_LOGIC: Class = Class {
    name: "NoPeeMineLogic",
    superclass: Some(&crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC),
    on_game_frame,
    ..crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC
};

/// The mine spawns at a y in this range.
const SPAWN_MIN_Y_POS: f32 = 100.0;
const SPAWN_MAX_Y_POS: f32 = 800.0;

/// The mine goes off with the pygmy within this distance.
const HIT_RADIUS: f32 = 50.0;

/// `[[NoPeeMineLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_MINE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeMineLogic initWithDisplayObject:]` @ 0x1000d58b4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeMineLogic::default());
    no_pee_object_logic::init_with_display_object(world, this, display_object);
}

/// `-[NoPeeMineLogic setNoPeeMineLogicListener:]` @ 0x1000d58f0
pub fn set_no_pee_mine_logic_listener(world: &mut World, this: Entity, no_pee_mine_logic_listener: Entity) {
    world.obj_mut::<NoPeeMineLogic>(this).no_pee_mine_logic_listener = Some(no_pee_mine_logic_listener);
    no_pee_object_logic::set_no_pee_object_logic_listener(world, this, no_pee_mine_logic_listener);
}

/// `-[NoPeeMineLogic setNoPeeMineType:]` @ 0x1000d5960
pub fn set_no_pee_mine_type(world: &mut World, this: Entity, no_pee_mine_type: i32) {
    world.obj_mut::<NoPeeMineLogic>(this).no_pee_mine_type = no_pee_mine_type;
}

/// `-[NoPeeMineLogic activate]` @ 0x1000d5970
pub fn activate(world: &mut World, this: Entity) {
    world.obj_mut::<NoPeeMineLogic>(this).mine_hit = false;
    logic::set_behavior(world, this, Some("NoPeeMineInit"));

    no_pee_object_logic::activate(world, this);
    no_pee_object_logic::random_spawn_position_no_overlap_with_min_y_pos(world, this, SPAWN_MIN_Y_POS, SPAWN_MAX_Y_POS);
}

/// `-[NoPeeMineLogic onGameFrame:]` @ 0x1000d5a00
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let object_listener = world.obj::<NoPeeObjectLogic>(this).no_pee_object_logic_listener;
    let pygmy = match object_listener {
        Some(object_listener) => no_pee_logic::no_pee_pygmy_logic(world, object_listener),
        None => None,
    };

    if let Some(pygmy) = pygmy {
        let mine_hit = world.obj::<NoPeeMineLogic>(this).mine_hit;
        if !mine_hit && no_pee_object_logic::check_collision(world, this, pygmy, HIT_RADIUS) {
            world.obj_mut::<NoPeeMineLogic>(this).mine_hit = true;
            crate::pygmy::hit_by_no_pee_mine(world, pygmy, this);
            logic::set_behavior(world, this, Some("NoPeeMineHit"));
        }
    }

    no_pee_object_logic::on_game_frame(world, this, dt);
}
