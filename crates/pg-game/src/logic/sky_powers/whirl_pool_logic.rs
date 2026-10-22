//! `WhirlPoolLogic` (superclass `SpriteLogic`).
//!
//! The underwater counterpart of `HurricaneLogic`: a flat vortex around (240, 160). The display
//! object is always a `WhirlPoolModel` (`PocketGodViewController(Game)::maintainWhirlPool:`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::sky_powers::whirl_pool_model;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::trunc_i32;

/// `WhirlPoolLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct WhirlPoolLogic {
    /// `mWhirlPoolLogicListener`: `@"<WhirlPoolLogicListener>"`
    pub whirl_pool_logic_listener: Option<Entity>,

    /// `mbStopWhirlPool`: `B`
    pub stop_whirl_pool: bool,

    /// `mbWhirlPoolActive`: `B`
    pub whirl_pool_active: bool,

    /// `mbPickUpObjectsActive`: `B`
    pub pick_up_objects_active: bool,

    /// `mbTouchActive`: `B`
    pub touch_active: bool,

    /// `mWindRot`: `f`
    pub wind_rot: f32,

    /// `mWindRotVel`: `f`
    pub wind_rot_vel: f32,

    /// `mWindVel`: `f`
    pub wind_vel: f32,

    /// `mPickedUpLogicArray`: `@"NSMutableArray"`
    pub picked_up_logic_array: Vec<Entity>,
}

pub const WHIRL_POOL_LOGIC: Class = Class {
    name: "WhirlPoolLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Each circling touch speeds the current up by this much, up to the maximum.
const WIND_VEL_STEP: f32 = 0.03;
const MAX_WIND_VEL: f32 = 3.0;

/// The current slows down by this much every frame, and by the stop decay more while stopping.
const WIND_VEL_DECAY: f32 = -0.01;
const STOP_WIND_VEL_DECAY: f32 = -0.05;

/// From this speed on the whirlpool is fully visible (and plays `WhirlPoolStart`).
const ACTIVE_WIND_VEL: f32 = 0.5;

/// From this speed on it pulls things in.
const PICK_UP_WIND_VEL: f32 = 2.0;

/// A wind record's segment phase wraps to 0 past this.
const MAX_DRAW_POS: f32 = 180.0;

/// The ribbons' alpha at full visibility.
const MODEL_ALPHA: f32 = 0.7;

/// The centre of the vortex.
const VORTEX_X_POS: f32 = 240.0;
const VORTEX_Y_POS: f32 = 160.0;

/// Wind record constants.
const WIND_ROT_VEL: f32 = 0.2;
const WIND_LINE_WIDTH: f32 = 2.0;

/// Objects closer to the centre are pushed out to this radius.
const MIN_SPIN_RADIUS: f32 = 20.0;

/// Share of the way to the target radius covered each frame.
const RADIUS_EASE: f32 = 0.2;
const MAX_SPIN_STRENGTH: f32 = 1.0;

/// Target radius bounds of a caught object, in tenths: at least its distance (at most 50),
/// below its distance plus a tenth (at most 140.1).
const MAX_INNER_TARGET_RADIUS: f32 = 50.0;
const MAX_INNER_TARGET_RADIUS_TENTHS: i32 = 500;
const MAX_OUTER_TARGET_RADIUS: f32 = 140.0;
const MAX_OUTER_TARGET_RADIUS_TENTHS: i32 = 1401;

/// A sprite waits a random 1–30 frames before the whirlpool changes its state.
const STALL_FRAME_RANGE: u32 = 30;

/// Sequence-item handlers implemented by `WhirlPoolLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[WhirlPoolLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&WHIRL_POOL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[WhirlPoolLogic initWithDisplayObject:]` @ 0x100137600
///
/// Seeds the five spiral ribbons around the centre (invisible until the current picks up).
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(WhirlPoolLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<WhirlPoolLogic>(this).wind_vel = 0.0;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).set_alpha(0.0);

    let mut wind_pos_array = {
        let wind_pos_array = whirl_pool_model::whirl_pool_wind_pos_array(world, display_object);
        wind_pos_array.clone()
    };
    for wind_pos in wind_pos_array.iter_mut() {
        wind_pos.pos.x = VORTEX_X_POS;
        wind_pos.pos.y = VORTEX_Y_POS;
        wind_pos.pos.z = 0.0;
        wind_pos.pos_vel.x = 0.0;
        wind_pos.pos_vel.y = 0.0;
        wind_pos.pos_vel.z = 0.0;
        wind_pos.wind_rot = 0.0;
        wind_pos.wind_rot_vel = WIND_ROT_VEL;

        let radius_roll = world.resource_mut::<Random>().arc4random_mod(401);
        wind_pos.radius = (radius_roll + 1000) as f32 / 10.0;

        let radius_vel_roll = world.resource_mut::<Random>().arc4random_mod(21);
        wind_pos.radius_vel = (radius_vel_roll as i32 - 100) as f32 / 100.0;

        let draw_pos_roll = world.resource_mut::<Random>().arc4random_mod(1801);
        wind_pos.draw_pos = draw_pos_roll as f32 / 10.0;

        let draw_pos_vel_roll = world.resource_mut::<Random>().arc4random_mod(21);
        wind_pos.draw_pos_vel = (draw_pos_vel_roll + 80) as f32 / 100.0;

        wind_pos.line_width = WIND_LINE_WIDTH;

        let segment_roll = world.resource_mut::<Random>().arc4random() & 3;
        wind_pos.segment_count = (segment_roll + 5) as i32;
    }
    *whirl_pool_model::whirl_pool_wind_pos_array(world, display_object) = wind_pos_array;

    world.obj_mut::<WhirlPoolLogic>(this).picked_up_logic_array = Vec::with_capacity(5);
}

/// `-[WhirlPoolLogic setWhirlPoolLogicListener:]` @ 0x100137824
pub fn set_whirl_pool_logic_listener(world: &mut World, this: Entity, whirl_pool_logic_listener: Entity) {
    world.obj_mut::<WhirlPoolLogic>(this).whirl_pool_logic_listener = Some(whirl_pool_logic_listener);
}

/// `-[WhirlPoolLogic dealloc]` @ 0x1001379a4
///
/// For whoever releases the whirlpool last (the view controller's `onWhirlPoolDie:`, right
/// after it took the whirlpool out of the display tree and shut it down). `BCLogic`'s release
/// of the display object frees the `WhirlPoolModel` once it has left the tree; the held
/// sprites belong to the controller.
pub fn dealloc(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);

    // `[super dealloc]`: `SpriteLogic` only releases its cloud style string.
    logic::dealloc(world, this);

    let model_out_of_display_tree = match world.get::<DisplayObject>(display_object) {
        Some(model) => model.display_group.is_none(),
        None => false,
    };
    if model_out_of_display_tree {
        world.despawn(display_object);
    }
}

/// `-[WhirlPoolLogic setTouchActive:]` @ 0x100137880
pub fn set_touch_active(world: &mut World, this: Entity, touch_active: bool) {
    world.obj_mut::<WhirlPoolLogic>(this).touch_active = touch_active;
}

/// `-[WhirlPoolLogic stopWhirlPoolNow]` @ 0x100137890
///
/// Lets go of every held sprite (last first) and dies.
pub fn stop_whirl_pool_now(world: &mut World, this: Entity) {
    world.obj_mut::<WhirlPoolLogic>(this).stop_whirl_pool = true;

    loop {
        let last_held = world.obj::<WhirlPoolLogic>(this).picked_up_logic_array.last().copied();
        let Some(sprite) = last_held else {
            break;
        };
        world.obj_mut::<WhirlPoolLogic>(this).picked_up_logic_array.retain(|held| *held != sprite);
        crate::send::dropped_by_whirl_pool(world, Some(sprite), this);
    }

    logic::set_behavior(world, this, Some("WhirlPoolDie"));
}

/// `-[WhirlPoolLogic stopWhirlPool]` @ 0x100137958
pub fn stop_whirl_pool(world: &mut World, this: Entity) {
    world.obj_mut::<WhirlPoolLogic>(this).stop_whirl_pool = true;
}

/// `-[WhirlPoolLogic setWindVel:]` @ 0x10013796c
///
/// The argument is ignored: every call speeds the current up by 0.03 (at most 3) and cancels
/// a stop.
pub fn set_wind_vel(world: &mut World, this: Entity, _wind_vel: f32) {
    let mut whirl_pool = world.obj_mut::<WhirlPoolLogic>(this);
    whirl_pool.stop_whirl_pool = false;
    let wind_vel = whirl_pool.wind_vel + WIND_VEL_STEP;
    whirl_pool.wind_vel = if wind_vel > MAX_WIND_VEL { MAX_WIND_VEL } else { wind_vel };
}

/// `-[WhirlPoolLogic spinDisplayObject:whirlPoolSpinState:]` @ 0x100137a20
///
/// Moves one object along its circle around the centre, its radius easing toward the target,
/// and advances its angle and strength.
pub fn spin_display_object(world: &mut World, _this: Entity, display_object: Entity, spin_state: &mut WhirlPoolSpinState) {
    let mut radius = spin_state.radius;
    if radius < MIN_SPIN_RADIUS {
        spin_state.radius = MIN_SPIN_RADIUS;
        radius = MIN_SPIN_RADIUS;
    }

    let angle = spin_state.angle;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = radius * angle.cos() + VORTEX_X_POS;
        object.pos.y = radius * angle.sin() + VORTEX_Y_POS;
    }

    spin_state.angle = spin_state.angle_vel + angle;
    spin_state.radius = radius + (spin_state.target_radius - radius) * RADIUS_EASE;
    let next_strength = spin_state.strength_vel + spin_state.strength;
    spin_state.strength = if next_strength > MAX_SPIN_STRENGTH { MAX_SPIN_STRENGTH } else { next_strength };
}

/// `-[WhirlPoolLogic spinSpriteLogics]` @ 0x100137aec
pub fn spin_sprite_logics(world: &mut World, this: Entity) {
    let picked_up_logics = world.obj::<WhirlPoolLogic>(this).picked_up_logic_array.clone();
    for sprite in picked_up_logics {
        let sprite_display_object = logic::display_object(world, sprite);
        let mut spin_state = *sprite_logic::whirl_pool_spin_state(world, sprite);
        spin_display_object(world, this, sprite_display_object, &mut spin_state);
        *sprite_logic::whirl_pool_spin_state(world, sprite) = spin_state;
    }
}

/// `-[WhirlPoolLogic countdownStall:]` @ 0x100137c50
///
/// Counts the sprite's random 1–30 frame delay down; true when it runs out, or at once while
/// the whirlpool is stopping.
pub fn countdown_stall(world: &mut World, this: Entity, sprite: Entity) -> bool {
    let mut stall = sprite_logic::whirl_pool_spin_state(world, sprite).stall;
    if stall == 0 {
        let roll = world.resource_mut::<Random>().arc4random_mod(STALL_FRAME_RANGE);
        stall = roll as i32 + 1;
        sprite_logic::whirl_pool_spin_state(world, sprite).stall = stall;
    }

    let remaining_stall = stall - 1;
    sprite_logic::whirl_pool_spin_state(world, sprite).stall = remaining_stall;
    if remaining_stall == 0 {
        return true;
    }
    world.obj::<WhirlPoolLogic>(this).stop_whirl_pool
}

/// `-[WhirlPoolLogic whirlPoolActive]` @ 0x100137cdc
///
/// Whether the current is strong enough to pull things in.
pub fn whirl_pool_active(world: &mut World, this: Entity) -> bool {
    world.obj::<WhirlPoolLogic>(this).wind_vel >= PICK_UP_WIND_VEL
}

/// `-[WhirlPoolLogic onGameFrame:]` @ 0x100137cf8
///
/// Animates the ribbons, starts and stops the whirlpool at speed 0.5, then works through every
/// sprite of the listener: while pick-up is on (speed ≥ 2) each one that accepts is caught at
/// its current distance and angle after its delay; otherwise held sprites are let go after
/// theirs. With no touch and no current left the whirlpool dies.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }
    let display_object = logic::display_object(world, this);

    let wind_vel = world.obj::<WhirlPoolLogic>(this).wind_vel;
    {
        let mut wind_pos_array = whirl_pool_model::whirl_pool_wind_pos_array(world, display_object);
        for wind_pos in wind_pos_array.iter_mut() {
            let draw_pos = wind_pos.draw_pos + wind_pos.draw_pos_vel * wind_vel;
            wind_pos.draw_pos = if draw_pos > MAX_DRAW_POS { 0.0 } else { draw_pos };
        }
    }

    let alpha = {
        let mut whirl_pool = world.obj_mut::<WhirlPoolLogic>(this);
        whirl_pool.wind_vel = whirl_pool.wind_vel + WIND_VEL_DECAY;
        if whirl_pool.stop_whirl_pool {
            whirl_pool.wind_vel = whirl_pool.wind_vel + STOP_WIND_VEL_DECAY;
        }
        if whirl_pool.wind_vel < 0.0 {
            whirl_pool.wind_vel = 0.0;
        }
        let wind_vel = whirl_pool.wind_vel;
        if wind_vel < ACTIVE_WIND_VEL { (wind_vel + wind_vel) * MODEL_ALPHA } else { MODEL_ALPHA }
    };
    display::object_mut(world, display_object).set_alpha(alpha);

    let (wind_vel, whirl_pool_active) = {
        let whirl_pool = world.obj::<WhirlPoolLogic>(this);
        (whirl_pool.wind_vel, whirl_pool.whirl_pool_active)
    };
    if wind_vel < ACTIVE_WIND_VEL && whirl_pool_active {
        world.obj_mut::<WhirlPoolLogic>(this).whirl_pool_active = false;
        logic::set_behavior(world, this, Some("WhirlPoolStop"));
    } else if wind_vel >= ACTIVE_WIND_VEL && !whirl_pool_active {
        world.obj_mut::<WhirlPoolLogic>(this).whirl_pool_active = true;
        logic::set_behavior(world, this, Some("WhirlPoolStart"));
    }

    {
        let mut whirl_pool = world.obj_mut::<WhirlPoolLogic>(this);
        let wind_vel = whirl_pool.wind_vel;
        if wind_vel < PICK_UP_WIND_VEL && whirl_pool.pick_up_objects_active {
            whirl_pool.pick_up_objects_active = false;
        } else if wind_vel >= PICK_UP_WIND_VEL && !whirl_pool.pick_up_objects_active {
            whirl_pool.pick_up_objects_active = true;
        }
    }

    let listener = world.obj::<WhirlPoolLogic>(this).whirl_pool_logic_listener;
    let sprites = match listener {
        Some(listener) => crate::controller::world_registry::sprite_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for sprite in sprites {
        let pick_up_objects_active = world.obj::<WhirlPoolLogic>(this).pick_up_objects_active;
        let picked_up = sprite_logic::is_picked_up_by_whirl_pool(world, sprite);

        if !pick_up_objects_active {
            if picked_up && countdown_stall(world, this, sprite) {
                world.obj_mut::<WhirlPoolLogic>(this).picked_up_logic_array.retain(|held| *held != sprite);
                crate::send::dropped_by_whirl_pool(world, Some(sprite), this);
            }
            continue;
        }

        if picked_up || !countdown_stall(world, this, sprite) {
            continue;
        }
        let accepted = crate::send::picked_up_by_whirl_pool(world, Some(sprite), this);
        if !accepted {
            continue;
        }

        // The sprite circles at its current distance and angle from the centre, its target
        // radius drawn near that distance.
        let sprite_display_object = logic::display_object(world, sprite);
        let sprite_pos = display::object(world, sprite_display_object).pos;
        let x_offset = sprite_pos.x - VORTEX_X_POS;
        let y_offset = sprite_pos.y - VORTEX_Y_POS;
        let radius = (x_offset * x_offset + y_offset * y_offset).sqrt();
        let angle = y_offset.atan2(x_offset);

        let radius_tenths = trunc_i32(radius * 10.0);
        let min_target_tenths = if radius < MAX_INNER_TARGET_RADIUS { radius_tenths } else { MAX_INNER_TARGET_RADIUS_TENTHS };
        let max_target_tenths = if radius > MAX_OUTER_TARGET_RADIUS { MAX_OUTER_TARGET_RADIUS_TENTHS } else { radius_tenths + 1 };
        let target_range = (max_target_tenths - min_target_tenths) as u32;

        let target_roll = world.resource_mut::<Random>().arc4random_mod(target_range);
        let angle_vel_roll = world.resource_mut::<Random>().arc4random() & 7;
        let strength_vel_roll = world.resource_mut::<Random>().arc4random_mod(31);
        {
            let target_tenths = (target_roll as i32).wrapping_add(min_target_tenths);
            let mut spin_state = sprite_logic::whirl_pool_spin_state(world, sprite);
            spin_state.radius = radius;
            spin_state.target_radius = target_tenths as f32 / 10.0;
            spin_state.angle = angle;
            spin_state.angle_vel = (angle_vel_roll + 3) as f32 / 100.0;
            spin_state.strength = 0.0;
            spin_state.strength_vel = (strength_vel_roll + 20) as f32 / 1000.0;
        }
        world.obj_mut::<WhirlPoolLogic>(this).picked_up_logic_array.push(sprite);
    }

    let (pick_up_objects_active, picked_up_count, listener) = {
        let whirl_pool = world.obj::<WhirlPoolLogic>(this);
        let picked_up_count = whirl_pool.picked_up_logic_array.len();
        (whirl_pool.pick_up_objects_active, picked_up_count, whirl_pool.whirl_pool_logic_listener)
    };
    if !pick_up_objects_active && picked_up_count == 0 {
        if let Some(listener) = listener {
            crate::controller::god_powers::on_whirl_pool_cant_pick_up_objects(world, listener, this);
        }
    }

    let (touch_active, wind_vel) = {
        let whirl_pool = world.obj::<WhirlPoolLogic>(this);
        (whirl_pool.touch_active, whirl_pool.wind_vel)
    };
    if !touch_active && wind_vel <= 0.0 {
        logic::set_behavior(world, this, Some("WhirlPoolDie"));
    } else {
        spin_sprite_logics(world, this);
        sprite_logic::on_game_frame(world, this, dt);
    }
}

/// `-[WhirlPoolLogic dropSpriteLogic:]` @ 0x10013837c
///
/// A held sprite leaving the whirlpool by itself.
pub fn drop_sprite_logic(world: &mut World, this: Entity, sprite: Entity) {
    world.obj_mut::<WhirlPoolLogic>(this).picked_up_logic_array.retain(|held| *held != sprite);
}

/// `-[WhirlPoolLogic die:::]` @ 0x100138398
///
/// Tells the listener, which releases the whirlpool.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<WhirlPoolLogic>(this).whirl_pool_logic_listener;
    if let Some(listener) = listener {
        crate::controller::god_powers::on_whirl_pool_die(world, listener, this);
    }
    ItemResult::Done
}
