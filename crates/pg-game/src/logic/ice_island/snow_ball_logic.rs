//! `SnowBallLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ice_island::ice_monster_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};

/// `SnowBallLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SnowBallLogic {
    /// `mSnowBallLogicListener`: `@"<SnowBallLogicListener>"`
    pub snow_ball_logic_listener: Option<Entity>,

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

    /// `mbHitPygmy`: `B`
    pub hit_pygmy: bool,
}

pub const SNOW_BALL_LOGIC: Class = Class {
    name: "SnowBallLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `mGravityForce` as `onGameFrame:` resets it every frame.
const GRAVITY_FORCE: Vec4 = Vec4::new(0.0, -3000.0, 0.0, 1.0);

/// The island floor a snowball lands on.
const ISLAND_FLOOR_Y_POS: f32 = 80.0;

/// Sequence-item handlers implemented by `SnowBallLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[SnowBallLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SNOW_BALL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SnowBallLogic initWithDisplayObject:]` @ 0x1000103dc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SnowBallLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::set_behavior(world, this, Some("SnowBallFly"));
}

/// `-[SnowBallLogic setSnowBallLogicListener:]` @ 0x100010448
pub fn set_snow_ball_logic_listener(world: &mut World, this: Entity, snow_ball_logic_listener: Entity) {
    world.obj_mut::<SnowBallLogic>(this).snow_ball_logic_listener = Some(snow_ball_logic_listener);
}

/// `-[SnowBallLogic exitIsland:]` @ 0x1000104a4
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let in_water = world.obj::<SnowBallLogic>(this).in_water;
    if in_water && !exit_island {
        return;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<SnowBallLogic>(this).enable_island_floor_collision = false;
    pg_engine::logic::set_behavior(world, this, Some("SnowBallDie"));
}

/// `-[SnowBallLogic checkPygmyCollision:]` @ 0x100010558
///
/// A flying snowball near the sprite depth hits the first pygmy that is close enough and
/// accepts it (`mbHitPygmy` is never set, so it keeps checking afterwards).
pub fn check_pygmy_collision(world: &mut World, this: Entity, _check_pygmy_collision: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    let (hit_pygmy, on_island, in_water) = {
        let snow_ball = world.obj::<SnowBallLogic>(this);
        (snow_ball.hit_pygmy, snow_ball.on_island, snow_ball.in_water)
    };
    if hit_pygmy || on_island || in_water {
        return;
    }

    let z_pos = pg_engine::display::object(world, display_object).pos.z;
    if z_pos < -377.12 || -227.12 < z_pos {
        return;
    }

    let listener = world.obj::<SnowBallLogic>(this).snow_ball_logic_listener;
    let pygmies = match listener {
        Some(ice_monster) => ice_monster_logic::pygmy_logic_array(world, ice_monster, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_pos = pg_engine::display::object(world, pygmy_display_object).pos;
        let pos = pg_engine::display::object(world, display_object).pos;

        let delta_x = (pos.x - (pygmy_pos.x + 0.0)).abs();
        if !(delta_x < 40.0) {
            continue;
        }
        let delta_y = ((pos.y + 28.0) - (pygmy_pos.y + 20.0)).abs();
        if !(delta_y < 40.0) {
            continue;
        }

        if crate::pygmy::hit_by_snow_ball(world, pygmy, this) {
            let listener = world.obj::<SnowBallLogic>(this).snow_ball_logic_listener;
            if let Some(ice_monster) = listener {
                ice_monster_logic::on_snow_ball_hit_pygmy(world, ice_monster, this);
            }
            return;
        }
    }
}

/// `-[SnowBallLogic onGameFrame:]` @ 0x1000107b4
///
/// Its own integration (velocity first, then position), kept on screen; a snowball falling
/// through the island floor (y 80) over the island lands on it once the `fall` item enabled
/// the floor.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = pg_engine::logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;
    let previous_y_pos = pg_engine::display::object(world, display_object).pos.y;

    world.obj_mut::<SpriteLogic>(this).gravity_force = GRAVITY_FORCE;
    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
    if !ignore_gravity {
        pos_force.x = GRAVITY_FORCE.x + pos_force.x;
        pos_force.y = GRAVITY_FORCE.y + pos_force.y;
        pos_force.z = GRAVITY_FORCE.z + pos_force.z;
    }

    let pos_vel = {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_force = pos_force;
        state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
        state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
        state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
        state.pos_vel
    };
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
    }

    let min_pos = pg_engine::display::min_pos(world, display_object);
    let max_pos = pg_engine::display::max_pos(world, display_object);
    if min_pos.x < 0.0 {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = (object.pos.x - min_pos.x) + 0.0;
    }
    if 480.0 < max_pos.x {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = 480.0 - (max_pos.x - object.pos.x);
    }

    world.obj_mut::<SnowBallLogic>(this).island_floor_collision = false;

    let listener = world.obj::<SnowBallLogic>(this).snow_ball_logic_listener;
    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(ice_monster) = listener {
        island_min_x = ice_monster_logic::island_min_x_pos(world, ice_monster);
        island_max_x = ice_monster_logic::island_max_x_pos(world, ice_monster);
    }

    let pos = pg_engine::display::object(world, display_object).pos;
    let below_floor = pos.y < ISLAND_FLOOR_Y_POS;
    let over_island = island_min_x <= pos.x && pos.x <= island_max_x;
    let was_above_floor = previous_y_pos >= ISLAND_FLOOR_Y_POS;
    let floor_enabled = world.obj::<SnowBallLogic>(this).enable_island_floor_collision;

    if below_floor && over_island && was_above_floor && floor_enabled {
        {
            let mut state = pg_engine::logic::logic_mut(world, this);
            state.pos_vel.y = 0.0;
            state.pos_vel.z = 0.0;
        }

        let on_island = world.obj::<SnowBallLogic>(this).on_island;
        if !on_island {
            {
                let mut snow_ball = world.obj_mut::<SnowBallLogic>(this);
                snow_ball.falling = false;
                snow_ball.on_island = true;
            }
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<SnowBallLogic>(this).island_floor_collision = true;
        }

        let mut object = pg_engine::display::object_mut(world, display_object);
        object.rot.z = 0.0;
        object.pos.y = ISLAND_FLOOR_Y_POS;
    } else {
        world.obj_mut::<SnowBallLogic>(this).on_island = false;
    }

    check_pygmy_collision(world, this, dt);
}

/// `-[SnowBallLogic fall:::]` @ 0x100010b00
///
/// Completes when the snowball lands on the island and is replaced by `SnowBallSplash` when
/// it drops into the sea (y < 30); both count as a miss.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let island_floor_collision = world.obj::<SnowBallLogic>(this).island_floor_collision;
        if !island_floor_collision {
            world.obj_mut::<SnowBallLogic>(this).on_island = false;
        }
        world.obj_mut::<SnowBallLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SnowBallLogic>(this).falling = true;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let island_floor_collision = world.obj::<SnowBallLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<SnowBallLogic>(this).falling = false;

        let listener = world.obj::<SnowBallLogic>(this).snow_ball_logic_listener;
        if let Some(ice_monster) = listener {
            ice_monster_logic::on_snow_ball_miss_pygmy(world, ice_monster, this);
        }
        return ItemResult::Done;
    }

    let y_pos = pg_engine::display::object(world, display_object).pos.y;
    if !(y_pos < 30.0) {
        return ItemResult::Pending;
    }

    {
        let mut snow_ball = world.obj_mut::<SnowBallLogic>(this);
        snow_ball.falling = false;
        snow_ball.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
        state.pos_vel.z = 0.0;
    }
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;

    let listener = world.obj::<SnowBallLogic>(this).snow_ball_logic_listener;
    if let Some(ice_monster) = listener {
        ice_monster_logic::on_snow_ball_miss_pygmy(world, ice_monster, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("SnowBallSplash"));
    ItemResult::Replaced
}

/// `-[SnowBallLogic die:::]` @ 0x100010c80
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut snow_ball = world.obj_mut::<SnowBallLogic>(this);
        snow_ball.in_water = true;
        snow_ball.falling = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let listener = world.obj::<SnowBallLogic>(this).snow_ball_logic_listener;
    if let Some(ice_monster) = listener {
        ice_monster_logic::on_snow_ball_die(world, ice_monster, this);
    }
    ItemResult::Done
}
