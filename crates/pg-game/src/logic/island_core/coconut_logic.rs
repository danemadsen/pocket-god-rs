//! `CoconutLogic` (superclass `SpriteLogic`).
//!
//! `CoconutLogic` is the reference implementation of the throwable-object pattern
//! (`docs/re/logic-catalog.md` §2.2: gravity, tilt, island floor, spring drag). The
//! `throwable_*` items at the end of this module are the steps of that pattern, exact to the
//! original's f32 operation order, for every throwable to share with its own constants.

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::{length3, normalize3};
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `CoconutLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct CoconutLogic {
    /// `mCoconutLogicListener`: `@"<CoconutLogicListener>"`
    pub coconut_logic_listener: Option<Entity>,

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

    /// `mEatingPygmyLogic`: `@"PygmyLogic"`
    pub eating_pygmy_logic: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbDontHitUntilBounce`: `B`
    pub dont_hit_until_bounce: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbOnTree`: `B`
    pub on_tree: bool,

    /// `mbBeingEaten`: `B`
    pub being_eaten: bool,

    /// `mbSplit`: `B`
    pub split: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const COCONUT_LOGIC: Class = Class {
    name: "CoconutLogic",
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

/// The island floor (`setYPos:` in `onGameFrame:`).
const FLOOR_Y: f32 = 92.0;

/// `slide:::` and `fall:::` splash below this y.
const WATER_LINE_Y: f32 = 50.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkPygmyCollision`: the hit radius (20, squared) around a point 31 above the pygmy.
const PYGMY_HIT_DISTANCE_SQUARED: f32 = 400.0;
const PYGMY_HIT_Y_OFFSET: f32 = 31.0;

/// Sequence-item handlers implemented by `CoconutLogic` (`respondsToSelector:` order).
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

/// `[[CoconutLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&COCONUT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[CoconutLogic initWithDisplayObject:]` @ 0x100188444
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(CoconutLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut coconut = world.obj_mut::<CoconutLogic>(this);
        coconut.on_tree = true;
        coconut.tilt_state = 0;
        coconut.slide_friction = SLIDE_FRICTION;
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[CoconutLogic setCoconutLogicListener:]` @ 0x1001884e4
pub fn set_coconut_logic_listener(world: &mut World, this: Entity, coconut_logic_listener: Entity) {
    world.obj_mut::<CoconutLogic>(this).coconut_logic_listener = Some(coconut_logic_listener);
}

/// `-[CoconutLogic isEdible]` @ 0x1001885a4
pub fn is_edible(world: &mut World, this: Entity) -> bool {
    let coconut = world.obj::<CoconutLogic>(this);
    coconut.split && coconut.on_island && !coconut.being_eaten && coconut.active
}

/// `-[CoconutLogic isInAir]` @ 0x1001885f4
pub fn is_in_air(world: &mut World, this: Entity) -> bool {
    world.obj::<CoconutLogic>(this).falling
}

/// `-[CoconutLogic thrownByCharlie:]` @ 0x100188604
pub fn thrown_by_charlie(world: &mut World, this: Entity, thrown_by_charlie: Entity) {
    {
        let mut coconut = world.obj_mut::<CoconutLogic>(this);
        coconut.eating_pygmy_logic = None;
        coconut.dont_hit_until_bounce = true;
        coconut.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let charlie_display_object = logic::display_object(world, thrown_by_charlie);
    let charlie_scale = display::object(world, charlie_display_object).scale;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = charlie_scale.x * -200.0;
        state.pos_vel.y = 400.0;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("CoconutDropWhileEating"));
}

/// `-[CoconutLogic dropWhileEating]` @ 0x1001886e4
pub fn drop_while_eating(world: &mut World, this: Entity) {
    {
        let mut coconut = world.obj_mut::<CoconutLogic>(this);
        coconut.eating_pygmy_logic = None;
        coconut.dont_hit_until_bounce = true;
        coconut.being_eaten = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    logic::logic_mut(world, this).pos_vel.y = 400.0;
    logic::set_behavior(world, this, Some("CoconutDropWhileEating"));
}

/// `-[CoconutLogic startEating:]` @ 0x10018874c
pub fn start_eating(world: &mut World, this: Entity, eating: Entity) {
    world.obj_mut::<CoconutLogic>(this).being_eaten = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<CoconutLogic>(this).eating_pygmy_logic = Some(eating);
}

/// `-[CoconutLogic pickUpForEating]` @ 0x10018877c
pub fn pick_up_for_eating(_world: &mut World, _this: Entity) {}

/// `-[CoconutLogic finishEating]` @ 0x100188780
pub fn finish_eating(world: &mut World, this: Entity) {
    world.obj_mut::<CoconutLogic>(this).eating_pygmy_logic = None;
    logic::set_behavior(world, this, Some("CoconutDie"));
}

/// `-[CoconutLogic isOnTree]` @ 0x1001887a4
pub fn is_on_tree(world: &mut World, this: Entity) -> bool {
    world.obj::<CoconutLogic>(this).on_tree
}

/// `-[CoconutLogic moveToIslandLayer]` @ 0x1001887b4
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[CoconutLogic moveToDragLayer]` @ 0x10018881c
pub fn move_to_drag_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[CoconutLogic moveToTreeLayer]` @ 0x100188884
pub fn move_to_tree_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "StartCoconutDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[CoconutLogic moveToBehindIslandLayer]` @ 0x1001888ec
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "IslandDisplayMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[CoconutLogic reset]` @ 0x100188954
pub fn reset(world: &mut World, this: Entity) {
    let dragging = world.obj::<CoconutLogic>(this).dragging;
    if dragging {
        world.obj_mut::<CoconutLogic>(this).dragging = false;
        let listener = world.obj::<CoconutLogic>(this).coconut_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_coconut_drag_complete(world, listener, this);
        }
    }

    let being_eaten = world.obj::<CoconutLogic>(this).being_eaten;
    if being_eaten {
        let eating_pygmy = world.obj::<CoconutLogic>(this).eating_pygmy_logic;
        if let Some(eating_pygmy) = eating_pygmy {
            crate::pygmy::stop_eating(world, eating_pygmy);
        }
        world.obj_mut::<CoconutLogic>(this).eating_pygmy_logic = None;
    }

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos.z = ISLAND_Z_POS;
    move_to_tree_layer(world, this);

    {
        let mut coconut = world.obj_mut::<CoconutLogic>(this);
        coconut.on_tree = true;
        coconut.split = false;
        coconut.falling = false;
        coconut.in_water = false;
        coconut.being_eaten = false;
        coconut.apply_spring = false;
        coconut.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<CoconutLogic>(this).eating_pygmy_logic = None;
}

/// `-[CoconutLogic enterIsland]` @ 0x100188ad4
pub fn enter_island(world: &mut World, this: Entity) {
    reset(world, this);
    world.obj_mut::<CoconutLogic>(this).active = true;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("CoconutEnterIsland"));
}

/// `-[CoconutLogic exitIsland]` @ 0x100188b4c
pub fn exit_island(world: &mut World, this: Entity) {
    reset(world, this);
    world.obj_mut::<CoconutLogic>(this).active = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("CoconutExitIsland"));
}

/// `-[CoconutLogic grow]` @ 0x100188bc0
pub fn grow(world: &mut World, this: Entity) {
    let active = world.obj::<CoconutLogic>(this).active;
    if !active {
        return;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("CoconutGrow"));
}

/// `-[CoconutLogic fallOffTree]` @ 0x100188c34
pub fn fall_off_tree(world: &mut World, this: Entity) {
    let active = world.obj::<CoconutLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<CoconutLogic>(this).falling = true;
    move_to_island_layer(world, this);
    world.obj_mut::<CoconutLogic>(this).tilt_state = 0;
    logic::set_behavior(world, this, Some("CoconutFall"));
}

/// `-[CoconutLogic isSplit]` @ 0x100188cb4
pub fn is_split(world: &mut World, this: Entity) -> bool {
    world.obj::<CoconutLogic>(this).split
}

/// `-[CoconutLogic split:]` @ 0x100188cc4
pub fn split(world: &mut World, this: Entity, split: bool) {
    let texture = pg_engine::library::texture_def_id(world, "CoconutHalf");
    let display_object = logic::display_object(world, this);
    display::set_texture_def(world, display_object, texture);
    logic::set_behavior(world, this, Some("CoconutSplit"));

    {
        let mut coconut = world.obj_mut::<CoconutLogic>(this);
        coconut.on_tree = false;
        coconut.split = true;
        coconut.active = true;
    }

    if split {
        let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(display_object));
        }

        // Pushed 100 against its motion.
        let vel_x = logic::logic(world, this).pos_vel.x;
        let push = if vel_x < 0.0 { 100.0 } else { -100.0 };
        logic::logic_mut(world, this).pos_vel.x = vel_x + push;
    }
}

/// `-[CoconutLogic checkPygmyCollision]` @ 0x100188e00
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let coconut = world.obj::<CoconutLogic>(this);
    if coconut.being_eaten || coconut.dont_hit_until_bounce {
        return;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let x_pos = pos.x;
    let y_pos = pos.y;

    let listener = world.obj::<CoconutLogic>(this).coconut_logic_listener;
    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmy_logic_array {
        let vel_y = logic::logic(world, this).pos_vel.y;
        let moving_down = vel_y < 0.0;
        if !moving_down {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let delta_x = x_pos - (pygmy_pos.x + 0.0);
        let delta_y = y_pos - (pygmy_pos.y + PYGMY_HIT_Y_OFFSET);
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        let within_reach = distance_squared < PYGMY_HIT_DISTANCE_SQUARED;
        if !within_reach {
            continue;
        }

        let hit = crate::pygmy::hit_by_coconut(world, pygmy, this);
        if !hit {
            continue;
        }

        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.y = 700.0;
            state.pos_vel.x = delta_x * 10.0;
        }

        let dragging = world.obj::<CoconutLogic>(this).dragging;
        if dragging {
            {
                let mut coconut = world.obj_mut::<CoconutLogic>(this);
                coconut.falling = true;
                coconut.dragging = false;
                coconut.active_touch = None;
                coconut.apply_spring = false;
                coconut.touch_began_inside = false;
            }
            if let Some(listener) = listener {
                crate::controller::on_coconut_drag_complete(world, listener, this);
            }
            logic::set_behavior(world, this, Some("CoconutSplit"));
        }

        if let Some(listener) = listener {
            crate::controller::on_coconut_hit_pygmy(world, listener, this);
        }
        return;
    }
}

/// `-[CoconutLogic isAccelEnabled]` @ 0x100189110
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<CoconutLogic>(this).coconut_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[CoconutLogic onGameFrame:]` @ 0x100189180
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    let active = world.obj::<CoconutLogic>(this).active;
    if shutdown || !active {
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

    let coconut = world.obj::<CoconutLogic>(this);
    let tilt_state = coconut.tilt_state;
    let slide_friction = coconut.slide_friction;
    let spring_force = coconut.spring_force;
    let apply_spring = coconut.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);
    clamp_throwable_to_screen(world, display_object);

    world.obj_mut::<CoconutLogic>(this).island_floor_collision = false;
    let listener = world.obj::<CoconutLogic>(this).coconut_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let reached_floor = reaches_island_floor(previous_y, pos, island_min_x_pos, island_max_x_pos, FLOOR_Y);
    let enable_island_floor_collision = world.obj::<CoconutLogic>(this).enable_island_floor_collision;
    if reached_floor && enable_island_floor_collision {
        let on_island = world.obj::<CoconutLogic>(this).on_island;
        let mut pos_vel = logic::logic(world, this).pos_vel;
        bounce_throwable_velocity(&mut pos_vel, on_island);
        logic::logic_mut(world, this).pos_vel = pos_vel;

        if pos_vel.y < THROWABLE_BOUNCE_SPEED {
            logic::logic_mut(world, this).pos_vel.y = 0.0;
            if !on_island {
                {
                    let mut coconut = world.obj_mut::<CoconutLogic>(this);
                    coconut.falling = false;
                    coconut.on_island = true;
                }
                logic::logic_mut(world, this).pos_vel.x = 0.0;
                world.obj_mut::<CoconutLogic>(this).island_floor_collision = true;
                if let Some(listener) = listener {
                    crate::controller::on_coconut_hit_ground(world, listener, this);
                }
            }
            display::object_mut(world, display_object).rot.z = 0.0;
        } else {
            logic::set_behavior(world, this, Some("CoconutBounce"));
        }

        world.obj_mut::<CoconutLogic>(this).dont_hit_until_bounce = false;
        display::object_mut(world, display_object).pos.y = FLOOR_Y;
    } else {
        world.obj_mut::<CoconutLogic>(this).on_island = false;
    }

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("CoconutDie"));
    }

    check_pygmy_collision(world, this);
}

/// `-[CoconutLogic refreshTiltState]` @ 0x10018971c
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let coconut = world.obj::<CoconutLogic>(this);
    if coconut.dragging || coconut.falling || !coconut.on_island {
        return;
    }

    let tilt_state = throwable_tilt_state(world);
    let current_tilt_state = world.obj::<CoconutLogic>(this).tilt_state;
    if tilt_state != current_tilt_state {
        if tilt_state == 0 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("CoconutStopSlide"));
            world.obj_mut::<CoconutLogic>(this).tilt_state = 0;
        } else if tilt_state == 1 {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            logic::set_behavior(world, this, Some("CoconutTiltSlide"));
            world.obj_mut::<CoconutLogic>(this).tilt_state = 1;
        }
    }

    if is_island_tipped_over(world) {
        {
            let mut coconut = world.obj_mut::<CoconutLogic>(this);
            coconut.falling = true;
            coconut.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("CoconutFall"));
    }
}

/// `-[CoconutLogic undoTiltState]` @ 0x1001898fc
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let coconut = world.obj::<CoconutLogic>(this);
    if coconut.dragging || coconut.falling || !coconut.on_island {
        return;
    }

    if coconut.tilt_state == 1 {
        logic::logic_mut(world, this).pos_vel.x = 0.0;
        logic::set_behavior(world, this, Some("CoconutStopSlide"));
    }
    world.obj_mut::<CoconutLogic>(this).tilt_state = 0;
}

/// The splash of `slide:::`, `fall:::` and `fallFromHurricane:::`: the coconut stops in the
/// water and plays `CoconutSplash`.
fn splash(world: &mut World, this: Entity) {
    {
        let mut coconut = world.obj_mut::<CoconutLogic>(this);
        coconut.falling = false;
        coconut.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<CoconutLogic>(this).tilt_state = 0;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("CoconutSplash"));
}

/// `-[CoconutLogic slide:::]` @ 0x100189990
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<CoconutLogic>(this).enable_island_floor_collision = true;
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

/// `-[CoconutLogic drag:::]` @ 0x100189a90
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<CoconutLogic>(this).enable_island_floor_collision = false;
        move_to_drag_layer(world, this);
    }

    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<CoconutLogic>(this).touch_screen_pos;
    let offset = throwable_drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<CoconutLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[CoconutLogic fall:::]` @ 0x100189c54
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut coconut = world.obj_mut::<CoconutLogic>(this);
            if !coconut.island_floor_collision {
                coconut.on_island = false;
            }
            coconut.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<CoconutLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    let island_floor_collision = world.obj::<CoconutLogic>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<CoconutLogic>(this).falling = false;
        return ItemResult::Done;
    }

    if y_pos < WATER_LINE_Y {
        splash(world, this);
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// `-[CoconutLogic fallFromHurricane:::]` @ 0x100189da0
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<CoconutLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<CoconutLogic>(this).falling = true;
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

/// `-[CoconutLogic die:::]` @ 0x100189ee4
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    reset(world, this);

    let texture = pg_engine::library::texture_def_id(world, "Coconut_06");
    let display_object = logic::display_object(world, this);
    display::set_texture_def(world, display_object, texture);

    let listener = world.obj::<CoconutLogic>(this).coconut_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_coconut_die(world, listener, this);
    }
    ItemResult::Done
}

/// `-[CoconutLogic pickedUpByHurricane:]` @ 0x100189f88
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, picked_up_by_hurricane: Entity) -> bool {
    let coconut = world.obj::<CoconutLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let game_frame_running = logic::logic(world, this).game_frame_running;
    let can_be_picked_up = !coconut.touch_began_inside
        && !already_picked_up
        && !coconut.dragging
        && !coconut.falling
        && !coconut.being_eaten
        && !coconut.in_water
        && coconut.active
        && game_frame_running;
    if !can_be_picked_up {
        return false;
    }

    world.obj_mut::<CoconutLogic>(this).on_tree = false;
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = Some(picked_up_by_hurricane);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    world.obj_mut::<CoconutLogic>(this).tilt_state = 0;
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
    logic::set_behavior(world, this, Some("CoconutPickedUpByHurricane"));
    true
}

/// `-[CoconutLogic droppedByHurricane:]` @ 0x10018a110
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<CoconutLogic>(this).falling = true;
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
    logic::set_behavior(world, this, Some("CoconutDroppedByHurricane"));
}

/// `-[CoconutLogic touchBeganInside:allTouches:]` @ 0x10018a204
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<CoconutLogic>(this).coconut_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let coconut = world.obj::<CoconutLogic>(this);
    let busy = coconut.falling || coconut.in_water || coconut.being_eaten || coconut.active_touch.is_some();
    if story_active || busy {
        return;
    }

    world.obj_mut::<CoconutLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut coconut = world.obj_mut::<CoconutLogic>(this);
        coconut.touch_began_inside = true;
        coconut.touch_screen_pos = touch_pos;
        coconut.on_tree = false;
        coconut.dragging = true;
        coconut.apply_spring = true;
        // Original quirk: cleared again straight away.
        coconut.touch_began_inside = false;
        coconut.tilt_state = 0;
    }

    if let Some(listener) = listener {
        crate::controller::on_coconut_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("CoconutDrag"));
}

/// `-[CoconutLogic touchBeganOutside:allTouches:]` @ 0x10018a358
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<CoconutLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut coconut = world.obj_mut::<CoconutLogic>(this);
    coconut.touch_screen_pos = touch_pos;
    coconut.touch_began_inside = false;
}

/// `-[CoconutLogic touchMovedEntered:allTouches:]` @ 0x10018a3b4
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<CoconutLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<CoconutLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[CoconutLogic touchMovedExited:allTouches:]` @ 0x10018a40c
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<CoconutLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<CoconutLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[CoconutLogic touchMovedInside:allTouches:]` @ 0x10018a464
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[CoconutLogic touchMovedOutside:allTouches:]` @ 0x10018a474
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<CoconutLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<CoconutLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[CoconutLogic touchEndedInside:allTouches:]` @ 0x10018a4cc
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<CoconutLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<CoconutLogic>(this).dragging;
    if dragging {
        {
            let mut coconut = world.obj_mut::<CoconutLogic>(this);
            coconut.falling = true;
            coconut.dragging = false;
            coconut.apply_spring = false;
        }
        move_to_island_layer(world, this);

        let listener = world.obj::<CoconutLogic>(this).coconut_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_coconut_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("CoconutFall"));
    }

    let mut coconut = world.obj_mut::<CoconutLogic>(this);
    coconut.touch_began_inside = false;
    coconut.active_touch = None;
}

/// `-[CoconutLogic touchEndedOutside:allTouches:]` @ 0x10018a594
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}

// Throwable-object physics (docs/re/logic-catalog.md §2.2) ---------------------------------
//
// The steps of `CoconutLogic::onGameFrame:`, `refreshTiltState`, `drag:::` and
// `fallFromHurricane:::` that the other throwables repeat with their own floor, friction,
// spring and behaviour ids. Each helper keeps the original's operation order.

/// The standard depth of island objects (`0xc38a8f5c`): `reset` puts throwables there, and a
/// throwable behind it is behind the island.
pub const ISLAND_Z_POS: f32 = -277.12;

/// `mGravityForce` while the accelerometer does not tilt the world: `(0, -3000, 0, 1)`.
pub const THROWABLE_GRAVITY_FORCE: Vec4 = Vec4::new(0.0, -3000.0, 0.0, 1.0);

/// A floor contact this fast upwards (after the contact's damping) bounces instead of landing.
pub const THROWABLE_BOUNCE_SPEED: f32 = 100.0;

/// `DAT_10029db08`: the splash line of `fallFromHurricane:::` in front of the island (50) and
/// behind it (86).
pub const THROWABLE_HURRICANE_WATER_LINE_Y: [f32; 2] = [50.0, 86.0];

/// The gravity force while the accelerometer tilts the world: `(-accelForce.y, accelForce.x,
/// 0, 1)` normalised by its xyz length (`FUN_1001a157c`, w included), then x, y and z times 3000.
pub fn throwable_tilted_gravity_force(world: &World) -> Vec4 {
    let accel_force = world.resource::<BcView>().accel_force;
    let tilted = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
    let normalized = normalize3(tilted);

    let gravity_x = normalized.x * 3000.0;
    let gravity_y = normalized.y * 3000.0;
    let gravity_z = normalized.z * 3000.0;
    Vec4::new(gravity_x, gravity_y, gravity_z, normalized.w)
}

/// The tilt state of `refreshTiltState` for `BCView.rawAccelRotDegrees.z`: 0 beyond ±130°
/// (resting), 1 between ±90° and ±130° (sliding), 2 within ±90° (also for NaN).
pub fn throwable_tilt_state(world: &World) -> i32 {
    let z = world.resource::<BcView>().raw_accel_rot_degrees.z;
    if z < -130.0 || z > 130.0 {
        return 0;
    }
    if z > 90.0 || z < -90.0 {
        return 1;
    }
    2
}

/// Whether the filtered roll `BCView.accelRotDegrees.z` lies in (−90, 90): the island is tipped
/// far enough for a resting throwable to fall (`refreshTiltState`) and for the bonfire to drop
/// its logs (`BonFireLogic::onGameFrame:`).
pub fn is_island_tipped_over(world: &World) -> bool {
    let z = world.resource::<BcView>().accel_rot_degrees.z;
    z > -90.0 && z < 90.0
}

/// The gravity's x for the tilt state: scaled by the slide friction while sliding (state 1),
/// zero in every other state.
pub fn apply_throwable_slide_friction(gravity_force: &mut Vec4, tilt_state: i32, slide_friction: f32) {
    if tilt_state == 1 {
        gravity_force.x = slide_friction * gravity_force.x;
    } else {
        gravity_force.x = 0.0;
    }
}

/// `mPosForce`: `(0, 0, 0, 1)`, plus the gravity force unless gravity is ignored, plus the
/// spring force while the spring applies (x, y and z).
pub fn throwable_pos_force(gravity_force: Vec4, ignore_gravity: bool, spring_force: Vec4, apply_spring: bool) -> Vec4 {
    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    if !ignore_gravity {
        pos_force.x = gravity_force.x + pos_force.x;
        pos_force.y = gravity_force.y + pos_force.y;
        pos_force.z = gravity_force.z + pos_force.z;
    }
    if apply_spring {
        pos_force.x = spring_force.x + pos_force.x;
        pos_force.y = spring_force.y + pos_force.y;
        pos_force.z = spring_force.z + pos_force.z;
    }
    pos_force
}

/// One integration step of a throwable: `mPosVel += force·dt` first, then the display object
/// moves by the new velocity (`pos += vel·dt`), x, y and z.
pub fn integrate_throwable(world: &mut World, this: Entity, pos_force: Vec4, dt: f32) {
    let display_object = logic::display_object(world, this);

    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
    state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
    state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
    let pos_vel = state.pos_vel;

    let mut display = display::object_mut(world, display_object);
    display.pos.x = display.pos.x + pos_vel.x * dt;
    display.pos.y = display.pos.y + pos_vel.y * dt;
    display.pos.z = display.pos.z + pos_vel.z * dt;
}

/// Keeps a throwable on the 480-point-wide screen. Both bounds are measured first; then
/// `minPos.x < 0` moves x to `(x − minPos.x) + 0`, and `maxPos.x > 480` moves the (possibly
/// already moved) x to `480 − (maxPos.x − x)`.
pub fn clamp_throwable_to_screen(world: &mut World, display_object: Entity) {
    let min_pos = display::min_pos(world, display_object);
    let max_pos = display::max_pos(world, display_object);

    if min_pos.x < 0.0 {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = (display.pos.x - min_pos.x) + 0.0;
    }
    if max_pos.x > 480.0 {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = 480.0 - (max_pos.x - display.pos.x);
    }
}

/// Whether this frame's step reached the island floor: the previous y was at or above it, the
/// new y is below it, and x lies within the listener's `islandMinXPos`…`islandMaxXPos`.
pub fn reaches_island_floor(previous_y: f32, pos: Vec4, island_min_x_pos: f32, island_max_x_pos: f32, floor_y: f32) -> bool {
    let within_island = pos.x >= island_min_x_pos && pos.x <= island_max_x_pos;
    pos.y < floor_y && within_island && previous_y >= floor_y
}

/// The velocity change of a floor contact: on the first contact (not yet on the island) x and
/// z keep 40 % and y reverses at 40 %; on later contacts y stops.
pub fn bounce_throwable_velocity(pos_vel: &mut Vec4, on_island: bool) {
    if on_island {
        pos_vel.y = 0.0;
        return;
    }
    pos_vel.x = pos_vel.x * 0.4;
    pos_vel.z = pos_vel.z * 0.4;
    pos_vel.y = -(pos_vel.y * 0.4);
}

/// Whether a throwable has left the play area (`y < −100` or `y > 820`): it then dies.
pub fn is_throwable_out_of_range(y_pos: f32) -> bool {
    y_pos < -100.0 || y_pos > 820.0
}

/// The splash line of `fallFromHurricane:::` for a throwable at depth `z_pos`.
pub fn throwable_hurricane_water_line_y(z_pos: f32) -> f32 {
    let behind_island = z_pos < ISLAND_Z_POS;
    THROWABLE_HURRICANE_WATER_LINE_Y[behind_island as usize]
}

/// The offset of `drag:::`: x and y are the display position minus the touch point in the
/// display group's space (`viewToDisplayObject:`; a nil group answers the zero point), z is
/// `pos.z − pos.z` (0 for any finite depth, so the drag spring has no z stretch), and w is
/// `pos.w`.
pub fn throwable_drag_offset(world: &World, display_object: Entity, touch_screen_pos: Vec4) -> Vec4 {
    let display = display::object(world, display_object);
    let pos = display.pos;
    let display_group = display.display_group;

    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    // Original bug, kept: drag::: reloads pos.z over the touch z (ldr s2,[x20,#8]), so the z
    // offset is pos.z - pos.z.
    let offset_z = pos.z - pos.z;
    Vec4::new(offset_x, offset_y, offset_z, pos.w)
}

/// The rotation that points a dragged throwable along the drag: `atan2f(−dy, −dx)·180/π − 90`
/// degrees (`drag:::` applies it only while the offset has a y component).
pub fn throwable_drag_z_rot(offset: Vec4) -> f32 {
    let angle = (-offset.y).atan2(-offset.x);
    angle * 180.0 / std::f32::consts::PI - 90.0
}

/// The drag spring of `drag:::` (`mSpringForce`): `((length·−stiffness) / length)·offset −
/// damping·vel` for x, y and z, with w taken from the offset. With the offset of
/// `throwable_drag_offset`, z is only the damping `−damping·vel.z`.
pub fn throwable_spring_force(offset: Vec4, length: f32, pos_vel: Vec4, stiffness: f32, damping: f32) -> Vec4 {
    let stretch = (length * -stiffness) / length;

    let damping_x = -damping * pos_vel.x;
    let damping_y = -damping * pos_vel.y;
    let damping_z = -damping * pos_vel.z;

    let spring_x = damping_x + stretch * offset.x;
    let spring_y = damping_y + stretch * offset.y;
    let spring_z = damping_z + stretch * offset.z;
    Vec4::new(spring_x, spring_y, spring_z, offset.w)
}
