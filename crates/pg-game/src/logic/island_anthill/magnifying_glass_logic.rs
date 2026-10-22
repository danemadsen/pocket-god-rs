//! `MagnifyingGlassLogic` (superclass `SpriteLogic`).
//!
//! The magnifying glass: a throwable (`docs/re/logic-catalog.md` §2.2) that, held up on a
//! sunny day, focuses a sun beam whose hot spot burns the pygmies and ants under it.

use crate::prelude::*;

use crate::logic::island_anthill::magnifying_beam_model::{self, MAGNIFYING_BEAM_MODEL};
use crate::logic::island_anthill::{ant_hill_logic, ant_logic};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::texture::Textures;
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `MagnifyingGlassLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MagnifyingGlassLogic {
    /// `mMagnifyingGlassLogicListener`: `@"<MagnifyingGlassLogicListener>"`
    pub magnifying_glass_logic_listener: Option<Entity>,

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

    /// `mBurntPygmyLogic`: `@"PygmyLogic"`
    pub burnt_pygmy_logic: Option<Entity>,

    /// `mBeamDisplayObject`: `@"MagnifyingBeamModel"`
    pub beam_display_object: Option<Entity>,

    /// `mHotSpotDisplayObject`: `@"BCDisplayObject"`
    pub hot_spot_display_object: Option<Entity>,

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

pub const MAGNIFYING_GLASS_LOGIC: Class = Class {
    name: "MagnifyingGlassLogic",
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

/// The resting height of the glass on the island (`setYPos:126`).
const FLOOR_Y_POS: f32 = 126.0;

/// Below this height the glass splashes into the sea.
const WATER_Y_POS: f32 = 50.0;

/// The gravity magnitude, straight down or along the device tilt.
const GRAVITY: f32 = 3000.0;

/// How close (in x) the hot spot must be to something to burn it.
const BURN_RANGE: f32 = 10.0;

/// The glass's heights at which the beam starts and stops focusing.
const BEAM_MIN_Y_POS: f32 = 150.0;
const BEAM_MAX_Y_POS: f32 = 210.0;

/// The hot spot's height on the ground.
const HOT_SPOT_Y_POS: f32 = 90.0;

/// The beam's widths at the glass and at its far end.
const BEAM_TOP_WIDTH: f32 = 74.0;
const BEAM_BOTTOM_WIDTH: f32 = 51.8;

/// Sequence-item handlers implemented by `MagnifyingGlassLogic` (`respondsToSelector:` order).
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

/// `[[MagnifyingGlassLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MAGNIFYING_GLASS_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[MagnifyingGlassLogic initWithDisplayObject:]` @ 0x1000d2620
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MagnifyingGlassLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
        glass.tilt_state = 0;
        glass.slide_friction = 0.025;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MagnifyingGlassLogic>(this).enable_island_floor_collision = true;

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[MagnifyingGlassLogic setMagnifyingGlassLogicListener:]` @ 0x1000d26d0
pub fn set_magnifying_glass_logic_listener(world: &mut World, this: Entity, magnifying_glass_logic_listener: Entity) {
    world.obj_mut::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener = Some(magnifying_glass_logic_listener);
}

/// `-[MagnifyingGlassLogic enterIsland:]` @ 0x1000d272c
///
/// Drops the glass in from (158, 400), or puts it straight on the floor at startup.
pub fn enter_island(world: &mut World, this: Entity, startup: bool) {
    if world.obj::<MagnifyingGlassLogic>(this).falling {
        return;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<MagnifyingGlassLogic>(this).in_water = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.is_selectable = true;
        object.pos.x = 158.0;
        object.pos.y = 400.0;
        if startup {
            object.pos.y = FLOOR_Y_POS;
        }
    }

    let marker = pg_engine::library::display_marker(world, "IslandBackDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("MagnifyingGlassEnterIsland"));
}

/// `-[MagnifyingGlassLogic exitIsland:]` @ 0x1000d287c
///
/// Ends a drag; then removes the glass at once, or lets it fall through the island.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    if world.obj::<MagnifyingGlassLogic>(this).dragging {
        {
            let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
            glass.falling = true;
            glass.dragging = false;
            glass.apply_spring = false;
        }
        let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_magnifying_glass_drag_complete(world, listener, this);
        }
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MagnifyingGlassLogic>(this).enable_island_floor_collision = false;

    if now {
        logic::set_behavior(world, this, Some("MagnifyingGlassDie"));
        return;
    }
    if world.obj::<MagnifyingGlassLogic>(this).in_water {
        return;
    }

    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("MagnifyingGlassFallIgnoreIsland"));
}

/// `-[MagnifyingGlassLogic exitIslandNow]` @ 0x1000d29cc
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[MagnifyingGlassLogic isAccelEnabled]` @ 0x1000d2a7c
///
/// The device tilt moves the glass only while gravity is on and the glass can be picked up.
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }

    let display_object = logic::display_object(world, this);
    if !display::object(world, display_object).is_selectable {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[MagnifyingGlassLogic onPygmyDie:]` @ 0x1000d2b10
pub fn on_pygmy_die(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
    if glass.burnt_pygmy_logic == Some(pygmy_logic) {
        glass.burnt_pygmy_logic = None;
    }
}

/// `-[MagnifyingGlassLogic checkPygmyCollision:hotSpotPercent:]` @ 0x1000d2b2c
///
/// A lit hot spot sets fire to the first pygmy under it, or else to every ant under it. The
/// burning pygmy is let go when the hot spot goes out or the glass moves away from it.
pub fn check_pygmy_collision(world: &mut World, this: Entity, _dt: f32, hot_spot_percent: f32) {
    let display_object = logic::display_object(world, this);
    let glass_x_pos = display::object(world, display_object).pos.x;

    let burnt_pygmy = world.obj::<MagnifyingGlassLogic>(this).burnt_pygmy_logic;
    if let Some(burnt_pygmy) = burnt_pygmy {
        let pygmy_display_object = logic::display_object(world, burnt_pygmy);
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
        let distance = (glass_x_pos - pygmy_x_pos).abs();
        if hot_spot_percent == 0.0 || BURN_RANGE < distance {
            crate::pygmy::unburnt_by_magnifying_glass(world, burnt_pygmy, this);
            world.obj_mut::<MagnifyingGlassLogic>(this).burnt_pygmy_logic = None;
        }
        return;
    }

    if hot_spot_percent == 0.0 {
        return;
    }

    let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        if !crate::pygmy::can_be_burnt_by_magnifying_glass(world, pygmy, this) {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
        let distance = (glass_x_pos - pygmy_x_pos).abs();
        if distance < BURN_RANGE {
            crate::pygmy::burnt_by_magnifying_glass(world, pygmy, this);
            world.obj_mut::<MagnifyingGlassLogic>(this).burnt_pygmy_logic = Some(pygmy);
            return;
        }
    }

    let ant_hill = match listener {
        Some(listener) => crate::controller::ant_hill_logic(world, listener, this),
        None => None,
    };
    let ants = match ant_hill {
        Some(ant_hill) => ant_hill_logic::ant_logic_array(world, ant_hill),
        None => Vec::new(),
    };
    for ant in ants {
        if !ant_logic::can_be_burnt_by_magnifying_glass(world, ant, this) {
            continue;
        }

        let ant_display_object = logic::display_object(world, ant);
        let ant_x_pos = display::object(world, ant_display_object).pos.x;
        let distance = (glass_x_pos - ant_x_pos).abs();
        if distance < BURN_RANGE {
            ant_logic::burnt_by_magnifying_glass(world, ant, this);
        }
    }
}

/// `-[MagnifyingGlassLogic onGameFrame:]` @ 0x1000d2eb0
///
/// The throwable physics (gravity or tilt, spring, screen edges, island floor), then the sun
/// beam: held between y 150 and 210 on a sunny day the beam focuses, sharpest at y 180, and
/// its hot spot on the ground burns what is under it.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let previous_y_pos = display::object(world, display_object).pos.y;

    if is_accel_enabled(world, this) {
        let accel_force = world.resource::<BcView>().accel_force;
        let tilt = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let mut gravity = pg_engine::math::normalize3(tilt);
        gravity.x = gravity.x * GRAVITY;
        gravity.y = gravity.y * GRAVITY;
        gravity.z = gravity.z * GRAVITY;
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = Vec4::new(0.0, -GRAVITY, 0.0, 1.0);
        undo_tilt_state(world, this);
    }

    // Only a tilt slide keeps a (damped) sideways pull.
    let (tilt_state, slide_friction, apply_spring, spring_force) = {
        let glass = world.obj::<MagnifyingGlassLogic>(this);
        (glass.tilt_state, glass.slide_friction, glass.apply_spring, glass.spring_force)
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

    // Semi-implicit Euler: the position moves with the new velocity.
    let vel = {
        let mut state = logic::logic_mut(world, this);
        state.pos_force = force;
        state.pos_vel.x = state.pos_vel.x + force.x * dt;
        state.pos_vel.y = state.pos_vel.y + force.y * dt;
        state.pos_vel.z = state.pos_vel.z + force.z * dt;
        state.pos_vel
    };
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = object.pos.x + vel.x * dt;
        object.pos.y = object.pos.y + vel.y * dt;
        object.pos.z = object.pos.z + vel.z * dt;
    }

    // Keep the glass on screen.
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);
    if min_pos.x < 0.0 {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = (object.pos.x - min_pos.x) + 0.0;
    }
    if 480.0 < max_pos.x {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = 480.0 - (max_pos.x - object.pos.x);
    }

    world.obj_mut::<MagnifyingGlassLogic>(this).island_floor_collision = false;
    let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let enable_island_floor_collision = world.obj::<MagnifyingGlassLogic>(this).enable_island_floor_collision;
    let over_island = pos.x >= island_min_x_pos && pos.x <= island_max_x_pos;
    let crossed_floor = pos.y < FLOOR_Y_POS && previous_y_pos >= FLOOR_Y_POS;
    let mut pos_y = pos.y;
    if crossed_floor && over_island && enable_island_floor_collision {
        // Bounce on the first contact, then come to rest.
        let on_island = world.obj::<MagnifyingGlassLogic>(this).on_island;
        let vel = {
            let mut state = logic::logic_mut(world, this);
            if !on_island {
                state.pos_vel.x = state.pos_vel.x * 0.4;
                state.pos_vel.y = state.pos_vel.y * -0.4;
            } else {
                state.pos_vel.y = 0.0;
            }
            state.pos_vel
        };

        let resting = vel.x.abs() < 30.0 && vel.y < 30.0;
        if resting {
            logic::logic_mut(world, this).pos_vel.y = 0.0;
            if !world.obj::<MagnifyingGlassLogic>(this).on_island {
                world.obj_mut::<MagnifyingGlassLogic>(this).on_island = true;
                logic::logic_mut(world, this).pos_vel.x = 0.0;
                world.obj_mut::<MagnifyingGlassLogic>(this).island_floor_collision = true;
            }
        } else if 100.0 < vel.y {
            logic::set_behavior(world, this, Some("MagnifyingGlassBounce"));
        }

        display::object_mut(world, display_object).pos.y = FLOOR_Y_POS;
        pos_y = display::object(world, display_object).pos.y;
    } else {
        world.obj_mut::<MagnifyingGlassLogic>(this).on_island = false;
    }

    if pos_y < -100.0 || 820.0 < pos_y {
        logic::set_behavior(world, this, Some("MagnifyingGlassDie"));
    }

    let mut hot_spot_percent = 0.0;
    let beam = world.obj::<MagnifyingGlassLogic>(this).beam_display_object;
    if let Some(beam) = beam {
        let glass_pos = display::object(world, display_object).pos;

        let mut focus_percent = 0.0;
        let below_focus_range = glass_pos.y < BEAM_MIN_Y_POS;
        let above_focus_range = BEAM_MAX_Y_POS < glass_pos.y;
        if !below_focus_range && !above_focus_range {
            if let Some(listener) = listener {
                // `isStormActive` is only asked on a day.
                let day_active = crate::controller::is_day_active(world, listener);
                let sunny = day_active && !crate::controller::is_storm_active(world, listener);
                if sunny {
                    focus_percent = (glass_pos.y - BEAM_MIN_Y_POS) / 60.0;
                }
            }
        }

        // The original computes the beam's alpha in double precision.
        let focus_offset = (focus_percent - 0.5).abs();
        let beam_alpha = (focus_offset as f64 * -2.0 + 1.0) as f32;
        hot_spot_percent = beam_alpha - 0.5;
        hot_spot_percent = hot_spot_percent + hot_spot_percent;
        if hot_spot_percent <= 0.0 {
            hot_spot_percent = 0.0;
        }

        let focus_height = focus_percent * 120.0 + 30.0;
        magnifying_beam_model::set_top_width(world, beam, BEAM_TOP_WIDTH);
        magnifying_beam_model::set_bottom_width(world, beam, BEAM_BOTTOM_WIDTH);
        magnifying_beam_model::set_focus_height(world, beam, focus_height);
        {
            let mut beam_object = display::object_mut(world, beam);
            beam_object.set_alpha(beam_alpha);
            beam_object.pos.x = glass_pos.x;
            beam_object.pos.y = glass_pos.y;
            beam_object.pos.z = glass_pos.z;
        }

        let hot_spot = world.obj::<MagnifyingGlassLogic>(this).hot_spot_display_object;
        if let Some(hot_spot) = hot_spot {
            let mut hot_spot_object = display::object_mut(world, hot_spot);
            hot_spot_object.pos.x = glass_pos.x;
            hot_spot_object.pos.y = HOT_SPOT_Y_POS;
            hot_spot_object.pos.z = glass_pos.z;
            hot_spot_object.set_alpha(hot_spot_percent);
        }
    }

    check_pygmy_collision(world, this, dt, hot_spot_percent);
}

/// `-[MagnifyingGlassLogic refreshTiltState]` @ 0x1000d3628
///
/// On the floor the glass slides with a moderate tilt and falls off with a steep one.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island) = {
        let glass = world.obj::<MagnifyingGlassLogic>(this);
        (glass.dragging, glass.falling, glass.on_island)
    };
    if dragging || falling || !on_island {
        return;
    }

    let raw_roll = world.resource::<BcView>().raw_accel_rot_degrees.z;
    let tilt_state = if raw_roll < -130.0 || 130.0 < raw_roll {
        0
    } else if 90.0 < raw_roll || raw_roll < -90.0 {
        1
    } else {
        2
    };

    if tilt_state != world.obj::<MagnifyingGlassLogic>(this).tilt_state {
        if tilt_state == 0 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("MagnifyingGlassStopSlide"));
            world.obj_mut::<MagnifyingGlassLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("MagnifyingGlassTiltSlide"));
            world.obj_mut::<MagnifyingGlassLogic>(this).tilt_state = 1;
        }
    }

    let roll = world.resource::<BcView>().accel_rot_degrees.z;
    if -90.0 < roll && roll < 90.0 {
        {
            let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
            glass.falling = true;
            glass.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("MagnifyingGlassFall"));
    }
}

/// `-[MagnifyingGlassLogic undoTiltState]` @ 0x1000d3808
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let (dragging, falling, on_island, tilt_state) = {
        let glass = world.obj::<MagnifyingGlassLogic>(this);
        (glass.dragging, glass.falling, glass.on_island, glass.tilt_state)
    };
    if dragging || falling || !on_island {
        return;
    }

    if tilt_state == 1 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        logic::set_behavior(world, this, Some("MagnifyingGlassStopSlide"));
    }
    world.obj_mut::<MagnifyingGlassLogic>(this).tilt_state = 0;
}

/// `-[MagnifyingGlassLogic slide:::]` @ 0x1000d389c
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<MagnifyingGlassLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = logic::display_object(world, this);
    let pos_y = display::object(world, display_object).pos.y;
    let below_water_line = pos_y < WATER_Y_POS;
    if !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
        glass.falling = false;
        glass.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<MagnifyingGlassLogic>(this).tilt_state = 0;
    logic::set_behavior(world, this, Some("MagnifyingGlassSplash"));
    ItemResult::Replaced
}

/// `-[MagnifyingGlassLogic drag:::]` @ 0x1000d3984
///
/// On the first call lifts the glass to the drag layer with the hot spot and the beam in front
/// of it; then pulls the glass towards the finger with a damped spring.
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<MagnifyingGlassLogic>(this).enable_island_floor_collision = false;

        let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(display_object));
        }

        let hot_spot_texture = pg_engine::library::texture_def_id(world, "HotSpot_01");
        let hot_spot = display::model::spawn(world, Some("MagnifyingHotSpot"), hot_spot_texture);
        world.obj_mut::<MagnifyingGlassLogic>(this).hot_spot_display_object = Some(hot_spot);
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(hot_spot));
        }

        // Centred on its position. PORT: a missing texture reads as size 0 (the original
        // would dereference NULL); `HotSpot_01` is always there.
        let (frame_width, frame_height) = match hot_spot_texture {
            Some(texture) => {
                let def = world.resource::<Textures>().def(texture);
                (def.frame_width, def.frame_height)
            }
            None => (0.0, 0.0),
        };
        {
            let mut hot_spot_object = display::object_mut(world, hot_spot);
            hot_spot_object.origin_pos.x = frame_width * -0.5;
            hot_spot_object.origin_pos.y = frame_height * -0.5;
        }

        // `[mHotSpotDisplayObject release]`: the drag layer owns the hot spot from here on, and
        // likewise the beam.
        let beam = world.spawn(Isa(&MAGNIFYING_BEAM_MODEL)).id();
        magnifying_beam_model::init_with_id(world, beam, "MagnifyingBeam");
        world.obj_mut::<MagnifyingGlassLogic>(this).beam_display_object = Some(beam);
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(beam));
        }
    }

    let (pos, display_group) = {
        let object = display::object(world, display_object);
        (object.pos, object.display_group)
    };
    let touch_screen_pos = world.obj::<MagnifyingGlassLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The original subtracts the glass's own z, so the spring never acts in depth.
    let mut delta = pos;
    delta.x = pos.x - touch_pos.x;
    delta.y = pos.y - touch_pos.y;
    delta.z = pos.z - pos.z;

    let distance = pg_engine::math::length3(delta);
    if distance != 0.0 {
        let vel = logic::logic(world, this).pos_vel;
        let spring = (distance * -50.0) / distance;

        let mut spring_force = delta;
        spring_force.x = -6.0 * vel.x + spring * delta.x;
        spring_force.y = -6.0 * vel.y + spring * delta.y;
        spring_force.z = -6.0 * vel.z + spring * delta.z;
        world.obj_mut::<MagnifyingGlassLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[MagnifyingGlassLogic fall:::]` @ 0x1000d3c68
///
/// Falls until the glass lands on the island, or splashes below the water line.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
            if !glass.island_floor_collision {
                glass.on_island = false;
            }
            glass.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<MagnifyingGlassLogic>(this).falling = true;
        if call.bool("ignoreIsland") {
            world.obj_mut::<MagnifyingGlassLogic>(this).enable_island_floor_collision = false;
        }
    }

    let display_object = logic::display_object(world, this);
    let pos_y = display::object(world, display_object).pos.y;
    if world.obj::<MagnifyingGlassLogic>(this).island_floor_collision {
        world.obj_mut::<MagnifyingGlassLogic>(this).falling = false;
        return ItemResult::Done;
    }

    let below_water_line = pos_y < WATER_Y_POS;
    if !below_water_line {
        return ItemResult::Pending;
    }

    {
        let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
        glass.falling = false;
        glass.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    logic::set_behavior(world, this, Some("MagnifyingGlassSplash"));
    ItemResult::Replaced
}

/// `-[MagnifyingGlassLogic die:::]` @ 0x1000d3db4
///
/// Resets the glass and drops it in again while the ant hill is enabled.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    if world.obj::<MagnifyingGlassLogic>(this).dragging {
        world.obj_mut::<MagnifyingGlassLogic>(this).dragging = false;
        let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_magnifying_glass_drag_complete(world, listener, this);
        }
    }

    {
        let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
        glass.dragging = false;
        glass.falling = false;
        glass.in_water = false;
        glass.apply_spring = false;
        glass.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
    let ant_hill_enabled = match listener {
        Some(listener) => crate::controller::is_ant_hill_enabled(world, listener),
        None => false,
    };
    if ant_hill_enabled {
        enter_island(world, this, false);
    }
    ItemResult::Done
}

/// `-[MagnifyingGlassLogic touchBeganInside:allTouches:]` @ 0x1000d3eb0
///
/// Picks the glass up (refused during a story, while it falls or in the water).
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    let (falling, in_water, active_touch) = {
        let glass = world.obj::<MagnifyingGlassLogic>(this);
        (glass.falling, glass.in_water, glass.active_touch)
    };
    if story_active || falling || in_water || active_touch.is_some() {
        return;
    }

    world.obj_mut::<MagnifyingGlassLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
        glass.touch_screen_pos = touch_pos;
        glass.tilt_state = 0;
        glass.dragging = true;
        glass.apply_spring = true;
        glass.enable_island_floor_collision = false;
    }

    if let Some(listener) = listener {
        crate::controller::on_magnifying_glass_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("MagnifyingGlassDrag"));
}

/// `-[MagnifyingGlassLogic touchBeganOutside:allTouches:]` @ 0x1000d3fd8
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    if world.obj::<MagnifyingGlassLogic>(this).active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<MagnifyingGlassLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[MagnifyingGlassLogic touchMovedEntered:allTouches:]` @ 0x1000d4028
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MagnifyingGlassLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<MagnifyingGlassLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[MagnifyingGlassLogic touchMovedExited:allTouches:]` @ 0x1000d4080
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MagnifyingGlassLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<MagnifyingGlassLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[MagnifyingGlassLogic touchMovedInside:allTouches:]` @ 0x1000d40d8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[MagnifyingGlassLogic touchMovedOutside:allTouches:]` @ 0x1000d40e8
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MagnifyingGlassLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<MagnifyingGlassLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[MagnifyingGlassLogic touchEndedInside:allTouches:]` @ 0x1000d4140
///
/// Lets go of a dragged glass: the beam and hot spot go out and the glass falls back behind
/// the island.
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MagnifyingGlassLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    if world.obj::<MagnifyingGlassLogic>(this).dragging {
        {
            let mut glass = world.obj_mut::<MagnifyingGlassLogic>(this);
            glass.falling = true;
            glass.dragging = false;
            glass.apply_spring = false;
        }

        // The drag layer held the last reference to the beam and to the hot spot.
        let beam = world.obj::<MagnifyingGlassLogic>(this).beam_display_object;
        if let Some(beam) = beam {
            display::remove_from_display_group(world, beam);
            magnifying_beam_model::dealloc(world, beam);
        }
        world.obj_mut::<MagnifyingGlassLogic>(this).beam_display_object = None;

        let hot_spot = world.obj::<MagnifyingGlassLogic>(this).hot_spot_display_object;
        if let Some(hot_spot) = hot_spot {
            display::remove_from_display_group(world, hot_spot);
            world.despawn(hot_spot);
        }
        world.obj_mut::<MagnifyingGlassLogic>(this).hot_spot_display_object = None;

        let display_object = logic::display_object(world, this);
        let marker = pg_engine::library::display_marker(world, "IslandBackDisplayMarker");
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(display_object));
        }

        let listener = world.obj::<MagnifyingGlassLogic>(this).magnifying_glass_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_magnifying_glass_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("MagnifyingGlassFall"));
    }

    world.obj_mut::<MagnifyingGlassLogic>(this).active_touch = None;
}

/// `-[MagnifyingGlassLogic touchEndedOutside:allTouches:]` @ 0x1000d427c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
