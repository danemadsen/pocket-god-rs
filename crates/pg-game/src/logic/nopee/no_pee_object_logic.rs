//! `NoPeeObjectLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::nopee::no_pee_logic;
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeObjectLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeObjectLogic {
    /// `mNoPeeObjectLogicListener`: `@"<NoPeeObjectLogicListener>"`
    pub no_pee_object_logic_listener: Option<Entity>,
}

pub const NO_PEE_OBJECT_LOGIC: Class = Class {
    name: "NoPeeObjectLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The depth of every object of the No Pee world (`0xc38a8f5c`).
pub const NO_PEE_Z_POS: f32 = -277.12;

/// The screen centre's x in the No Pee world is `SCREEN_CENTER_X − group.x`.
pub const SCREEN_CENTER_X: f32 = 240.0;

/// Objects stay within this distance of the screen centre: the world wraps every
/// `WORLD_WIDTH` points.
const WORLD_HALF_WIDTH: f32 = 1500.0;
const WORLD_WIDTH: f32 = 3000.0;

/// `randomSpawnPositionNoOverlapWithMinYPos:maxYPos:` keeps this far from every other
/// collidable object.
const SPAWN_CLEARANCE: f32 = 200.0;

/// The visible part of the world: 480 × 320 points from `−group.pos`.
const SCREEN_WIDTH: f32 = 480.0;
const SCREEN_HEIGHT: f32 = 320.0;

/// `[[NoPeeObjectLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_OBJECT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeObjectLogic initWithDisplayObject:]` @ 0x100034670
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeObjectLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[NoPeeObjectLogic setNoPeeObjectLogicListener:]` @ 0x1000346ac
pub fn set_no_pee_object_logic_listener(world: &mut World, this: Entity, no_pee_object_logic_listener: Entity) {
    world.obj_mut::<NoPeeObjectLogic>(this).no_pee_object_logic_listener = Some(no_pee_object_logic_listener);
}

/// `-[NoPeeObjectLogic activate]` @ 0x100034708
pub fn activate(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "NoPeePygmyDisplayMarker");

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    if display_group.is_some() {
        return;
    }

    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[NoPeeObjectLogic shouldHideNoPeeMapItem]` @ 0x1000347a8
pub fn should_hide_no_pee_map_item(_world: &mut World, _this: Entity) -> bool {
    false
}

/// `-[NoPeeObjectLogic randomSpawnPosition]` @ 0x1000347b0
pub fn random_spawn_position(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);

    let x_random = world.resource_mut::<Random>().arc4random_mod(30001);
    let x_pos = x_random as i32 as f32 / 10.0;
    display::object_mut(world, display_object).pos.x = x_pos;

    let y_random = world.resource_mut::<Random>().arc4random_mod(7001);
    let y_tenths = y_random.wrapping_add(1000);
    let y_pos = y_tenths as i32 as f32 / 10.0;

    let mut display = display::object_mut(world, display_object);
    display.pos.y = y_pos;
    display.pos.z = NO_PEE_Z_POS;
}

/// `-[NoPeeObjectLogic randomSpawnPositionNoOverlapWithMinYPos:maxYPos:]` @ 0x10003486c
///
/// Draws a position within ±1500 of the screen centre (in tenths of a point) until it keeps
/// clear of every other collidable object, both where it is and one world width away, and lies
/// off screen.
pub fn random_spawn_position_no_overlap_with_min_y_pos(world: &mut World, this: Entity, min_y_pos: f32, max_y_pos: f32) {
    let min_y_tenths = (min_y_pos * 10.0) as i32;
    let max_y_tenths = (max_y_pos * 10.0) as i32;
    let y_range = 1_i32.wrapping_sub(min_y_tenths).wrapping_add(max_y_tenths) as u32;

    // The listener is always the `NoPeeLogic` (a nil one would crash the original here).
    let listener = world.obj::<NoPeeObjectLogic>(this).no_pee_object_logic_listener;
    let Some(listener) = listener else {
        return;
    };
    let display_object = logic::display_object(world, this);

    loop {
        let no_pee = no_pee_logic::no_pee_logic(world, listener);
        let group_pos = no_pee_logic::world_group_pos(world, no_pee);
        let center_x_pos = SCREEN_CENTER_X - group_pos.x;

        let min_x_tenths = ((center_x_pos + -WORLD_HALF_WIDTH) * 10.0) as i32;
        let max_x_tenths = ((center_x_pos + WORLD_HALF_WIDTH) * 10.0) as i32;
        let x_range = max_x_tenths.wrapping_add(1).wrapping_sub(min_x_tenths) as u32;
        let x_random = world.resource_mut::<Random>().arc4random_mod(x_range);
        let x_tenths = x_random.wrapping_add(min_x_tenths as u32);
        let x_pos = x_tenths as i32 as f32 / 10.0;

        let y_random = world.resource_mut::<Random>().arc4random_mod(y_range);
        let y_tenths = y_random.wrapping_add(min_y_tenths as u32);
        let y_pos = y_tenths as i32 as f32 / 10.0;

        {
            let mut display = display::object_mut(world, display_object);
            display.pos.x = x_pos;
            display.pos.y = y_pos;
            display.pos.z = NO_PEE_Z_POS;
        }

        // The same spot one world width away, on the other side of the screen centre.
        let wrap_x_offset = if x_pos > center_x_pos { -WORLD_WIDTH } else { WORLD_WIDTH };
        let wrapped_x_pos = x_pos + wrap_x_offset;

        let collidables = no_pee_logic::no_pee_collidable_logic_array(world, listener);
        let mut overlaps = false;
        for collidable in collidables {
            if collidable == this {
                continue;
            }

            let collidable_display_object = logic::display_object(world, collidable);
            let collidable_pos = display::object(world, collidable_display_object).pos;

            let delta_x = x_pos - collidable_pos.x;
            let delta_y = y_pos - collidable_pos.y;
            let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

            let wrapped_delta_x = wrapped_x_pos - collidable_pos.x;
            let wrapped_delta_y = y_pos - collidable_pos.y;
            let wrapped_distance = (wrapped_delta_x * wrapped_delta_x + wrapped_delta_y * wrapped_delta_y).sqrt();

            if distance < SPAWN_CLEARANCE || wrapped_distance < SPAWN_CLEARANCE {
                overlaps = true;
                break;
            }
        }
        if overlaps {
            continue;
        }

        let on_screen_x = x_pos > -group_pos.x && x_pos < SCREEN_WIDTH - group_pos.x;
        if !on_screen_x {
            break;
        }

        let on_screen_y = y_pos > -group_pos.y && y_pos < SCREEN_HEIGHT - group_pos.y;
        if !on_screen_y {
            break;
        }
    }
}

/// `-[NoPeeObjectLogic checkCollision:collisionRadius:]` @ 0x100034bd8
pub fn check_collision(world: &mut World, this: Entity, pygmy_logic: Entity, collision_radius: f32) -> bool {
    let can_be_hit = crate::pygmy::can_be_hit_by_no_pee_object(world, pygmy_logic);
    if !can_be_hit {
        return false;
    }

    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;

    let delta_x = pygmy_pos.x - pos.x;
    let delta_y = pygmy_pos.y - pos.y;
    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
    distance < collision_radius
}

/// `-[NoPeeObjectLogic onGameFrame:]` @ 0x100034c98
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let listener = world.obj::<NoPeeObjectLogic>(this).no_pee_object_logic_listener;
    let Some(listener) = listener else {
        return;
    };
    let pygmy = no_pee_logic::no_pee_pygmy_logic(world, listener);
    if pygmy.is_none() {
        return;
    }

    let no_pee = no_pee_logic::no_pee_logic(world, listener);
    let group_pos = no_pee_logic::world_group_pos(world, no_pee);
    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;

    // Wraps the object around the screen centre by one world width.
    let center_offset = x_pos - (SCREEN_CENTER_X - group_pos.x);
    let wrap_x_offset = if center_offset < -WORLD_HALF_WIDTH {
        WORLD_WIDTH
    } else if center_offset > WORLD_HALF_WIDTH {
        -WORLD_WIDTH
    } else {
        return;
    };
    display::object_mut(world, display_object).pos.x = x_pos + wrap_x_offset;
}
