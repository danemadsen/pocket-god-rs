//! `TRexEggLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::monsters::t_rex_logic;
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `TRexEggLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TRexEggLogic {
    /// `mTRexEggLogicListener`: `@"<TRexEggLogicListener>"`
    pub t_rex_egg_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mCrushedPygmyLogic`: `@"PygmyLogic"`
    pub crushed_pygmy_logic: Option<Entity>,

    /// `mbCallingForMom`: `B`
    pub calling_for_mom: bool,

    /// `mbResetBounce`: `B`
    pub reset_bounce: bool,

    /// `mCrackCount`: `i`
    pub crack_count: i32,

    /// `mBowledPygmyCount`: `i`
    pub bowled_pygmy_count: i32,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const T_REX_EGG_LOGIC: Class = Class {
    name: "TRexEggLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The resting height of the egg on the island (`setYPos:119`).
const FLOOR_Y_POS: f32 = 119.0;

/// Below this height the egg splashes into the sea.
const WATER_Y_POS: f32 = 50.0;

/// Where `enterIsland:` drops the egg in from.
const ENTER_X_POS: f32 = 284.0;
const ENTER_Y_POS: f32 = 400.0;

/// `mCrushedPygmyLogic` pushes the egg away at this speed, by side (`DAT_10029d410`).
const CRUSH_X_VELS: [f32; 2] = [100.0, -100.0];

/// The cracks after which the baby calls its mother.
const CRACKS_TO_HATCH: i32 = 3;

/// Sequence-item handlers implemented by `TRexEggLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "callForMom" => seq_call_for_mom(world, this, call),
        "slide" => seq_slide(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[TRexEggLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&T_REX_EGG_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TRexEggLogic initWithDisplayObject:]` @ 0x1001710e0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TRexEggLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.tilt_state = 0;
        egg.slide_friction = 0.025;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<TRexEggLogic>(this).enable_island_floor_collision = true;

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, own_display_object);
}

/// `-[TRexEggLogic setTRexEggLogicListener:]` @ 0x100171190
pub fn set_t_rex_egg_logic_listener(world: &mut World, this: Entity, t_rex_egg_logic_listener: Entity) {
    world.obj_mut::<TRexEggLogic>(this).t_rex_egg_logic_listener = Some(t_rex_egg_logic_listener);
}

/// `-[TRexEggLogic clearPygmyLogic:]` @ 0x1001711ec
pub fn clear_pygmy_logic(world: &mut World, this: Entity, clear_pygmy_logic: Entity) {
    let mut egg = world.obj_mut::<TRexEggLogic>(this);
    if egg.crushed_pygmy_logic == Some(clear_pygmy_logic) {
        egg.crushed_pygmy_logic = None;
    }
}

/// `[listener islandMinXPos]` and `[listener islandMaxXPos]` (zero for a nil listener).
fn island_x_range(world: &mut World, listener: Option<Entity>) -> (f32, f32) {
    let Some(listener) = listener else {
        return (0.0, 0.0);
    };

    let island_min_x = crate::controller::island_min_x_pos(world, listener);
    let island_max_x = crate::controller::island_max_x_pos(world, listener);
    (island_min_x, island_max_x)
}

/// `[mCrushedPygmyLogic uncrushedByTRexEgg:self dontWait:]`, then forgets the pygmy.
fn release_crushed_pygmy(world: &mut World, this: Entity, dont_wait: bool) {
    let crushed = world.obj::<TRexEggLogic>(this).crushed_pygmy_logic;
    if let Some(crushed) = crushed {
        crate::pygmy::uncrushed_by_t_rex_egg(world, crushed, this, dont_wait);
        world.obj_mut::<TRexEggLogic>(this).crushed_pygmy_logic = None;
    }
}

/// `-[TRexEggLogic enterIsland:]` @ 0x100171208
///
/// Drops a fresh egg in from (284, 400), or puts it straight on the floor at startup.
pub fn enter_island(world: &mut World, this: Entity, enter_island: bool) {
    if world.obj::<TRexEggLogic>(this).falling {
        return;
    }

    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.calling_for_mom = false;
        egg.reset_bounce = false;
        egg.crack_count = 0;
    }
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    world.obj_mut::<TRexEggLogic>(this).in_water = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.is_selectable = true;
        object.pos.x = ENTER_X_POS;
        object.pos.y = ENTER_Y_POS;
        if enter_island {
            object.pos.y = FLOOR_Y_POS;
        }
    }

    let marker = pg_engine::library::display_marker(world, "IslandBackDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("TRexEggEnterIsland"));
}

/// `-[TRexEggLogic exitIsland:]` @ 0x10017137c
///
/// Lets go of everything; the egg dies at once (`now`) or falls off behind the island.
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    release_crushed_pygmy(world, this, exit_island);

    if world.obj::<TRexEggLogic>(this).dragging {
        {
            let mut egg = world.obj_mut::<TRexEggLogic>(this);
            egg.falling = true;
            egg.dragging = false;
            egg.apply_spring = false;
        }
        let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_t_rex_egg_drag_complete(world, listener, this);
        }
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<TRexEggLogic>(this).enable_island_floor_collision = false;

    if exit_island {
        pg_engine::logic::set_behavior(world, this, Some("TRexEggDie"));
        return;
    }
    if world.obj::<TRexEggLogic>(this).in_water {
        return;
    }

    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("TRexEggFallIgnoreIsland"));
}

/// `-[TRexEggLogic exitIslandNow]` @ 0x1001714f8
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[TRexEggLogic checkPygmyCollision:]` @ 0x100171570
///
/// While the egg is over the island at or above the floor: Charlie may smack a moving egg
/// away, a pygmy under a dropped egg is crushed (and pins it), and a fast egg bowls pygmies
/// over.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _check_pygmy_collision: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    let egg_pos = pg_engine::display::object(world, display_object).pos;

    let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
    let (island_min_x, island_max_x) = island_x_range(world, listener);

    let (crushed, enable_island_floor_collision) = {
        let egg = world.obj::<TRexEggLogic>(this);
        (egg.crushed_pygmy_logic, egg.enable_island_floor_collision)
    };
    if crushed.is_some() {
        return;
    }
    let over_island = island_min_x <= egg_pos.x && egg_pos.x <= island_max_x;
    let above_floor = FLOOR_Y_POS <= egg_pos.y;
    if !(enable_island_floor_collision && over_island && above_floor) {
        return;
    }

    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_pos = pg_engine::display::object(world, pygmy_display_object).pos;
        let delta_x = egg_pos.x - (pygmy_pos.x + 0.0);
        let delta_y = egg_pos.y - (pygmy_pos.y + 31.0);
        let touching = delta_x * delta_x + delta_y * delta_y < 900.0;
        if !touching {
            continue;
        }

        let personality = crate::pygmy::persona_personality(world, pygmy);
        if personality == 1 {
            let vel = pg_engine::logic::logic(world, this).pos_vel;
            let moving = 20.0 < vel.x.abs() || 20.0 < vel.y.abs();
            if !moving {
                continue;
            }
            if !crate::pygmy::charlie_smack_t_rex_egg(world, pygmy, this) {
                continue;
            }

            let mut logic = pg_engine::logic::logic_mut(world, this);
            logic.pos_vel.x = 200.0;
            logic.pos_vel.y = 600.0;
            if delta_x < 0.0 {
                logic.pos_vel.x = -200.0;
            }
            return;
        }

        let vel = pg_engine::logic::logic(world, this).pos_vel;
        let speed_x = vel.x.abs();
        let dropped_on_pygmy = vel.y < -20.0 && speed_x < 30.0;
        if dropped_on_pygmy {
            if crate::pygmy::crushed_by_t_rex_egg(world, pygmy, this) {
                let push_index = usize::from(0.0 < delta_x);
                pg_engine::logic::logic_mut(world, this).pos_vel.x = CRUSH_X_VELS[push_index];
                world.obj_mut::<TRexEggLogic>(this).crushed_pygmy_logic = Some(pygmy);
                return;
            }
            continue;
        }

        let fast = 200.0 < speed_x || 200.0 < vel.y.abs();
        if fast && crate::pygmy::hit_by_t_rex_egg(world, pygmy, this, delta_x * -3.0) {
            world.obj_mut::<TRexEggLogic>(this).bowled_pygmy_count += 1;
            pg_engine::logic::logic_mut(world, this).pos_vel.y = 300.0;
        }
    }
}

/// `-[TRexEggLogic isAccelEnabled]` @ 0x100171988
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    if !pg_engine::display::object(world, display_object).is_selectable {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[TRexEggLogic onGameFrame:]` @ 0x100171a1c
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let previous_y = pg_engine::display::object(world, display_object).pos.y;

    // Gravity follows the device tilt while the accelerometer drives the egg.
    if is_accel_enabled(world, this) {
        let accel_force = world.resource::<BcView>().accel_force;
        let tilt = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let mut gravity = pg_engine::math::normalize3(tilt);
        gravity.x = gravity.x * 3000.0;
        gravity.y = gravity.y * 3000.0;
        gravity.z = gravity.z * 3000.0;
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = Vec4::new(0.0, -3000.0, 0.0, 1.0);
        undo_tilt_state(world, this);
    }

    let (tilt_state, slide_friction, apply_spring, spring_force) = {
        let egg = world.obj::<TRexEggLogic>(this);
        (egg.tilt_state, egg.slide_friction, egg.apply_spring, egg.spring_force)
    };
    let (ignore_gravity, gravity) = {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        if tilt_state == 1 {
            sprite.gravity_force.x = slide_friction * sprite.gravity_force.x;
        } else {
            sprite.gravity_force.x = 0.0;
        }
        (sprite.ignore_gravity, sprite.gravity_force)
    };

    let mut force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !ignore_gravity {
        force.x = gravity.x + force.x;
        force.y = gravity.y + force.y;
        force.z = gravity.z + force.z;
    }
    if apply_spring {
        force.x = spring_force.x + force.x;
        force.y = spring_force.y + force.y;
        force.z = spring_force.z + force.z;
    }

    let vel = {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_force = force;
        logic.pos_vel.x = logic.pos_vel.x + force.x * dt;
        logic.pos_vel.y = logic.pos_vel.y + force.y * dt;
        logic.pos_vel.z = logic.pos_vel.z + force.z * dt;
        logic.pos_vel
    };
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = object.pos.x + vel.x * dt;
        object.pos.y = object.pos.y + vel.y * dt;
        object.pos.z = object.pos.z + vel.z * dt;
    }

    // Keep the egg on the 480-point-wide screen; both bounds are measured before either moves it.
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

    // A crushed pygmy pins the egg's x.
    let crushed = world.obj::<TRexEggLogic>(this).crushed_pygmy_logic;
    if let Some(crushed) = crushed {
        let crushed_display_object = pg_engine::logic::display_object(world, crushed);
        let crushed_x = pg_engine::display::object(world, crushed_display_object).pos.x;

        let vel_x = pg_engine::logic::logic(world, this).pos_vel.x;
        let pos_x = pg_engine::display::object(world, display_object).pos.x;
        if 0.0 < vel_x && crushed_x < pos_x {
            pg_engine::display::object_mut(world, display_object).pos.x = crushed_x;
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        }

        let vel_x = pg_engine::logic::logic(world, this).pos_vel.x;
        let pos_x = pg_engine::display::object(world, display_object).pos.x;
        if vel_x < 0.0 && pos_x < crushed_x {
            pg_engine::display::object_mut(world, display_object).pos.x = crushed_x;
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        }
    }

    world.obj_mut::<TRexEggLogic>(this).island_floor_collision = false;
    let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
    let (island_min_x, island_max_x) = island_x_range(world, listener);

    let pos = pg_engine::display::object(world, display_object).pos;
    let enable_island_floor_collision = world.obj::<TRexEggLogic>(this).enable_island_floor_collision;
    let over_island = island_min_x <= pos.x && pos.x <= island_max_x;
    let crossed_floor = pos.y < FLOOR_Y_POS && FLOOR_Y_POS <= previous_y;
    if crossed_floor && over_island && enable_island_floor_collision {
        land_on_floor(world, this, listener);
        pg_engine::display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;
    } else {
        world.obj_mut::<TRexEggLogic>(this).on_island = false;
    }

    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    if pos_y < -100.0 || 820.0 < pos_y {
        pg_engine::logic::set_behavior(world, this, Some("TRexEggDie"));
    }

    check_pygmy_collision(world, this, dt);
}

/// The floor contact of `onGameFrame:`: a hard bounce plays `TRexEggBounce`, or the next crack
/// once the egg has settled since the last one (and the T-Rex is enabled); settling after the
/// last crack makes the baby call its mother, and settling reports the pygmies bowled over.
fn land_on_floor(world: &mut World, this: Entity, listener: Option<Entity>) {
    let on_island = world.obj::<TRexEggLogic>(this).on_island;
    let vel = {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        if on_island {
            logic.pos_vel.y = 0.0;
        } else {
            logic.pos_vel.x = logic.pos_vel.x * 0.4;
            logic.pos_vel.y = logic.pos_vel.y * -0.4;
        }
        logic.pos_vel
    };

    let settled = vel.x.abs() < 30.0 && vel.y < 30.0;
    if !settled {
        let calling_for_mom = world.obj::<TRexEggLogic>(this).calling_for_mom;
        if 100.0 < vel.y && !calling_for_mom {
            let behavior = bounce_behavior(world, this, listener);
            pg_engine::logic::set_behavior(world, this, Some(behavior));
        }
        return;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
    if world.obj::<TRexEggLogic>(this).on_island {
        return;
    }

    if world.obj::<TRexEggLogic>(this).crack_count == CRACKS_TO_HATCH {
        {
            let mut egg = world.obj_mut::<TRexEggLogic>(this);
            egg.falling = false;
            egg.crack_count = 0;
            egg.calling_for_mom = true;
        }
        pg_engine::logic::set_behavior(world, this, Some("TRexEggCallForMom"));
    }

    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.reset_bounce = true;
        egg.on_island = true;
    }
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<TRexEggLogic>(this).island_floor_collision = true;

    let bowled_pygmy_count = world.obj::<TRexEggLogic>(this).bowled_pygmy_count;
    if 0 < bowled_pygmy_count {
        if let Some(listener) = listener {
            crate::controller::on_pygmies_bowled(world, listener, bowled_pygmy_count);
        }
        world.obj_mut::<TRexEggLogic>(this).bowled_pygmy_count = 0;
    }
}

/// The behaviour of a hard bounce: the next crack (counting it) when the egg has settled since
/// the last crack and the T-Rex is enabled, else a plain bounce.
fn bounce_behavior(world: &mut World, this: Entity, listener: Option<Entity>) -> &'static str {
    if !world.obj::<TRexEggLogic>(this).reset_bounce {
        return "TRexEggBounce";
    }
    let t_rex_enabled = match listener {
        Some(listener) => crate::controller::is_t_rex_enabled(world, listener),
        None => false,
    };
    if !t_rex_enabled {
        return "TRexEggBounce";
    }

    let crack_count = {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.reset_bounce = false;
        let crack_count = egg.crack_count;
        egg.crack_count = crack_count + 1;
        crack_count
    };
    match crack_count {
        0 => "TRexEggBounceCrack1",
        1 => "TRexEggBounceCrack2",
        2 => "TRexEggBounceCrack3",
        _ => "TRexEggBounce",
    }
}

/// `-[TRexEggLogic refreshTiltState]` @ 0x100172154
///
/// On the floor the egg slides with a moderate tilt and falls off with a steep one; a steep
/// tilt also frees a crushed pygmy.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island) = {
        let egg = world.obj::<TRexEggLogic>(this);
        (egg.dragging, egg.falling, egg.on_island)
    };

    if !dragging && !falling && on_island {
        let raw_roll = world.resource::<BcView>().raw_accel_rot_degrees.z;
        let mut tilt_state = 0;
        let beyond_slide = raw_roll < -130.0 || 130.0 < raw_roll;
        if !beyond_slide {
            let sliding = 90.0 < raw_roll || raw_roll < -90.0;
            tilt_state = if sliding { 1 } else { 2 };
        }

        if tilt_state != world.obj::<TRexEggLogic>(this).tilt_state {
            if tilt_state == 0 {
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                if !world.obj::<TRexEggLogic>(this).calling_for_mom {
                    pg_engine::logic::set_behavior(world, this, Some("TRexEggStopSlide"));
                }
                world.obj_mut::<TRexEggLogic>(this).tilt_state = 0;
            } else if tilt_state == 1 {
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                if !world.obj::<TRexEggLogic>(this).calling_for_mom {
                    pg_engine::logic::set_behavior(world, this, Some("TRexEggTiltSlide"));
                }
                world.obj_mut::<TRexEggLogic>(this).tilt_state = 1;
            }
        }

        let roll = world.resource::<BcView>().accel_rot_degrees.z;
        let tipped_over = -90.0 < roll && roll < 90.0;
        if !tipped_over {
            return;
        }
        {
            let mut egg = world.obj_mut::<TRexEggLogic>(this);
            egg.falling = true;
            egg.tilt_state = 0;
        }
        if !world.obj::<TRexEggLogic>(this).calling_for_mom {
            pg_engine::logic::set_behavior(world, this, Some("TRexEggFall"));
        }
    } else {
        let roll = world.resource::<BcView>().accel_rot_degrees.z;
        let tipped_over = -90.0 < roll && roll < 90.0;
        if !tipped_over {
            return;
        }
    }

    release_crushed_pygmy(world, this, false);
}

/// `-[TRexEggLogic undoTiltState]` @ 0x1001723d0
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island, tilt_state) = {
        let egg = world.obj::<TRexEggLogic>(this);
        (egg.dragging, egg.falling, egg.on_island, egg.tilt_state)
    };
    if dragging || falling || !on_island {
        return;
    }

    if tilt_state == 1 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        if !world.obj::<TRexEggLogic>(this).calling_for_mom {
            pg_engine::logic::set_behavior(world, this, Some("TRexEggStopSlide"));
        }
    }
    world.obj_mut::<TRexEggLogic>(this).tilt_state = 0;
}

/// `-[TRexEggLogic callForMom:::]` @ 0x100172470
pub fn seq_call_for_mom(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.falling = false;
        egg.calling_for_mom = false;
    }

    let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
    let t_rex = match listener {
        Some(listener) => crate::controller::t_rex_logic(world, listener, this),
        None => None,
    };
    if let Some(t_rex) = t_rex {
        t_rex_logic::call_mom(world, t_rex);
    }
    ItemResult::Done
}

/// `-[TRexEggLogic slide:::]` @ 0x1001724d0
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<TRexEggLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    let below_water = pos_y < WATER_Y_POS;
    if !below_water {
        return ItemResult::Pending;
    }

    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.falling = false;
        egg.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    world.obj_mut::<TRexEggLogic>(this).tilt_state = 0;
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("TRexEggSplash"));
    ItemResult::Replaced
}

/// `-[TRexEggLogic drag:::]` @ 0x1001725d0
///
/// Pulls the egg towards the finger with a damped spring.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<TRexEggLogic>(this).enable_island_floor_collision = false;

        let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
        if let Some(marker) = marker {
            pg_engine::display::insert_after(world, marker, Some(display_object));
        }
    }

    let (pos, display_group) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<TRexEggLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => pg_engine::display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The original subtracts the egg's own z, so the spring never acts in depth.
    let mut delta = pos;
    delta.x = pos.x - touch_pos.x;
    delta.y = pos.y - touch_pos.y;
    delta.z = pos.z - pos.z;

    let distance = pg_engine::math::length3(delta);
    if distance != 0.0 {
        let vel = pg_engine::logic::logic(world, this).pos_vel;
        let spring = (distance * -50.0) / distance;

        let mut spring_force = delta;
        spring_force.x = -6.0 * vel.x + spring * delta.x;
        spring_force.y = -6.0 * vel.y + spring * delta.y;
        spring_force.z = -6.0 * vel.z + spring * delta.z;
        world.obj_mut::<TRexEggLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[TRexEggLogic fall:::]` @ 0x100172770
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut egg = world.obj_mut::<TRexEggLogic>(this);
            if !egg.island_floor_collision {
                egg.on_island = false;
            }
            egg.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<TRexEggLogic>(this).falling = true;
        if call.bool("ignoreIsland") {
            world.obj_mut::<TRexEggLogic>(this).enable_island_floor_collision = false;
        }
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    if world.obj::<TRexEggLogic>(this).island_floor_collision {
        world.obj_mut::<TRexEggLogic>(this).falling = false;
        return ItemResult::Done;
    }
    let below_water = pos_y < WATER_Y_POS;
    if !below_water {
        return ItemResult::Pending;
    }

    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.falling = false;
        egg.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("TRexEggSplash"));
    ItemResult::Replaced
}

/// `-[TRexEggLogic die:::]` @ 0x1001728d4
///
/// Resets the egg and drops a new one in while the feature is enabled.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);

    let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
    let bowled_pygmy_count = world.obj::<TRexEggLogic>(this).bowled_pygmy_count;
    if 0 < bowled_pygmy_count {
        if let Some(listener) = listener {
            crate::controller::on_pygmies_bowled(world, listener, bowled_pygmy_count);
        }
        world.obj_mut::<TRexEggLogic>(this).bowled_pygmy_count = 0;
    }

    release_crushed_pygmy(world, this, false);

    if world.obj::<TRexEggLogic>(this).dragging {
        world.obj_mut::<TRexEggLogic>(this).dragging = false;
        let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_t_rex_egg_drag_complete(world, listener, this);
        }
    }

    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.falling = false;
        egg.in_water = false;
        egg.apply_spring = false;
        egg.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<TRexEggLogic>(this).crushed_pygmy_logic = None;

    let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
    let t_rex_egg_enabled = match listener {
        Some(listener) => crate::controller::is_t_rex_egg_enabled(world, listener),
        None => false,
    };
    if t_rex_egg_enabled {
        enter_island(world, this, false);
    }
    ItemResult::Done
}

/// `-[TRexEggLogic touchBeganInside:allTouches:]` @ 0x100172a48
///
/// Picks the egg up, unless a story runs or the egg is falling, swimming or hatching.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    let (falling, in_water, calling_for_mom, active_touch) = {
        let egg = world.obj::<TRexEggLogic>(this);
        (egg.falling, egg.in_water, egg.calling_for_mom, egg.active_touch)
    };
    if story_active || falling || in_water || calling_for_mom || active_touch.is_some() {
        return;
    }

    release_crushed_pygmy(world, this, false);

    world.obj_mut::<TRexEggLogic>(this).active_touch = Some(touch);
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    world.obj_mut::<TRexEggLogic>(this).touch_began_inside = true;

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut egg = world.obj_mut::<TRexEggLogic>(this);
        egg.touch_screen_pos = touch_pos;
        egg.tilt_state = 0;
        egg.dragging = true;
        egg.apply_spring = true;
        egg.touch_began_inside = false;
        egg.enable_island_floor_collision = false;
    }

    if let Some(listener) = listener {
        crate::controller::on_t_rex_egg_drag(world, listener, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("TRexEggDrag"));
}

/// `-[TRexEggLogic touchBeganOutside:allTouches:]` @ 0x100172bcc
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<TRexEggLogic>(this).active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut egg = world.obj_mut::<TRexEggLogic>(this);
    egg.touch_screen_pos = touch_pos;
    egg.touch_began_inside = false;
}

/// Follows the finger: the active touch, or any touch while none is active.
fn track_touch(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<TRexEggLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<TRexEggLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[TRexEggLogic touchMovedEntered:allTouches:]` @ 0x100172c28
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    track_touch(world, this, touch);
}

/// `-[TRexEggLogic touchMovedExited:allTouches:]` @ 0x100172c80
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    track_touch(world, this, touch);
}

/// `-[TRexEggLogic touchMovedInside:allTouches:]` @ 0x100172cd8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[TRexEggLogic touchMovedOutside:allTouches:]` @ 0x100172ce8
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    track_touch(world, this, touch);
}

/// `-[TRexEggLogic touchEndedInside:allTouches:]` @ 0x100172d40
///
/// Lets go of a dragged egg: it falls back behind the island.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<TRexEggLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    if world.obj::<TRexEggLogic>(this).dragging {
        {
            let mut egg = world.obj_mut::<TRexEggLogic>(this);
            egg.falling = true;
            egg.dragging = false;
            egg.apply_spring = false;
        }

        let display_object = pg_engine::logic::display_object(world, this);
        let marker = pg_engine::library::display_marker(world, "IslandBackDisplayMarker");
        if let Some(marker) = marker {
            pg_engine::display::insert_after(world, marker, Some(display_object));
        }

        let listener = world.obj::<TRexEggLogic>(this).t_rex_egg_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_t_rex_egg_drag_complete(world, listener, this);
        }
        pg_engine::logic::set_behavior(world, this, Some("TRexEggFall"));
    }

    let mut egg = world.obj_mut::<TRexEggLogic>(this);
    egg.touch_began_inside = false;
    egg.active_touch = None;
}

/// `-[TRexEggLogic touchEndedOutside:allTouches:]` @ 0x100172e44
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
