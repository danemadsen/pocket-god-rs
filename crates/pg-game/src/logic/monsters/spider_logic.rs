//! `SpiderLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ice_island::health_bar_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::island_objects::spear_logic;
use pg_engine::view::BcView;

/// `SpiderLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SpiderLogic {
    /// `mSpiderLogicListener`: `@"<SpiderLogicListener>"`
    pub spider_logic_listener: Option<Entity>,

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

    /// `mbCalledByBabySpider`: `B`
    pub called_by_baby_spider: bool,

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

pub const SPIDER_LOGIC: Class = Class {
    name: "SpiderLogic",
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

/// The height the spider stands at on the island.
const FLOOR_Y_POS: f32 = 86.0;

/// The island edges a walking spider stops at.
const ISLAND_LEFT_X_POS: f32 = 60.0;
const ISLAND_RIGHT_X_POS: f32 = 400.0;

/// Where `callMom` places the spider, off the right edge of the screen.
const START_X_POS: f32 = 500.0;
const START_Y_POS: f32 = 91.0;

/// Full health; the health bar shows `mHealth / 100`.
const MAX_HEALTH: f32 = 100.0;

/// The animation channel that carries the pygmy in the spider's jaws.
const PYGMY_CHANNEL: usize = 1;

/// The jaws are this far ahead of the spider's position (times `xScale`).
const MOUTH_X_OFFSET: f32 = 54.0;

/// A tap right of this pokes the spider off the island; left of it, it only jumps.
const POKE_EXIT_X_POS: f32 = 340.0;

/// `ChallengeManager` game number of the spider challenge.
const SPIDER_GAME: i32 = 8;

/// Sequence-item handlers implemented by `SpiderLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "jumpOntoIsland" => seq_jump_onto_island(world, this, call),
        "fall" => seq_fall(world, this, call),
        "eatPygmy" => seq_eat_pygmy(world, this, call),
        "die" => seq_die(world, this, call),
        "whichWayDoIGo" => seq_which_way_do_i_go(world, this, call),
        "walk" => seq_walk(world, this, call),
        "idle" => seq_idle(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[SpiderLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SPIDER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SpiderLogic initWithDisplayObject:]` @ 0x100199ff0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SpiderLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpiderLogic>(this).slide_friction = 0.025;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<SpiderLogic>(this).enable_island_floor_collision = true;

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, own_display_object);
}

/// `-[SpiderLogic setSpiderLogicListener:]` @ 0x10019a094
pub fn set_spider_logic_listener(world: &mut World, this: Entity, spider_logic_listener: Entity) {
    world.obj_mut::<SpiderLogic>(this).spider_logic_listener = Some(spider_logic_listener);
}

/// `[mSpiderLogicListener healthBarLogic:self]`.
fn listener_health_bar(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<SpiderLogic>(this).spider_logic_listener;
    match listener {
        Some(listener) => crate::controller::health_bar_logic(world, listener, this),
        None => None,
    }
}

/// `[[ChallengeManager instance] isGameActive:8]`: the spider challenge is running.
fn is_spider_game_active(world: &mut World) -> bool {
    let challenge_manager = crate::managers::challenge_manager::instance(world);
    match challenge_manager {
        Some(challenge_manager) => crate::managers::challenge_manager::is_game_active(world, challenge_manager, SPIDER_GAME),
        None => false,
    }
}

/// `-[SpiderLogic clearPygmyLogic:]` @ 0x10019a0f0
pub fn clear_pygmy_logic(world: &mut World, this: Entity, clear_pygmy_logic: Entity) {
    if world.obj::<SpiderLogic>(this).eaten_pygmy_logic != Some(clear_pygmy_logic) {
        return;
    }

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    world.obj_mut::<SpiderLogic>(this).eaten_pygmy_logic = None;
}

/// `-[SpiderLogic uneatPygmy]` @ 0x10019a148
///
/// Lets go of the pygmy in its jaws, if any.
pub fn uneat_pygmy(world: &mut World, this: Entity) {
    let eaten = world.obj::<SpiderLogic>(this).eaten_pygmy_logic;
    let Some(eaten) = eaten else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    crate::pygmy::uneaten_by_spider(world, eaten, this);
    world.obj_mut::<SpiderLogic>(this).eaten_pygmy_logic = None;
}

/// `-[SpiderLogic isActive]` @ 0x10019a1b4
pub fn is_active(world: &mut World, this: Entity) -> bool {
    let spider = world.obj::<SpiderLogic>(this);
    spider.called_by_baby_spider && !spider.killed
}

/// `-[SpiderLogic callMom]` @ 0x10019a1e4
///
/// The little spiders' call: at full health the giant spider comes onto the island from the
/// right, the spears are handed out and its health bar appears.
pub fn call_mom(world: &mut World, this: Entity) {
    let (called, listener) = {
        let spider = world.obj::<SpiderLogic>(this);
        (spider.called_by_baby_spider, spider.spider_logic_listener)
    };
    if called {
        return;
    }
    let Some(listener) = listener else {
        return;
    };
    if !crate::controller::is_spider_enabled(world, listener) {
        return;
    }

    {
        let mut spider = world.obj_mut::<SpiderLogic>(this);
        spider.killed = false;
        spider.health = MAX_HEALTH;
        spider.called_by_baby_spider = true;
        spider.enable_island_side_collision = false;
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
    crate::controller::on_spider_visible(world, listener, this);

    let marker = pg_engine::library::display_marker(world, "TRexMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("SpiderEnterIsland"));
}

/// `-[SpiderLogic canBeHitBySpear:]` @ 0x10019a480
pub fn can_be_hit_by_spear(_world: &mut World, _this: Entity, _can_be_hit_by_spear: Entity) -> bool {
    true
}

/// `-[SpiderLogic hitBySpear:]` @ 0x10019a488
///
/// A spear costs 10 health, up to twice that when thrown from close by
/// (`10·(1 + (300 − d)/300)`, in double precision); the hit animation depends on whether the
/// spear flew the way the spider faces (hit from behind).
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
        let mut spider = world.obj_mut::<SpiderLogic>(this);
        spider.health = (f64::from(spider.health) - damage) as f32;
        spider.health
    };

    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::set_health_percent(world, health_bar, health / 100.0);
    }

    let listener = world.obj::<SpiderLogic>(this).spider_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_spider_hit_by_spear(world, listener, this);
    }

    let health = world.obj::<SpiderLogic>(this).health;
    let killed = health <= 0.0;
    if killed {
        {
            let mut spider = world.obj_mut::<SpiderLogic>(this);
            spider.killed = true;
            spider.called_by_baby_spider = false;
        }
        pg_engine::display::object_mut(world, display_object).is_selectable = false;

        if let Some(listener) = listener {
            crate::controller::on_spider_killed_by_spear(world, listener, this);
        }
        let health_bar = listener_health_bar(world, this);
        if let Some(health_bar) = health_bar {
            health_bar_logic::deactivate(world, health_bar);
        }
        if let Some(listener) = listener {
            crate::controller::on_achievement(world, listener, "ConqueredSpider", -1, true);
        }
    }

    let spear_display_object = pg_engine::logic::display_object(world, hit_by_spear);
    let spear_x_scale = pg_engine::display::object(world, spear_display_object).scale.x;
    let own_x_scale = pg_engine::display::object(world, display_object).scale.x;
    let from_behind = 0.0 < spear_x_scale * own_x_scale;

    let behavior = match (killed, from_behind) {
        (true, false) => "SpiderHitAndKilledBySpear",
        (true, true) => "SpiderHitFromBehindAndKilledBySpear",
        (false, false) => "SpiderHitBySpear",
        (false, true) => "SpiderHitFromBehindBySpear",
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[SpiderLogic enterIsland:]` @ 0x10019a728
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<SpiderLogic>(this).called_by_baby_spider {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;
}

/// `-[SpiderLogic exitIsland]` @ 0x10019a760
///
/// Calls off a running hunt.
pub fn exit_island(world: &mut World, this: Entity) {
    if !world.obj::<SpiderLogic>(this).called_by_baby_spider {
        return;
    }

    let listener = world.obj::<SpiderLogic>(this).spider_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_spider_cancel_hunt(world, listener, this);
    }
    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::deactivate(world, health_bar);
    }

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    uneat_pygmy(world, this);
    world.obj_mut::<SpiderLogic>(this).called_by_baby_spider = false;
    pg_engine::logic::set_behavior(world, this, Some("SpiderDie"));
}

/// `-[SpiderLogic checkPygmyCollision:]` @ 0x10019a8a4
///
/// A hunting spider with empty jaws grabs the first pygmy within 10 points of them.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _check_pygmy_collision: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    let (x_pos, x_scale) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos.x, object.scale.x)
    };

    let (eaten, falling, called, listener) = {
        let spider = world.obj::<SpiderLogic>(this);
        (spider.eaten_pygmy_logic, spider.falling, spider.called_by_baby_spider, spider.spider_logic_listener)
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
        if !in_reach || !crate::pygmy::eaten_by_spider(world, pygmy, this) {
            continue;
        }

        world.obj_mut::<SpiderLogic>(this).eaten_pygmy_logic = Some(pygmy);
        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

        pg_engine::logic::set_behavior(world, this, Some("SpiderSwallowPygmy"));
        return;
    }
}

/// `-[SpiderLogic isAccelEnabled]` @ 0x10019ab0c
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<SpiderLogic>(this).spider_logic_listener;
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

/// `-[SpiderLogic onGameFrame:]` @ 0x10019aba0
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

    // The island edges stop a spider that walks across them.
    world.obj_mut::<SpiderLogic>(this).island_side_collision = false;
    let (enable_island_side_collision, last_x_pos) = {
        let spider = world.obj::<SpiderLogic>(this);
        (spider.enable_island_side_collision, spider.last_pos.x)
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
            world.obj_mut::<SpiderLogic>(this).island_side_collision = true;
        }
    }

    world.obj_mut::<SpiderLogic>(this).island_floor_collision = false;
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    let enable_island_floor_collision = world.obj::<SpiderLogic>(this).enable_island_floor_collision;
    let crossed_floor = FLOOR_Y_POS <= previous_y && pos_y < FLOOR_Y_POS;
    if crossed_floor && enable_island_floor_collision {
        pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
        if !world.obj::<SpiderLogic>(this).on_island {
            world.obj_mut::<SpiderLogic>(this).on_island = true;
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<SpiderLogic>(this).island_floor_collision = true;
        }
        pg_engine::display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;
    } else {
        world.obj_mut::<SpiderLogic>(this).on_island = false;
    }

    // Out of range the spider is held at the edge of the play area.
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

    let health = world.obj::<SpiderLogic>(this).health;
    if health < MAX_HEALTH {
        let health = health + 0.005;
        world.obj_mut::<SpiderLogic>(this).health = health;

        let health_bar = listener_health_bar(world, this);
        if let Some(health_bar) = health_bar {
            health_bar_logic::set_health_percent(world, health_bar, health / 100.0);
        }
    }

    check_pygmy_collision(world, this, dt);

    let pos = pg_engine::display::object(world, display_object).pos;
    world.obj_mut::<SpiderLogic>(this).last_pos = pos;
}

/// `-[SpiderLogic refreshTiltState]` @ 0x10019afc0
///
/// Tipping the device over makes a standing spider fall off the island, dropping its prey.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let (falling, on_island) = {
        let spider = world.obj::<SpiderLogic>(this);
        (spider.falling, spider.on_island)
    };
    if falling || !on_island {
        return;
    }

    let roll = world.resource::<BcView>().accel_rot_degrees.z;
    let tipped_over = -90.0 < roll && roll < 90.0;
    if !tipped_over {
        return;
    }

    world.obj_mut::<SpiderLogic>(this).falling = true;
    uneat_pygmy(world, this);
    pg_engine::logic::set_behavior(world, this, Some("SpiderFall"));
}

/// `-[SpiderLogic jumpOntoIsland:::]` @ 0x10019b088
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

/// `-[SpiderLogic fall:::]` @ 0x10019b0e0
///
/// Falls until it lands on the island, where it stands upright and may walk to the edges.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut spider = world.obj_mut::<SpiderLogic>(this);
            if !spider.island_floor_collision {
                spider.on_island = false;
            }
            spider.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SpiderLogic>(this).falling = true;
        if call.bool("ignoreIsland") {
            world.obj_mut::<SpiderLogic>(this).enable_island_floor_collision = false;
        }
    }

    if !world.obj::<SpiderLogic>(this).island_floor_collision {
        return ItemResult::Pending;
    }

    world.obj_mut::<SpiderLogic>(this).falling = false;
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<SpiderLogic>(this).enable_island_side_collision = true;

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    ItemResult::Done
}

/// `-[SpiderLogic eatPygmy:::]` @ 0x10019b1d8
///
/// Swallowing heals 10 points. Unlike the T-Rex, the spider does not check for a plague.
pub fn seq_eat_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let health = {
        let mut spider = world.obj_mut::<SpiderLogic>(this);
        let raised_health = spider.health + 10.0;
        spider.health = if MAX_HEALTH <= raised_health { MAX_HEALTH } else { raised_health };
        spider.health
    };

    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::set_health_percent(world, health_bar, health / 100.0);
    }

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, PYGMY_CHANNEL);

    let eaten = world.obj::<SpiderLogic>(this).eaten_pygmy_logic;
    if let Some(eaten) = eaten {
        crate::pygmy::swallowed_by_spider(world, eaten, this);
    }
    world.obj_mut::<SpiderLogic>(this).eaten_pygmy_logic = None;
    ItemResult::Done
}

/// `-[SpiderLogic die:::]` @ 0x10019b2a8
///
/// Leaves the island and takes the spears away.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);

    uneat_pygmy(world, this);

    let listener = world.obj::<SpiderLogic>(this).spider_logic_listener;
    let spears = match listener {
        Some(listener) => crate::controller::spear_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for spear in spears {
        spear_logic::deactivate(world, spear);
    }
    ItemResult::Done
}

/// `-[SpiderLogic healthBarType]` @ 0x10019b404
pub fn health_bar_type(_world: &mut World, _this: Entity) -> i32 {
    1
}

/// `-[SpiderLogic shouldHealthBarBeVisible:]` @ 0x10019b40c
pub fn should_health_bar_be_visible(world: &mut World, this: Entity, _should_health_bar_be_visible: Entity) -> bool {
    let listener = world.obj::<SpiderLogic>(this).spider_logic_listener;
    let Some(listener) = listener else {
        return false;
    };

    if !crate::controller::is_spear_enabled(world, listener) {
        return false;
    }
    if !crate::controller::is_spider_enabled(world, listener) {
        return false;
    }
    world.obj::<SpiderLogic>(this).called_by_baby_spider
}

/// `-[SpiderLogic whichWayDoIGo:::]` @ 0x10019b47c
///
/// Picks a random direction (right at the left edge, left at the right edge) and walks on if
/// the spider already faces it, else turns first.
pub fn seq_which_way_do_i_go(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let (x_pos, x_scale) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos.x, object.scale.x)
    };

    let random = world.resource_mut::<Random>().arc4random();
    let mut go_right = random & 1;
    if x_pos <= ISLAND_LEFT_X_POS {
        go_right = 1;
    }
    if ISLAND_RIGHT_X_POS <= x_pos {
        go_right = 0;
    }

    let facing_way = (go_right == 1 && 0.0 < x_scale) || (go_right == 0 && x_scale < 0.0);
    let behavior = if facing_way { "SpiderWalk" } else { "SpiderTurn" };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[SpiderLogic walk:::]` @ 0x10019b55c
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

        world.obj_mut::<SpiderLogic>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let mut result = ItemResult::Pending;
    if world.obj::<SpiderLogic>(this).island_side_collision {
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

/// `-[SpiderLogic idle:::]` @ 0x10019b6c4
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

/// `-[SpiderLogic touchBeganInside:allTouches:]` @ 0x10019b7d8
///
/// A poke (outside the spider challenge) makes it drop its prey and jump: back onto the island
/// near the left, off the island (ending the hunt) near the right.
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    if world.obj::<SpiderLogic>(this).active_touch.is_some() {
        return;
    }
    if is_spider_game_active(world) {
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
        pg_engine::logic::set_behavior(world, this, Some("SpiderPoke"));
        return;
    }

    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 400.0;
        logic.pos_vel.y = 800.0;
    }
    {
        let mut spider = world.obj_mut::<SpiderLogic>(this);
        spider.called_by_baby_spider = false;
        spider.enable_island_side_collision = false;
    }
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::deactivate(world, health_bar);
    }
    pg_engine::logic::set_behavior(world, this, Some("SpiderPokeExitIsland"));
}

/// `-[SpiderLogic touchBeganOutside:allTouches:]` @ 0x10019b944
pub fn touch_began_outside(world: &mut World, this: Entity, _touch: Entity) {
    let mut spider = world.obj_mut::<SpiderLogic>(this);
    if spider.active_touch.is_none() {
        spider.touch_began_inside = false;
    }
}

/// `-[SpiderLogic touchMovedEntered:allTouches:]` @ 0x10019b964
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderLogic touchMovedExited:allTouches:]` @ 0x10019b968
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderLogic touchMovedInside:allTouches:]` @ 0x10019b96c
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[SpiderLogic touchMovedOutside:allTouches:]` @ 0x10019b97c
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[SpiderLogic touchEndedInside:allTouches:]` @ 0x10019b980
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let mut spider = world.obj_mut::<SpiderLogic>(this);
    if spider.active_touch.is_none() || spider.active_touch == Some(touch) {
        spider.touch_began_inside = false;
        spider.active_touch = None;
    }
}

/// `-[SpiderLogic touchEndedOutside:allTouches:]` @ 0x10019b9ac
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
