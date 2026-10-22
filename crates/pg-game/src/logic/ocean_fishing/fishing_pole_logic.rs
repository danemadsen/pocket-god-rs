//! `FishingPoleLogic` (superclass `SpriteLogic`).
//!
//! The fishing pole (`docs/re/logic-catalog.md` §4.3): a throwable of §2.2 (slide friction
//! 0.04, floor at y 72, no tilt sliding) that a pygmy picks up to fish. `startFishing:` adds
//! the line and the bob (a [`fishing_bob_logic`]) next to it, and while the pygmy fishes
//! `onGameFrame:` keeps the line stretched from the pole to the bob.

use crate::prelude::*;

use crate::logic::island_core::bon_fire_logic;
use crate::logic::island_core::coconut_logic::{
    ISLAND_Z_POS, THROWABLE_GRAVITY_FORCE, apply_throwable_slide_friction, bounce_throwable_velocity,
    clamp_throwable_to_screen, integrate_throwable, is_island_tipped_over, is_throwable_out_of_range,
    throwable_drag_z_rot, throwable_pos_force, throwable_spring_force, throwable_tilted_gravity_force,
};
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_fishing::{fish_logic, fishing_bob_logic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::{length3, rad_to_deg};
use pg_engine::texture::Textures;
use pg_engine::touch::BcTouch;

/// `FishingPoleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct FishingPoleLogic {
    /// `mFishingPoleLogicListener`: `@"<FishingPoleLogicListener>"`
    pub fishing_pole_logic_listener: Option<Entity>,

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

    /// `mFishingBobDisplayObject`: `@"BCDisplayObject"`
    pub fishing_bob_display_object: Option<Entity>,

    /// `mFishingStringDisplayObject`: `@"BCDisplayObject"`
    pub fishing_string_display_object: Option<Entity>,

    /// `mFishingBobLogic`: `@"FishingBobLogic"`
    pub fishing_bob_logic: Option<Entity>,

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

    /// `mbFishingIdle`: `B`
    pub fishing_idle: bool,

    /// `mbPygmyFacingRight`: `B`
    pub pygmy_facing_right: bool,

    /// `mbBeingUsed`: `B`
    pub being_used: bool,
}

pub const FISHING_POLE_LOGIC: Class = Class {
    name: "FishingPoleLogic",
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
const FLOOR_Y: f32 = 72.0;

/// The tilt of a pole lying on the island (`0xc149eb85`); `fall:::` eases toward it.
const RESTING_Z_ROT: f32 = -12.62;

/// A floor contact this fast upwards (after the contact's damping) bounces.
const BOUNCE_SPEED: f32 = 100.0;

/// `enterIsland:` drops the pole from here, or puts it straight on the floor.
const ENTER_X_POS: f32 = 200.0;
const ENTER_ABOVE_Y_POS: f32 = 400.0;

/// `fall:::` splashes below this y; `fallFromHurricane:::` too in front of the island, and
/// below the floor height behind it.
const WATER_LINE_Y: f32 = 20.0;
const BEHIND_ISLAND_WATER_LINE_Y: f32 = 72.0;

/// `dropWhileFishing`: the pole hops up as the pygmy lets go.
const DROP_Y_VEL: f32 = 400.0;

/// The spring of `drag:::`.
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `checkProximityToBonFire:`: the pole burns within (fire x + min, fire x + max), the window
/// (−30, 50) while a pygmy uses it and (−70, 35) while it lies on the island; the maxima are
/// `DAT_10029dbe0`, indexed by "not used".
const BON_FIRE_USED_MIN_X_OFFSET: f32 = -30.0;
const BON_FIRE_LYING_MIN_X_OFFSET: f32 = -70.0;
const BON_FIRE_MAX_X_OFFSETS: [f32; 2] = [50.0, 35.0];

/// `fall:::`: landing within (fire x − 70, fire x + 35) of a bonfire with a log, the pole falls
/// on into the fire.
const BON_FIRE_LANDING_MIN_X_OFFSET: f32 = -70.0;
const BON_FIRE_LANDING_MAX_X_OFFSET: f32 = 35.0;

/// `startFishing:`: the bob's touch box, (−8, −8)–(24, 24).
const BOB_COL_MIN: f32 = -8.0;
const BOB_COL_MAX: f32 = 24.0;

/// Sequence-item handlers implemented by `FishingPoleLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "fallFromHurricane" => seq_fall_from_hurricane(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[FishingPoleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&FISHING_POLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[FishingPoleLogic initWithDisplayObject:]` @ 0x1001a1c44
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(FishingPoleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.tilt_state = 0;
        pole.slide_friction = SLIDE_FRICTION;
    }
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(String::from("FishingPole"));
    world.obj_mut::<FishingPoleLogic>(this).exit_island = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<FishingPoleLogic>(this).enable_island_floor_collision = true;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[FishingPoleLogic setFishingPoleLogicListener:]` @ 0x1001a1d18
pub fn set_fishing_pole_logic_listener(world: &mut World, this: Entity, fishing_pole_logic_listener: Entity) {
    world.obj_mut::<FishingPoleLogic>(this).fishing_pole_logic_listener = Some(fishing_pole_logic_listener);
}

/// `-[FishingPoleLogic removeDisplayObjects]` @ 0x1001a1d74
///
/// PORT: the original releases the line and bob models and the bob logic here. The models
/// can still be bound to the fishing pygmy's animation channels (whose retain keeps them alive
/// in the original), and the bob logic can still be reached through the bob model and a
/// hooked fish; so the port takes them out of the display tree, shuts the bob down and keeps
/// all three entities.
pub fn remove_display_objects(world: &mut World, this: Entity) {
    let fishing_string = world.obj::<FishingPoleLogic>(this).fishing_string_display_object;
    if let Some(fishing_string) = fishing_string {
        display::remove_from_display_group(world, fishing_string);
    }
    world.obj_mut::<FishingPoleLogic>(this).fishing_string_display_object = None;

    let fishing_bob_display_object = world.obj::<FishingPoleLogic>(this).fishing_bob_display_object;
    if let Some(fishing_bob_display_object) = fishing_bob_display_object {
        display::remove_from_display_group(world, fishing_bob_display_object);
    }
    world.obj_mut::<FishingPoleLogic>(this).fishing_bob_display_object = None;

    world.obj_mut::<FishingPoleLogic>(this).pygmy_logic = None;

    let fishing_bob = world.obj::<FishingPoleLogic>(this).fishing_bob_logic;
    let Some(fishing_bob) = fishing_bob else {
        return;
    };

    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_fishing_pole_destroy_bob(world, listener, this, fishing_bob);
    }
    logic::shutdown(world, fishing_bob);
    world.obj_mut::<FishingPoleLogic>(this).fishing_bob_logic = None;
}

/// `-[FishingPoleLogic moveToIslandLayer]` @ 0x1001a1ecc
pub fn move_to_island_layer(world: &mut World, this: Entity) {
    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[FishingPoleLogic moveToBehindIslandLayer]` @ 0x1001a1f34
pub fn move_to_behind_island_layer(world: &mut World, this: Entity) {
    let island_display_group = pg_engine::library::display_group(world, "IslandDisplayGroup");
    let marker_index = match island_display_group {
        Some(island_display_group) => display::group::get_display_object_index(world, island_display_group, "IslandDisplayMarker"),
        None => 0,
    };

    let display_object = logic::display_object(world, this);
    display::move_to_display_group_at_index(world, display_object, island_display_group, marker_index);
}

/// `-[FishingPoleLogic enterIsland:]` @ 0x1001a1fc0
///
/// `startup` puts the pole straight on the floor instead of dropping it in.
pub fn enter_island(world: &mut World, this: Entity, startup: bool) {
    let falling = world.obj::<FishingPoleLogic>(this).falling;
    if !falling {
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
            state.rot_vel.z = 0.0;
        }
        world.obj_mut::<FishingPoleLogic>(this).in_water = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<FishingPoleLogic>(this).exit_island = false;

        let display_object = logic::display_object(world, this);
        {
            let mut display = display::object_mut(world, display_object);
            display.is_selectable = true;
            display.pos.x = ENTER_X_POS;
            display.pos.y = ENTER_ABOVE_Y_POS;
            if startup {
                display.pos.y = FLOOR_Y;
            }
        }
        move_to_island_layer(world, this);
        logic::set_behavior(world, this, Some("FishingPoleEnterIsland"));
    }

    sprite_logic::show_apocalypse_cloud(world, this);
}

/// `-[FishingPoleLogic exitIsland:]` @ 0x1001a2104
///
/// `now` makes the pole vanish (`FishingPoleDie`) instead of falling off the island.
pub fn exit_island(world: &mut World, this: Entity, now: bool) {
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    sprite_logic::hide_apocalypse_cloud(world, this);

    let fishing_bob = world.obj::<FishingPoleLogic>(this).fishing_bob_logic;
    if let Some(fishing_bob) = fishing_bob {
        fishing_bob_logic::drop_while_fishing(world, fishing_bob);
    }

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.picked_up_by_hurricane_logic = None;
        sprite.picked_up_by_hurricane = false;
    }

    let pygmy = world.obj::<FishingPoleLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::stop_fishing_from_pole(world, pygmy, this);
        world.obj_mut::<FishingPoleLogic>(this).pygmy_logic = None;
    }

    let dragging = world.obj::<FishingPoleLogic>(this).dragging;
    if dragging {
        {
            let mut pole = world.obj_mut::<FishingPoleLogic>(this);
            pole.falling = true;
            pole.dragging = false;
            pole.apply_spring = false;
        }
        let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_fishing_pole_drag_complete(world, listener, this);
        }
    }

    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.being_used = false;
        pole.fishing_idle = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.exit_island = true;
        pole.enable_island_floor_collision = false;
    }
    remove_display_objects(world, this);

    if now {
        logic::set_behavior(world, this, Some("FishingPoleDie"));
        return;
    }

    let in_water = world.obj::<FishingPoleLogic>(this).in_water;
    if !in_water {
        logic::set_behavior(world, this, Some("FishingPoleFallIgnoreIsland"));
    }
}

/// `-[FishingPoleLogic exitIslandNow]` @ 0x1001a22e4
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[FishingPoleLogic isAvailable]` @ 0x1001a22f8
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
    let fish_enabled = match listener {
        Some(listener) => crate::controller::is_fish_enabled(world, listener),
        None => false,
    };
    if !fish_enabled {
        return false;
    }

    let pole = world.obj::<FishingPoleLogic>(this);
    pole.on_island && !pole.being_used
}

/// `-[FishingPoleLogic pygmyLogic]` @ 0x1001a2360
pub fn pygmy_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<FishingPoleLogic>(this).pygmy_logic
}

/// `-[FishingPoleLogic fishingBobLogic]` @ 0x1001a2370
pub fn fishing_bob_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<FishingPoleLogic>(this).fishing_bob_logic
}

/// `-[FishingPoleLogic fishingBobDisplayObject]` @ 0x1001a2380
pub fn fishing_bob_display_object(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<FishingPoleLogic>(this).fishing_bob_display_object
}

/// `-[FishingPoleLogic fishingStringDisplayObject]` @ 0x1001a2390
pub fn fishing_string_display_object(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<FishingPoleLogic>(this).fishing_string_display_object
}

/// `-[FishingPoleLogic catchShark]` @ 0x1001a23a0
pub fn catch_shark(world: &mut World, this: Entity) {
    world.obj_mut::<FishingPoleLogic>(this).fishing_idle = false;
}

/// `-[FishingPoleLogic die]` @ 0x1001a23b0
pub fn die(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("FishingPoleDie"));
}

/// `-[FishingPoleLogic dropWhileFishing]` @ 0x1001a23c8
pub fn drop_while_fishing(world: &mut World, this: Entity) {
    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.being_used = false;
        pole.fishing_idle = false;
    }
    logic::logic_mut(world, this).pos_vel.y = DROP_Y_VEL;
    world.obj_mut::<FishingPoleLogic>(this).pygmy_logic = None;

    let fishing_bob = world.obj::<FishingPoleLogic>(this).fishing_bob_logic;
    if let Some(fishing_bob) = fishing_bob {
        fishing_bob_logic::drop_while_fishing(world, fishing_bob);
    }

    remove_display_objects(world, this);
    logic::set_behavior(world, this, Some("FishingPoleFall"));
}

/// `-[FishingPoleLogic checkProximityToBonFire:]` @ 0x1001a246c
pub fn check_proximity_to_bon_fire(world: &mut World, this: Entity, check_proximity_to_bon_fire: Entity) {
    let pole = world.obj::<FishingPoleLogic>(this);
    let being_used = pole.being_used;
    if !pole.on_island && !being_used {
        return;
    }

    let min_x_offset = if being_used { BON_FIRE_USED_MIN_X_OFFSET } else { BON_FIRE_LYING_MIN_X_OFFSET };
    let max_x_offset = BON_FIRE_MAX_X_OFFSETS[usize::from(!being_used)];

    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let bon_fire_display_object = logic::display_object(world, check_proximity_to_bon_fire);
    let bon_fire_x_pos = display::object(world, bon_fire_display_object).pos.x;

    let in_fire = min_x_offset + bon_fire_x_pos < x_pos && x_pos < max_x_offset + bon_fire_x_pos;
    if in_fire {
        exit_island(world, this, false);
    }
}

/// `-[FishingPoleLogic startIceFishing:]` @ 0x1001a2598
pub fn start_ice_fishing(world: &mut World, this: Entity, ice_fishing: Entity) {
    world.obj_mut::<FishingPoleLogic>(this).being_used = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut pole = world.obj_mut::<FishingPoleLogic>(this);
    pole.pygmy_logic = Some(ice_fishing);
    pole.pygmy_facing_right = true;
}

/// `-[FishingPoleLogic startFishing:]` @ 0x1001a25d4
///
/// The line (textured later by the pygmy's animation) goes right after the pole in its display
/// group and the bob right after the line, both at the pole's position.
pub fn start_fishing(world: &mut World, this: Entity, fishing: Entity) {
    world.obj_mut::<FishingPoleLogic>(this).being_used = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.pygmy_logic = Some(fishing);
        pole.pygmy_facing_right = true;
    }

    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let pole_index = match display_group {
        Some(display_group) => display::group::index_of_display_object(world, display_group, display_object),
        None => 0,
    };
    let pos = display::object(world, display_object).pos;

    let fishing_string = display::model::spawn(world, Some("FishingString"), None);
    display::move_to_display_group_at_index(world, fishing_string, display_group, pole_index + 1);
    {
        let mut string_display = display::object_mut(world, fishing_string);
        string_display.pos.x = pos.x;
        string_display.pos.y = pos.y;
        string_display.pos.z = pos.z;
    }
    world.obj_mut::<FishingPoleLogic>(this).fishing_string_display_object = Some(fishing_string);

    let fishing_bob_display_object = display::multi_model::spawn(world, Some("FishingBob"));
    display::move_to_display_group_at_index(world, fishing_bob_display_object, display_group, pole_index + 2);
    {
        let mut bob_display = display::object_mut(world, fishing_bob_display_object);
        bob_display.is_selectable = true;
        bob_display.col_x_min = BOB_COL_MIN;
        bob_display.col_x_max = BOB_COL_MAX;
        bob_display.col_y_min = BOB_COL_MIN;
        bob_display.col_y_max = BOB_COL_MAX;
        bob_display.pos.x = pos.x;
        bob_display.pos.y = pos.y;
        bob_display.pos.z = pos.z;
    }
    world.obj_mut::<FishingPoleLogic>(this).fishing_bob_display_object = Some(fishing_bob_display_object);

    let fishing_bob = fishing_bob_logic::spawn(world, fishing_bob_display_object);
    world.obj_mut::<FishingPoleLogic>(this).fishing_bob_logic = Some(fishing_bob);

    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_fishing_pole_create_bob(world, listener, this, fishing_bob);
    }
}

/// `-[FishingPoleLogic startFishingIdle]` @ 0x1001a28b0
pub fn start_fishing_idle(world: &mut World, this: Entity) {
    world.obj_mut::<FishingPoleLogic>(this).pygmy_facing_right = false;

    let display_object = logic::display_object(world, this);
    let x_scale = display::object(world, display_object).scale.x;
    if 0.0 < x_scale {
        world.obj_mut::<FishingPoleLogic>(this).pygmy_facing_right = true;
    }
    world.obj_mut::<FishingPoleLogic>(this).fishing_idle = true;

    let fishing_bob = world.obj::<FishingPoleLogic>(this).fishing_bob_logic;
    if let Some(fishing_bob) = fishing_bob {
        fishing_bob_logic::start_fishing_idle(world, fishing_bob);
    }
}

/// `-[FishingPoleLogic pickupPole]` @ 0x1001a2930
pub fn pickup_pole(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("FishingPoleStartFishingIdle"));
}

/// `-[FishingPoleLogic catchFish]` @ 0x1001a2948
pub fn catch_fish(world: &mut World, this: Entity) {
    world.obj_mut::<FishingPoleLogic>(this).fishing_idle = false;
}

/// `-[FishingPoleLogic isAccelEnabled]` @ 0x1001a2958
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
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

/// `-[FishingPoleLogic onGameFrame:]` @ 0x1001a29ec
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

    let pole = world.obj::<FishingPoleLogic>(this);
    let tilt_state = pole.tilt_state;
    let slide_friction = pole.slide_friction;
    let spring_force = pole.spring_force;
    let apply_spring = pole.apply_spring;

    let mut sprite = world.obj_mut::<SpriteLogic>(this);
    apply_throwable_slide_friction(&mut sprite.gravity_force, tilt_state, slide_friction);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;

    let pos_force = throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    integrate_throwable(world, this, pos_force, dt);
    clamp_throwable_to_screen(world, display_object);

    world.obj_mut::<FishingPoleLogic>(this).island_floor_collision = false;
    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    let pos = display::object(world, display_object).pos;
    let enable_island_floor_collision = world.obj::<FishingPoleLogic>(this).enable_island_floor_collision;
    let missed_floor = FLOOR_Y <= pos.y
        || pos.x < island_min_x_pos
        || island_max_x_pos < pos.x
        || previous_y < FLOOR_Y
        || !enable_island_floor_collision;
    if missed_floor {
        world.obj_mut::<FishingPoleLogic>(this).on_island = false;
    } else {
        land_on_island_floor(world, this, display_object);
    }

    let y_pos = display::object(world, display_object).pos.y;
    if is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("FishingPoleDie"));
    }

    let pole = world.obj::<FishingPoleLogic>(this);
    if !pole.fishing_idle {
        return;
    }
    let Some(pygmy) = pole.pygmy_logic else {
        return;
    };
    let (Some(bob_display_object), Some(string_display_object)) =
        (pole.fishing_bob_display_object, pole.fishing_string_display_object)
    else {
        // PORT: `die:::` leaves `mbFishingIdle` set, so an ice-fishing pygmy (no line, no bob)
        // can reach this with nil models, whose positions the original reads through a NULL
        // pointer (a crash); the port leaves the line alone.
        return;
    };
    update_fishing_line(world, this, pygmy, bob_display_object, string_display_object);
}

/// The island floor contact of `onGameFrame:`: on the first contact the pole bounces, or lands
/// at its resting tilt and depth; every contact puts it on the floor.
fn land_on_island_floor(world: &mut World, this: Entity, display_object: Entity) {
    let on_island = world.obj::<FishingPoleLogic>(this).on_island;
    let mut pos_vel = logic::logic(world, this).pos_vel;
    bounce_throwable_velocity(&mut pos_vel, on_island);
    logic::logic_mut(world, this).pos_vel = pos_vel;

    if BOUNCE_SPEED <= pos_vel.y {
        logic::set_behavior(world, this, Some("FishingPoleBounce"));
    } else {
        logic::logic_mut(world, this).pos_vel.y = 0.0;
        if !on_island {
            world.obj_mut::<FishingPoleLogic>(this).on_island = true;
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            {
                let mut display = display::object_mut(world, display_object);
                display.rot.z = RESTING_Z_ROT;
                display.pos.z = ISLAND_Z_POS;
            }
            world.obj_mut::<FishingPoleLogic>(this).island_floor_collision = true;
        }
    }

    display::object_mut(world, display_object).pos.y = FLOOR_Y;
}

/// The fishing part of `onGameFrame:`: turns the pygmy toward the bob when the bob crosses to
/// its other side, and stretches the line model from its origin to the bob.
fn update_fishing_line(world: &mut World, this: Entity, pygmy: Entity, bob_display_object: Entity, string_display_object: Entity) {
    let bob_pos = display::object(world, bob_display_object).pos;
    let string_pos = display::object(world, string_display_object).pos;
    let pygmy_display_object = logic::display_object(world, pygmy);
    let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;

    let facing_right = world.obj::<FishingPoleLogic>(this).pygmy_facing_right;
    let mut turned = false;
    if pygmy_x_pos <= bob_pos.x || !facing_right {
        let keeps_facing = bob_pos.x <= pygmy_x_pos || facing_right;
        if !keeps_facing {
            world.obj_mut::<FishingPoleLogic>(this).pygmy_facing_right = true;
            turned = true;
        }
    } else {
        world.obj_mut::<FishingPoleLogic>(this).pygmy_facing_right = false;
        turned = true;
    }
    if turned {
        logic::set_behavior(world, pygmy, Some("PygmyFishingFaceOtherway"));
    }

    let delta_x = bob_pos.x - string_pos.x;
    let delta_y = bob_pos.y - string_pos.y;
    let angle = delta_y.atan2(delta_x);
    display::object_mut(world, string_display_object).rot.z = rad_to_deg(angle);

    let texture = display::texture_def(world, string_display_object);
    let Some(texture) = texture else {
        return;
    };
    let frame_width = world.resource::<Textures>().def(texture).frame_width;
    let length = (delta_x * delta_x + delta_y * delta_y).sqrt();
    display::object_mut(world, string_display_object).scale.x = length / frame_width;
}

/// `-[FishingPoleLogic refreshTiltState]` @ 0x1001a30bc
///
/// The pole never slides; it only falls when the island tips over.
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let pole = world.obj::<FishingPoleLogic>(this);
    if pole.dragging || pole.falling || !pole.on_island {
        return;
    }

    if is_island_tipped_over(world) {
        {
            let mut pole = world.obj_mut::<FishingPoleLogic>(this);
            pole.falling = true;
            pole.tilt_state = 0;
        }
        logic::set_behavior(world, this, Some("FishingPoleFall"));
    }
}

/// `-[FishingPoleLogic undoTiltState]` @ 0x1001a318c
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    let mut pole = world.obj_mut::<FishingPoleLogic>(this);
    if !pole.dragging && !pole.falling && pole.on_island {
        pole.tilt_state = 0;
    }
}

/// `-[FishingPoleLogic drag:::]` @ 0x1001a31cc
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<FishingPoleLogic>(this).enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    let touch_screen_pos = world.obj::<FishingPoleLogic>(this).touch_screen_pos;
    let offset = fish_logic::drag_offset(world, display_object, touch_screen_pos);
    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<FishingPoleLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// The splash of `fall:::` and `fallFromHurricane:::`: the pole stops in the water and plays
/// `FishingPoleSplash`.
fn splash(world: &mut World, this: Entity) {
    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.falling = false;
        pole.in_water = true;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<FishingPoleLogic>(this).tilt_state = 0;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    logic::set_behavior(world, this, Some("FishingPoleSplash"));
}

/// `-[FishingPoleLogic fall:::]` @ 0x1001a337c
///
/// `[0] ignoreIsland` falls through the island floor.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        {
            let mut pole = world.obj_mut::<FishingPoleLogic>(this);
            if !pole.island_floor_collision {
                pole.on_island = false;
            }
        }
        {
            let mut display = display::object_mut(world, display_object);
            display.scale.x = 1.0;
            display.scale.y = 1.0;
        }
        world.obj_mut::<FishingPoleLogic>(this).enable_island_floor_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<FishingPoleLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<FishingPoleLogic>(this).enable_island_floor_collision = false;
        }
    }

    // Each frame the pole turns halfway to its resting tilt.
    {
        let mut display = display::object_mut(world, display_object);
        let z_rot = display.rot.z;
        display.rot.z = z_rot + (RESTING_Z_ROT - z_rot) * 0.5;
    }

    let island_floor_collision = world.obj::<FishingPoleLogic>(this).island_floor_collision;
    if !island_floor_collision {
        let y_pos = display::object(world, display_object).pos.y;
        if WATER_LINE_Y <= y_pos {
            return ItemResult::Pending;
        }
        splash(world, this);
        return ItemResult::Replaced;
    }

    world.obj_mut::<FishingPoleLogic>(this).falling = false;
    display::object_mut(world, display_object).rot.z = RESTING_Z_ROT;

    // Landed next to a bonfire with a log: the pole falls on into the fire.
    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
    let Some(listener) = listener else {
        return ItemResult::Done;
    };
    let bon_fire_enabled = crate::controller::is_bon_fire_enabled(world, listener);
    if !bon_fire_enabled {
        return ItemResult::Done;
    }
    let bon_fire = crate::controller::bon_fire_logic(world, listener, this);
    let Some(bon_fire) = bon_fire else {
        return ItemResult::Done;
    };

    let bon_fire_display_object = logic::display_object(world, bon_fire);
    let has_log = bon_fire_logic::has_at_least_one_log(world, bon_fire);
    if !has_log {
        return ItemResult::Done;
    }
    let x_pos = display::object(world, display_object).pos.x;
    let bon_fire_x_pos = display::object(world, bon_fire_display_object).pos.x;
    let over_fire = bon_fire_x_pos + BON_FIRE_LANDING_MIN_X_OFFSET < x_pos && x_pos < bon_fire_x_pos + BON_FIRE_LANDING_MAX_X_OFFSET;
    if !over_fire {
        return ItemResult::Done;
    }

    world.obj_mut::<FishingPoleLogic>(this).falling = true;
    logic::set_behavior(world, this, Some("FishingPoleFallIgnoreIsland"));
    ItemResult::Replaced
}

/// `-[FishingPoleLogic fallFromHurricane:::]` @ 0x1001a3650
pub fn seq_fall_from_hurricane(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<FishingPoleLogic>(this).enable_island_floor_collision = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<FishingPoleLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let water_line_y = if ISLAND_Z_POS <= pos.z { WATER_LINE_Y } else { BEHIND_ISLAND_WATER_LINE_Y };
    if water_line_y <= pos.y {
        return ItemResult::Pending;
    }

    // The original sets y between the velocity and the tilt state; nothing reads it there.
    display::object_mut(world, display_object).pos.y = water_line_y;
    splash(world, this);
    ItemResult::Replaced
}

/// `-[FishingPoleLogic die:::]` @ 0x1001a3794
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos.z = ISLAND_Z_POS;
    remove_display_objects(world, this);
    display::remove_from_display_group(world, display_object);

    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
    let dragging = world.obj::<FishingPoleLogic>(this).dragging;
    if dragging {
        world.obj_mut::<FishingPoleLogic>(this).dragging = false;
        if let Some(listener) = listener {
            crate::controller::on_fishing_pole_drag_complete(world, listener, this);
        }
    }

    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.falling = false;
        pole.in_water = false;
        pole.being_used = false;
        pole.apply_spring = false;
        pole.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let fish_enabled = match listener {
        Some(listener) => crate::controller::is_fish_enabled(world, listener),
        None => false,
    };
    if fish_enabled {
        enter_island(world, this, false);
    }
    ItemResult::Done
}

/// `-[FishingPoleLogic pickedUpByHurricane:]` @ 0x1001a38d4
///
/// Unlike the other throwables, a pole a pygmy is fishing with can be taken.
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, picked_up_by_hurricane: Entity) -> bool {
    let pole = world.obj::<FishingPoleLogic>(this);
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let game_frame_running = logic::logic(world, this).game_frame_running;
    let can_be_picked_up = !pole.touch_began_inside
        && !already_picked_up
        && !pole.dragging
        && !pole.falling
        && !pole.in_water
        && !pole.exit_island
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
    world.obj_mut::<FishingPoleLogic>(this).tilt_state = 0;
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
    logic::set_behavior(world, this, Some("FishingPolePickedUpByHurricane"));
    true
}

/// `-[FishingPoleLogic droppedByHurricane:]` @ 0x1001a3a40
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _dropped_by_hurricane: Entity) {
    let picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up {
        return;
    }

    world.obj_mut::<FishingPoleLogic>(this).falling = true;
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
    logic::set_behavior(world, this, Some("FishingPoleDroppedByHurricane"));
}

/// `-[FishingPoleLogic touchBeganInside:allTouches:]` @ 0x1001a3b34
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let pole = world.obj::<FishingPoleLogic>(this);
    let busy = pole.falling || pole.in_water || pole.being_used || pole.active_touch.is_some();
    if story_active || busy {
        return;
    }

    world.obj_mut::<FishingPoleLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut pole = world.obj_mut::<FishingPoleLogic>(this);
        pole.touch_began_inside = true;
        pole.touch_screen_pos = touch_pos;
        pole.dragging = true;
        pole.apply_spring = true;
        // Original quirk: cleared again straight away.
        pole.touch_began_inside = false;
        pole.tilt_state = 0;
    }

    if let Some(listener) = listener {
        crate::controller::on_fishing_pole_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("FishingPoleDrag"));
}

/// `-[FishingPoleLogic touchBeganOutside:allTouches:]` @ 0x1001a3c7c
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishingPoleLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut pole = world.obj_mut::<FishingPoleLogic>(this);
    pole.touch_screen_pos = touch_pos;
    pole.touch_began_inside = false;
}

/// `-[FishingPoleLogic touchMovedEntered:allTouches:]` @ 0x1001a3cd8
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishingPoleLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<FishingPoleLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[FishingPoleLogic touchMovedExited:allTouches:]` @ 0x1001a3d30
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishingPoleLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<FishingPoleLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[FishingPoleLogic touchMovedInside:allTouches:]` @ 0x1001a3d88
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[FishingPoleLogic touchMovedOutside:allTouches:]` @ 0x1001a3d98
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishingPoleLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<FishingPoleLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[FishingPoleLogic touchEndedInside:allTouches:]` @ 0x1001a3df0
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<FishingPoleLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let dragging = world.obj::<FishingPoleLogic>(this).dragging;
    if dragging {
        {
            let mut pole = world.obj_mut::<FishingPoleLogic>(this);
            pole.falling = true;
            pole.dragging = false;
            pole.apply_spring = false;
        }

        let listener = world.obj::<FishingPoleLogic>(this).fishing_pole_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_fishing_pole_drag_complete(world, listener, this);
        }
        logic::set_behavior(world, this, Some("FishingPoleFall"));
    }

    let mut pole = world.obj_mut::<FishingPoleLogic>(this);
    pole.touch_began_inside = false;
    pole.active_touch = None;
}

/// `-[FishingPoleLogic touchEndedOutside:allTouches:]` @ 0x1001a3ea4
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
