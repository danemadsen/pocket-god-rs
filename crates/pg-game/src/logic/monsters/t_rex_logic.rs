//! `TRexLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ice_island::health_bar_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::island_objects::spear_logic;
use pg_engine::view::BcView;

/// `TRexLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct TRexLogic {
    /// `mTRexLogicListener`: `@"<TRexLogicListener>"`
    pub t_rex_logic_listener: Option<Entity>,

    /// `mHealth`: `f`
    pub health: f32,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mEatenPygmyLogic`: `@"PygmyLogic"`
    pub eaten_pygmy_logic: Option<Entity>,

    /// `mbCalledByBabyTRex`: `B`
    pub called_by_baby_t_rex: bool,

    /// `mbKilled`: `B`
    pub killed: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbEnableIslandSideCollision`: `B`
    pub enable_island_side_collision: bool,

    /// `mbIslandSideCollision`: `B`
    pub island_side_collision: bool,

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

pub const T_REX_LOGIC: Class = Class {
    name: "TRexLogic",
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

/// The height the T-Rex stands at on the island.
const FLOOR_Y_POS: f32 = 86.0;

/// The island edges a walking T-Rex stops at.
const ISLAND_LEFT_X_POS: f32 = 60.0;
const ISLAND_RIGHT_X_POS: f32 = 400.0;

/// Where the T-Rex waits off the right edge of the screen until it is called.
const START_X_POS: f32 = 500.0;
const START_Y_POS: f32 = 91.0;

/// Full health; the health bar shows `mHealth / 100`.
const MAX_HEALTH: f32 = 100.0;

/// The animation channel that carries the pygmy in the T-Rex's mouth.
const PYGMY_CHANNEL: usize = 1;

/// The mouth is this far ahead of the T-Rex's position (times `xScale`).
const MOUTH_X_OFFSET: f32 = 54.0;

/// A tap right of this pokes the T-Rex off the island; left of it, it only jumps.
const POKE_EXIT_X_POS: f32 = 340.0;

/// `ChallengeManager` game number of the T-Rex hunt.
const T_REX_HUNT_GAME: i32 = 7;

/// `timeTemplePygmyType` of a baby pygmy.
const BABY_PYGMY_TYPE: i32 = 2;

/// `apocalypseDiseaseType` of a plague-infected pygmy.
const PLAGUE_DISEASE_TYPE: i32 = 1;

/// Sequence-item handlers implemented by `TRexLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "jumpOntoIsland" => seq_jump_onto_island(world, this, call),
        "fall" => seq_fall(world, this, call),
        "eatPygmy" => seq_eat_pygmy(world, this, call),
        "die" => seq_die(world, this, call),
        "chooseIdleSequence" => seq_choose_idle_sequence(world, this, call),
        "whichWayDoIGo" => seq_which_way_do_i_go(world, this, call),
        "walk" => seq_walk(world, this, call),
        "idle" => seq_idle(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[TRexLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&T_REX_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[TRexLogic initWithDisplayObject:]` @ 0x10017fa84
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(TRexLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<TRexLogic>(this).slide_friction = 0.025;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<TRexLogic>(this).enable_island_floor_collision = true;

    let own_display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, own_display_object);
        object.pos.x = START_X_POS;
        object.pos.y = START_Y_POS;
    }
    pg_engine::display::remove_from_display_group(world, own_display_object);
    pg_engine::logic::set_behavior(world, this, Some("TRexEnterIsland"));
}

/// `-[TRexLogic setTRexLogicListener:]` @ 0x10017fb7c
pub fn set_t_rex_logic_listener(world: &mut World, this: Entity, t_rex_logic_listener: Entity) {
    world.obj_mut::<TRexLogic>(this).t_rex_logic_listener = Some(t_rex_logic_listener);
}

/// `[mTRexLogicListener healthBarLogic:self]`.
fn listener_health_bar(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<TRexLogic>(this).t_rex_logic_listener;
    match listener {
        Some(listener) => crate::controller::health_bar_logic(world, listener, this),
        None => None,
    }
}

/// `[[ChallengeManager instance] isGameActive:7]`: the T-Rex hunt challenge is running.
fn is_t_rex_hunt_active(world: &mut World) -> bool {
    let challenge_manager = crate::managers::challenge_manager::instance(world);
    match challenge_manager {
        Some(challenge_manager) => crate::managers::challenge_manager::is_game_active(world, challenge_manager, T_REX_HUNT_GAME),
        None => false,
    }
}

/// `-[TRexLogic clearPygmyLogic:]` @ 0x10017fbd8
pub fn clear_pygmy_logic(world: &mut World, this: Entity, clear_pygmy_logic: Entity) {
    if world.obj::<TRexLogic>(this).eaten_pygmy_logic != Some(clear_pygmy_logic) {
        return;
    }

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    world.obj_mut::<TRexLogic>(this).eaten_pygmy_logic = None;
}

/// `-[TRexLogic uneatPygmy]` @ 0x10017fc30
///
/// Spits out the pygmy in its mouth, if any.
pub fn uneat_pygmy(world: &mut World, this: Entity) {
    let eaten = world.obj::<TRexLogic>(this).eaten_pygmy_logic;
    let Some(eaten) = eaten else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    crate::pygmy::uneaten_by_t_rex(world, eaten, this);
    world.obj_mut::<TRexLogic>(this).eaten_pygmy_logic = None;
}

/// `-[TRexLogic isActive]` @ 0x10017fc9c
pub fn is_active(world: &mut World, this: Entity) -> bool {
    let t_rex = world.obj::<TRexLogic>(this);
    t_rex.called_by_baby_t_rex && !t_rex.killed
}

/// `-[TRexLogic callMom]` @ 0x10017fccc
///
/// The hatchling's call: at full health the T-Rex jumps onto the island from the right, the
/// spears are handed out and its health bar appears.
pub fn call_mom(world: &mut World, this: Entity) {
    let (called, listener) = {
        let t_rex = world.obj::<TRexLogic>(this);
        (t_rex.called_by_baby_t_rex, t_rex.t_rex_logic_listener)
    };
    if called {
        return;
    }
    let Some(listener) = listener else {
        return;
    };
    if !crate::controller::is_t_rex_enabled(world, listener) {
        return;
    }

    {
        let mut t_rex = world.obj_mut::<TRexLogic>(this);
        t_rex.killed = false;
        t_rex.health = MAX_HEALTH;
        t_rex.called_by_baby_t_rex = true;
        t_rex.enable_island_side_collision = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = START_X_POS;
        object.pos.y = START_Y_POS;
    }

    let spears = crate::controller::spear_logic_array(world, listener, this);
    for spear in spears {
        spear_logic::activate(world, spear);
    }

    let health_bar = crate::controller::health_bar_logic(world, listener, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::set_health_bar_logic_listener(world, health_bar, this);
        health_bar_logic::activate(world, health_bar);
    }

    pg_engine::display::object_mut(world, display_object).is_selectable = true;
    crate::controller::on_t_rex_visible(world, listener, this);

    let marker = pg_engine::library::display_marker(world, "TRexMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("TRexEnterIsland"));
}

/// `-[TRexLogic canBeHitBySpear:]` @ 0x10017ff68
pub fn can_be_hit_by_spear(_world: &mut World, _this: Entity, _can_be_hit_by_spear: Entity) -> bool {
    true
}

/// `-[TRexLogic hitBySpear:]` @ 0x10017ff70
///
/// A spear costs 10 health, up to twice that when thrown from close by
/// (`10·(1 + (300 − d)/300)`, in double precision); the hit animation depends on whether the
/// spear flew the way the T-Rex faces (hit from behind).
pub fn hit_by_spear(world: &mut World, this: Entity, hit_by_spear: Entity) {
    uneat_pygmy(world, this);

    let throw_x_pos = spear_logic::throw_x_pos(world, hit_by_spear);
    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = pg_engine::display::object(world, display_object).pos.x;

    let throw_distance = (throw_x_pos - x_pos).abs();
    let closeness = f64::from(300.0 - throw_distance);
    let close_damage = (closeness / 300.0 + 1.0) * 10.0;
    let damage = if 300.0 < throw_distance { 10.0 } else { close_damage };

    let health = {
        let mut t_rex = world.obj_mut::<TRexLogic>(this);
        t_rex.health = (f64::from(t_rex.health) - damage) as f32;
        t_rex.health
    };

    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::set_health_percent(world, health_bar, health / 100.0);
    }

    let listener = world.obj::<TRexLogic>(this).t_rex_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_t_rex_hit_by_spear(world, listener, this);
    }

    let health = world.obj::<TRexLogic>(this).health;
    let killed = health <= 0.0;
    if killed {
        {
            let mut t_rex = world.obj_mut::<TRexLogic>(this);
            t_rex.killed = true;
            t_rex.called_by_baby_t_rex = false;
        }
        pg_engine::display::object_mut(world, display_object).is_selectable = false;

        if let Some(listener) = listener {
            crate::controller::on_t_rex_killed_by_spear(world, listener, this);
        }
        let health_bar = listener_health_bar(world, this);
        if let Some(health_bar) = health_bar {
            health_bar_logic::deactivate(world, health_bar);
        }
        if let Some(listener) = listener {
            crate::controller::on_achievement(world, listener, "ConqueredTyrannosaurus", -1, true);
        }
    }

    let spear_display_object = pg_engine::logic::display_object(world, hit_by_spear);
    let spear_x_scale = pg_engine::display::object(world, spear_display_object).scale.x;
    let own_x_scale = pg_engine::display::object(world, display_object).scale.x;
    let from_behind = 0.0 < spear_x_scale * own_x_scale;

    let behavior = match (killed, from_behind) {
        (true, false) => "TRexHitAndKilledBySpear",
        (true, true) => "TRexHitFromBehindAndKilledBySpear",
        (false, false) => "TRexHitBySpear",
        (false, true) => "TRexHitFromBehindBySpear",
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[TRexLogic enterIsland:]` @ 0x100180210
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<TRexLogic>(this).called_by_baby_t_rex {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;
}

/// `-[TRexLogic exitIsland]` @ 0x100180248
///
/// Calls off a running hunt.
pub fn exit_island(world: &mut World, this: Entity) {
    if !world.obj::<TRexLogic>(this).called_by_baby_t_rex {
        return;
    }

    let listener = world.obj::<TRexLogic>(this).t_rex_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_t_rex_cancel_hunt(world, listener, this);
    }
    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::deactivate(world, health_bar);
    }

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    uneat_pygmy(world, this);
    world.obj_mut::<TRexLogic>(this).called_by_baby_t_rex = false;
    pg_engine::logic::set_behavior(world, this, Some("TRexDie"));
}

/// `-[TRexLogic checkPygmyCollision:]` @ 0x10018038c
///
/// A hunting T-Rex with an empty mouth snaps up the first pygmy within 10 points of its mouth.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _check_pygmy_collision: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    let (x_pos, x_scale) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos.x, object.scale.x)
    };

    let (eaten, falling, called, listener) = {
        let t_rex = world.obj::<TRexLogic>(this);
        (t_rex.eaten_pygmy_logic, t_rex.falling, t_rex.called_by_baby_t_rex, t_rex.t_rex_logic_listener)
    };
    if eaten.is_some() || falling || !called {
        return;
    }

    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    let mouth_x_pos = x_pos + x_scale * MOUTH_X_OFFSET;
    for pygmy in pygmies {
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_x_pos = pg_engine::display::object(world, pygmy_display_object).pos.x;
        let in_reach = (mouth_x_pos - pygmy_x_pos).abs() < 10.0;
        if !in_reach || !crate::pygmy::eaten_by_t_rex(world, pygmy, this) {
            continue;
        }

        world.obj_mut::<TRexLogic>(this).eaten_pygmy_logic = Some(pygmy);
        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

        let pygmy_type = crate::pygmy::time_temple_pygmy_type(world, pygmy);
        let behavior = if pygmy_type == BABY_PYGMY_TYPE { "TRexSwallowPygmyBaby" } else { "TRexSwallowPygmy" };
        pg_engine::logic::set_behavior(world, this, Some(behavior));
        return;
    }
}

/// `-[TRexLogic isAccelEnabled]` @ 0x10018061c
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<TRexLogic>(this).t_rex_logic_listener;
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

/// `-[TRexLogic onGameFrame:]` @ 0x1001806b0
///
/// Falls under gravity (pulled straight down; tilt only matters through `refreshTiltState`),
/// stops at the island edges while walking and at the floor, slowly heals, and hunts.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let previous_y = pg_engine::display::object(world, display_object).pos.y;

    if is_accel_enabled(world, this) {
        let accel_force = world.resource::<BcView>().accel_force;
        let tilt = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let mut gravity = pg_engine::math::normalize3(tilt);
        gravity.y = gravity.y * 3000.0;
        gravity.z = gravity.z * 3000.0;
        gravity.x = 0.0;
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = Vec4::new(0.0, -3000.0, 0.0, 1.0);
    }

    let (ignore_gravity, gravity) = {
        let sprite = world.obj::<SpriteLogic>(this);
        (sprite.ignore_gravity, sprite.gravity_force)
    };
    let mut force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !ignore_gravity {
        force.x = gravity.x + force.x;
        force.y = gravity.y + force.y;
        force.z = gravity.z + force.z;
    }

    let vel = {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_force = force;
        logic.pos_vel.x = logic.pos_vel.x + force.x * dt;
        logic.pos_vel.y = logic.pos_vel.y + force.y * dt;
        logic.pos_vel.z = logic.pos_vel.z + force.z * dt;
        logic.pos_vel
    };
    let pos_x = {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = object.pos.x + vel.x * dt;
        object.pos.y = object.pos.y + vel.y * dt;
        object.pos.z = object.pos.z + vel.z * dt;
        object.pos.x
    };

    // The island edges stop a T-Rex that walks across them.
    world.obj_mut::<TRexLogic>(this).island_side_collision = false;
    let (enable_island_side_collision, last_x_pos) = {
        let t_rex = world.obj::<TRexLogic>(this);
        (t_rex.enable_island_side_collision, t_rex.last_pos.x)
    };
    if enable_island_side_collision {
        let mut edge_x_pos = None;
        if pos_x < ISLAND_LEFT_X_POS && ISLAND_LEFT_X_POS <= last_x_pos {
            edge_x_pos = Some(ISLAND_LEFT_X_POS);
        } else if ISLAND_RIGHT_X_POS < pos_x && last_x_pos <= ISLAND_RIGHT_X_POS {
            edge_x_pos = Some(ISLAND_RIGHT_X_POS);
        }

        if let Some(edge_x_pos) = edge_x_pos {
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            pg_engine::display::object_mut(world, display_object).pos.x = edge_x_pos;
            world.obj_mut::<TRexLogic>(this).island_side_collision = true;
        }
    }

    world.obj_mut::<TRexLogic>(this).island_floor_collision = false;
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    let enable_island_floor_collision = world.obj::<TRexLogic>(this).enable_island_floor_collision;
    let crossed_floor = FLOOR_Y_POS <= previous_y && pos_y < FLOOR_Y_POS;
    if crossed_floor && enable_island_floor_collision {
        pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
        if !world.obj::<TRexLogic>(this).on_island {
            world.obj_mut::<TRexLogic>(this).on_island = true;
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<TRexLogic>(this).island_floor_collision = true;
        }
        pg_engine::display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;
    } else {
        world.obj_mut::<TRexLogic>(this).on_island = false;
    }

    // Out of range the T-Rex is held at the edge of the play area.
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    let mut held_y_pos = None;
    if pos_y < -100.0 {
        held_y_pos = Some(-100.0);
    } else if 820.0 < pos_y {
        held_y_pos = Some(820.0);
    }
    if let Some(held_y_pos) = held_y_pos {
        pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
        pg_engine::display::object_mut(world, display_object).pos.y = held_y_pos;
    }

    let health = world.obj::<TRexLogic>(this).health;
    if health < MAX_HEALTH {
        let health = health + 0.005;
        world.obj_mut::<TRexLogic>(this).health = health;

        let health_bar = listener_health_bar(world, this);
        if let Some(health_bar) = health_bar {
            health_bar_logic::set_health_percent(world, health_bar, health / 100.0);
        }
    }

    check_pygmy_collision(world, this, dt);

    let pos = pg_engine::display::object(world, display_object).pos;
    world.obj_mut::<TRexLogic>(this).last_pos = pos;
}

/// `-[TRexLogic refreshTiltState]` @ 0x100180ad0
///
/// Tipping the device over makes a standing T-Rex fall off the island, dropping its prey.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let (falling, on_island) = {
        let t_rex = world.obj::<TRexLogic>(this);
        (t_rex.falling, t_rex.on_island)
    };
    if falling || !on_island {
        return;
    }

    let roll = world.resource::<BcView>().accel_rot_degrees.z;
    let tipped_over = -90.0 < roll && roll < 90.0;
    if !tipped_over {
        return;
    }

    world.obj_mut::<TRexLogic>(this).falling = true;
    uneat_pygmy(world, this);
    pg_engine::logic::set_behavior(world, this, Some("TRexFall"));
}

/// `-[TRexLogic jumpOntoIsland:::]` @ 0x100180b98
pub fn seq_jump_onto_island(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = -200.0;
        logic.pos_vel.y = 800.0;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).scale.x = -1.0;
    ItemResult::Done
}

/// `-[TRexLogic fall:::]` @ 0x100180bf0
///
/// Falls until it lands on the island, where it stands upright and may walk to the edges.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut t_rex = world.obj_mut::<TRexLogic>(this);
            if !t_rex.island_floor_collision {
                t_rex.on_island = false;
            }
            t_rex.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<TRexLogic>(this).falling = true;
        if call.bool("ignoreIsland") {
            world.obj_mut::<TRexLogic>(this).enable_island_floor_collision = false;
        }
    }

    if !world.obj::<TRexLogic>(this).island_floor_collision {
        return ItemResult::Pending;
    }

    world.obj_mut::<TRexLogic>(this).falling = false;
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<TRexLogic>(this).enable_island_side_collision = true;

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    ItemResult::Done
}

/// `-[TRexLogic eatPygmy:::]` @ 0x100180ce8
///
/// Swallowing heals 10 points; a plague-infected pygmy kills the T-Rex.
pub fn seq_eat_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let health = {
        let mut t_rex = world.obj_mut::<TRexLogic>(this);
        let raised_health = t_rex.health + 10.0;
        t_rex.health = if MAX_HEALTH <= raised_health { MAX_HEALTH } else { raised_health };
        t_rex.health
    };

    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::set_health_percent(world, health_bar, health / 100.0);
    }

    let eaten = world.obj::<TRexLogic>(this).eaten_pygmy_logic;
    let Some(eaten) = eaten else {
        return ItemResult::Done;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);
    crate::pygmy::swallowed_by_t_rex(world, eaten, this);
    let disease_type = sprite_logic::apocalypse_disease_type(world, eaten);
    world.obj_mut::<TRexLogic>(this).eaten_pygmy_logic = None;

    if disease_type != PLAGUE_DISEASE_TYPE {
        return ItemResult::Done;
    }

    {
        let mut t_rex = world.obj_mut::<TRexLogic>(this);
        t_rex.killed = true;
        t_rex.called_by_baby_t_rex = false;
    }
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    let listener = world.obj::<TRexLogic>(this).t_rex_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_t_rex_killed_by_spear(world, listener, this);
    }
    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::deactivate(world, health_bar);
    }
    pg_engine::logic::set_behavior(world, this, Some("TRexEatPlaguePygmy"));
    ItemResult::Replaced
}

/// `-[TRexLogic die:::]` @ 0x100180e8c
///
/// Leaves the island and takes the spears away.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);

    uneat_pygmy(world, this);

    let listener = world.obj::<TRexLogic>(this).t_rex_logic_listener;
    let spears = match listener {
        Some(listener) => crate::controller::spear_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for spear in spears {
        spear_logic::deactivate(world, spear);
    }
    ItemResult::Done
}

/// `-[TRexLogic healthBarType]` @ 0x100180fe8
pub fn health_bar_type(_world: &mut World, _this: Entity) -> i32 {
    0
}

/// `-[TRexLogic shouldHealthBarBeVisible:]` @ 0x100180ff0
pub fn should_health_bar_be_visible(world: &mut World, this: Entity, _should_health_bar_be_visible: Entity) -> bool {
    let listener = world.obj::<TRexLogic>(this).t_rex_logic_listener;
    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_spear_enabled(world, listener) {
        return false;
    }
    if !crate::controller::is_t_rex_enabled(world, listener) {
        return false;
    }
    world.obj::<TRexLogic>(this).called_by_baby_t_rex
}

/// `-[TRexLogic chooseIdleSequence:::]` @ 0x100181060
pub fn seq_choose_idle_sequence(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let behavior = if is_t_rex_hunt_active(world) { "TRexGameIdle" } else { "TRexWanderIdle" };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[TRexLogic whichWayDoIGo:::]` @ 0x1001810d8
///
/// Picks a random direction and walks on if the T-Rex already faces it, else turns first.
pub fn seq_which_way_do_i_go(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let (x_pos, x_scale) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos.x, object.scale.x)
    };

    let random = world.resource_mut::<Random>().arc4random();
    let mut go_right = random & 1;
    // Original quirk: exact comparisons; they work because the side collision puts a walking
    // T-Rex exactly on 60 or 400.
    if x_pos == ISLAND_LEFT_X_POS {
        go_right = 1;
    }
    if x_pos == ISLAND_RIGHT_X_POS {
        go_right = 0;
    }

    let facing_way = (go_right == 1 && 0.0 < x_scale) || (go_right == 0 && x_scale < 0.0);
    let hunt_active = is_t_rex_hunt_active(world);
    let behavior = match (facing_way, hunt_active) {
        (true, false) => "TRexWalk",
        (true, true) => "TRexHuntWalk",
        (false, false) => "TRexTurn",
        (false, true) => "TRexHuntTurn",
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[TRexLogic walk:::]` @ 0x100181230
///
/// Walks for a random `[minTime, maxTime]` seconds, or up to 100 s with `endOfIsland`, until it
/// reaches an island edge.
pub fn seq_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let stall_time = if call.has("endOfIsland") {
            100.0
        } else {
            let min_time = call.float("minTime");
            let max_time = call.float("maxTime");
            let random = world.resource_mut::<Random>().arc4random();
            pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random)
        };
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        world.obj_mut::<TRexLogic>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let mut result = ItemResult::Pending;
    if world.obj::<TRexLogic>(this).island_side_collision {
        result = ItemResult::Done;
    }

    let mut logic = pg_engine::logic::logic_mut(world, this);
    if logic.stall_time <= 0.0 {
        logic.pos_vel.x = 0.0;
        result = ItemResult::Done;
    }
    logic.stall_time = logic.stall_time - call.delta_time;
    result
}

/// `-[TRexLogic idle:::]` @ 0x100181398
///
/// Stands still for a random `[minTime, maxTime]` seconds.
pub fn seq_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let mut logic = pg_engine::logic::logic_mut(world, this);
    if logic.stall_time <= 0.0 {
        return ItemResult::Done;
    }
    logic.stall_time = logic.stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[TRexLogic touchBeganInside:allTouches:]` @ 0x1001814ac
///
/// A poke (outside the T-Rex hunt) makes it drop its prey and jump: back onto the island near
/// the left, off the island (ending the hunt) near the right.
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    if world.obj::<TRexLogic>(this).active_touch.is_some() {
        return;
    }
    if is_t_rex_hunt_active(world) {
        return;
    }

    uneat_pygmy(world, this);

    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = pg_engine::display::object(world, display_object).pos.x;
    let pokes_off_island = POKE_EXIT_X_POS < x_pos;
    if !pokes_off_island {
        {
            let mut logic = pg_engine::logic::logic_mut(world, this);
            logic.pos_vel.x = 200.0;
            logic.pos_vel.y = 800.0;
        }
        pg_engine::logic::set_behavior(world, this, Some("TRexPoke"));
        return;
    }

    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 400.0;
        logic.pos_vel.y = 800.0;
    }
    {
        let mut t_rex = world.obj_mut::<TRexLogic>(this);
        t_rex.called_by_baby_t_rex = false;
        t_rex.enable_island_side_collision = false;
    }
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::deactivate(world, health_bar);
    }
    pg_engine::logic::set_behavior(world, this, Some("TRexPokeExitIsland"));
}

/// `-[TRexLogic touchBeganOutside:allTouches:]` @ 0x100181618
pub fn touch_began_outside(world: &mut World, this: Entity, _touch: Entity) {
    let mut t_rex = world.obj_mut::<TRexLogic>(this);
    if t_rex.active_touch.is_none() {
        t_rex.touch_began_inside = false;
    }
}

/// `-[TRexLogic touchMovedEntered:allTouches:]` @ 0x100181638
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TRexLogic touchMovedExited:allTouches:]` @ 0x10018163c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TRexLogic touchMovedInside:allTouches:]` @ 0x100181640
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[TRexLogic touchMovedOutside:allTouches:]` @ 0x100181650
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[TRexLogic touchEndedInside:allTouches:]` @ 0x100181654
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let mut t_rex = world.obj_mut::<TRexLogic>(this);
    if t_rex.active_touch.is_none() || t_rex.active_touch == Some(touch) {
        t_rex.touch_began_inside = false;
        t_rex.active_touch = None;
    }
}

/// `-[TRexLogic touchEndedOutside:allTouches:]` @ 0x100181680
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
