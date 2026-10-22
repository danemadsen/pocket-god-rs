//! `BatLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::math::normalize3;
use pg_engine::view::BcView;

/// `BatLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BatLogic {
    /// `mBatLogicListener`: `@"<BatLogicListener>"`
    pub bat_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mHoverTime`: `f`
    pub hover_time: f32,

    /// `mTotalHoverTime`: `f`
    pub total_hover_time: f32,

    /// `mHoverTargetXPos`: `f`
    pub hover_target_x_pos: f32,

    /// `mHoverTargetYPos`: `f`
    pub hover_target_y_pos: f32,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbFlyFromMoon`: `B`
    pub fly_from_moon: bool,

    /// `mbInWater`: `B`
    pub in_water: bool,

    /// `mbFalling`: `B`
    pub falling: bool,
}

pub const BAT_LOGIC: Class = Class {
    name: "BatLogic",
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

/// Sequence-item handlers implemented by `BatLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "flyFromMoonToFront" => seq_fly_from_moon_to_front(world, this, call),
        "flyFromFrontToIsland" => seq_fly_from_front_to_island(world, this, call),
        "hover" => seq_hover(world, this, call),
        "flyToPygmy" => seq_fly_to_pygmy(world, this, call),
        "flyOffScreen" => seq_fly_off_screen(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[BatLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BAT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[BatLogic initWithDisplayObject:]` @ 0x100115a10
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BatLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<BatLogic>(this).in_water = false;
    logic::set_behavior(world, this, Some("BatFlyFromMoon"));
}

/// `-[BatLogic setBatLogicListener:]` @ 0x100115a88
pub fn set_bat_logic_listener(world: &mut World, this: Entity, bat_logic_listener: Entity) {
    world.obj_mut::<BatLogic>(this).bat_logic_listener = Some(bat_logic_listener);
}

/// `-[BatLogic isAccelEnabled]` @ 0x100115b48
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<BatLogic>(this).bat_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[BatLogic onGameFrame:]` @ 0x100115bb8
///
/// Gravity follows the device tilt when the accelerometer is on (3000 along the tilt, else
/// straight down). Unlike `SpriteLogic`, the position moves with the new speed. Leaving the
/// screen vertically kills the bat; while the hurricane holds it, it spins 45° per frame.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let accel_enabled = is_accel_enabled(world, this);
    let gravity_force = if accel_enabled {
        let accel_force = world.resource::<BcView>().accel_force;
        let tilt = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let mut gravity = normalize3(tilt);
        gravity.x = gravity.x * 3000.0;
        gravity.y = gravity.y * 3000.0;
        gravity.z = gravity.z * 3000.0;
        gravity
    } else {
        Vec4::new(0.0, -3000.0, 0.0, 1.0)
    };
    world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;

    let mut pos_force = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
    if !ignore_gravity {
        pos_force.x = gravity_force.x + pos_force.x;
        pos_force.y = gravity_force.y + pos_force.y;
        pos_force.z = gravity_force.z + pos_force.z;
    }

    let pos_vel = {
        let mut state = logic::logic_mut(world, this);
        state.pos_force = pos_force;
        state.pos_vel.x = state.pos_vel.x + pos_force.x * dt;
        state.pos_vel.y = state.pos_vel.y + pos_force.y * dt;
        state.pos_vel.z = state.pos_vel.z + pos_force.z * dt;
        state.pos_vel
    };

    let y_pos = {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = display.pos.x + pos_vel.x * dt;
        display.pos.y = display.pos.y + pos_vel.y * dt;
        display.pos.z = display.pos.z + pos_vel.z * dt;
        display.pos.y
    };

    if y_pos < -100.0 || y_pos > 820.0 {
        logic::set_behavior(world, this, Some("BatDie"));
    }

    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if picked_up_by_hurricane {
        let mut display = display::object_mut(world, display_object);
        display.rot.z = display.rot.z + 45.0;
    }
}

/// Zeroes the x, y and z speed (`w` is kept).
fn stop_moving(world: &mut World, this: Entity) {
    let mut state = logic::logic_mut(world, this);
    state.pos_vel.x = 0.0;
    state.pos_vel.y = 0.0;
    state.pos_vel.z = 0.0;
}

/// The view controller's `pygmyLogicArray:` (empty without a listener).
fn pygmy_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<BatLogic>(this).bat_logic_listener;
    match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

/// `-[BatLogic flyFromMoonToFront:::]` @ 0x100115e58
///
/// Fades in while flying from the moon (far behind, `z = −700`) to the front plane at
/// (240, 200, −177) in 3 s.
pub fn seq_fly_from_moon_to_front(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        {
            let mut bat = world.obj_mut::<BatLogic>(this);
            bat.falling = false;
            bat.fly_from_moon = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

        let pos = {
            let mut display = display::object_mut(world, display_object);
            display.set_alpha(0.0);
            display.pos.z = -700.0;
            display.pos
        };

        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = (240.0 - pos.x) / 3.0;
        state.pos_vel.y = (200.0 - pos.y) / 3.0;
        state.pos_vel.z = (-177.0 - pos.z) / 3.0;
    }

    let z_pos = {
        let mut display = display::object_mut(world, display_object);
        let alpha = display.alpha;
        display.set_alpha(alpha + (1.0 - alpha) * 0.1);
        display.pos.z
    };
    if !(z_pos > -177.0) {
        return ItemResult::Pending;
    }

    stop_moving(world, this);
    world.obj_mut::<BatLogic>(this).fly_from_moon = false;
    display::object_mut(world, display_object).set_alpha(1.0);
    ItemResult::Done
}

/// `-[BatLogic flyFromFrontToIsland:::]` @ 0x100116004
///
/// Glides to (350, 250) on the island plane; any pygmy it can bite on the way sends it at that
/// pygmy (`BatFlyToPygmyFromBack`).
pub fn seq_fly_from_front_to_island(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        let pos = display::object(world, display_object).pos;
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = (350.0 - pos.x) * 0.5;
        state.pos_vel.y = (250.0 - pos.y) * 0.5;
        state.pos_vel.z = (SPRITE_Z - pos.z) * 0.5;
    }

    let mut result = ItemResult::Pending;

    // Every biteable pygmy restarts the behaviour; the loop does not stop at the first.
    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        let can_be_bit = crate::pygmy::can_be_bit_by_bat(world, pygmy);
        if can_be_bit {
            stop_moving(world, this);
            logic::set_behavior(world, this, Some("BatFlyToPygmyFromBack"));
            result = ItemResult::Done;
        }
    }

    let z_pos = display::object(world, display_object).pos.z;
    if z_pos < SPRITE_Z {
        stop_moving(world, this);
        display::object_mut(world, display_object).pos.z = SPRITE_Z;
        result = ItemResult::Done;
    }
    result
}

/// `-[BatLogic hover:::]` @ 0x100116264
///
/// Eases toward a random point of the sky (x 100–380, y 160–300) over a time proportional to
/// the distance, unless a biteable pygmy shows up (`BatFlyToPygmy`).
pub fn seq_hover(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        world.obj_mut::<BatLogic>(this).falling = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

        let x_random = world.resource_mut::<Random>().arc4random();
        let target_x = (x_random % 2801 + 1000) as f32 / 10.0;
        world.obj_mut::<BatLogic>(this).hover_target_x_pos = target_x;

        let y_random = world.resource_mut::<Random>().arc4random();
        let target_y = (y_random % 1401 + 1600) as f32 / 10.0;
        world.obj_mut::<BatLogic>(this).hover_target_y_pos = target_y;

        let pos = display::object(world, display_object).pos;
        let x_scale = if pos.x > target_x { -1.0 } else { 1.0 };
        display::object_mut(world, display_object).scale.x = x_scale;

        let delta_x = target_x - pos.x;
        let delta_y = target_y - pos.y;
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

        let mut bat = world.obj_mut::<BatLogic>(this);
        bat.hover_time = (f64::from(distance) * 0.1) as f32;
        bat.total_hover_time = 0.0;
    }

    let mut result = ItemResult::Pending;

    // Every biteable pygmy restarts the behaviour; the loop does not stop at the first.
    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        let can_be_bit = crate::pygmy::can_be_bit_by_bat(world, pygmy);
        if can_be_bit {
            stop_moving(world, this);
            logic::set_behavior(world, this, Some("BatFlyToPygmy"));
            result = ItemResult::Done;
        }
    }

    let bat = world.obj::<BatLogic>(this);
    let target_x = bat.hover_target_x_pos;
    let target_y = bat.hover_target_y_pos;
    let elapsed = bat.total_hover_time / bat.hover_time;
    let progress = if elapsed > 1.0 { 1.0 } else { elapsed };

    let x_pos = {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = display.pos.x + progress * (target_x - display.pos.x);
        display.pos.y = display.pos.y + progress * (target_y - display.pos.y);
        display.pos.x
    };
    if target_x - 3.0 <= x_pos && x_pos <= target_x + 3.0 {
        result = ItemResult::Done;
    }

    let delta_time = call.delta_time;
    let mut bat = world.obj_mut::<BatLogic>(this);
    bat.total_hover_time = delta_time + bat.total_hover_time;
    result
}

/// `-[BatLogic flyToPygmy:::]` @ 0x100116618
///
/// Heads for the front-most biteable pygmy (highest index in its display group), drawn just in
/// front of it, and bites it once over it. Without one, the bat goes idle in front of the
/// dragged objects.
pub fn seq_fly_to_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        world.obj_mut::<BatLogic>(this).falling = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    }

    let mut target: Option<Entity> = None;
    let mut target_index: i32 = -1;
    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        let can_be_bit = crate::pygmy::can_be_bit_by_bat(world, pygmy);
        if !can_be_bit {
            continue;
        }

        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_group = display::object(world, pygmy_display_object).display_group;
        let pygmy_index = match pygmy_group {
            Some(pygmy_group) => group::index_of_display_object(world, pygmy_group, pygmy_display_object),
            None => 0,
        };
        if pygmy_index > target_index {
            target = Some(pygmy);
            target_index = pygmy_index;
        }
    }

    let bat_group = display::object(world, display_object).display_group;

    let Some(target) = target else {
        stop_moving(world, this);
        let drag_marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
        if let Some(drag_marker) = drag_marker {
            display::insert_after(world, drag_marker, Some(display_object));
        }
        logic::set_behavior(world, this, Some("BatIdle"));
        return ItemResult::Done;
    };

    let target_display_object = logic::display_object(world, target);
    let target_pos = display::object(world, target_display_object).pos;
    let bat_pos = display::object(world, display_object).pos;

    let vel_x = target_pos.x - bat_pos.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = vel_x;
        state.pos_vel.y = target_pos.y - bat_pos.y;
        state.pos_vel.z = target_pos.z - bat_pos.z;
    }

    display::move_to_display_group_at_index(world, display_object, bat_group, target_index + 1);

    let x_scale = if vel_x < 0.0 { -1.0 } else { 1.0 };
    display::object_mut(world, display_object).scale.x = x_scale;

    let over_target_x = bat_pos.x > target_pos.x - 1.0 && bat_pos.x < target_pos.x + 1.0;
    if !over_target_x {
        return ItemResult::Pending;
    }
    if !(bat_pos.y <= target_pos.y + 1.0) {
        return ItemResult::Pending;
    }

    stop_moving(world, this);
    crate::pygmy::bit_by_bat(world, target, this);
    ItemResult::Done
}

/// `-[BatLogic flyOffScreen:::]` @ 0x1001169f8
///
/// Flies up and out through the nearer side of the screen.
pub fn seq_fly_off_screen(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);

    if call.first_time {
        let x_pos = display::object(world, display_object).pos.x;
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 200.0;
        state.pos_vel.y = 100.0;
        if x_pos < 240.0 {
            state.pos_vel.x = -200.0;
        }
    }

    let x_pos = display::object(world, display_object).pos.x;
    if x_pos < -80.0 || x_pos > 560.0 {
        stop_moving(world, this);
        return ItemResult::Done;
    }
    ItemResult::Pending
}

/// `-[BatLogic fall:::]` @ 0x100116ac0
///
/// Falls with gravity; a bat in front of the island plane that reaches the sea (when there is
/// one) splashes in.
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<BatLogic>(this).falling = true;
    }

    let display_object = logic::display_object(world, this);
    let bat_display = display::object(world, display_object);
    let pos = bat_display.pos;
    let origin_pos = bat_display.origin_pos;

    if pos.z >= SPRITE_Z {
        let listener = world.obj::<BatLogic>(this).bat_logic_listener;
        let ocean_enabled = match listener {
            Some(listener) => crate::controller::is_ocean_enabled(world, listener),
            None => false,
        };

        if ocean_enabled && pos.y + origin_pos.y < 10.0 {
            {
                let mut bat = world.obj_mut::<BatLogic>(this);
                bat.falling = false;
                bat.in_water = true;
            }
            {
                let mut state = logic::logic_mut(world, this);
                state.pos_vel.x = 0.0;
                state.pos_vel.y = 0.0;
            }
            display::object_mut(world, display_object).pos.y = 10.0 - origin_pos.y;
            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            display::object_mut(world, display_object).rot.z = 0.0;
            logic::set_behavior(world, this, Some("BatSplash"));
            return ItemResult::Replaced;
        }
    }
    ItemResult::Pending
}

/// `-[BatLogic die:::]` @ 0x100116c2c
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_moving(world, this);
    world.obj_mut::<BatLogic>(this).active_touch = None;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let listener = world.obj::<BatLogic>(this).bat_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_bat_die(world, listener, this);
    }
    ItemResult::Done
}

/// `-[BatLogic pickedUpByHurricane:]` @ 0x100116c9c
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, _hurricane: Entity) -> bool {
    let bat = world.obj::<BatLogic>(this);
    let touch_began_inside = bat.touch_began_inside;
    let falling = bat.falling;
    let in_water = bat.in_water;
    let already_picked_up = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if touch_began_inside || already_picked_up || falling || in_water {
        return false;
    }

    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = true;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = true;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.z = SPRITE_Z;
        display.is_selectable = false;
    }
    logic::set_behavior(world, this, Some("BatPickedUpByHurricane"));
    true
}

/// `-[BatLogic droppedByHurricane:]` @ 0x100116da8
///
/// A bat dropped behind the island plane goes back into the island group, just before
/// `IslandDisplayMarker`.
pub fn dropped_by_hurricane(world: &mut World, this: Entity, _hurricane: Entity) {
    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    if !picked_up_by_hurricane {
        return;
    }

    world.obj_mut::<BatLogic>(this).falling = true;
    {
        let mut sprite = world.obj_mut::<SpriteLogic>(this);
        sprite.ignore_gravity = false;
        sprite.blown_by_hurricane = false;
        sprite.picked_up_by_hurricane = false;
    }

    let display_object = logic::display_object(world, this);
    let z_pos = display::object(world, display_object).pos.z;
    if z_pos < SPRITE_Z {
        let island_group = pg_engine::library::display_group(world, "IslandDisplayGroup");
        let marker_index = match island_group {
            Some(island_group) => group::get_display_object_index(world, island_group, "IslandDisplayMarker"),
            None => 0,
        };
        display::remove_from_display_group(world, display_object);
        display::move_to_display_group_at_index(world, display_object, island_group, marker_index);
    }
    logic::set_behavior(world, this, Some("BatDroppedByHurricane"));
}

/// `-[BatLogic touchBeganInside:allTouches:]` @ 0x100116eec
///
/// A tap swats a flying bat out of the air.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let bat = world.obj::<BatLogic>(this);
    if bat.falling || bat.fly_from_moon || bat.in_water || bat.active_touch.is_some() {
        return;
    }

    world.obj_mut::<BatLogic>(this).active_touch = Some(touch);
    stop_moving(world, this);
    {
        let mut bat = world.obj_mut::<BatLogic>(this);
        bat.in_water = false;
        bat.touch_began_inside = false;
    }
    logic::set_behavior(world, this, Some("BatFall"));
}

/// `-[BatLogic touchBeganOutside:allTouches:]` @ 0x100116f74
pub fn touch_began_outside(world: &mut World, this: Entity, _touch: Entity) {
    let mut bat = world.obj_mut::<BatLogic>(this);
    if bat.active_touch.is_none() {
        bat.touch_began_inside = false;
    }
}

/// `-[BatLogic touchMovedEntered:allTouches:]` @ 0x100116f94
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BatLogic touchMovedExited:allTouches:]` @ 0x100116f98
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BatLogic touchMovedInside:allTouches:]` @ 0x100116f9c
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[BatLogic touchMovedOutside:allTouches:]` @ 0x100116fac
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[BatLogic touchEndedInside:allTouches:]` @ 0x100116fb0
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let mut bat = world.obj_mut::<BatLogic>(this);
    let active_touch = bat.active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        bat.touch_began_inside = false;
        bat.active_touch = None;
    }
}

/// `-[BatLogic touchEndedOutside:allTouches:]` @ 0x100116fdc
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
