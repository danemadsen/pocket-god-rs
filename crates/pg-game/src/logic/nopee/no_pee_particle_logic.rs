//! `NoPeeParticleLogic` (superclass `NoPeeObjectLogic`).

use crate::prelude::*;

use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_object_logic::{self, NoPeeObjectLogic, NO_PEE_Z_POS, SCREEN_CENTER_X};
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeParticleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeParticleLogic {
    /// `mNoPeeParticleLogicListener`: `@"<NoPeeParticleLogicListener>"`
    pub no_pee_particle_logic_listener: Option<Entity>,

    /// `mNoPeeParticleType`: `i`
    pub no_pee_particle_type: i32,

    /// `mbInitPos`: `B`
    pub init_pos: bool,
}

pub const NO_PEE_PARTICLE_LOGIC: Class = Class {
    name: "NoPeeParticleLogic",
    superclass: Some(&crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC),
    on_game_frame,
    ..crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC
};

/// The screen centre's y in the No Pee world is `SCREEN_CENTER_Y − group.y`.
const SCREEN_CENTER_Y: f32 = 160.0;

/// A particle leaving this box around the screen centre comes back in on the other side.
const WRAP_HALF_WIDTH: f32 = 270.0;
const WRAP_HALF_HEIGHT: f32 = 190.0;

/// `[[NoPeeParticleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_PARTICLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeParticleLogic initWithDisplayObject:]` @ 0x100175360
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeParticleLogic::default());
    no_pee_object_logic::init_with_display_object(world, this, display_object);

    logic::set_behavior(world, this, Some("NoPeeParticleInit"));
}

/// `-[NoPeeParticleLogic setNoPeeParticleLogicListener:]` @ 0x1001753cc
pub fn set_no_pee_particle_logic_listener(world: &mut World, this: Entity, no_pee_particle_logic_listener: Entity) {
    world.obj_mut::<NoPeeParticleLogic>(this).no_pee_particle_logic_listener = Some(no_pee_particle_logic_listener);
    no_pee_object_logic::set_no_pee_object_logic_listener(world, this, no_pee_particle_logic_listener);
}

/// `-[NoPeeParticleLogic setNoPeeParticleType:]` @ 0x10017543c
pub fn set_no_pee_particle_type(world: &mut World, this: Entity, no_pee_particle_type: i32) {
    world.obj_mut::<NoPeeParticleLogic>(this).no_pee_particle_type = no_pee_particle_type;
}

/// `-[NoPeeParticleLogic activate]` @ 0x10017544c
pub fn activate(world: &mut World, this: Entity) {
    world.obj_mut::<NoPeeParticleLogic>(this).init_pos = false;

    let x_random = world.resource_mut::<Random>().arc4random_mod(1001);
    let x_tenths = x_random.wrapping_sub(500);
    logic::logic_mut(world, this).pos_vel.x = x_tenths as i32 as f32 / 10.0;

    let y_random = world.resource_mut::<Random>().arc4random_mod(201);
    let y_tenths = y_random.wrapping_sub(100);
    logic::logic_mut(world, this).pos_vel.y = y_tenths as i32 as f32 / 10.0;

    no_pee_object_logic::activate(world, this);
}

/// `-[NoPeeParticleLogic onGameFrame:]` @ 0x100175528
///
/// While a pygmy plays, the particle first appears somewhere around the screen centre and wraps
/// around a 540 × 380 box centred on the screen; it always drifts on.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let display_object = logic::display_object(world, this);
    let listener = world.obj::<NoPeeObjectLogic>(this).no_pee_object_logic_listener;
    let pygmy = match listener {
        Some(listener) => no_pee_logic::no_pee_pygmy_logic(world, listener),
        None => None,
    };

    if let (Some(listener), Some(_)) = (listener, pygmy) {
        let no_pee = no_pee_logic::no_pee_logic(world, listener);
        let group_pos = no_pee_logic::world_group_pos(world, no_pee);
        let center_x_pos = SCREEN_CENTER_X - group_pos.x;
        let center_y_pos = SCREEN_CENTER_Y - group_pos.y;

        let init_pos = world.obj::<NoPeeParticleLogic>(this).init_pos;
        if !init_pos {
            world.obj_mut::<NoPeeParticleLogic>(this).init_pos = true;
            place_around_screen_center(world, display_object, center_x_pos, center_y_pos);
        }

        let pos = display::object(world, display_object).pos;
        let x_offset = pos.x - center_x_pos;
        let y_offset = pos.y - center_y_pos;

        let mut display = display::object_mut(world, display_object);
        if x_offset < -WRAP_HALF_WIDTH {
            display.pos.x = center_x_pos + WRAP_HALF_WIDTH;
        } else if x_offset > WRAP_HALF_WIDTH {
            display.pos.x = center_x_pos + -WRAP_HALF_WIDTH;
        }
        if y_offset < -WRAP_HALF_HEIGHT {
            display.pos.y = center_y_pos + WRAP_HALF_HEIGHT;
        } else if y_offset > WRAP_HALF_HEIGHT {
            display.pos.y = center_y_pos + -WRAP_HALF_HEIGHT;
        }
    }

    let pos_vel = logic::logic(world, this).pos_vel;
    let mut display = display::object_mut(world, display_object);
    display.pos.x = display.pos.x + pos_vel.x * dt;
    display.pos.y = display.pos.y + pos_vel.y * dt;
    display.pos.z = display.pos.z + pos_vel.z * dt;
}

/// The first placement of `onGameFrame:`: x within ±270 and y within ±190 of the screen centre.
fn place_around_screen_center(world: &mut World, display_object: Entity, center_x_pos: f32, center_y_pos: f32) {
    let x_random = world.resource_mut::<Random>().arc4random_mod(5401);
    let x_tenths = x_random.wrapping_sub(2700);
    let x_pos = center_x_pos + x_tenths as i32 as f32 / 10.0;
    display::object_mut(world, display_object).pos.x = x_pos;

    let y_random = world.resource_mut::<Random>().arc4random_mod(3801);
    let y_tenths = y_random.wrapping_sub(1900);
    let y_pos = center_y_pos + y_tenths as i32 as f32 / 10.0;

    let mut display = display::object_mut(world, display_object);
    display.pos.y = y_pos;
    display.pos.z = NO_PEE_Z_POS;
}
