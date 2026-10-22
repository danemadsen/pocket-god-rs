//! `NoPeeSeaBlobLogic` (superclass `NoPeeObjectLogic`).

use crate::prelude::*;

use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_object_logic::{self, NoPeeObjectLogic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;

/// `NoPeeSeaBlobLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeSeaBlobLogic {
    /// `mNoPeeSeaBlobLogicListener`: `@"<NoPeeSeaBlobLogicListener>"`
    pub no_pee_sea_blob_logic_listener: Option<Entity>,

    /// `mNoPeeSeaBlobType`: `i`
    pub no_pee_sea_blob_type: i32,

    /// `mTargetPosVel`: `{?="x"f"y"f"z"f"w"f}`
    pub target_pos_vel: Vec4,

    /// `mSwimTimer`: `f`
    pub swim_timer: f32,

    /// `mChasePosVel`: `f`
    pub chase_pos_vel: f32,

    /// `mTargetYOffset`: `f`
    pub target_y_offset: f32,
}

pub const NO_PEE_SEA_BLOB_LOGIC: Class = Class {
    name: "NoPeeSeaBlobLogic",
    superclass: Some(&crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC),
    on_game_frame,
    ..crate::logic::nopee::no_pee_object_logic::NO_PEE_OBJECT_LOGIC
};

/// The blob spawns, and keeps drifting, within this y range.
const SWIM_MIN_Y_POS: f32 = 100.0;
const SWIM_MAX_Y_POS: f32 = 800.0;

/// The blob never leaves this y range.
const MIN_Y_POS: f32 = 0.0;
const MAX_Y_POS: f32 = 1000.0;

/// Farther from the pygmy than this, the blob aims `mTargetYOffset` above or below it.
const CHASE_OFFSET_DISTANCE: f32 = 300.0;

/// How fast a chasing blob's velocity turns towards its target, per second.
const CHASE_X_ACCELERATION: f32 = 200.0;
const CHASE_Y_ACCELERATION: f32 = 100.0;

/// The blob catches the pygmy within this distance.
const EAT_RADIUS: f32 = 20.0;

/// `[[NoPeeSeaBlobLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_SEA_BLOB_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeSeaBlobLogic initWithDisplayObject:]` @ 0x100178edc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeSeaBlobLogic::default());
    no_pee_object_logic::init_with_display_object(world, this, display_object);
}

/// `-[NoPeeSeaBlobLogic setNoPeeSeaBlobLogicListener:]` @ 0x100178f18
pub fn set_no_pee_sea_blob_logic_listener(world: &mut World, this: Entity, no_pee_sea_blob_logic_listener: Entity) {
    world.obj_mut::<NoPeeSeaBlobLogic>(this).no_pee_sea_blob_logic_listener = Some(no_pee_sea_blob_logic_listener);
    no_pee_object_logic::set_no_pee_object_logic_listener(world, this, no_pee_sea_blob_logic_listener);
}

/// `-[NoPeeSeaBlobLogic setNoPeeSeaBlobType:]` @ 0x100178f88
pub fn set_no_pee_sea_blob_type(world: &mut World, this: Entity, no_pee_sea_blob_type: i32) {
    world.obj_mut::<NoPeeSeaBlobLogic>(this).no_pee_sea_blob_type = no_pee_sea_blob_type;
}

/// `-[NoPeeSeaBlobLogic activate]` @ 0x100178f98
pub fn activate(world: &mut World, this: Entity) {
    world.obj_mut::<NoPeeSeaBlobLogic>(this).swim_timer = 0.0;
    logic::set_behavior(world, this, Some("NoPeeSeaBlobInit"));

    no_pee_object_logic::activate(world, this);
    no_pee_object_logic::random_spawn_position_no_overlap_with_min_y_pos(world, this, SWIM_MIN_Y_POS, SWIM_MAX_Y_POS);
}

/// `-[NoPeeSeaBlobLogic onGameFrame:]` @ 0x100179028
///
/// While the game is not active, the pygmy cannot be hit or the swim timer runs, the blob
/// drifts; otherwise it chases the pygmy. Either way it moves, stays within y 0…1000 and eats
/// the pygmy within 20 points.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let object_listener = world.obj::<NoPeeObjectLogic>(this).no_pee_object_logic_listener;
    let pygmy = match object_listener {
        Some(object_listener) => no_pee_logic::no_pee_pygmy_logic(world, object_listener),
        None => None,
    };

    if let Some(pygmy) = pygmy {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let display_object = logic::display_object(world, this);

        {
            let mut sea_blob = world.obj_mut::<NoPeeSeaBlobLogic>(this);
            sea_blob.swim_timer = sea_blob.swim_timer - dt;
        }

        let game_active = match object_listener {
            Some(object_listener) => no_pee_logic::is_game_active(world, object_listener),
            None => false,
        };
        let mut chase = false;
        if game_active {
            let can_be_hit = crate::pygmy::can_be_hit_by_no_pee_object(world, pygmy);
            if can_be_hit {
                let swim_timer = world.obj::<NoPeeSeaBlobLogic>(this).swim_timer;
                chase = swim_timer <= 0.0;
            }
        }

        if chase {
            chase_pygmy(world, this, display_object, pygmy_pos, dt);
        } else {
            drift(world, this);
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        {
            let mut display = display::object_mut(world, display_object);
            display.pos.x = display.pos.x + pos_vel.x * dt;
            display.pos.y = display.pos.y + pos_vel.y * dt;
            display.pos.z = display.pos.z + pos_vel.z * dt;

            if display.pos.y < MIN_Y_POS {
                display.pos.y = MIN_Y_POS;
            } else if display.pos.y > MAX_Y_POS {
                display.pos.y = MAX_Y_POS;
            }
        }

        let caught = no_pee_object_logic::check_collision(world, this, pygmy, EAT_RADIUS);
        if caught {
            {
                let mut state = logic::logic_mut(world, this);
                state.pos_vel.x = 0.0;
                state.pos_vel.y = 0.0;
            }
            crate::pygmy::hit_by_no_pee_sea_blob(world, pygmy, this);
            logic::set_behavior(world, this, Some("NoPeeSeaBlobEatPygmy"));
        }
    }

    no_pee_object_logic::on_game_frame(world, this, dt);
}

/// The drifting half of `onGameFrame:`: a new swim target once the timer has run out, then the
/// velocity eases half way towards it.
fn drift(world: &mut World, this: Entity) {
    let swim_timer = world.obj::<NoPeeSeaBlobLogic>(this).swim_timer;
    if swim_timer <= 0.0 {
        let random = world.resource_mut::<Random>().arc4random();
        let tenths = (random & 0xf) + 5;
        world.obj_mut::<NoPeeSeaBlobLogic>(this).swim_timer = tenths as i32 as f32 / 10.0;
        calculate_swim_pos_vel(world, this);
    }

    let target_pos_vel = world.obj::<NoPeeSeaBlobLogic>(this).target_pos_vel;
    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = state.pos_vel.x + (target_pos_vel.x - state.pos_vel.x) * 0.5;
    state.pos_vel.y = state.pos_vel.y + (target_pos_vel.y - state.pos_vel.y) * 0.5;
}

/// The chasing half of `onGameFrame:`: the target velocity points at the pygmy (offset by
/// `mTargetYOffset` while far away) with length `mChasePosVel`, and the velocity steps towards
/// it by 200 (x) and 100 (y) per second.
fn chase_pygmy(world: &mut World, this: Entity, display_object: Entity, pygmy_pos: Vec4, dt: f32) {
    let pos = display::object(world, display_object).pos;
    let pygmy_offset = Vec4::new(pos.x - pygmy_pos.x, pos.y - pygmy_pos.y, pos.z - pygmy_pos.z, pos.w);
    let pygmy_distance = length3(pygmy_offset);

    let mut target_y_pos = pygmy_pos.y;
    if pygmy_distance > CHASE_OFFSET_DISTANCE {
        let target_y_offset = world.obj::<NoPeeSeaBlobLogic>(this).target_y_offset;
        target_y_pos = target_y_pos + target_y_offset;
    }

    let mut target_pos_vel = world.obj::<NoPeeSeaBlobLogic>(this).target_pos_vel;
    target_pos_vel.x = pygmy_pos.x - pos.x;
    target_pos_vel.y = target_y_pos - pos.y;

    let chase_pos_vel = world.obj::<NoPeeSeaBlobLogic>(this).chase_pos_vel;
    let target_offset = Vec4::new(pos.x - pygmy_pos.x, pos.y - target_y_pos, pos.z - pygmy_pos.z, pos.w);
    let target_distance = length3(target_offset);
    let speed = chase_pos_vel / target_distance;
    target_pos_vel.x = speed * target_pos_vel.x;
    target_pos_vel.y = speed * target_pos_vel.y;
    target_pos_vel.z = speed * target_pos_vel.z;
    world.obj_mut::<NoPeeSeaBlobLogic>(this).target_pos_vel = target_pos_vel;

    let mut state = logic::logic_mut(world, this);

    let x_step = dt * CHASE_X_ACCELERATION;
    let vel_x = state.pos_vel.x;
    state.pos_vel.x = if vel_x < target_pos_vel.x { x_step + vel_x } else { vel_x - x_step };

    let y_step = dt * CHASE_Y_ACCELERATION;
    let vel_y = state.pos_vel.y;
    state.pos_vel.y = if vel_y < target_pos_vel.y { y_step + vel_y } else { vel_y - y_step };
}

/// `-[NoPeeSeaBlobLogic calculateSwimPosVel]` @ 0x10017940c
pub fn calculate_swim_pos_vel(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);

    let x_random = world.resource_mut::<Random>().arc4random_mod(2001);
    let x_tenths = x_random.wrapping_sub(1000);
    world.obj_mut::<NoPeeSeaBlobLogic>(this).target_pos_vel.x = x_tenths as i32 as f32 / 10.0;

    let y_random = world.resource_mut::<Random>().arc4random_mod(401);
    let y_tenths = y_random.wrapping_sub(200);
    world.obj_mut::<NoPeeSeaBlobLogic>(this).target_pos_vel.y = y_tenths as i32 as f32 / 10.0;

    // Swims back towards the middle when outside y 100…800.
    let y_pos = display::object(world, display_object).pos.y;
    let mut sea_blob = world.obj_mut::<NoPeeSeaBlobLogic>(this);
    if y_pos < SWIM_MIN_Y_POS {
        sea_blob.target_pos_vel.y = sea_blob.target_pos_vel.y.abs();
    } else if y_pos > SWIM_MAX_Y_POS {
        sea_blob.target_pos_vel.y = -sea_blob.target_pos_vel.y.abs();
    }
}

/// `-[NoPeeSeaBlobLogic onCollectNoPeeTreasure:]` @ 0x100179580
///
/// Each treasure makes the blobs faster and quicker to give chase (at once from the fifth).
pub fn on_collect_no_pee_treasure(world: &mut World, this: Entity, collect_no_pee_treasure: i32) {
    let score = collect_no_pee_treasure;

    let offset_random = world.resource_mut::<Random>().arc4random_mod(4001);
    let offset_tenths = offset_random.wrapping_sub(2000);
    world.obj_mut::<NoPeeSeaBlobLogic>(this).target_y_offset = offset_tenths as i32 as f32 / 10.0;

    let speed_random = world.resource_mut::<Random>().arc4random_mod(501);
    let speed_tenths = speed_random.wrapping_add(2300);
    let base_speed = speed_tenths as i32 as f32 / 10.0;
    world.obj_mut::<NoPeeSeaBlobLogic>(this).chase_pos_vel = score as f32 * 10.0 + base_speed;

    let timer_random = world.resource_mut::<Random>().arc4random_mod(11);
    let timer_tenths = timer_random.wrapping_add(10);
    let base_timer = timer_tenths as i32 as f32 / 10.0;
    world.obj_mut::<NoPeeSeaBlobLogic>(this).swim_timer = base_timer + score as f32 / -5.0;
    if score >= 5 {
        world.obj_mut::<NoPeeSeaBlobLogic>(this).swim_timer = 0.0;
    }

    calculate_swim_pos_vel(world, this);
}
