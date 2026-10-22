//! `WoodLogic` (superclass `SpriteLogic`).
//!
//! A throwable on the pattern of `CoconutLogic` (`docs/re/logic-catalog.md` §2.2), built from
//! the shared `throwable_*` steps of `coconut_logic`.

use crate::prelude::*;

use crate::logic::island_core::bon_fire_logic;
use crate::logic::island_core::coconut_logic::{
    ISLAND_Z_POS, THROWABLE_BOUNCE_SPEED, THROWABLE_GRAVITY_FORCE, apply_throwable_slide_friction,
    bounce_throwable_velocity, clamp_throwable_to_screen, integrate_throwable, is_island_tipped_over,
    is_throwable_out_of_range, reaches_island_floor, throwable_drag_offset, throwable_drag_z_rot,
    throwable_hurricane_water_line_y, throwable_pos_force, throwable_spring_force, throwable_tilt_state,
    throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::length3;
use pg_engine::touch::BcTouch;

/// `WoodLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct WoodLogic {
    /// `mWoodLogicListener`: `@"<WoodLogicListener>"`
    pub wood_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mStartXPos`: `f`
    pub start_x_pos: f32,

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

    /// `mCarryingPygmyLogic`: `@"PygmyLogic"`
    pub carrying_pygmy_logic: Option<Entity>,

    /// `mBonFireLogic`: `@"BonFireLogic"`
    pub bon_fire_logic: Option<Entity>,

    /// `mbExitIsland`: `B`
    pub exit_island: bool,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbInPile`: `B`
    pub in_pile: bool,

    /// `mbOnFire`: `B`
    pub on_fire: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbBeingCarried`: `B`
    pub being_carried: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const WOOD_LOGIC: Class = Class {
    name: "WoodLogic",
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

/// `mSlideFriction` (`0x3d0f5c29`).
const SLIDE_FRICTION: f32 = 0.035;

/// The island floor (`setYPos:` in `onGameFrame:`).
const FLOOR_Y: f32 = 80.0;

/// `slide:::` and `fall:::` splash below this y.
const WATER_LINE_Y: f32 = 50.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `enterIsland:` drops the log in from this height.
const ENTER_ISLAND_Y_POS: f32 = 400.0;

/// Sequence-item handlers implemented by `WoodLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "slide" => seq_slide(world, this, call),
        "drag" => seq_drag(world, this, call),
        "throwOnPile" => seq_throw_on_pile(world, this, call),
        "fall" => seq_fall(world, this, call),
        "fallFromHurricane" => seq_fall_from_hurricane(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[WoodLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&WOOD_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[WoodLogic initWithDisplayObject:]` @ 0x10017a2cc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(WoodLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.tilt_state = 0;
        wood.slide_friction = SLIDE_FRICTION;
    }
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(String::from("Wood"));
    world.obj_mut::<WoodLogic>(this).exit_island = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<WoodLogic>(this).enable_island_floor_collision = true;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[WoodLogic setWoodLogicListener:]` @ 0x10017a3a0
pub fn set_wood_logic_listener(world: &mut World, this: Entity, wood_logic_listener: Entity) {
    world.obj_mut::<WoodLogic>(this).wood_logic_listener = Some(wood_logic_listener);
}

/// `-[WoodLogic isAvailable]` @ 0x10017a460
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
    let wood_enabled = match listener {
        Some(listener) => crate::controller::is_wood_enabled(world, listener),
        None => false,
    };

    let wood = world.obj::<WoodLogic>(this);
    wood_enabled && !wood.in_pile && wood.on_island && !wood.being_carried && wood.active
}

/// `-[WoodLogic dropWhileCarrying]` @ 0x10017a4e8
pub fn drop_while_carrying(world: &mut World, this: Entity) {
    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.carrying_pygmy_logic = None;
        wood.being_carried = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    logic::logic_mut(world, this).pos_vel.y = 400.0;
    logic::set_behavior(world, this, Some("WoodDropWhileCarrying"));
}

/// `-[WoodLogic startCarrying:]` @ 0x10017a540
pub fn start_carrying(world: &mut World, this: Entity, carrying: Entity) {
    world.obj_mut::<WoodLogic>(this).being_carried = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<WoodLogic>(this).carrying_pygmy_logic = Some(carrying);
}

/// `-[WoodLogic pickUpForCarrying]` @ 0x10017a570
pub fn pick_up_for_carrying(_world: &mut World, _this: Entity) {}

/// `-[WoodLogic throwOnPile]` @ 0x10017a574
pub fn throw_on_pile(world: &mut World, this: Entity) {
    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.being_carried = false;
        wood.carrying_pygmy_logic = None;
    }

    let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
    let bon_fire = match listener {
        Some(listener) => crate::controller::bon_fire_logic(world, listener, this),
        None => None,
    };
    world.obj_mut::<WoodLogic>(this).bon_fire_logic = bon_fire;

    logic::logic_mut(world, this).pos_vel.y = 100.0;
    logic::set_behavior(world, this, Some("WoodThrowOnPile"));
}

/// `-[WoodLogic fallOutOfBonFire]` @ 0x10017a604
pub fn fall_out_of_bon_fire(world: &mut World, this: Entity) {
    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.bon_fire_logic = None;
        wood.enable_island_floor_collision = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<WoodLogic>(this).falling = true;

    // (rand % 6001 - 3000) / 10: -300.0 … 300.0 in steps of 0.1.
    let roll = world.resource_mut::<Random>().arc4random_mod(6001);
    let offset = roll.wrapping_sub(3000) as i32;
    let vel_x = offset as f32 / 10.0;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = vel_x;
        state.pos_vel.y = 500.0;
    }
    logic::set_behavior(world, this, Some("WoodFallOutOfBonFire"));
}

/// `-[WoodLogic moveToIslandLayer]` @ 0x10017a6bc
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[WoodLogic moveToDragLayer]` @ 0x10017a724
pub fn move_to_drag_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[WoodLogic moveToBehindIslandLayer]` @ 0x10017a78c
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "IslandDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[WoodLogic reset]` @ 0x10017a7f4
pub fn reset(world: &mut World, this: Entity) {
    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    let dragging = world.obj::<WoodLogic>(this).dragging;
    if dragging {
        {
            let mut wood = world.obj_mut::<WoodLogic>(this);
            wood.falling = true;
            wood.dragging = false;
            wood.apply_spring = false;
        }
        let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_wood_drag_complete(world, listener, this);
        }
    }

    let being_carried = world.obj::<WoodLogic>(this).being_carried;
    if being_carried {
        let carrying_pygmy = world.obj::<WoodLogic>(this).carrying_pygmy_logic;
        if let Some(carrying_pygmy) = carrying_pygmy {
            crate::pygmy::stop_carrying_from_wood(world, carrying_pygmy);
        }
        world.obj_mut::<WoodLogic>(this).carrying_pygmy_logic = None;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos.z = ISLAND_Z_POS;
    move_to_island_layer(world, this);

    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.falling = false;
        wood.in_water = false;
        wood.in_pile = false;
        wood.on_fire = false;
        wood.being_carried = false;
        wood.apply_spring = false;
        wood.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.carrying_pygmy_logic = None;
        wood.bon_fire_logic = None;
    }
}

/// `-[WoodLogic enterIsland:]` @ 0x10017a998
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let falling = world.obj::<WoodLogic>(this).falling;
    if !falling {
        let display_object = logic::display_object(world, this);

        let start_x_pos = world.obj::<WoodLogic>(this).start_x_pos;
        if start_x_pos == 0.0 {
            let x_pos = display::object(world, display_object).pos.x;
            world.obj_mut::<WoodLogic>(this).start_x_pos = x_pos;
        }

        reset(world, this);
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
            state.rot_vel.z = 0.0;
        }
        {
            let mut wood = world.obj_mut::<WoodLogic>(this);
            wood.active = true;
            wood.in_water = false;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<WoodLogic>(this).exit_island = false;

        let start_x_pos = world.obj::<WoodLogic>(this).start_x_pos;
        {
            let mut display = display::object_mut(world, display_object);
            display.is_selectable = true;
            display.pos.x = start_x_pos;
            display.pos.y = ENTER_ISLAND_Y_POS;
        }
        move_to_island_layer(world, this);
        logic::set_behavior(world, this, Some("WoodEnterIsland"));
    }

    sprite_logic::show_apocalypse_cloud(world, this);
}

/// `-[WoodLogic exitIsland:]` @ 0x10017ab10
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let active = world.obj::<WoodLogic>(this).active;
    if !active {
        return;
    }

    sprite_logic::hide_apocalypse_cloud(world, this);
    world.obj_mut::<WoodLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;

    // A floating log only leaves at once.
    let in_water = world.obj::<WoodLogic>(this).in_water;
    if in_water && !now {
        return;
    }

    reset(world, this);
    world.obj_mut::<WoodLogic>(this).exit_island = true;
    let behavior = if now { "WoodDie" } else { "WoodFallIgnoreIsland" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[WoodLogic exitIslandNow]` @ 0x10017abf8
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[WoodLogic isAccelEnabled]` @ 0x10017ac0c
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[WoodLogic onGameFrame:]` @ 0x10017ac7c
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

    let wood = world.obj::<WoodLogic>(this);
    let tilt_state = wood.tilt_state;
    let slide_friction = wood.slide_friction;
    let spring_force = wood.spring_force;
    let apply_spring = wood.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);
    clamp_throwable_to_screen(world, display_object);

    world.obj_mut::<WoodLogic>(this).island_floor_collision = false;
    let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let reached_floor = reaches_island_floor(previous_y, pos, island_min_x_pos, island_max_x_pos, FLOOR_Y);
    let enable_island_floor_collision = world.obj::<WoodLogic>(this).enable_island_floor_collision;
    if reached_floor && enable_island_floor_collision {
        let on_island = world.obj::<WoodLogic>(this).on_island;
        let mut pos_vel = logic::logic(world, this).pos_vel;
        bounce_throwable_velocity(&mut pos_vel, on_island);
        logic::logic_mut(world, this).pos_vel = pos_vel;

        if pos_vel.y < THROWABLE_BOUNCE_SPEED {
            logic::logic_mut(world, this).pos_vel.y = 0.0;
            if !on_island {
                {
                    let mut wood = world.obj_mut::<WoodLogic>(this);
                    wood.falling = false;
                    wood.on_island = true;
                }
                logic::logic_mut(world, this).pos_vel.x = 0.0;
                world.obj_mut::<WoodLogic>(this).island_floor_collision = true;
            }
            display::object_mut(world, display_object).rot.z = 0.0;
        } else {
            logic::set_behavior(world, this, Some("WoodBounce"));
        }

        display::object_mut(world, display_object).pos.y = FLOOR_Y;
    } else {
        world.obj_mut::<WoodLogic>(this).on_island = false;
    }

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("WoodDie"));
    }
}

/// `-[WoodLogic refreshTiltState]` @ 0x10017b1d0
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let wood = world.obj::<WoodLogic>(this);
    if wood.dragging || wood.falling || !wood.on_island {
        return;
    }

    let tilt_state = throwable_tilt_state(world);
    let current_tilt_state = world.obj::<WoodLogic>(this).tilt_state;
    if tilt_state != current_tilt_state {
        if tilt_state == 0 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("WoodStopSlide"));
            world.obj_mut::<WoodLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("WoodTiltSlide"));
            world.obj_mut::<WoodLogic>(this).tilt_state = 1;
        }
    }

    if is_island_tipped_over(world) {
        {
            let mut wood = world.obj_mut::<WoodLogic>(this);
            wood.falling = true;
            wood.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("WoodFall"));
    }
}

/// `-[WoodLogic undoTiltState]` @ 0x10017b3b0
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let wood = world.obj::<WoodLogic>(this);
    if wood.dragging || wood.falling || !wood.on_island {
        return;
    }

    if wood.tilt_state == 1 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        logic::set_behavior(world, this, Some("WoodStopSlide"));
    }
    world.obj_mut::<WoodLogic>(this).tilt_state = 0;
}

/// The splash of `slide:::`, `fall:::` and `fallFromHurricane:::`: the log stops in the water
/// and plays `WoodSplash`.
fn splash(world: &mut World, this: Entity) {
    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.falling = false;
        wood.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<WoodLogic>(this).tilt_state = 0;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("WoodSplash"));
}

/// `-[WoodLogic slide:::]` @ 0x10017b444
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<WoodLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let display_object = logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if y_pos < WATER_LINE_Y {
        splash(world, this);
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// `-[WoodLogic drag:::]` @ 0x10017b544
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<WoodLogic>(this).enable_island_floor_collision = false;
        move_to_drag_layer(world, this);
    }

    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<WoodLogic>(this).touch_screen_pos;
    let offset = throwable_drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<WoodLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[WoodLogic throwOnPile:::]` @ 0x10017b708
pub fn seq_throw_on_pile(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut wood = world.obj_mut::<WoodLogic>(this);
            if !wood.island_floor_collision {
                wood.on_island = false;
            }
            wood.enable_island_floor_collision = false;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<WoodLogic>(this).falling = true;
    }

    let bon_fire = world.obj::<WoodLogic>(this).bon_fire_logic;
    let Some(bon_fire) = bon_fire else {
        // PORT: the original reads the position of a nil bonfire (NULL) and crashes; the log is
        // always given its bonfire by `throwOnPile` before this item runs.
        return ItemResult::Pending;
    };

    let display_object = logic::display_object(world, this);
    let bon_fire_display_object = logic::display_object(world, bon_fire);
    let bon_fire_pos = display::object(world, bon_fire_display_object).pos;
    let next_wood_x_offset = bon_fire_logic::next_wood_x_offset(world, bon_fire);
    let next_wood_y_offset = bon_fire_logic::next_wood_y_offset(world, bon_fire);

    // Halfway to the next log slot each frame.
    let pos = display::object(world, display_object).pos;
    let slot_x_pos = next_wood_x_offset + bon_fire_pos.x;
    let x_pos = pos.x + (slot_x_pos - pos.x) * 0.5;
    display::object_mut(world, display_object).pos.x = x_pos;

    let vel_y = logic::logic(world, this).pos_vel.y;
    let slot_y_pos = next_wood_y_offset + bon_fire_pos.y;
    let landed_in_slot = vel_y < 0.0 && pos.y < slot_y_pos;
    if !landed_in_slot {
        return ItemResult::Pending;
    }

    world.obj_mut::<WoodLogic>(this).falling = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    bon_fire_logic::add_wood_logic(world, bon_fire, this);
    ItemResult::Done
}

/// `-[WoodLogic fall:::]` @ 0x10017b898
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut wood = world.obj_mut::<WoodLogic>(this);
            if !wood.island_floor_collision {
                wood.on_island = false;
            }
            wood.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<WoodLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<WoodLogic>(this).enable_island_floor_collision = false;
        }
    }

    let display_object = logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let island_floor_collision = world.obj::<WoodLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<WoodLogic>(this).falling = false;
        return ItemResult::Done;
    }

    if y_pos < WATER_LINE_Y {
        splash(world, this);
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// `-[WoodLogic fallFromHurricane:::]` @ 0x10017ba08
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<WoodLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<WoodLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let water_line_y = throwable_hurricane_water_line_y(pos.z);
    if pos.y < water_line_y {
        // The original sets y between the velocity and the tilt state; nothing reads it there.
        display::object_mut(world, display_object).pos.y = water_line_y;
        splash(world, this);
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// `-[WoodLogic die:::]` @ 0x10017bb4c
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    reset(world, this);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
    let wood_enabled = match listener {
        Some(listener) => crate::controller::is_wood_enabled(world, listener),
        None => false,
    };
    if wood_enabled {
        enter_island(world, this, false);
    }
    ItemResult::Done
}

/// `-[WoodLogic pickedUpByHurricane:]` @ 0x10017bbd4
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, picked_up_by_hurricane: Entity) -> bool {
    let wood = world.obj::<WoodLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let game_frame_running = logic::logic(world, this).game_frame_running;
    let can_be_picked_up = !wood.touch_began_inside
        && !already_picked_up
        && !wood.dragging
        && !wood.falling
        && !wood.being_carried
        && wood.bon_fire_logic.is_none()
        && !wood.in_water
        && wood.active
        && !wood.exit_island
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
    world.obj_mut::<WoodLogic>(this).tilt_state = 0;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.rot.z = 0.0;
        display.is_selectable = false;
    }
    move_to_island_layer(world, this);
    logic::set_behavior(world, this, Some("WoodPickedUpByHurricane"));
    true
}

/// `-[WoodLogic droppedByHurricane:]` @ 0x10017bd74
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<WoodLogic>(this).falling = true;
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
    logic::set_behavior(world, this, Some("WoodDroppedByHurricane"));
}

/// `-[WoodLogic touchBeganInside:allTouches:]` @ 0x10017be68
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let wood = world.obj::<WoodLogic>(this);
    let busy = wood.falling || wood.in_water || wood.being_carried || wood.active_touch.is_some();
    if story_active || busy {
        return;
    }

    world.obj_mut::<WoodLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.touch_began_inside = true;
        wood.touch_screen_pos = touch_pos;
    }

    let bon_fire = world.obj::<WoodLogic>(this).bon_fire_logic;
    if let Some(bon_fire) = bon_fire {
        bon_fire_logic::remove_wood_logic(world, bon_fire, Some(this));
        world.obj_mut::<WoodLogic>(this).bon_fire_logic = None;
    }

    {
        let mut wood = world.obj_mut::<WoodLogic>(this);
        wood.dragging = true;
        wood.apply_spring = true;
        // Original quirk: cleared again straight away.
        wood.touch_began_inside = false;
        wood.tilt_state = 0;
    }

    if let Some(listener) = listener {
        crate::controller::on_wood_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("WoodDrag"));
}

/// `-[WoodLogic touchBeganOutside:allTouches:]` @ 0x10017bfd8
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WoodLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut wood = world.obj_mut::<WoodLogic>(this);
    wood.touch_screen_pos = touch_pos;
    wood.touch_began_inside = false;
}

/// `-[WoodLogic touchMovedEntered:allTouches:]` @ 0x10017c034
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WoodLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<WoodLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[WoodLogic touchMovedExited:allTouches:]` @ 0x10017c08c
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WoodLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<WoodLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[WoodLogic touchMovedInside:allTouches:]` @ 0x10017c0e4
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[WoodLogic touchMovedOutside:allTouches:]` @ 0x10017c0f4
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WoodLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<WoodLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[WoodLogic touchEndedInside:allTouches:]` @ 0x10017c14c
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<WoodLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<WoodLogic>(this).dragging;
    if dragging {
        {
            let mut wood = world.obj_mut::<WoodLogic>(this);
            wood.falling = true;
            wood.dragging = false;
            wood.apply_spring = false;
        }
        move_to_island_layer(world, this);

        let listener = world.obj::<WoodLogic>(this).wood_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_wood_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("WoodFall"));
    }

    let mut wood = world.obj_mut::<WoodLogic>(this);
    wood.touch_began_inside = false;
    wood.active_touch = None;
}

/// `-[WoodLogic touchEndedOutside:allTouches:]` @ 0x10017c214
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
