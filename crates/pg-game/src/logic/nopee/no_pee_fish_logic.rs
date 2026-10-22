//! `NoPeeFishLogic` (superclass `NoPeeObjectLogic`).

use crate::prelude::*;

use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_object_logic::{self, NoPeeObjectLogic, NO_PEE_Z_POS, SCREEN_CENTER_X};
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeFishLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeFishLogic {
    /// `mNoPeeFishLogicListener`: `@"<NoPeeFishLogicListener>"`
    pub no_pee_fish_logic_listener: Option<Entity>,

    /// `mNoPeeFishType`: `i`
    pub no_pee_fish_type: i32,

    /// `mbInitPos`: `B`
    pub init_pos: bool,
}

pub const NO_PEE_FISH_LOGIC: Class = Class {
    name: "NoPeeFishLogic",
    superclass: Some(&crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC),
    on_game_frame,
    ..crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC
};

/// The screen centre's y in the No Pee world is `SCREEN_CENTER_Y − group.y`.
const SCREEN_CENTER_Y: f32 = 160.0;

/// A fish leaving this box around the screen centre comes back in on the other side.
const WRAP_HALF_WIDTH: f32 = 440.0;
const WRAP_HALF_HEIGHT: f32 = 360.0;

/// `[[NoPeeFishLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_FISH_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeFishLogic initWithDisplayObject:]` @ 0x100135968
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeFishLogic::default());
    no_pee_object_logic::init_with_display_object(world, this, display_object);

    logic::set_behavior(world, this, Some("NoPeeFishInit"));
}

/// `-[NoPeeFishLogic setNoPeeFishLogicListener:]` @ 0x1001359d4
pub fn set_no_pee_fish_logic_listener(world: &mut World, this: Entity, no_pee_fish_logic_listener: Entity) {
    world.obj_mut::<NoPeeFishLogic>(this).no_pee_fish_logic_listener = Some(no_pee_fish_logic_listener);
    no_pee_object_logic::set_no_pee_object_logic_listener(world, this, no_pee_fish_logic_listener);
}

/// `-[NoPeeFishLogic setNoPeeFishType:]` @ 0x100135a44
pub fn set_no_pee_fish_type(world: &mut World, this: Entity, no_pee_fish_type: i32) {
    world.obj_mut::<NoPeeFishLogic>(this).no_pee_fish_type = no_pee_fish_type;
}

/// `-[NoPeeFishLogic activate]` @ 0x100135a54
///
/// A random swimming velocity, at least 100 points per second sideways.
pub fn activate(world: &mut World, this: Entity) {
    world.obj_mut::<NoPeeFishLogic>(this).init_pos = false;

    let x_random = world.resource_mut::<Random>().arc4random_mod(1001);
    let x_tenths = x_random.wrapping_sub(500);
    logic::logic_mut(world, this).pos_vel.x = x_tenths as i32 as f32 / 10.0;

    let y_random = world.resource_mut::<Random>().arc4random_mod(201);
    let y_tenths = y_random.wrapping_sub(100);
    logic::logic_mut(world, this).pos_vel.y = y_tenths as i32 as f32 / 10.0;

    {
        let mut state = logic::logic_mut(world, this);
        let vel_x = state.pos_vel.x;
        let push = if vel_x <= 0.0 { -100.0 } else { 100.0 };
        state.pos_vel.x = vel_x + push;
    }

    no_pee_object_logic::activate(world, this);
}

/// `-[NoPeeFishLogic onGameFrame:]` @ 0x100135b58
///
/// While a pygmy plays, the fish first appears somewhere around the screen centre (facing its
/// direction) and wraps around a 880 × 720 box centred on the screen; it always swims on.
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

        let init_pos = world.obj::<NoPeeFishLogic>(this).init_pos;
        if !init_pos {
            world.obj_mut::<NoPeeFishLogic>(this).init_pos = true;
            place_around_screen_center(world, display_object, center_x_pos, center_y_pos);

            let vel_x = logic::logic(world, this).pos_vel.x;
            let mut display = display::object_mut(world, display_object);
            display.scale.x = 1.0;
            if vel_x < 0.0 {
                display.scale.x = -1.0;
            }
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

/// The first placement of `onGameFrame:`: x within ±440 and y within ±360 of the screen centre.
fn place_around_screen_center(world: &mut World, display_object: Entity, center_x_pos: f32, center_y_pos: f32) {
    let x_random = world.resource_mut::<Random>().arc4random_mod(8801);
    let x_tenths = x_random.wrapping_sub(4400);
    let x_pos = center_x_pos + x_tenths as i32 as f32 / 10.0;
    display::object_mut(world, display_object).pos.x = x_pos;

    let y_random = world.resource_mut::<Random>().arc4random_mod(7201);
    let y_tenths = y_random.wrapping_sub(3600);
    let y_pos = center_y_pos + y_tenths as i32 as f32 / 10.0;

    let mut display = display::object_mut(world, display_object);
    display.pos.y = y_pos;
    display.pos.z = NO_PEE_Z_POS;
}
