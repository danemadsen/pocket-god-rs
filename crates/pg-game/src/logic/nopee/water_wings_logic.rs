//! `WaterWingsLogic` (superclass `SpriteLogic`).
//!
//! A throwable of `docs/re/logic-catalog.md` §2.2 (the `throwable_*` steps of
//! [`crate::logic::island_core::coconut_logic`]) with slide friction 0.04 and the floor at
//! y 80. Dropped onto a pygmy it straps itself on, and the pygmy dives into the No Pee game.

use crate::prelude::*;

use crate::logic::island_core::coconut_logic::{
    ISLAND_Z_POS, THROWABLE_BOUNCE_SPEED, THROWABLE_GRAVITY_FORCE, apply_throwable_slide_friction,
    bounce_throwable_velocity, clamp_throwable_to_screen, integrate_throwable, is_island_tipped_over,
    is_throwable_out_of_range, reaches_island_floor, throwable_drag_z_rot, throwable_pos_force,
    throwable_spring_force, throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::nopee::no_pee_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `WaterWingsLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct WaterWingsLogic {
    /// `mWaterWingsLogicListener`: `@"<WaterWingsLogicListener>"`
    pub water_wings_logic_listener: Option<Entity>,

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

pub const WATER_WINGS_LOGIC: Class = Class {
    name: "WaterWingsLogic",
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

/// `mSlideFriction` (`0x3d23d70a`).
const SLIDE_FRICTION: f32 = 0.04;

/// The island floor.
const FLOOR_Y: f32 = 80.0;

/// `enterIsland:` drops the wings in at this x, onto the floor or from above the screen.
const ENTER_X_POS: f32 = 300.0;
const ENTER_ABOVE_Y_POS: f32 = 400.0;

/// `fall:::` splashes below this y; `fallFromHurricane:::` below 20 in front of the island and
/// 80 behind it.
const WATER_LINE_Y: f32 = 20.0;
const BEHIND_ISLAND_WATER_LINE_Y: f32 = 80.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkPygmyCollision`: falling faster than this, the wings strap onto a pygmy whose point
/// 31 above its origin lies within 30 points on both axes.
const STRAP_MIN_FALL_SPEED: f32 = -20.0;
const STRAP_REACH: f32 = 30.0;
const STRAP_PYGMY_Y_OFFSET: f32 = 31.0;

/// The wings face away from the screen centre once strapped on.
const SCREEN_CENTER_X: f32 = 240.0;

/// The animation channel the wings bind the wearing pygmy to.
const PYGMY_CHANNEL: usize = 1;

/// Sequence-item handlers implemented by `WaterWingsLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "waterWingsSplashComplete" => seq_water_wings_splash_complete(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "fallFromHurricane" => seq_fall_from_hurricane(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[WaterWingsLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&WATER_WINGS_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[WaterWingsLogic initWithDisplayObject:]` @ 0x10010fbd4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(WaterWingsLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
        water_wings.tilt_state = 0;
        water_wings.slide_friction = SLIDE_FRICTION;
        water_wings.exit_island = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<WaterWingsLogic>(this).enable_island_floor_collision = true;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let mut display = display::object_mut(world, display_object);
    display.col_x_min = -15.0;
    display.col_x_max = 50.0;
    display.col_y_min = -20.0;
    display.col_y_max = 20.0;
}

/// `-[WaterWingsLogic setWaterWingsLogicListener:]` @ 0x10010fcf4
pub fn set_water_wings_logic_listener(world: &mut World, this: Entity, water_wings_logic_listener: Entity) {
    world.obj_mut::<WaterWingsLogic>(this).water_wings_logic_listener = Some(water_wings_logic_listener);
}

/// `-[WaterWingsLogic moveToIslandLayer]` @ 0x10010fdb4
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[WaterWingsLogic moveToBehindIslandLayer]` @ 0x10010fe1c
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let island_display_group = pg_engine::library::display_group(world, "IslandDisplayGroup");
    let marker_index = match island_display_group {
        Some(island_display_group) => display::group::get_display_object_index(world, island_display_group, "IslandDisplayMarker"),
        None => 0,
    };

    let display_object = logic::display_object(world, this);
    display::move_to_display_group_at_index(world, display_object, island_display_group, marker_index);
}

/// `-[WaterWingsLogic enterIsland:]` @ 0x10010fea8
///
/// Drops the wings in at x 300: onto the floor when `on_floor`, else from above the screen.
pub fn enter_island(world: &mut World, this: Entity, on_floor: bool) {
    let falling = world.obj::<WaterWingsLogic>(this).falling;
    if falling {
        return;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
        state.rot_vel.z = 0.0;
    }
    world.obj_mut::<WaterWingsLogic>(this).in_water = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
        water_wings.exit_island = false;
        water_wings.disable_collision = true;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.is_selectable = true;
        display.pos.x = ENTER_X_POS;
        display.pos.y = if on_floor { FLOOR_Y } else { ENTER_ABOVE_Y_POS };
    }

    move_to_island_layer(world, this);
    logic::set_behavior(world, this, Some("WaterWingsEnterIsland"));
}

/// `-[WaterWingsLogic exitIsland:]` @ 0x10010ffe0
///
/// Lets go of the hurricane, the wearing pygmy and the drag; then the wings die at once
/// (`now`) or fall off the island (unless already in the water).
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    let pygmy = world.obj::<WaterWingsLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::eject_from_water_wings(world, pygmy);
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        world.obj_mut::<WaterWingsLogic>(this).pygmy_logic = None;
    }

    let dragging = world.obj::<WaterWingsLogic>(this).dragging;
    if dragging {
        {
            let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
            water_wings.falling = true;
            water_wings.dragging = false;
            water_wings.apply_spring = false;
        }
        let listener = world.obj::<WaterWingsLogic>(this).water_wings_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_water_wings_drag_complete(world, listener, this);
        }
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
        water_wings.exit_island = true;
        water_wings.enable_island_floor_collision = false;
    }

    if now {
        logic::set_behavior(world, this, Some("WaterWingsDie"));
        return;
    }

    let in_water = world.obj::<WaterWingsLogic>(this).in_water;
    if in_water {
        return;
    }
    logic::set_behavior(world, this, Some("WaterWingsFallIgnoreIsland"));
}

/// `-[WaterWingsLogic exitIslandNow]` @ 0x100110180
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[WaterWingsLogic wearingWaterWingsComplete:]` @ 0x100110194
pub fn wearing_water_wings_complete(world: &mut World, this: Entity, wearing_water_wings_complete: Entity) {
    let pygmy = world.obj::<WaterWingsLogic>(this).pygmy_logic;
    if pygmy == Some(wearing_water_wings_complete) {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        world.obj_mut::<WaterWingsLogic>(this).pygmy_logic = None;
    }

    enter_island(world, this, false);
}

/// `-[WaterWingsLogic pygmyLogic]` @ 0x100110200
pub fn pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<WaterWingsLogic>(this).pygmy_logic
}

/// `-[WaterWingsLogic die]` @ 0x100110210
pub fn die(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("WaterWingsDie"));
}

/// `-[WaterWingsLogic checkPygmyCollision]` @ 0x100110228
///
/// Falling wings without a wearer strap onto the first pygmy that they drop on and that
/// accepts them.
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let water_wings = world.obj::<WaterWingsLogic>(this);
    let can_strap_on = water_wings.falling && water_wings.pygmy_logic.is_none() && !water_wings.disable_collision;
    if !can_strap_on {
        return;
    }

    let listener = water_wings.water_wings_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };

    let display_object = logic::display_object(world, this);
    for pygmy in pygmies {
        let pos = display::object(world, display_object).pos;
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;

        let within_x = (pos.x - (pygmy_pos.x + 0.0)).abs() < STRAP_REACH;
        let within_y = (pos.y - (pygmy_pos.y + STRAP_PYGMY_Y_OFFSET)).abs() < STRAP_REACH;
        if !within_x || !within_y {
            continue;
        }

        let vel_y = logic::logic(world, this).pos_vel.y;
        if !(vel_y < STRAP_MIN_FALL_SPEED) {
            continue;
        }

        let wearing = crate::pygmy::wear_water_wings(world, pygmy, this);
        if !wearing {
            continue;
        }

        strap_on_pygmy(world, this, pygmy, pygmy_display_object);
        return;
    }
}

/// The strapping of `checkPygmyCollision`: the wings stop, snap onto the pygmy facing away from
/// the screen centre, bind it on channel 1 and play `WaterWingsStrapOnPygmy`.
fn strap_on_pygmy(world: &mut World, this: Entity, pygmy: Entity, pygmy_display_object: Entity) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    world.obj_mut::<WaterWingsLogic>(this).falling = false;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let pygmy_pos = display::object(world, pygmy_display_object).pos;
    display::object_mut(world, display_object).pos = pygmy_pos;
    world.obj_mut::<WaterWingsLogic>(this).pygmy_logic = Some(pygmy);

    {
        let mut display = display::object_mut(world, display_object);
        display.scale.x = 1.0;
        if display.pos.x > SCREEN_CENTER_X {
            display.scale.x = -1.0;
        }
    }

    let animation = logic::animation(world, this);
    let wearer_display_object = logic::display_object(world, pygmy);
    pg_engine::animation::set_tweenable(world, animation, Some(wearer_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("WaterWingsStrapOnPygmy"));
}

/// `-[WaterWingsLogic waterWingsSplashComplete:::]` @ 0x100110558
///
/// The wearing pygmy has jumped into the sea: the No Pee game starts with it.
pub fn seq_water_wings_splash_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let no_pee = no_pee_logic::instance(world);
    let pygmy = world.obj::<WaterWingsLogic>(this).pygmy_logic;
    // The strap sequence only runs while a pygmy wears the wings (every path that lets go of
    // it also replaces the sequence).
    if let (Some(no_pee), Some(pygmy)) = (no_pee, pygmy) {
        no_pee_logic::start_no_pee(world, no_pee, pygmy);
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    world.obj_mut::<WaterWingsLogic>(this).pygmy_logic = None;
    enter_island(world, this, false);
    ItemResult::Done
}

/// `-[WaterWingsLogic isAccelEnabled]` @ 0x1001105ec
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<WaterWingsLogic>(this).water_wings_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }

    let display_object = logic::display_object(world, this);
    let selectable = display::object(world, display_object).is_selectable;
    if !selectable {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[WaterWingsLogic onGameFrame:]` @ 0x100110680
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);
    let previous_y = display::object(world, display_object).pos.y;

    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        let gravity_force = throwable_tilted_gravity_force(world);
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = THROWABLE_GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    let water_wings = world.obj::<WaterWingsLogic>(this);
    let tilt_state = water_wings.tilt_state;
    let slide_friction = water_wings.slide_friction;
    let spring_force = water_wings.spring_force;
    let apply_spring = water_wings.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);
    clamp_throwable_to_screen(world, display_object);

    world.obj_mut::<WaterWingsLogic>(this).island_floor_collision = false;
    let listener = world.obj::<WaterWingsLogic>(this).water_wings_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let mut y_pos = pos.y;
    let reached_floor = reaches_island_floor(previous_y, pos, island_min_x_pos, island_max_x_pos, FLOOR_Y);
    let enable_island_floor_collision = world.obj::<WaterWingsLogic>(this).enable_island_floor_collision;
    if reached_floor && enable_island_floor_collision {
        land_on_island_floor(world, this, display_object);
        y_pos = FLOOR_Y;
    } else {
        world.obj_mut::<WaterWingsLogic>(this).on_island = false;
    }

    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("WaterWingsDie"));
    }

    check_pygmy_collision(world, this);
}

/// The floor contact of `onGameFrame:`: a bounce while still fast, else the wings settle (on
/// the first contact they land: upright, at island depth, colliding with pygmies again).
fn land_on_island_floor(world: &mut World, this: Entity, display_object: Entity) {
    let on_island = world.obj::<WaterWingsLogic>(this).on_island;
    let mut pos_vel = logic::logic(world, this).pos_vel;
    bounce_throwable_velocity(&mut pos_vel, on_island);
    logic::logic_mut(world, this).pos_vel = pos_vel;

    if pos_vel.y < THROWABLE_BOUNCE_SPEED {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        world.obj_mut::<WaterWingsLogic>(this).disable_collision = false;

        let on_island = world.obj::<WaterWingsLogic>(this).on_island;
        if !on_island {
            world.obj_mut::<WaterWingsLogic>(this).on_island = true;
            {
                let mut display = display::object_mut(world, display_object);
                display.rot.z = 0.0;
                display.pos.z = ISLAND_Z_POS;
            }
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            world.obj_mut::<WaterWingsLogic>(this).island_floor_collision = true;
        }
    } else {
        logic::set_behavior(world, this, Some("WaterWingsBounce"));
    }

    display::object_mut(world, display_object).pos.y = FLOOR_Y;
}

/// `-[WaterWingsLogic refreshTiltState]` @ 0x100110bc0
///
/// Resting wings fall once the island is tipped over; they never slide.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let water_wings = world.obj::<WaterWingsLogic>(this);
    if water_wings.dragging || water_wings.falling || !water_wings.on_island {
        return;
    }

    if is_island_tipped_over(world) {
        {
            let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
            water_wings.falling = true;
            water_wings.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("WaterWingsFall"));
    }
}

/// `-[WaterWingsLogic undoTiltState]` @ 0x100110c90
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let water_wings = world.obj::<WaterWingsLogic>(this);
    if water_wings.dragging || water_wings.falling || !water_wings.on_island {
        return;
    }
    world.obj_mut::<WaterWingsLogic>(this).tilt_state = 0;
}

/// `-[WaterWingsLogic drag:::]` @ 0x100110cd0
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<WaterWingsLogic>(this).enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let display_group = display.display_group;

    let touch_screen_pos = world.obj::<WaterWingsLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // The machine code takes the touch point on the wings' own plane (it reloads `pos.z` over
    // the touch's z), so the offset's z is `pos.z − pos.z`.
    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    let offset_z = pos.z - pos.z;
    let offset = Vec4::new(offset_x, offset_y, offset_z, pos.w);

    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<WaterWingsLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[WaterWingsLogic fall:::]` @ 0x100110e88
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        let island_floor_collision = world.obj::<WaterWingsLogic>(this).island_floor_collision;
        if !island_floor_collision {
            world.obj_mut::<WaterWingsLogic>(this).on_island = false;
        }

        {
            let mut display = display::object_mut(world, display_object);
            display.scale.x = 1.0;
            display.scale.y = 1.0;
        }
        world.obj_mut::<WaterWingsLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<WaterWingsLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<WaterWingsLogic>(this).enable_island_floor_collision = false;
        }
    }

    {
        let mut display = display::object_mut(world, display_object);
        display.rot.z = display.rot.z + display.rot.z * -0.5;
    }

    let island_floor_collision = world.obj::<WaterWingsLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<WaterWingsLogic>(this).falling = false;
        display::object_mut(world, display_object).rot.z = 0.0;
        return ItemResult::Done;
    }

    let y_pos = display::object(world, display_object).pos.y;
    if y_pos < WATER_LINE_Y {
        {
            let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
            water_wings.falling = false;
            water_wings.in_water = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        world.obj_mut::<WaterWingsLogic>(this).tilt_state = 0;
        display::object_mut(world, display_object).rot.z = 0.0;
        logic::set_behavior(world, this, Some("WaterWingsSplash"));
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// `-[WaterWingsLogic fallFromHurricane:::]` @ 0x100111044
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<WaterWingsLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<WaterWingsLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let water_line_y = if pos.z < ISLAND_Z_POS { BEHIND_ISLAND_WATER_LINE_Y } else { WATER_LINE_Y };
    if pos.y < water_line_y {
        {
            let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
            water_wings.falling = false;
            water_wings.in_water = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<WaterWingsLogic>(this).tilt_state = 0;
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        {
            let mut display = display::object_mut(world, display_object);
            display.pos.y = water_line_y;
            display.rot.z = 0.0;
        }
        logic::set_behavior(world, this, Some("WaterWingsSplash"));
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// `-[WaterWingsLogic die:::]` @ 0x100111188
///
/// The wings leave the display tree and, when the island has them, come back in.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos.z = ISLAND_Z_POS;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    display::remove_from_display_group(world, display_object);

    let listener = world.obj::<WaterWingsLogic>(this).water_wings_logic_listener;
    let dragging = world.obj::<WaterWingsLogic>(this).dragging;
    if dragging {
        world.obj_mut::<WaterWingsLogic>(this).dragging = false;
        if let Some(listener) = listener {
            crate::controller::on_water_wings_drag_complete(world, listener, this);
        }
    }

    {
        let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
        water_wings.falling = false;
        water_wings.in_water = false;
        water_wings.apply_spring = false;
        water_wings.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let listener = world.obj::<WaterWingsLogic>(this).water_wings_logic_listener;
    let enabled = match listener {
        Some(listener) => crate::controller::are_water_wings_enabled(world, listener),
        None => false,
    };
    if enabled {
        enter_island(world, this, false);
    }
    ItemResult::Done
}

/// `-[WaterWingsLogic pickedUpByHurricane:]` @ 0x1001112ac
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, picked_up_by_hurricane: Entity) -> bool {
    let water_wings = world.obj::<WaterWingsLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let game_frame_running = logic::logic(world, this).game_frame_running;
    let can_be_picked_up = !water_wings.touch_began_inside
        && !already_picked_up
        && !water_wings.dragging
        && !water_wings.falling
        && !water_wings.in_water
        && !water_wings.exit_island
        && game_frame_running;
    if !can_be_picked_up {
        return false;
    }

    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = Some(picked_up_by_hurricane);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    world.obj_mut::<WaterWingsLogic>(this).tilt_state = 0;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    display::object_mut(world, display_object).is_selectable = false;

    move_to_island_layer(world, this);
    logic::set_behavior(world, this, Some("WaterWingsPickedUpByHurricane"));
    true
}

/// `-[WaterWingsLogic droppedByHurricane:]` @ 0x100111418
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<WaterWingsLogic>(this).falling = true;
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = false;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = false;
        sprite.picked_up_by_hurricane_logic = None;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    let z_pos = display::object(world, display_object).pos.z;
    if z_pos < ISLAND_Z_POS {
        move_to_behind_island_layer(world, this);
    }
    logic::set_behavior(world, this, Some("WaterWingsDroppedByHurricane"));
}

/// `-[WaterWingsLogic touchBeganInside:allTouches:]` @ 0x10011150c
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<WaterWingsLogic>(this).water_wings_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let water_wings = world.obj::<WaterWingsLogic>(this);
    let busy = water_wings.falling || water_wings.in_water || water_wings.active_touch.is_some();
    if story_active || busy {
        return;
    }

    world.obj_mut::<WaterWingsLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
        water_wings.touch_began_inside = true;
        water_wings.touch_screen_pos = touch_pos;
        water_wings.dragging = true;
        water_wings.apply_spring = true;
        // Original quirk: cleared again straight away.
        water_wings.touch_began_inside = false;
        water_wings.tilt_state = 0;
    }

    if let Some(listener) = listener {
        crate::controller::on_water_wings_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("WaterWingsDrag"));
}

/// `-[WaterWingsLogic touchBeganOutside:allTouches:]` @ 0x100111644
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WaterWingsLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
    water_wings.touch_screen_pos = touch_pos;
    water_wings.touch_began_inside = false;
}

/// `-[WaterWingsLogic touchMovedEntered:allTouches:]` @ 0x1001116a0
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WaterWingsLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<WaterWingsLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[WaterWingsLogic touchMovedExited:allTouches:]` @ 0x1001116f8
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WaterWingsLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<WaterWingsLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[WaterWingsLogic touchMovedInside:allTouches:]` @ 0x100111750
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[WaterWingsLogic touchMovedOutside:allTouches:]` @ 0x100111760
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WaterWingsLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<WaterWingsLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[WaterWingsLogic touchEndedInside:allTouches:]` @ 0x1001117b8
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WaterWingsLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<WaterWingsLogic>(this).dragging;
    if dragging {
        {
            let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
            water_wings.falling = true;
            water_wings.dragging = false;
            water_wings.apply_spring = false;
        }

        let listener = world.obj::<WaterWingsLogic>(this).water_wings_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_water_wings_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("WaterWingsFall"));
    }

    let mut water_wings = world.obj_mut::<WaterWingsLogic>(this);
    water_wings.touch_began_inside = false;
    water_wings.active_touch = None;
}

/// `-[WaterWingsLogic touchEndedOutside:allTouches:]` @ 0x10011186c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
