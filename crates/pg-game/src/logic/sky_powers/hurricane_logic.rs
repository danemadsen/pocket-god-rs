//! `HurricaneLogic` (superclass `SpriteLogic`).
//!
//! The display object is always a `HurricaneModel` (`PocketGodViewController(Game)::
//! startHurricane:`), whose wind records this class seeds and animates.

use std::f32::consts::PI;

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::sky_powers::hurricane_model;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::library;
use pg_engine::logic;

/// `HurricaneLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct HurricaneLogic {
    /// `mHurricaneLogicListener`: `@"<HurricaneLogicListener>"`
    pub hurricane_logic_listener: Option<Entity>,

    /// `mbStopHurricane`: `B`
    pub stop_hurricane: bool,

    /// `mbHurricaneActive`: `B`
    pub hurricane_active: bool,

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

    /// `mRainArray`: `@"NSMutableArray"`
    pub rain_array: Vec<Entity>,

    /// `mParticleArray`: `@"NSMutableArray"`
    pub particle_array: Vec<Entity>,

    /// `mPickedUpLogicArray`: `@"NSMutableArray"`
    pub picked_up_logic_array: Vec<Entity>,

    /// `mHurricaneSpinStateArray`: `[4{HurricaneSpinState="mStall"i"mRadius"f"mRadiusRot"f"mRadiusRotVel"f"mYPos"f"mYPosRot"f"mYPosRotVel"f"mAngle"f"mAngleVel"f"mStrength"f"mStrengthVel"f}]`
    pub hurricane_spin_state_array: [HurricaneSpinState; 4],
}

pub const HURRICANE_LOGIC: Class = Class {
    name: "HurricaneLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Each circling touch speeds the wind up by this much, up to the maximum.
const WIND_VEL_STEP: f32 = 0.03;
const MAX_WIND_VEL: f32 = 3.0;

/// The wind slows down by this much every frame, and by the stop decay more while stopping.
const WIND_VEL_DECAY: f32 = -0.01;
const STOP_WIND_VEL_DECAY: f32 = -0.05;

/// From this wind speed on the hurricane blows things over (and is fully visible).
const ACTIVE_WIND_VEL: f32 = 0.5;

/// From this wind speed on it lifts things into the vortex.
const PICK_UP_WIND_VEL: f32 = 2.0;

/// A wind record's segment phase wraps to 0 past this.
const MAX_DRAW_POS: f32 = 180.0;

/// The wind ribbons' share of the visibility.
const MODEL_ALPHA_SCALE: f32 = 0.5;

/// Wind records the model holds.
const WIND_POS_HEIGHT: f32 = 86.0;
const WIND_POS_VEL_Y: f32 = 1.0;
const WIND_ROT_VEL: f32 = 0.2;
const WIND_LINE_WIDTH: f32 = 2.0;

/// Texture and model id of the rain drops.
const RAINDROP_TEXTURE_ID: &str = "Raindrop";
const RAIN_MODEL_ID: &str = "Rain";
const RAIN_DROP_COUNT: usize = 10;

/// Rain drops fall slanted by this many degrees.
const RAIN_Z_ROT: f32 = 30.0;

/// Per-frame rain movement.
const RAIN_X_DRIFT: f32 = 5.0;
const RAIN_Y_FALL: f32 = -30.0;

/// A drop below this height starts again at the top of the screen.
const RAIN_BOTTOM_Y_POS: f32 = -30.0;
const RAIN_TOP_Y_POS: f32 = 320.0;

/// Rain and debris go right after this marker.
const CLOUD_FRONT_DISPLAY_MARKER_ID: &str = "CloudFrontDisplayMarker";

/// The debris swirling in the vortex, one per spin state.
const PARTICLE_TEXTURE_IDS: [&str; 4] = ["WindLeaf", "WindRock1", "WindRock2", "WindWater"];
const PARTICLE_MODEL_ID: &str = "Particle";

/// Degrees a particle turns every frame.
const PARTICLE_Z_ROT_STEP: f32 = 10.0;

/// The vertical axis of the vortex.
const VORTEX_X_POS: f32 = 240.0;

/// Objects closer to the axis are pushed out to this radius.
const MIN_SPIN_RADIUS: f32 = 20.0;

/// How far a spinning object swings in depth, as a share of its radius at full strength.
const SPIN_DEPTH_SCALE: f32 = 0.5;

/// Above this strength an object rises toward its spin height, by this share per frame.
const LIFT_STRENGTH: f32 = 0.1;
const LIFT_EASE: f32 = 0.05;
const MAX_SPIN_STRENGTH: f32 = 1.0;

/// A sprite waits a random 1–30 frames before the hurricane changes its state.
const STALL_FRAME_RANGE: u32 = 30;

/// Sequence-item handlers implemented by `HurricaneLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[HurricaneLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&HURRICANE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// A rain drop's x when it (re)appears: `(rand % 5301 − 500) / 10`.
fn random_rain_x_pos(world: &mut World) -> f32 {
    let roll = world.resource_mut::<Random>().arc4random_mod(5301);
    (roll as i32 - 500) as f32 / 10.0
}

/// `-[HurricaneLogic initWithDisplayObject:]` @ 0x100113cd8
///
/// Seeds the five wind ribbons, then creates ten rain drops and four debris particles after
/// `CloudFrontDisplayMarker` (invisible until the wind picks up).
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(HurricaneLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<HurricaneLogic>(this).wind_vel = 0.0;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).set_alpha(0.0);

    let mut wind_pos_array = {
        let wind_pos_array = hurricane_model::hurricane_wind_pos_array(world, display_object);
        wind_pos_array.clone()
    };
    for wind_pos in wind_pos_array.iter_mut() {
        let x_roll = world.resource_mut::<Random>().arc4random_mod(401);
        wind_pos.pos.x = (x_roll + 2200) as f32 / 10.0;
        wind_pos.pos.y = WIND_POS_HEIGHT;
        wind_pos.pos.z = 0.0;
        wind_pos.pos_vel.x = 0.0;
        wind_pos.pos_vel.y = WIND_POS_VEL_Y;
        wind_pos.pos_vel.z = 0.0;
        wind_pos.wind_rot = 0.0;
        wind_pos.wind_rot_vel = WIND_ROT_VEL;

        let radius_roll = world.resource_mut::<Random>().arc4random_mod(201);
        wind_pos.radius = (radius_roll + 500) as f32 / 10.0;

        let radius_vel_roll = world.resource_mut::<Random>().arc4random_mod(21);
        wind_pos.radius_vel = (radius_vel_roll + 30) as f32 / 100.0;

        let draw_pos_roll = world.resource_mut::<Random>().arc4random_mod(1801);
        wind_pos.draw_pos = draw_pos_roll as f32 / 10.0;

        let draw_pos_vel_roll = world.resource_mut::<Random>().arc4random_mod(151);
        wind_pos.draw_pos_vel = (draw_pos_vel_roll + 150) as f32 / 100.0;

        wind_pos.line_width = WIND_LINE_WIDTH;

        let segment_roll = world.resource_mut::<Random>().arc4random_mod(5);
        wind_pos.segment_count = (segment_roll + 6) as i32;
    }
    *hurricane_model::hurricane_wind_pos_array(world, display_object) = wind_pos_array;

    {
        let mut hurricane = world.obj_mut::<HurricaneLogic>(this);
        hurricane.picked_up_logic_array = Vec::with_capacity(5);
        hurricane.particle_array = Vec::with_capacity(5);
        hurricane.rain_array = Vec::with_capacity(5);
    }

    let cloud_front_marker = library::display_marker(world, CLOUD_FRONT_DISPLAY_MARKER_ID);
    let raindrop_texture = library::texture_def_id(world, RAINDROP_TEXTURE_ID);
    for _ in 0..RAIN_DROP_COUNT {
        let rain = display::model::spawn(world, Some(RAIN_MODEL_ID), raindrop_texture);
        if let Some(cloud_front_marker) = cloud_front_marker {
            display::insert_after(world, cloud_front_marker, Some(rain));
        }

        let x_pos = random_rain_x_pos(world);
        let y_roll = world.resource_mut::<Random>().arc4random_mod(5001);
        let y_pos = (y_roll + 3200) as f32 / 10.0;
        {
            let mut rain_display = display::object_mut(world, rain);
            rain_display.pos.x = x_pos;
            rain_display.pos.y = y_pos;
            rain_display.pos.z = SPRITE_Z;
        }
        world.obj_mut::<HurricaneLogic>(this).rain_array.push(rain);
    }

    for (index, texture_id) in PARTICLE_TEXTURE_IDS.iter().enumerate() {
        let texture = library::texture_def_id(world, texture_id);
        let particle = display::model::spawn(world, Some(PARTICLE_MODEL_ID), texture);
        if let Some(cloud_front_marker) = cloud_front_marker {
            display::insert_after(world, cloud_front_marker, Some(particle));
        }

        let x_roll = world.resource_mut::<Random>().arc4random_mod(801);
        let x_pos = (x_roll + 3400) as f32 / 10.0;
        let y_roll = world.resource_mut::<Random>().arc4random_mod(1201);
        let y_pos = (y_roll + 1200) as f32 / 10.0;
        {
            let mut particle_display = display::object_mut(world, particle);
            particle_display.pos.x = x_pos;
            particle_display.pos.y = y_pos;
            particle_display.pos.z = SPRITE_Z;
            particle_display.set_alpha(0.0);
        }

        let angle_roll = world.resource_mut::<Random>().arc4random() & 0x1f;
        let angle_vel_roll = world.resource_mut::<Random>().arc4random_mod(3);
        let strength_vel_roll = world.resource_mut::<Random>().arc4random_mod(31);
        {
            let mut hurricane = world.obj_mut::<HurricaneLogic>(this);
            let spin_state = &mut hurricane.hurricane_spin_state_array[index];
            spin_state.radius = x_pos - VORTEX_X_POS;
            spin_state.y_pos = y_pos;
            spin_state.angle = angle_roll as f32 / 10.0;
            spin_state.angle_vel = (angle_vel_roll + 1) as f32 / 10.0;
            spin_state.strength = MAX_SPIN_STRENGTH;
            spin_state.strength_vel = (strength_vel_roll + 20) as f32 / 1000.0;
        }
        world.obj_mut::<HurricaneLogic>(this).particle_array.push(particle);
    }
}

/// `-[HurricaneLogic setHurricaneLogicListener:]` @ 0x100114408
pub fn set_hurricane_logic_listener(world: &mut World, this: Entity, hurricane_logic_listener: Entity) {
    world.obj_mut::<HurricaneLogic>(this).hurricane_logic_listener = Some(hurricane_logic_listener);
}

/// `-[HurricaneLogic dealloc]` @ 0x1001144d4
///
/// For whoever releases the hurricane last (the view controller's `onHurricaneDie:`, right
/// after it took the hurricane out of the display tree and shut it down). Releasing the rain
/// and particle arrays frees the drops and debris that are out of the display tree (`die:::`
/// has normally freed them already), and `BCLogic`'s release of the display object frees the
/// `HurricaneModel` once it has left the tree too. The held sprites belong to the controller.
pub fn dealloc(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let (rain_array, particle_array) = {
        let mut hurricane = world.obj_mut::<HurricaneLogic>(this);
        let rain_array = std::mem::take(&mut hurricane.rain_array);
        let particle_array = std::mem::take(&mut hurricane.particle_array);
        (rain_array, particle_array)
    };

    for object in rain_array.into_iter().chain(particle_array) {
        let in_display_tree = display::object(world, object).display_group.is_some();
        if !in_display_tree {
            world.despawn(object);
        }
    }

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

/// `-[HurricaneLogic setTouchActive:]` @ 0x100114464
pub fn set_touch_active(world: &mut World, this: Entity, touch_active: bool) {
    world.obj_mut::<HurricaneLogic>(this).touch_active = touch_active;
}

/// `-[HurricaneLogic stopHurricaneNow]` @ 0x100114474
pub fn stop_hurricane_now(world: &mut World, this: Entity) {
    world.obj_mut::<HurricaneLogic>(this).stop_hurricane = true;
}

/// `-[HurricaneLogic stopHurricane]` @ 0x100114488
pub fn stop_hurricane(world: &mut World, this: Entity) {
    world.obj_mut::<HurricaneLogic>(this).stop_hurricane = true;
}

/// `-[HurricaneLogic setWindVel:]` @ 0x10011449c
///
/// The argument is ignored: every call speeds the wind up by 0.03 (at most 3) and cancels a
/// stop.
pub fn set_wind_vel(world: &mut World, this: Entity, _wind_vel: f32) {
    let mut hurricane = world.obj_mut::<HurricaneLogic>(this);
    hurricane.stop_hurricane = false;
    let wind_vel = hurricane.wind_vel + WIND_VEL_STEP;
    hurricane.wind_vel = if wind_vel > MAX_WIND_VEL { MAX_WIND_VEL } else { wind_vel };
}

/// `-[HurricaneLogic spinDisplayObject:hurricaneSpinState:]` @ 0x100114578
///
/// Moves one object along its circle around the vortex (swinging in depth around the
/// hurricane's own z), lifts it toward its spin height once it is held firmly enough and
/// advances its angle and strength.
pub fn spin_display_object(world: &mut World, this: Entity, display_object: Entity, spin_state: &mut HurricaneSpinState) {
    let mut radius = spin_state.radius;
    if radius < MIN_SPIN_RADIUS {
        spin_state.radius = MIN_SPIN_RADIUS;
        radius = MIN_SPIN_RADIUS;
    }

    let angle = spin_state.angle;
    let x_offset = radius * angle.cos();
    let depth = radius * (spin_state.strength * SPIN_DEPTH_SCALE);
    let z_offset = angle.sin() * depth;

    let hurricane_display_object = logic::display_object(world, this);
    let hurricane_z_pos = display::object(world, hurricane_display_object).pos.z;
    let strength = spin_state.strength;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = x_offset + VORTEX_X_POS;
        object.pos.z = z_offset + hurricane_z_pos;
        if strength > LIFT_STRENGTH {
            object.pos.y = object.pos.y + (spin_state.y_pos - object.pos.y) * LIFT_EASE;
        }
    }

    spin_state.angle = spin_state.angle_vel + spin_state.angle;
    let next_strength = spin_state.strength_vel + strength;
    spin_state.strength = if next_strength > MAX_SPIN_STRENGTH { MAX_SPIN_STRENGTH } else { next_strength };
}

/// `-[HurricaneLogic spinSpriteLogics]` @ 0x10011468c
pub fn spin_sprite_logics(world: &mut World, this: Entity) {
    let picked_up_logics = world.obj::<HurricaneLogic>(this).picked_up_logic_array.clone();
    for sprite in picked_up_logics {
        let sprite_display_object = logic::display_object(world, sprite);
        let mut spin_state = *sprite_logic::hurricane_spin_state(world, sprite);
        spin_display_object(world, this, sprite_display_object, &mut spin_state);
        *sprite_logic::hurricane_spin_state(world, sprite) = spin_state;
    }
}

/// `-[HurricaneLogic spinParticles]` @ 0x1001147f0
///
/// Particle `i` spins with `mHurricaneSpinStateArray[i]`.
pub fn spin_particles(world: &mut World, this: Entity) {
    let particles = world.obj::<HurricaneLogic>(this).particle_array.clone();
    for (index, particle) in particles.into_iter().enumerate() {
        let mut spin_state = world.obj::<HurricaneLogic>(this).hurricane_spin_state_array[index];
        spin_display_object(world, this, particle, &mut spin_state);
        world.obj_mut::<HurricaneLogic>(this).hurricane_spin_state_array[index] = spin_state;
    }
}

/// `-[HurricaneLogic countdownStall:]` @ 0x10011494c
///
/// Counts the sprite's random 1–30 frame delay down; true when it runs out, or at once while
/// the hurricane is stopping.
pub fn countdown_stall(world: &mut World, this: Entity, sprite: Entity) -> bool {
    let mut stall = sprite_logic::hurricane_spin_state(world, sprite).stall;
    if stall == 0 {
        let roll = world.resource_mut::<Random>().arc4random_mod(STALL_FRAME_RANGE);
        stall = roll as i32 + 1;
        sprite_logic::hurricane_spin_state(world, sprite).stall = stall;
    }

    let remaining_stall = stall - 1;
    sprite_logic::hurricane_spin_state(world, sprite).stall = remaining_stall;
    if remaining_stall == 0 {
        return true;
    }
    world.obj::<HurricaneLogic>(this).stop_hurricane
}

/// `-[HurricaneLogic hurricaneActive]` @ 0x1001149d8
///
/// Whether the wind is strong enough to pick things up.
pub fn hurricane_active(world: &mut World, this: Entity) -> bool {
    world.obj::<HurricaneLogic>(this).wind_vel >= PICK_UP_WIND_VEL
}

/// `-[HurricaneLogic windActive]` @ 0x1001149f4
pub fn wind_active(world: &mut World, this: Entity) -> bool {
    world.obj::<HurricaneLogic>(this).hurricane_active
}

/// `-[HurricaneLogic onGameFrame:]` @ 0x100114a04
///
/// Animates the wind, rain and debris, starts and stops the storm at wind speed 0.5, then
/// works through every sprite of the listener: while pick-up is on (wind ≥ 2) each one that
/// accepts is lifted into the vortex after its delay; otherwise held sprites are dropped and
/// the others blown over or released, each after its delay. With no touch and no wind left
/// the hurricane dies.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }
    let display_object = logic::display_object(world, this);

    let wind_vel = world.obj::<HurricaneLogic>(this).wind_vel;
    {
        let mut wind_pos_array = hurricane_model::hurricane_wind_pos_array(world, display_object);
        for wind_pos in wind_pos_array.iter_mut() {
            let draw_pos = wind_pos.draw_pos + wind_pos.draw_pos_vel * wind_vel;
            wind_pos.draw_pos = if draw_pos > MAX_DRAW_POS { 0.0 } else { draw_pos };
        }
    }

    let visibility = {
        let mut hurricane = world.obj_mut::<HurricaneLogic>(this);
        hurricane.wind_vel = hurricane.wind_vel + WIND_VEL_DECAY;
        if hurricane.stop_hurricane {
            hurricane.wind_vel = hurricane.wind_vel + STOP_WIND_VEL_DECAY;
        }
        if hurricane.wind_vel < 0.0 {
            hurricane.wind_vel = 0.0;
        }
        let wind_vel = hurricane.wind_vel;
        if wind_vel < ACTIVE_WIND_VEL { wind_vel + wind_vel } else { 1.0 }
    };
    display::object_mut(world, display_object).set_alpha(visibility * MODEL_ALPHA_SCALE);

    let rain_array = world.obj::<HurricaneLogic>(this).rain_array.clone();
    for rain in rain_array {
        let fallen_y_pos = {
            let mut rain_display = display::object_mut(world, rain);
            rain_display.set_alpha(visibility);
            rain_display.rot.z = RAIN_Z_ROT;
            rain_display.pos.y = rain_display.pos.y + RAIN_Y_FALL;
            rain_display.pos.x = rain_display.pos.x + RAIN_X_DRIFT;
            rain_display.pos.y
        };

        if fallen_y_pos < RAIN_BOTTOM_Y_POS {
            let x_pos = random_rain_x_pos(world);
            let mut rain_display = display::object_mut(world, rain);
            rain_display.pos.x = x_pos;
            rain_display.pos.y = RAIN_TOP_Y_POS;
        }
    }

    let particle_array = world.obj::<HurricaneLogic>(this).particle_array.clone();
    for particle in particle_array {
        let mut particle_display = display::object_mut(world, particle);
        particle_display.set_alpha(visibility);
        particle_display.rot.z = particle_display.rot.z + PARTICLE_Z_ROT_STEP;
    }

    let (wind_vel, hurricane_active) = {
        let hurricane = world.obj::<HurricaneLogic>(this);
        (hurricane.wind_vel, hurricane.hurricane_active)
    };
    if wind_vel < ACTIVE_WIND_VEL && hurricane_active {
        world.obj_mut::<HurricaneLogic>(this).hurricane_active = false;
        logic::set_behavior(world, this, Some("HurricaneStop"));
    } else if wind_vel >= ACTIVE_WIND_VEL && !hurricane_active {
        world.obj_mut::<HurricaneLogic>(this).hurricane_active = true;
        logic::set_behavior(world, this, Some("HurricaneStart"));
    }

    {
        let mut hurricane = world.obj_mut::<HurricaneLogic>(this);
        let wind_vel = hurricane.wind_vel;
        if wind_vel < PICK_UP_WIND_VEL && hurricane.pick_up_objects_active {
            hurricane.pick_up_objects_active = false;
        } else if wind_vel >= PICK_UP_WIND_VEL && !hurricane.pick_up_objects_active {
            hurricane.pick_up_objects_active = true;
        }
    }

    let listener = world.obj::<HurricaneLogic>(this).hurricane_logic_listener;
    let sprites = match listener {
        Some(listener) => crate::controller::world_registry::sprite_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for sprite in sprites {
        let pick_up_objects_active = world.obj::<HurricaneLogic>(this).pick_up_objects_active;
        let picked_up = sprite_logic::is_picked_up_by_hurricane(world, sprite);

        if pick_up_objects_active {
            if picked_up || !countdown_stall(world, this, sprite) {
                continue;
            }
            let accepted = crate::send::picked_up_by_hurricane(world, Some(sprite), this);
            if !accepted {
                continue;
            }

            // The sprite circles at its distance from the axis, starting on its own side.
            let sprite_display_object = logic::display_object(world, sprite);
            let mut radius = display::object(world, sprite_display_object).pos.x - VORTEX_X_POS;
            let mut angle = 0.0;
            if radius < 0.0 {
                radius = -radius;
                angle = PI;
            }

            let y_roll = world.resource_mut::<Random>().arc4random_mod(1201);
            let angle_vel_roll = world.resource_mut::<Random>().arc4random_mod(3);
            let strength_vel_roll = world.resource_mut::<Random>().arc4random_mod(31);
            {
                let mut spin_state = sprite_logic::hurricane_spin_state(world, sprite);
                spin_state.radius = radius;
                spin_state.y_pos = (y_roll + 1200) as f32 / 10.0;
                spin_state.angle = angle;
                spin_state.angle_vel = (angle_vel_roll + 1) as f32 / 10.0;
                spin_state.strength = 0.0;
                spin_state.strength_vel = (strength_vel_roll + 20) as f32 / 1000.0;
            }
            world.obj_mut::<HurricaneLogic>(this).picked_up_logic_array.push(sprite);
        } else if picked_up {
            if !countdown_stall(world, this, sprite) {
                continue;
            }
            world.obj_mut::<HurricaneLogic>(this).picked_up_logic_array.retain(|held| *held != sprite);
            crate::send::dropped_by_hurricane(world, Some(sprite), this);
        } else {
            let hurricane_active = world.obj::<HurricaneLogic>(this).hurricane_active;
            let blown = sprite_logic::is_blown_by_hurricane(world, sprite);
            if hurricane_active {
                if !blown && countdown_stall(world, this, sprite) {
                    crate::send::blown_by_hurricane(world, Some(sprite), this);
                }
            } else if blown && countdown_stall(world, this, sprite) {
                crate::send::unblown_by_hurricane(world, Some(sprite), this);
            }
        }
    }

    let (pick_up_objects_active, picked_up_count, listener) = {
        let hurricane = world.obj::<HurricaneLogic>(this);
        let picked_up_count = hurricane.picked_up_logic_array.len();
        (hurricane.pick_up_objects_active, picked_up_count, hurricane.hurricane_logic_listener)
    };
    if !pick_up_objects_active && picked_up_count == 0 {
        if let Some(listener) = listener {
            crate::controller::god_powers::on_hurricane_cant_pick_up_objects(world, listener, this);
        }
    }

    let (touch_active, wind_vel) = {
        let hurricane = world.obj::<HurricaneLogic>(this);
        (hurricane.touch_active, hurricane.wind_vel)
    };
    if !touch_active && wind_vel <= 0.0 {
        logic::set_behavior(world, this, Some("HurricaneDie"));
    } else {
        spin_sprite_logics(world, this);
        spin_particles(world, this);
        sprite_logic::on_game_frame(world, this, dt);
    }
}

/// `-[HurricaneLogic dropSpriteLogic:]` @ 0x1001152ec
///
/// A held sprite leaving the vortex by itself.
pub fn drop_sprite_logic(world: &mut World, this: Entity, sprite: Entity) {
    world.obj_mut::<HurricaneLogic>(this).picked_up_logic_array.retain(|held| *held != sprite);
}

/// `-[HurricaneLogic die:::]` @ 0x100115308
///
/// Frees the rain and the debris and tells the listener, which releases the hurricane.
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let rain_array = world.obj::<HurricaneLogic>(this).rain_array.clone();
    for &rain in &rain_array {
        display::remove_from_display_group(world, rain);
    }

    // `removeAllObjects` released the drops' last reference.
    world.obj_mut::<HurricaneLogic>(this).rain_array.clear();
    for rain in rain_array {
        world.despawn(rain);
    }

    let particle_array = world.obj::<HurricaneLogic>(this).particle_array.clone();
    for &particle in &particle_array {
        display::remove_from_display_group(world, particle);
    }

    // `removeAllObjects` released the particles' last reference.
    world.obj_mut::<HurricaneLogic>(this).particle_array.clear();
    for particle in particle_array {
        world.despawn(particle);
    }

    let listener = world.obj::<HurricaneLogic>(this).hurricane_logic_listener;
    if let Some(listener) = listener {
        crate::controller::god_powers::on_hurricane_die(world, listener, this);
    }
    ItemResult::Done
}
