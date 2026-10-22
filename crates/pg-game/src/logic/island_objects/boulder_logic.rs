//! `BoulderLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::coconut_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `BoulderLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BoulderLogic {
    /// `mBoulderLogicListener`: `@"<BoulderLogicListener>"`
    pub boulder_logic_listener: Option<Entity>,

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

    /// `mBowledPygmyCount`: `i`
    pub bowled_pygmy_count: i32,

    /// `mEclipseTargetPercent`: `f`
    pub eclipse_target_percent: f32,

    /// `mEclipsePercent`: `f`
    pub eclipse_percent: f32,

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

pub const BOULDER_LOGIC: Class = Class {
    name: "BoulderLogic",
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

/// The resting height of the boulder on the island (`setYPos:126`).
const FLOOR_Y_POS: f32 = 126.0;

/// Below this height the boulder splashes into the sea.
const WATER_Y_POS: f32 = 50.0;

/// `mCrushedPygmyLogic` pushes the boulder away at this speed, by side (`DAT_10029d410`).
const CRUSH_X_VELS: [f32; 2] = [100.0, -100.0];

/// Sequence-item handlers implemented by `BoulderLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "slide" => seq_slide(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[BoulderLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BOULDER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[BoulderLogic initWithDisplayObject:]` @ 0x10001d2a0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BoulderLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut boulder = world.obj_mut::<BoulderLogic>(this);
        boulder.tilt_state = 0;
        boulder.slide_friction = 0.025;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BoulderLogic>(this).enable_island_floor_collision = true;

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, own_display_object);
}

/// `-[BoulderLogic setBoulderLogicListener:]` @ 0x10001d350
pub fn set_boulder_logic_listener(world: &mut World, this: Entity, boulder_logic_listener: Entity) {
    world.obj_mut::<BoulderLogic>(this).boulder_logic_listener = Some(boulder_logic_listener);
}

/// `-[BoulderLogic clearPygmyLogic:]` @ 0x10001d3ac
pub fn clear_pygmy_logic(world: &mut World, this: Entity, clear_pygmy_logic: Entity) {
    let mut boulder = world.obj_mut::<BoulderLogic>(this);
    if boulder.crushed_pygmy_logic == Some(clear_pygmy_logic) {
        boulder.crushed_pygmy_logic = None;
    }
}

/// `-[BoulderLogic enterIsland:]` @ 0x10001d3c8
///
/// Drops the boulder in from (158, 400), or puts it straight on the floor at startup.
pub fn enter_island(world: &mut World, this: Entity, enter_island: bool) {
    if world.obj::<BoulderLogic>(this).falling {
        return;
    }

    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
        logic.rot_vel.z = 0.0;
    }
    world.obj_mut::<BoulderLogic>(this).in_water = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.is_selectable = true;
        object.pos.x = 158.0;
        object.pos.y = 400.0;
        if enter_island {
            object.pos.y = FLOOR_Y_POS;
        }
    }

    let marker = pg_engine::library::display_marker(world, "IslandBackDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("BoulderEnterIsland"));
}

/// `-[BoulderLogic exitIsland:]` @ 0x10001d52c
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).is_selectable = false;

    let crushed = world.obj::<BoulderLogic>(this).crushed_pygmy_logic;
    if let Some(crushed) = crushed {
        crate::pygmy::uncrushed_by_boulder(world, crushed, this, exit_island);
        world.obj_mut::<BoulderLogic>(this).crushed_pygmy_logic = None;
    }

    if world.obj::<BoulderLogic>(this).dragging {
        {
            let mut boulder = world.obj_mut::<BoulderLogic>(this);
            boulder.falling = true;
            boulder.dragging = false;
            boulder.apply_spring = false;
        }
        let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_boulder_drag_complete(world, listener, this);
        }
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<BoulderLogic>(this).enable_island_floor_collision = false;

    if exit_island {
        pg_engine::logic::set_behavior(world, this, Some("BoulderDie"));
        return;
    }
    if world.obj::<BoulderLogic>(this).in_water {
        return;
    }

    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some("BoulderFallIgnoreIsland"));
}

/// `-[BoulderLogic exitIslandNow]` @ 0x10001d6a8
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[BoulderLogic checkPygmyCollision:]` @ 0x10001d720
///
/// While the boulder is over the island at or above the floor: Charlie may smack a moving
/// boulder away, a pygmy under a dropped boulder is crushed (and pins it), and a fast boulder
/// bowls pygmies over.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _check_pygmy_collision: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    let boulder_pos = pg_engine::display::object(world, display_object).pos;

    let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(listener) = listener {
        island_min_x = crate::controller::island_min_x_pos(world, listener);
        island_max_x = crate::controller::island_max_x_pos(world, listener);
    }

    let (crushed, enable_island_floor_collision) = {
        let boulder = world.obj::<BoulderLogic>(this);
        (boulder.crushed_pygmy_logic, boulder.enable_island_floor_collision)
    };
    if crushed.is_some() {
        return;
    }
    let outside_island = boulder_pos.x < island_min_x || boulder_pos.x > island_max_x;
    if !enable_island_floor_collision || outside_island || boulder_pos.y < FLOOR_Y_POS {
        return;
    }

    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_pos = pg_engine::display::object(world, pygmy_display_object).pos;
        let delta_x = boulder_pos.x - (pygmy_pos.x + 0.0);
        let delta_y = boulder_pos.y - (pygmy_pos.y + 31.0);
        let touching = delta_x * delta_x + delta_y * delta_y < 2500.0;
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
            if !crate::pygmy::charlie_smack_boulder(world, pygmy, this) {
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
            if crate::pygmy::crushed_by_boulder(world, pygmy, this) {
                let push_index = usize::from(0.0 < delta_x);
                pg_engine::logic::logic_mut(world, this).pos_vel.x = CRUSH_X_VELS[push_index];
                world.obj_mut::<BoulderLogic>(this).crushed_pygmy_logic = Some(pygmy);
                return;
            }
            continue;
        }

        let fast = 200.0 < speed_x || 200.0 < vel.y.abs();
        if fast && crate::pygmy::hit_by_boulder(world, pygmy, this, delta_x * -3.0) {
            world.obj_mut::<BoulderLogic>(this).bowled_pygmy_count += 1;
            pg_engine::logic::logic_mut(world, this).pos_vel.y = 300.0;
        }
    }
}

/// `-[BoulderLogic isAccelEnabled]` @ 0x10001db38
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
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

/// `-[BoulderLogic onGameFrame:]` @ 0x10001dbcc
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let previous_y = pg_engine::display::object(world, display_object).pos.y;

    // Gravity follows the device tilt while the accelerometer drives the boulder.
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
        let boulder = world.obj::<BoulderLogic>(this);
        (boulder.tilt_state, boulder.slide_friction, boulder.apply_spring, boulder.spring_force)
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
    pg_engine::display::min_pos(world, display_object);
    pg_engine::display::max_pos(world, display_object);

    // A crushed pygmy pins the boulder's x.
    let crushed = world.obj::<BoulderLogic>(this).crushed_pygmy_logic;
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

    world.obj_mut::<BoulderLogic>(this).island_floor_collision = false;
    let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(listener) = listener {
        island_min_x = crate::controller::island_min_x_pos(world, listener);
        island_max_x = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = pg_engine::display::object(world, display_object).pos;
    let enable_island_floor_collision = world.obj::<BoulderLogic>(this).enable_island_floor_collision;
    let over_island = island_min_x <= pos.x && pos.x <= island_max_x;
    let crossed_floor = pos.y < FLOOR_Y_POS && FLOOR_Y_POS <= previous_y;
    if !(crossed_floor && over_island && enable_island_floor_collision) {
        world.obj_mut::<BoulderLogic>(this).on_island = false;
    } else {
        land_on_floor(world, this, listener);
        pg_engine::display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;
    }

    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    if pos_y < -100.0 || 820.0 < pos_y {
        pg_engine::logic::set_behavior(world, this, Some("BoulderDie"));
    }

    check_pygmy_collision(world, this, dt);

    let eclipse_percent = {
        let mut boulder = world.obj_mut::<BoulderLogic>(this);
        let eclipse_step = (boulder.eclipse_target_percent - boulder.eclipse_percent) * 0.13;
        boulder.eclipse_percent = boulder.eclipse_percent + eclipse_step;
        boulder.eclipse_percent
    };
    let shade = 1.0 - eclipse_percent;
    let mut object = pg_engine::display::object_mut(world, display_object);
    object.color = true;
    object.red = shade;
    object.green = shade;
    object.blue = shade;
}

/// The floor contact of `onGameFrame:`: bounce on landing, roll, and settle (reporting the
/// pygmies bowled over since the boulder left the floor).
fn land_on_floor(world: &mut World, this: Entity, listener: Option<Entity>) {
    let on_island = world.obj::<BoulderLogic>(this).on_island;
    let vel = {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        if !on_island {
            logic.pos_vel.x = logic.pos_vel.x * 0.9;
            logic.pos_vel.y = logic.pos_vel.y * -0.4;
        } else {
            logic.pos_vel.y = 0.0;
        }
        logic.rot_vel.z = logic.pos_vel.x * -3.0;
        logic.pos_vel
    };

    let settled = vel.x.abs() < 30.0 && vel.y < 30.0;
    if !settled {
        if 100.0 < vel.y {
            pg_engine::logic::set_behavior(world, this, Some("BoulderBounce"));
        }
        return;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
    if world.obj::<BoulderLogic>(this).on_island {
        return;
    }

    world.obj_mut::<BoulderLogic>(this).on_island = true;
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<BoulderLogic>(this).island_floor_collision = true;

    let bowled_pygmy_count = world.obj::<BoulderLogic>(this).bowled_pygmy_count;
    if 0 < bowled_pygmy_count {
        if let Some(listener) = listener {
            crate::controller::on_pygmies_bowled(world, listener, bowled_pygmy_count);
        }
        world.obj_mut::<BoulderLogic>(this).bowled_pygmy_count = 0;
    }
}

/// `-[BoulderLogic setEclipsePercent:]` @ 0x10001e260
pub fn set_eclipse_percent(world: &mut World, this: Entity, eclipse_percent: f32) {
    world.obj_mut::<BoulderLogic>(this).eclipse_target_percent = eclipse_percent;
}

/// `-[BoulderLogic refreshTiltState]` @ 0x10001e270
///
/// On the floor the boulder slides with a moderate tilt and falls off with a steep one.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island) = {
        let boulder = world.obj::<BoulderLogic>(this);
        (boulder.dragging, boulder.falling, boulder.on_island)
    };

    if !dragging && !falling && on_island {
        let tilt_state = coconut_logic::throwable_tilt_state(world);
        if tilt_state != world.obj::<BoulderLogic>(this).tilt_state {
            if tilt_state == 0 {
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                pg_engine::logic::set_behavior(world, this, Some("BoulderStopSlide"));
                world.obj_mut::<BoulderLogic>(this).tilt_state = 0;
            } else if tilt_state == 1 {
                pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
                pg_engine::logic::set_behavior(world, this, Some("BoulderTiltSlide"));
                world.obj_mut::<BoulderLogic>(this).tilt_state = 1;
            }
        }

        if !coconut_logic::is_island_tipped_over(world) {
            return;
        }
        {
            let mut boulder = world.obj_mut::<BoulderLogic>(this);
            boulder.falling = true;
            boulder.tilt_state = 0;
        }
        pg_engine::logic::set_behavior(world, this, Some("BoulderFall"));
    } else if !coconut_logic::is_island_tipped_over(world) {
        return;
    }

    let crushed = world.obj::<BoulderLogic>(this).crushed_pygmy_logic;
    if let Some(crushed) = crushed {
        crate::pygmy::uncrushed_by_boulder(world, crushed, this, false);
        world.obj_mut::<BoulderLogic>(this).crushed_pygmy_logic = None;
    }
}

/// `-[BoulderLogic undoTiltState]` @ 0x10001e4bc
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island, tilt_state) = {
        let boulder = world.obj::<BoulderLogic>(this);
        (boulder.dragging, boulder.falling, boulder.on_island, boulder.tilt_state)
    };
    if dragging || falling || !on_island {
        return;
    }

    if tilt_state == 1 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        pg_engine::logic::set_behavior(world, this, Some("BoulderStopSlide"));
    }
    world.obj_mut::<BoulderLogic>(this).tilt_state = 0;
}

/// `-[BoulderLogic slide:::]` @ 0x10001e550
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<BoulderLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    let below_water_line = pos_y < WATER_Y_POS;
    if !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut boulder = world.obj_mut::<BoulderLogic>(this);
        boulder.falling = false;
        boulder.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
        logic.rot_vel.z = 0.0;
    }
    world.obj_mut::<BoulderLogic>(this).tilt_state = 0;
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("BoulderSplash"));
    ItemResult::Replaced
}

/// `-[BoulderLogic drag:::]` @ 0x10001e664
///
/// Pulls the boulder towards the finger with a damped spring.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    if call.first_time {
        pg_engine::logic::logic_mut(world, this).rot_vel.z = 0.0;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BoulderLogic>(this).enable_island_floor_collision = false;

        let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
        if let Some(marker) = marker {
            pg_engine::display::insert_after(world, marker, Some(display_object));
        }
    }

    let (pos, display_group) = {
        let object = pg_engine::display::object(world, display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<BoulderLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => pg_engine::display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The original subtracts the boulder's own z, so the spring never acts in depth.
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
        world.obj_mut::<BoulderLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[BoulderLogic fall:::]` @ 0x10001e818
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut boulder = world.obj_mut::<BoulderLogic>(this);
            if !boulder.island_floor_collision {
                boulder.on_island = false;
            }
            boulder.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BoulderLogic>(this).falling = true;
        if call.bool("ignoreIsland") {
            world.obj_mut::<BoulderLogic>(this).enable_island_floor_collision = false;
        }
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pos_y = pg_engine::display::object(world, display_object).pos.y;
    if world.obj::<BoulderLogic>(this).island_floor_collision {
        world.obj_mut::<BoulderLogic>(this).falling = false;
        return ItemResult::Done;
    }
    let below_water_line = pos_y < WATER_Y_POS;
    if !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut boulder = world.obj_mut::<BoulderLogic>(this);
        boulder.falling = false;
        boulder.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
        logic.rot_vel.z = 0.0;
    }
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::set_behavior(world, this, Some("BoulderSplash"));
    ItemResult::Replaced
}

/// `-[BoulderLogic die:::]` @ 0x10001e990
///
/// Resets the boulder and drops it in again while the feature is enabled.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);

    let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
    let bowled_pygmy_count = world.obj::<BoulderLogic>(this).bowled_pygmy_count;
    if 0 < bowled_pygmy_count {
        if let Some(listener) = listener {
            crate::controller::on_pygmies_bowled(world, listener, bowled_pygmy_count);
        }
        world.obj_mut::<BoulderLogic>(this).bowled_pygmy_count = 0;
    }

    let crushed = world.obj::<BoulderLogic>(this).crushed_pygmy_logic;
    if let Some(crushed) = crushed {
        crate::pygmy::uncrushed_by_boulder(world, crushed, this, false);
        world.obj_mut::<BoulderLogic>(this).crushed_pygmy_logic = None;
    }

    if world.obj::<BoulderLogic>(this).dragging {
        world.obj_mut::<BoulderLogic>(this).dragging = false;
        let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_boulder_drag_complete(world, listener, this);
        }
    }

    {
        let mut boulder = world.obj_mut::<BoulderLogic>(this);
        boulder.falling = false;
        boulder.in_water = false;
        boulder.apply_spring = false;
        boulder.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<BoulderLogic>(this).crushed_pygmy_logic = None;

    let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
    let boulder_enabled = match listener {
        Some(listener) => crate::controller::is_boulder_enabled(world, listener),
        None => false,
    };
    if boulder_enabled {
        enter_island(world, this, false);
    }
    ItemResult::Done
}

/// `-[BoulderLogic touchBeganInside:allTouches:]` @ 0x10001eb04
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    let (falling, in_water, active_touch) = {
        let boulder = world.obj::<BoulderLogic>(this);
        (boulder.falling, boulder.in_water, boulder.active_touch)
    };
    if story_active || falling || in_water || active_touch.is_some() {
        return;
    }

    let crushed = world.obj::<BoulderLogic>(this).crushed_pygmy_logic;
    if let Some(crushed) = crushed {
        crate::pygmy::uncrushed_by_boulder(world, crushed, this, false);
        world.obj_mut::<BoulderLogic>(this).crushed_pygmy_logic = None;
    }

    world.obj_mut::<BoulderLogic>(this).active_touch = Some(touch);
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    world.obj_mut::<BoulderLogic>(this).touch_began_inside = true;

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut boulder = world.obj_mut::<BoulderLogic>(this);
        boulder.touch_screen_pos = touch_pos;
        boulder.tilt_state = 0;
        boulder.dragging = true;
        boulder.apply_spring = true;
        boulder.touch_began_inside = false;
        boulder.enable_island_floor_collision = false;
    }

    if let Some(listener) = listener {
        crate::controller::on_boulder_drag(world, listener, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("BoulderDrag"));
}

/// `-[BoulderLogic touchBeganOutside:allTouches:]` @ 0x10001ec78
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<BoulderLogic>(this).active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut boulder = world.obj_mut::<BoulderLogic>(this);
    boulder.touch_screen_pos = touch_pos;
    boulder.touch_began_inside = false;
}

/// `-[BoulderLogic touchMovedEntered:allTouches:]` @ 0x10001ecd4
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BoulderLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BoulderLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BoulderLogic touchMovedExited:allTouches:]` @ 0x10001ed2c
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BoulderLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BoulderLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BoulderLogic touchMovedInside:allTouches:]` @ 0x10001ed84
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[BoulderLogic touchMovedOutside:allTouches:]` @ 0x10001ed94
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BoulderLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BoulderLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BoulderLogic touchEndedInside:allTouches:]` @ 0x10001edec
///
/// Lets go of a dragged boulder: it falls back behind the island.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BoulderLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    if world.obj::<BoulderLogic>(this).dragging {
        {
            let mut boulder = world.obj_mut::<BoulderLogic>(this);
            boulder.falling = true;
            boulder.dragging = false;
            boulder.apply_spring = false;
        }

        let display_object = pg_engine::logic::display_object(world, this);
        let marker = pg_engine::library::display_marker(world, "IslandBackDisplayMarker");
        if let Some(marker) = marker {
            pg_engine::display::insert_after(world, marker, Some(display_object));
        }

        let listener = world.obj::<BoulderLogic>(this).boulder_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_boulder_drag_complete(world, listener, this);
        }
        pg_engine::logic::set_behavior(world, this, Some("BoulderFall"));
    }

    let mut boulder = world.obj_mut::<BoulderLogic>(this);
    boulder.touch_began_inside = false;
    boulder.active_touch = None;
}

/// `-[BoulderLogic touchEndedOutside:allTouches:]` @ 0x10001eef0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
