//! `PygmyBottomLogic` (superclass `SpriteLogic`).
//!
//! Detached body parts: the legs of a pygmy cut in half (global logic 0) and the locust skulls
//! and bones (464..469). Only the legs get a listener (the view controller); the skulls and
//! bones keep a nil one, which answers NO to `isOceanEnabled`.

use pg_engine::display;
use pg_engine::logic;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::prelude::*;

/// `PygmyBottomLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyBottomLogic {
    /// `mPygmyBottomLogicListener`: `@"<PygmyBottomLogicListener>"`
    pub pygmy_bottom_logic_listener: Option<Entity>,

    /// `mbIsActive`: `B`
    pub is_active: bool,
}

pub const PYGMY_BOTTOM_LOGIC: Class = Class {
    name: "PygmyBottomLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `mGravityForce` as `initWithDisplayObject:` sets it: `(0, -3000, 0, 1)`.
const GRAVITY_FORCE: Vec4 = Vec4::new(0.0, -3000.0, 0.0, 1.0);

/// The water line: with the ocean on, a falling part stops at this height.
const OCEAN_Y_POS: f32 = 16.0;

/// Sequence-item handlers implemented by `PygmyBottomLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[PygmyBottomLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&PYGMY_BOTTOM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[PygmyBottomLogic initWithDisplayObject:]` @ 0x100112c9c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(PygmyBottomLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).gravity_force = GRAVITY_FORCE;
}

/// `-[PygmyBottomLogic setPygmyBottomLogicListener:]` @ 0x100112d04
pub fn set_pygmy_bottom_logic_listener(world: &mut World, this: Entity, pygmy_bottom_logic_listener: Entity) {
    world.obj_mut::<PygmyBottomLogic>(this).pygmy_bottom_logic_listener = Some(pygmy_bottom_logic_listener);
}

/// `-[PygmyBottomLogic dealloc]` @ 0x100112d60
///
/// For the part's last owner. The parts are global logic objects, so nothing releases them
/// during the game.
pub fn dealloc(world: &mut World, this: Entity) {
    // The listener goes with the component; `[super dealloc]`: `SpriteLogic` only releases its
    // cloud style string.
    logic::dealloc(world, this);
}

/// `-[PygmyBottomLogic isActive]` @ 0x100112dc4
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyBottomLogic>(this).is_active
}

/// `[mPygmyBottomLogicListener isOceanEnabled]` (NO for the nil listener of the locust parts).
fn is_ocean_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<PygmyBottomLogic>(this).pygmy_bottom_logic_listener;
    match listener {
        Some(listener) => crate::controller::is_ocean_enabled(world, listener),
        None => false,
    }
}

/// `-[PygmyBottomLogic onGameFrame:]` @ 0x100112dd4
///
/// Its own integration, without `SpriteLogic`'s: velocity from gravity (unless ignored), then
/// position from velocity. Falling out of the play area (every frame) sends it to `PygmyDie`.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let (shutdown, display_object) = {
        let bc_logic = logic::logic(world, this);
        (bc_logic.shutdown, bc_logic.display_object)
    };
    if shutdown {
        return;
    }

    let (ignore_gravity, gravity_force) = {
        let sprite = world.obj::<SpriteLogic>(this);
        (sprite.ignore_gravity, sprite.gravity_force)
    };
    if !ignore_gravity {
        let mut bc_logic = logic::logic_mut(world, this);
        bc_logic.pos_vel.x = bc_logic.pos_vel.x + gravity_force.x * dt;
        bc_logic.pos_vel.y = bc_logic.pos_vel.y + gravity_force.y * dt;
        bc_logic.pos_vel.z = bc_logic.pos_vel.z + gravity_force.z * dt;
    }

    let pos_vel = logic::logic(world, this).pos_vel;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
    }

    let ocean_enabled = is_ocean_enabled(world, this);
    let mut y_pos = display::object(world, display_object).pos.y;
    if ocean_enabled && y_pos < OCEAN_Y_POS {
        {
            let mut bc_logic = logic::logic_mut(world, this);
            bc_logic.pos_vel.x = 0.0;
            bc_logic.pos_vel.y = 0.0;
        }
        display::object_mut(world, display_object).pos.y = OCEAN_Y_POS;
        y_pos = OCEAN_Y_POS;
    }

    if y_pos < -100.0 || 820.0 < y_pos {
        logic::set_behavior(world, this, Some("PygmyDie"));
    }
}

/// `-[PygmyBottomLogic fall:::]` @ 0x100112f88
///
/// Falls under gravity until it lands on the water (only with the ocean on); the
/// `ignoreIsland` parameter is not read.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = logic::display_object(world, this);
    let ocean_enabled = is_ocean_enabled(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if !ocean_enabled || OCEAN_Y_POS < y_pos {
        return ItemResult::Pending;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut bc_logic = logic::logic_mut(world, this);
        bc_logic.pos_vel.x = 0.0;
        bc_logic.pos_vel.y = 0.0;
    }
    display::object_mut(world, display_object).pos.y = OCEAN_Y_POS;
    ItemResult::Done
}

/// `-[PygmyBottomLogic die:::]` @ 0x100113048
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyBottomLogic>(this).is_active = false;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[PygmyBottomLogic tossBottom:]` @ 0x100113088
///
/// The legs of a pygmy cut in half: they start where the pygmy is, a little faster than it
/// (x and y), and are drawn just behind it. `mbIsActive` is not set.
pub fn toss_bottom(world: &mut World, this: Entity, pygmy: Entity) {
    let display_object = logic::display_object(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let pygmy_pos_vel = logic::logic(world, pygmy).pos_vel;

    display::object_mut(world, display_object).pos = pygmy_pos;
    {
        let mut bc_logic = logic::logic_mut(world, this);
        bc_logic.pos_vel = pygmy_pos_vel;
        bc_logic.pos_vel.x = bc_logic.pos_vel.x * 1.2;
        bc_logic.pos_vel.y = bc_logic.pos_vel.y * 1.2;
    }

    display::insert_before(world, pygmy_display_object, Some(display_object));
    logic::set_behavior(world, this, Some("PygmyBottomInit"));
}

/// `-[PygmyBottomLogic spawnLocustSkull:]` @ 0x100113190
///
/// A skull flung up from a pygmy the locusts stripped: 26 above the pygmy, a random velocity
/// (x in -200..=200, y in 100..=150), gravity off, drawn just behind the pygmy.
pub fn spawn_locust_skull(world: &mut World, this: Entity, pygmy: Entity) {
    let display_object = logic::display_object(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos = pygmy_pos;
        object.pos.y = object.pos.y + 26.0;
    }

    let x_random = world.resource_mut::<Random>().arc4random_mod(4001);
    let x_offset = x_random as i32 - 2000;
    logic::logic_mut(world, this).pos_vel.x = x_offset as f32 / 10.0;

    let y_random = world.resource_mut::<Random>().arc4random_mod(501);
    let y_speed = y_random as i32 + 1000;
    logic::logic_mut(world, this).pos_vel.y = y_speed as f32 / 10.0;

    world.obj_mut::<PygmyBottomLogic>(this).is_active = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    display::insert_before(world, pygmy_display_object, Some(display_object));
    logic::set_behavior(world, this, Some("PygmyBottomLocustSkull"));
}

/// `-[PygmyBottomLogic spawnLocustBone:]` @ 0x100113300
///
/// Like `spawnLocustSkull:`, but 50 above the pygmy.
pub fn spawn_locust_bone(world: &mut World, this: Entity, pygmy: Entity) {
    let display_object = logic::display_object(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos = pygmy_pos;
        object.pos.y = object.pos.y + 50.0;
    }

    let x_random = world.resource_mut::<Random>().arc4random_mod(4001);
    let x_offset = x_random as i32 - 2000;
    logic::logic_mut(world, this).pos_vel.x = x_offset as f32 / 10.0;

    let y_random = world.resource_mut::<Random>().arc4random_mod(501);
    let y_speed = y_random as i32 + 1000;
    logic::logic_mut(world, this).pos_vel.y = y_speed as f32 / 10.0;

    world.obj_mut::<PygmyBottomLogic>(this).is_active = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    display::insert_before(world, pygmy_display_object, Some(display_object));
    logic::set_behavior(world, this, Some("PygmyBottomLocustBone"));
}
