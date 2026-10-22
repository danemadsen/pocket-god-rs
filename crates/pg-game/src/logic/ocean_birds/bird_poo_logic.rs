//! `BirdPooLogic` (superclass `SpriteLogic`).
//!
//! The seagull's dropping (`docs/re/logic-catalog.md` §4.3). `BirdLogic` creates it when a tap
//! on the bird is released and is its listener. It falls onto a pygmy's head, the island floor
//! (y 76) or the sea, and the bird releases it when its `die:::` item runs.

use crate::prelude::*;

use crate::logic::island_core::coconut_logic::{
    THROWABLE_GRAVITY_FORCE, clamp_throwable_to_screen, integrate_throwable, is_throwable_out_of_range,
    reaches_island_floor,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_birds::bird_logic;
use pg_engine::display;
use pg_engine::logic;

/// `BirdPooLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BirdPooLogic {
    /// `mBirdPooLogicListener`: `@"<BirdPooLogicListener>"`
    pub bird_poo_logic_listener: Option<Entity>,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbHitIsland`: `B`
    pub hit_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbDropOnHead`: `B`
    pub drop_on_head: bool,
}

pub const BIRD_POO_LOGIC: Class = Class {
    name: "BirdPooLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The island floor (`setYPos:` in `onGameFrame:`); `checkPygmyCollision:` also needs the
/// dropping at or above it.
const FLOOR_Y: f32 = 76.0;

/// `fall:::` splashes below this y.
const WATER_LINE_Y: f32 = 30.0;

/// `checkPygmyCollision:` lands on a pygmy whose point 31 above its origin lies within 20 on
/// both axes.
const PYGMY_HEAD_Y_OFFSET: f32 = 31.0;
const PYGMY_HEAD_REACH: f32 = 20.0;

/// Sequence-item handlers implemented by `BirdPooLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[BirdPooLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BIRD_POO_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[BirdPooLogic initWithDisplayObject:]` @ 0x100179698
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BirdPooLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::set_behavior(world, this, Some("BirdPooFall"));
}

/// `-[BirdPooLogic setBirdPooLogicListener:]` @ 0x100179704
pub fn set_bird_poo_logic_listener(world: &mut World, this: Entity, bird_poo_logic_listener: Entity) {
    world.obj_mut::<BirdPooLogic>(this).bird_poo_logic_listener = Some(bird_poo_logic_listener);
}

/// `-[BirdPooLogic exitIsland:]` @ 0x100179760
///
/// A dropping in the water finishes its splash unless the exit is `now`.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let in_water = world.obj::<BirdPooLogic>(this).in_water;
    if in_water && !now {
        return;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BirdPooLogic>(this).enable_island_floor_collision = false;
    logic::set_behavior(world, this, Some("BirdPooDie"));
}

/// `-[BirdPooLogic checkPygmyCollision:]` @ 0x100179814
///
/// Over the island and at or above its floor, the dropping lands on the head of the first pygmy
/// in reach that takes the hit.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _dt: f32) {
    let display_object = logic::display_object(world, this);
    let listener = world.obj::<BirdPooLogic>(this).bird_poo_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = bird_logic::island_min_x_pos(world, listener);
        island_max_x_pos = bird_logic::island_max_x_pos(world, listener);
    }

    let drop_on_head = world.obj::<BirdPooLogic>(this).drop_on_head;
    if drop_on_head {
        return;
    }

    let pos = display::object(world, display_object).pos;
    let off_island = pos.x < island_min_x_pos || island_max_x_pos < pos.x;
    if off_island || pos.y < FLOOR_Y {
        return;
    }

    let pygmy_logic_array = match listener {
        Some(listener) => bird_logic::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmy_logic_array {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let pos = display::object(world, display_object).pos;
        let distance_x = (pos.x - (pygmy_pos.x + 0.0)).abs();
        let distance_y = (pos.y - (pygmy_pos.y + PYGMY_HEAD_Y_OFFSET)).abs();
        let on_head = distance_x < PYGMY_HEAD_REACH && distance_y < PYGMY_HEAD_REACH;
        if !on_head {
            continue;
        }

        let hit = crate::pygmy::hit_by_bird_poo(world, pygmy, this);
        if !hit {
            continue;
        }

        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        display::object_mut(world, display_object).rot.z = 0.0;
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
        display::object_mut(world, display_object).pos.x = pygmy_x_pos;

        world.obj_mut::<BirdPooLogic>(this).drop_on_head = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        logic::set_behavior(world, this, Some("BirdPooOnPygmyHead"));
        if let Some(listener) = listener {
            bird_logic::on_bird_poo_hit_pygmy(world, listener, this);
        }
        return;
    }
}

/// `-[BirdPooLogic onGameFrame:]` @ 0x100179afc
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let previous_y = display::object(world, display_object).pos.y;

    world.obj_mut::<SpriteLogic>(this).gravity_force = THROWABLE_GRAVITY_FORCE;
    let sprite = world.obj::<SpriteLogic>(this);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !ignore_gravity {
        pos_force.x = gravity_force.x + pos_force.x;
        pos_force.y = gravity_force.y + pos_force.y;
        pos_force.z = gravity_force.z + pos_force.z;
    }
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);
    clamp_throwable_to_screen(world, display_object);

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("BirdPooDie"));
    }

    world.obj_mut::<BirdPooLogic>(this).island_floor_collision = false;
    let listener = world.obj::<BirdPooLogic>(this).bird_poo_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = bird_logic::island_min_x_pos(world, listener);
        island_max_x_pos = bird_logic::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let reached_floor = reaches_island_floor(previous_y, pos, island_min_x_pos, island_max_x_pos, FLOOR_Y);
    let enable_island_floor_collision = world.obj::<BirdPooLogic>(this).enable_island_floor_collision;
    if reached_floor && enable_island_floor_collision {
        logic::logic_mut(world, this).pos_vel.y = 0.0;

        let on_island = world.obj::<BirdPooLogic>(this).on_island;
        if !on_island {
            {
                let mut bird_poo = world.obj_mut::<BirdPooLogic>(this);
                bird_poo.falling = false;
                bird_poo.on_island = true;
            }
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<BirdPooLogic>(this).island_floor_collision = true;
        }

        let mut display = display::object_mut(world, display_object);
        display.rot.z = 0.0;
        display.pos.y = FLOOR_Y;
    } else {
        world.obj_mut::<BirdPooLogic>(this).on_island = false;
    }

    check_pygmy_collision(world, this, dt);
}

/// `-[BirdPooLogic fall:::]` @ 0x100179e88
///
/// Lands on the island (a miss, 2) or splashes into the sea below y 30 (a miss, `BirdPooSplash`).
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut bird_poo = world.obj_mut::<BirdPooLogic>(this);
            if !bird_poo.island_floor_collision {
                bird_poo.on_island = false;
            }
            bird_poo.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BirdPooLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let listener = world.obj::<BirdPooLogic>(this).bird_poo_logic_listener;
    let island_floor_collision = world.obj::<BirdPooLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<BirdPooLogic>(this).falling = false;
        if let Some(listener) = listener {
            bird_logic::on_bird_poo_miss_pygmy(world, listener, this);
        }
        return ItemResult::Done;
    }

    let ocean_enabled = match listener {
        Some(listener) => bird_logic::is_ocean_enabled(world, listener),
        None => false,
    };
    let y_pos = display::object(world, display_object).pos.y;
    let below_water_line = y_pos < WATER_LINE_Y;
    if !ocean_enabled || !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut bird_poo = world.obj_mut::<BirdPooLogic>(this);
        bird_poo.falling = false;
        bird_poo.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    display::object_mut(world, display_object).rot.z = 0.0;

    if let Some(listener) = listener {
        bird_logic::on_bird_poo_miss_pygmy(world, listener, this);
    }
    logic::set_behavior(world, this, Some("BirdPooSplash"));
    ItemResult::Replaced
}

/// `-[BirdPooLogic die:::]` @ 0x10017a028
///
/// The bird releases the dropping in `onBirdPooDie:`: nothing touches it afterwards.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut bird_poo = world.obj_mut::<BirdPooLogic>(this);
        bird_poo.in_water = true;
        bird_poo.falling = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let listener = world.obj::<BirdPooLogic>(this).bird_poo_logic_listener;
    if let Some(listener) = listener {
        bird_logic::on_bird_poo_die(world, listener, this);
    }
    ItemResult::Done
}
