//! `SpearLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::coconut_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::monsters::{spider_logic, t_rex_logic};
use crate::logic::sky_powers::hurricane_logic;
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `SpearLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct SpearLogic {
    /// `mSpearLogicListener`: `@"<SpearLogicListener>"`
    pub spear_logic_listener: Option<Entity>,

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

    /// `mThrowXPos`: `f`
    pub throw_x_pos: f32,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,

    /// `mbExitIsland`: `B`
    pub exit_island: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbDisableCollision`: `B`
    pub disable_collision: bool,

    /// `mbSpinSpear`: `B`
    pub spin_spear: bool,

    /// `mbThrownByPygmy`: `B`
    pub thrown_by_pygmy: bool,

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

    /// `mbBeingUsed`: `B`
    pub being_used: bool,

    /// `mbActive`: `B`
    pub active: bool,
}

pub const SPEAR_LOGIC: Class = Class {
    name: "SpearLogic",
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

/// The resting height of the spear on the island (`setYPos:72`).
const FLOOR_Y_POS: f32 = 72.0;

/// Below this height the spear splashes into the sea.
const WATER_Y_POS: f32 = 20.0;

/// The depth at which a sprite keeps its natural size (`setZPos:-277.12`).
const SPRITE_Z_POS: f32 = -277.12;

/// Sequence-item handlers implemented by `SpearLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "slide" => seq_slide(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "fallFromHurricane" => seq_fall_from_hurricane(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[SpearLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&SPEAR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[SpearLogic initWithDisplayObject:]` @ 0x10016eb34
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(SpearLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.tilt_state = 0;
        spear.slide_friction = 0.025;
        spear.exit_island = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.disable_collision = true;
        spear.enable_island_floor_collision = true;
    }

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, own_display_object);
}

/// `-[SpearLogic setSpearLogicListener:]` @ 0x10016ebfc
pub fn set_spear_logic_listener(world: &mut World, this: Entity, spear_logic_listener: Entity) {
    world.obj_mut::<SpearLogic>(this).spear_logic_listener = Some(spear_logic_listener);
}

/// `-[SpearLogic throwXPos]` @ 0x10016ec58
pub fn throw_x_pos(world: &mut World, this: Entity) -> f32 {
    world.obj::<SpearLogic>(this).throw_x_pos
}

/// `-[SpearLogic activate]` @ 0x10016ec68
///
/// Drops the spear onto the island from (150, 400).
pub fn activate(world: &mut World, this: Entity) {
    if world.obj::<SpearLogic>(this).exit_island {
        return;
    }

    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
        logic.rot_vel.z = 0.0;
    }
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.active = true;
        spear.in_water = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.disable_collision = true;
        spear.exit_island = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.is_selectable = true;
        object.pos.x = 150.0;
        object.pos.y = 400.0;
        object.pos.z = SPRITE_Z_POS;
    }

    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("SpearEnterIsland"));
}

/// `-[SpearLogic deactivate]` @ 0x10016edd8
///
/// Lets go of the spear (hurricane, drag) and drops it off the island.
pub fn deactivate(world: &mut World, this: Entity) {
    if !world.obj::<SpearLogic>(this).active {
        return;
    }
    world.obj_mut::<SpearLogic>(this).active = false;

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        hurricane_logic::drop_sprite_logic(world, hurricane, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    if world.obj::<SpearLogic>(this).dragging {
        {
            let mut spear = world.obj_mut::<SpearLogic>(this);
            spear.falling = true;
            spear.dragging = false;
            spear.apply_spring = false;
        }
        let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_spear_drag_complete(world, listener, this);
        }
    }

    world.obj_mut::<SpearLogic>(this).being_used = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.disable_collision = true;
        spear.enable_island_floor_collision = false;
    }

    if world.obj::<SpearLogic>(this).in_water {
        return;
    }
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("SpearFallIgnoreIsland"));
}

/// Whether the T-Rex or the spider is attacking and spears are enabled: the condition shared
/// by `enterIsland:` and `die:::` for (re)activating the spear.
fn is_spear_needed(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
    let Some(listener) = listener else {
        return false;
    };

    let t_rex = crate::controller::t_rex_logic(world, listener, this);
    let t_rex_active = match t_rex {
        Some(t_rex) => t_rex_logic::is_active(world, t_rex),
        None => false,
    };
    if !t_rex_active {
        let spider = crate::controller::spider_logic(world, listener, this);
        let spider_active = match spider {
            Some(spider) => spider_logic::is_active(world, spider),
            None => false,
        };
        if !spider_active {
            return false;
        }
    }
    crate::controller::is_spear_enabled(world, listener)
}

/// `-[SpearLogic enterIsland:]` @ 0x10016ef74
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.disable_collision = true;
        spear.exit_island = false;
    }
    if is_spear_needed(world, this) {
        activate(world, this);
    }
}

/// `-[SpearLogic exitIsland:]` @ 0x10016f048
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    world.obj_mut::<SpearLogic>(this).exit_island = true;
    deactivate(world, this);
    if exit_island {
        pg_engine::logic::set_behavior(world, this, Some("SpearDie"));
    }
}

/// `-[SpearLogic exitIslandNow]` @ 0x10016f0b0
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[SpearLogic isAvailable]` @ 0x10016f0c4
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
    let spear_enabled = match listener {
        Some(listener) => crate::controller::is_spear_enabled(world, listener),
        None => false,
    };

    let spear = world.obj::<SpearLogic>(this);
    if !spear_enabled || !spear.on_island {
        return false;
    }
    !spear.being_used
}

/// `-[SpearLogic pygmyLogic]` @ 0x10016f12c
pub fn pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<SpearLogic>(this).pygmy_logic
}

/// `-[SpearLogic throwSpear]` @ 0x10016f13c
///
/// The holding pygmy throws the spear the way it faces.
pub fn throw_spear(world: &mut World, this: Entity) {
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.thrown_by_pygmy = true;
        spear.falling = false;
        spear.being_used = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<SpearLogic>(this).disable_collision = false;

    let display_object = pg_engine::logic::display_object(world, this);
    let (x_scale, x_pos) = {
        let object = pg_engine::display::object(world, display_object);
        (object.scale.x, object.pos.x)
    };
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 500.0;
        if x_scale < 0.0 {
            logic.pos_vel.x = -500.0;
        }
        logic.pos_vel.y = 800.0;
    }
    world.obj_mut::<SpearLogic>(this).throw_x_pos = x_pos;

    pg_engine::logic::set_behavior(world, this, Some("SpearThrow"));
    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_spear_throw(world, listener, this);
    }
}

/// `-[SpearLogic dropWhileHolding]` @ 0x10016f24c
pub fn drop_while_holding(world: &mut World, this: Entity) {
    world.obj_mut::<SpearLogic>(this).being_used = false;
    pg_engine::logic::logic_mut(world, this).pos_vel.y = 400.0;
    pg_engine::logic::set_behavior(world, this, Some("SpearFall"));
}

/// `-[SpearLogic startHolding:]` @ 0x10016f288
pub fn start_holding(world: &mut World, this: Entity, holding: Entity) {
    world.obj_mut::<SpearLogic>(this).being_used = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<SpearLogic>(this).pygmy_logic = Some(holding);
}

/// `-[SpearLogic moveToIslandLayer]` @ 0x10016f31c
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[SpearLogic moveToBehindIslandLayer]` @ 0x10016f384
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "IslandDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[SpearLogic checkPygmyCollision]` @ 0x10016f3ec
///
/// A pygmy that can catch spears catches one falling on its head (not one a pygmy threw).
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let (falling, thrown_by_pygmy, disable_collision, listener) = {
        let spear = world.obj::<SpearLogic>(this);
        (spear.falling, spear.thrown_by_pygmy, spear.disable_collision, spear.spear_logic_listener)
    };
    if !falling || thrown_by_pygmy || disable_collision {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let spear_pos = pg_engine::display::object(world, display_object).pos;

    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;
        let moving_down = vel_y < 0.0;
        if !moving_down {
            continue;
        }
        if !crate::pygmy::can_catch_spear(world, pygmy) {
            continue;
        }

        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_pos = pg_engine::display::object(world, pygmy_display_object).pos;
        let delta_x = spear_pos.x - (pygmy_pos.x + 0.0);
        let delta_y = spear_pos.y - (pygmy_pos.y + -32.0);
        let caught = delta_x * delta_x + delta_y * delta_y < 2500.0;
        if !caught {
            continue;
        }

        {
            let mut logic = pg_engine::logic::logic_mut(world, this);
            logic.pos_vel.x = 0.0;
            logic.pos_vel.y = 0.0;
        }
        world.obj_mut::<SpearLogic>(this).falling = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        pg_engine::logic::set_behavior(world, this, Some("SpearPygmyCatch"));
        crate::pygmy::catch_spear(world, pygmy, this);
        return;
    }
}

/// Whether a thrown spear may hit a monster this frame.
fn can_hit_monster(world: &World, this: Entity) -> bool {
    let spear = world.obj::<SpearLogic>(this);
    spear.thrown_by_pygmy && !spear.disable_collision
}

/// Bounces a spear off the monster it hit and plays `behavior`.
fn bounce_off_monster(world: &mut World, this: Entity, behavior: &str) {
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = logic.pos_vel.x * -0.5;
        logic.pos_vel.y = 500.0;
    }
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.spin_spear = true;
        spear.disable_collision = true;
    }
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[SpearLogic checkTRexCollision]` @ 0x10016f6b8
///
/// A thrown spear hits the T-Rex within 40 of `(x + xScale·41, y + 103)`.
pub fn check_t_rex_collision(world: &mut World, this: Entity) {
    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
    let Some(listener) = listener else {
        return;
    };
    let t_rex = crate::controller::t_rex_logic(world, listener, this);

    if !can_hit_monster(world, this) {
        return;
    }
    if !crate::controller::is_t_rex_active(world, listener) {
        return;
    }
    let Some(t_rex) = t_rex else {
        return;
    };
    if !t_rex_logic::can_be_hit_by_spear(world, t_rex, this) {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let spear_pos = pg_engine::display::object(world, display_object).pos;
    let t_rex_display_object = pg_engine::logic::display_object(world, t_rex);
    let (t_rex_pos, t_rex_x_scale) = {
        let object = pg_engine::display::object(world, t_rex_display_object);
        (object.pos, object.scale.x)
    };

    let delta_x = spear_pos.x - (t_rex_pos.x + t_rex_x_scale * 41.0);
    let delta_y = spear_pos.y - (t_rex_pos.y + 103.0);
    if delta_y * delta_y + delta_x * delta_x < 1600.0 {
        bounce_off_monster(world, this, "SpearHitTRex");
        t_rex_logic::hit_by_spear(world, t_rex, this);
    }
}

/// `-[SpearLogic checkSpiderCollision]` @ 0x10016f8c4
///
/// A thrown spear hits the spider within 40 of `(x + xScale·35, y + 35)` or of
/// `(x − xScale·16, y + 32)`.
pub fn check_spider_collision(world: &mut World, this: Entity) {
    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
    let Some(listener) = listener else {
        return;
    };
    let spider = crate::controller::spider_logic(world, listener, this);

    if !can_hit_monster(world, this) {
        return;
    }
    if !crate::controller::is_spider_active(world, listener) {
        return;
    }
    let Some(spider) = spider else {
        return;
    };
    if !spider_logic::can_be_hit_by_spear(world, spider, this) {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let spear_pos = pg_engine::display::object(world, display_object).pos;
    let spider_display_object = pg_engine::logic::display_object(world, spider);
    let (spider_pos, spider_x_scale) = {
        let object = pg_engine::display::object(world, spider_display_object);
        (object.pos, object.scale.x)
    };

    let head_delta_x = spear_pos.x - (spider_pos.x + spider_x_scale * 35.0);
    let head_delta_y = spear_pos.y - (spider_pos.y + 35.0);
    let mut hit = head_delta_y * head_delta_y + head_delta_x * head_delta_x < 1600.0;
    if !hit {
        let body_delta_x = spear_pos.x - (spider_pos.x + spider_x_scale * -16.0);
        let body_delta_y = spear_pos.y - (spider_pos.y + 32.0);
        hit = body_delta_y * body_delta_y + body_delta_x * body_delta_x < 1600.0;
    }

    if hit {
        bounce_off_monster(world, this, "SpearHitSpider");
        spider_logic::hit_by_spear(world, spider, this);
    }
}

/// `-[SpearLogic isAccelEnabled]` @ 0x10016fafc
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
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

/// `-[SpearLogic onGameFrame:]` @ 0x10016fb90
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let previous_y = pg_engine::display::object(world, display_object).pos.y;

    // Gravity follows the device tilt while the accelerometer drives the spear.
    if is_accel_enabled(world, this) {
        let accel_force = world.resource::<BcView>().accel_force;
        let tilt = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let mut gravity = pg_engine::math::normalize3(tilt);
        gravity.x *= 3000.0;
        gravity.y *= 3000.0;
        gravity.z *= 3000.0;
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = Vec4::new(0.0, -3000.0, 0.0, 1.0);
        undo_tilt_state(world, this);
    }

    let (tilt_state, slide_friction, apply_spring, spring_force) = {
        let spear = world.obj::<SpearLogic>(this);
        (spear.tilt_state, spear.slide_friction, spear.apply_spring, spear.spring_force)
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

    let (vel, rot_vel) = {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_force = force;
        logic.pos_vel.x = logic.pos_vel.x + force.x * dt;
        logic.pos_vel.y = logic.pos_vel.y + force.y * dt;
        logic.pos_vel.z = logic.pos_vel.z + force.z * dt;
        (logic.pos_vel, logic.rot_vel)
    };
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = object.pos.x + vel.x * dt;
        object.pos.y = object.pos.y + vel.y * dt;
        object.pos.z = object.pos.z + vel.z * dt;
        object.rot.x = object.rot.x + rot_vel.x * dt;
        object.rot.y = object.rot.y + rot_vel.y * dt;
        object.rot.z = object.rot.z + rot_vel.z * dt;
    }

    world.obj_mut::<SpearLogic>(this).island_floor_collision = false;
    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(listener) = listener {
        island_min_x = crate::controller::island_min_x_pos(world, listener);
        island_max_x = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = pg_engine::display::object(world, display_object).pos;
    let enable_island_floor_collision = world.obj::<SpearLogic>(this).enable_island_floor_collision;
    let over_island = island_min_x <= pos.x && pos.x <= island_max_x;
    let crossed_floor = pos.y < FLOOR_Y_POS && FLOOR_Y_POS <= previous_y;
    if !(crossed_floor && over_island && enable_island_floor_collision) {
        world.obj_mut::<SpearLogic>(this).on_island = false;
    } else {
        land_on_floor(world, this, display_object);
        world.obj_mut::<SpearLogic>(this).disable_collision = true;
        pg_engine::display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;
    }

    let (thrown_by_pygmy, spin_spear) = {
        let spear = world.obj::<SpearLogic>(this);
        (spear.thrown_by_pygmy, spear.spin_spear)
    };
    if thrown_by_pygmy {
        let rot_z = if !spin_spear {
            // Points the flying spear along its path (the original computes this in double).
            let vel_y = pg_engine::logic::logic(world, this).pos_vel.y;
            let x_scale = pg_engine::display::object(world, display_object).scale.x;
            let pitch = vel_y as f64 * 0.04;
            (pitch * x_scale as f64) as f32
        } else {
            pg_engine::display::object(world, display_object).rot.z + 50.0
        };
        pg_engine::display::object_mut(world, display_object).rot.z = rot_z;
    }

    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    if pos_y < -100.0 || 820.0 < pos_y {
        pg_engine::logic::set_behavior(world, this, Some("SpearDie"));
    }

    check_pygmy_collision(world, this);
    check_t_rex_collision(world, this);
    check_spider_collision(world, this);
}

/// The floor contact of `onGameFrame:`: a landing spear bounces, then settles flat on the
/// floor, which ends a throw.
fn land_on_floor(world: &mut World, this: Entity, display_object: Entity) {
    let on_island = world.obj::<SpearLogic>(this).on_island;
    let vel_y = {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        if !on_island {
            logic.pos_vel.x = logic.pos_vel.x * 0.4;
            logic.pos_vel.y = -(logic.pos_vel.y * 0.4);
            logic.pos_vel.z = logic.pos_vel.z * 0.4;
        } else {
            logic.pos_vel.y = 0.0;
        }
        logic.pos_vel.y
    };
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;

    let settling = vel_y < 100.0;
    if !settling {
        world.obj_mut::<SpearLogic>(this).spin_spear = false;
        pg_engine::logic::set_behavior(world, this, Some("SpearBounce"));
        return;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
    if world.obj::<SpearLogic>(this).on_island {
        return;
    }

    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.thrown_by_pygmy = false;
        spear.on_island = true;
    }
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    pg_engine::display::object_mut(world, display_object).pos.z = SPRITE_Z_POS;
    world.obj_mut::<SpearLogic>(this).island_floor_collision = true;
}

/// `-[SpearLogic refreshTiltState]` @ 0x100170190
///
/// On the floor the spear slides with a moderate tilt and falls off with a steep one.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island) = {
        let spear = world.obj::<SpearLogic>(this);
        (spear.dragging, spear.falling, spear.on_island)
    };
    if dragging || falling || !on_island {
        return;
    }

    let tilt_state = coconut_logic::throwable_tilt_state(world);
    if tilt_state != world.obj::<SpearLogic>(this).tilt_state {
        if tilt_state == 0 {
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            pg_engine::logic::set_behavior(world, this, Some("SpearStopSlide"));
            world.obj_mut::<SpearLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            pg_engine::logic::set_behavior(world, this, Some("SpearTiltSlide"));
            world.obj_mut::<SpearLogic>(this).tilt_state = 1;
        }
    }

    if coconut_logic::is_island_tipped_over(world) {
        {
            let mut spear = world.obj_mut::<SpearLogic>(this);
            spear.falling = true;
            spear.tilt_state = 0;
        }
        pg_engine::logic::set_behavior(world, this, Some("SpearFall"));
    }
}

/// `-[SpearLogic undoTiltState]` @ 0x100170370
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island, tilt_state) = {
        let spear = world.obj::<SpearLogic>(this);
        (spear.dragging, spear.falling, spear.on_island, spear.tilt_state)
    };
    if dragging || falling || !on_island {
        return;
    }

    if tilt_state == 1 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        pg_engine::logic::set_behavior(world, this, Some("SpearStopSlide"));
    }
    world.obj_mut::<SpearLogic>(this).tilt_state = 0;
}

/// `-[SpearLogic slide:::]` @ 0x100170404
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpearLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    let below_water_line = pos_y < WATER_Y_POS;
    if !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.falling = false;
        spear.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
        logic.rot_vel.z = 0.0;
    }
    world.obj_mut::<SpearLogic>(this).tilt_state = 0;
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("SpearSplash"));
    ItemResult::Replaced
}

/// `-[SpearLogic drag:::]` @ 0x100170514
///
/// Pulls the spear towards the finger with a damped spring.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    if call.first_time {
        pg_engine::logic::logic_mut(world, this).rot_vel.z = 0.0;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SpearLogic>(this).enable_island_floor_collision = false;

        let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
        if let Some(marker) = marker {
            pg_engine::display::insert_after(world, marker, Some(display_object));
        }
    }

    let (pos, display_group) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<SpearLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => pg_engine::display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The original subtracts the spear's own z, so the spring never acts in depth.
    let mut delta = pos;
    delta.x = pos.x - touch_pos.x;
    delta.y = pos.y - touch_pos.y;
    delta.z = pos.z - pos.z;

    let distance = pg_engine::math::length3(delta);
    if distance != 0.0 {
        let vel = pg_engine::logic::logic(world, this).pos_vel;
        let spring = (distance * -100.0) / distance;

        let mut spring_force = delta;
        spring_force.x = -8.0 * vel.x + spring * delta.x;
        spring_force.y = -8.0 * vel.y + spring * delta.y;
        spring_force.z = -8.0 * vel.z + spring * delta.z;
        world.obj_mut::<SpearLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[SpearLogic fall:::]` @ 0x1001706c8
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut spear = world.obj_mut::<SpearLogic>(this);
            if !spear.island_floor_collision {
                spear.on_island = false;
            }
            spear.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SpearLogic>(this).falling = true;
        if call.bool("ignoreIsland") {
            world.obj_mut::<SpearLogic>(this).enable_island_floor_collision = false;
        }
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    if world.obj::<SpearLogic>(this).island_floor_collision {
        world.obj_mut::<SpearLogic>(this).falling = false;
        return ItemResult::Done;
    }
    let below_water_line = pos_y < WATER_Y_POS;
    if !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.spin_spear = false;
        spear.thrown_by_pygmy = false;
        spear.falling = false;
        spear.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
        logic.rot_vel.z = 0.0;
    }
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("SpearSplash"));
    ItemResult::Replaced
}

/// `-[SpearLogic fallFromHurricane:::]` @ 0x100170854
///
/// Falls without floor collision and splashes at y = 72 when deeper than sprite depth
/// (dropped behind the island), at y = 20 otherwise.
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpearLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<SpearLogic>(this).falling = true;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display_object).pos;
    let floor_y = if pos.z < SPRITE_Z_POS {
        FLOOR_Y_POS
    } else {
        WATER_Y_POS
    };
    let below_floor = pos.y < floor_y;
    if !below_floor {
        return ItemResult::Pending;
    }

    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.falling = false;
        spear.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    pg_engine::display::object_mut(world, display_object).pos.y = floor_y;
    world.obj_mut::<SpearLogic>(this).tilt_state = 0;
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("SpearSplash"));
    ItemResult::Replaced
}

/// `-[SpearLogic die:::]` @ 0x100170998
///
/// Resets the spear and drops it in again while a monster still attacks.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);

    if world.obj::<SpearLogic>(this).dragging {
        world.obj_mut::<SpearLogic>(this).dragging = false;
        let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_spear_drag_complete(world, listener, this);
        }
    }

    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.active = false;
        spear.falling = false;
        spear.in_water = false;
        spear.being_used = false;
        spear.apply_spring = false;
        spear.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    if is_spear_needed(world, this) {
        activate(world, this);
    }
    ItemResult::Done
}

/// `-[SpearLogic pickedUpByHurricane:]` @ 0x100170af8
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, picked_up_by_hurricane: Entity) -> bool {
    let spear_free = {
        let spear = world.obj::<SpearLogic>(this);
        !spear.touch_began_inside
            && !spear.dragging
            && !spear.falling
            && !spear.in_water
            && !spear.exit_island
    };
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let game_frame_running = pg_engine::logic::logic(world, this).game_frame_running;
    if already_picked_up || !spear_free || !game_frame_running {
        return false;
    }

    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = Some(picked_up_by_hurricane);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    world.obj_mut::<SpearLogic>(this).tilt_state = 0;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.rot.z = 0.0;
        object.is_selectable = false;
    }
    move_to_island_layer(world, this);
    pg_engine::logic::set_behavior(world, this, Some("SpearPickedUpByHurricane"));
    true
}

/// `-[SpearLogic droppedByHurricane:]` @ 0x100170c64
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    if !world.obj::<SpriteLogic>(this).picked_up_by_hurricane {
        return;
    }

    world.obj_mut::<SpearLogic>(this).falling = true;
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = false;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = false;
        sprite.picked_up_by_hurricane_logic = None;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = true;
    let z_pos = pg_engine::display::object(world, display_object).pos.z;
    if z_pos < SPRITE_Z_POS {
        move_to_behind_island_layer(world, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("SpearDroppedByHurricane"));
}

/// `-[SpearLogic touchBeganInside:allTouches:]` @ 0x100170d58
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let (falling, in_water, being_used, active_touch) = {
        let spear = world.obj::<SpearLogic>(this);
        (spear.falling, spear.in_water, spear.being_used, spear.active_touch)
    };
    if falling || in_water || being_used || active_touch.is_some() {
        return;
    }

    world.obj_mut::<SpearLogic>(this).active_touch = Some(touch);
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    world.obj_mut::<SpearLogic>(this).touch_began_inside = true;

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut spear = world.obj_mut::<SpearLogic>(this);
        spear.touch_screen_pos = touch_pos;
        spear.tilt_state = 0;
        spear.dragging = true;
        spear.apply_spring = true;
        spear.touch_began_inside = false;
        spear.enable_island_floor_collision = false;
    }

    let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_spear_drag(world, listener, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("SpearDrag"));
}

/// `-[SpearLogic touchBeganOutside:allTouches:]` @ 0x100170e84
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<SpearLogic>(this).active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut spear = world.obj_mut::<SpearLogic>(this);
    spear.touch_screen_pos = touch_pos;
    spear.touch_began_inside = false;
}

/// `-[SpearLogic touchMovedEntered:allTouches:]` @ 0x100170ee0
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SpearLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SpearLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SpearLogic touchMovedExited:allTouches:]` @ 0x100170f38
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SpearLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SpearLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SpearLogic touchMovedInside:allTouches:]` @ 0x100170f90
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[SpearLogic touchMovedOutside:allTouches:]` @ 0x100170fa0
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SpearLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<SpearLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[SpearLogic touchEndedInside:allTouches:]` @ 0x100170ff8
///
/// Lets go of a dragged spear: it falls back onto the island.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<SpearLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    if world.obj::<SpearLogic>(this).dragging {
        {
            let mut spear = world.obj_mut::<SpearLogic>(this);
            spear.falling = true;
            spear.dragging = false;
            spear.apply_spring = false;
            spear.disable_collision = false;
        }

        let listener = world.obj::<SpearLogic>(this).spear_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_spear_drag_complete(world, listener, this);
        }
        move_to_island_layer(world, this);
        pg_engine::logic::set_behavior(world, this, Some("SpearFall"));
    }

    let mut spear = world.obj_mut::<SpearLogic>(this);
    spear.touch_began_inside = false;
    spear.active_touch = None;
}

/// `-[SpearLogic touchEndedOutside:allTouches:]` @ 0x1001710cc
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
