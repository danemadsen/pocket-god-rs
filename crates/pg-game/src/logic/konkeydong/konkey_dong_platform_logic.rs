//! `KonkeyDongPlatformLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::konkeydong::kong_statue_logic;

/// `KonkeyDongPlatformLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct KonkeyDongPlatformLogic {
    /// `mKonkeyDongPlatformLogicListener`: `@"<KonkeyDongPlatformLogicListener>"`
    pub konkey_dong_platform_logic_listener: Option<Entity>,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mCollisionWidth`: `f`
    pub collision_width: f32,

    /// `mBoulderXPosVel`: `f`
    pub boulder_x_pos_vel: f32,

    /// `mbStartPlatform`: `B`
    pub start_platform: bool,

    /// `mbFinalPlatform`: `B`
    pub final_platform: bool,

    /// `mbFlipBoulderXPosVel`: `B`
    pub flip_boulder_x_pos_vel: bool,
}

pub const KONKEY_DONG_PLATFORM_LOGIC: Class = Class {
    name: "KonkeyDongPlatformLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The platform's `initWithDisplayObject:` behaviour (`@"KonkeyDongPlatform1Init"`).
const PLATFORM_INIT_BEHAVIOR: &str = "KonkeyDongPlatform1Init";

/// Sequence-item handlers implemented by `KonkeyDongPlatformLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "setPlatformParams" => seq_set_platform_params(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[KonkeyDongPlatformLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&KONKEY_DONG_PLATFORM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[KonkeyDongPlatformLogic initWithDisplayObject:]` @ 0x1000cd44c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(KonkeyDongPlatformLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::set_behavior(world, this, Some(PLATFORM_INIT_BEHAVIOR));
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[KonkeyDongPlatformLogic setKonkeyDongPlatformLogicListener:]` @ 0x1000cd4cc
pub fn set_konkey_dong_platform_logic_listener(world: &mut World, this: Entity, konkey_dong_platform_logic_listener: Entity) {
    world.obj_mut::<KonkeyDongPlatformLogic>(this).konkey_dong_platform_logic_listener = Some(konkey_dong_platform_logic_listener);
}

/// `-[KonkeyDongPlatformLogic platformMinXPos]` @ 0x1000cd528
pub fn platform_min_x_pos(world: &mut World, this: Entity) -> f32 {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object(world, display_object).pos.x
}

/// `-[KonkeyDongPlatformLogic platformMaxXPos]` @ 0x1000cd55c
///
/// The left end plus the collision width along the platform's slope.
pub fn platform_max_x_pos(world: &mut World, this: Entity) -> f32 {
    let display_object = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display_object);
    let pos_x = object.pos.x;
    let angle = slope_angle(object.rot.z);

    let collision_width = world.obj::<KonkeyDongPlatformLogic>(this).collision_width;
    pos_x + angle.cos() * collision_width
}

/// `(rot.z / 180) * π` in single precision, the slope every check feeds to `tanf`/`cosf`.
fn slope_angle(rot_z: f32) -> f32 {
    (rot_z / 180.0) * 3.1415927
}

/// `-[KonkeyDongPlatformLogic setFlipBoulderXPosVel:]` @ 0x1000cd5f0
pub fn set_flip_boulder_x_pos_vel(world: &mut World, this: Entity, flip_boulder_x_pos_vel: bool) {
    world.obj_mut::<KonkeyDongPlatformLogic>(this).flip_boulder_x_pos_vel = flip_boulder_x_pos_vel;
}

/// `-[KonkeyDongPlatformLogic flipBoulderXPosVel]` @ 0x1000cd600
pub fn flip_boulder_x_pos_vel(world: &mut World, this: Entity) -> bool {
    world.obj::<KonkeyDongPlatformLogic>(this).flip_boulder_x_pos_vel
}

/// `-[KonkeyDongPlatformLogic setBoulderXPosVel:]` @ 0x1000cd610
pub fn set_boulder_x_pos_vel(world: &mut World, this: Entity, boulder_x_pos_vel: f32) {
    world.obj_mut::<KonkeyDongPlatformLogic>(this).boulder_x_pos_vel = boulder_x_pos_vel;
}

/// `-[KonkeyDongPlatformLogic boulderXPosVel]` @ 0x1000cd620
pub fn boulder_x_pos_vel(world: &mut World, this: Entity) -> f32 {
    world.obj::<KonkeyDongPlatformLogic>(this).boulder_x_pos_vel
}

/// `-[KonkeyDongPlatformLogic setStartPlatform:]` @ 0x1000cd630
pub fn set_start_platform(world: &mut World, this: Entity, start_platform: bool) {
    world.obj_mut::<KonkeyDongPlatformLogic>(this).start_platform = start_platform;
}

/// `-[KonkeyDongPlatformLogic isStartPlatform]` @ 0x1000cd640
pub fn is_start_platform(world: &mut World, this: Entity) -> bool {
    world.obj::<KonkeyDongPlatformLogic>(this).start_platform
}

/// `-[KonkeyDongPlatformLogic setFinalPlatform:]` @ 0x1000cd650
pub fn set_final_platform(world: &mut World, this: Entity, final_platform: bool) {
    world.obj_mut::<KonkeyDongPlatformLogic>(this).final_platform = final_platform;
}

/// `-[KonkeyDongPlatformLogic isFinalPlatform]` @ 0x1000cd660
pub fn is_final_platform(world: &mut World, this: Entity) -> bool {
    world.obj::<KonkeyDongPlatformLogic>(this).final_platform
}

/// `-[KonkeyDongPlatformLogic activate]` @ 0x1000cd670
pub fn activate(_world: &mut World, _this: Entity) {}

/// `-[KonkeyDongPlatformLogic deactivate]` @ 0x1000cd674
pub fn deactivate(_world: &mut World, _this: Entity) {}

/// `-[KonkeyDongPlatformLogic checkCollision:lastPos:]` @ 0x1000cd678
///
/// Lands `logic` (the pygmy) on the sloped platform of the active level: inside the platform's
/// x range, at or below the surface now and at or above it at `last_pos`. A landing stops its
/// fall and puts it on the surface.
pub fn check_collision(world: &mut World, this: Entity, check_collision: Entity, last_pos: Vec4) -> bool {
    check_surface_collision(world, this, check_collision, true, None, last_pos)
}

/// `-[KonkeyDongPlatformLogic checkBeetleCollision:radius:lastPos:]` @ 0x1000cd824
///
/// `checkCollision:lastPos:` for a beetle, which lives in the level group's space (no level
/// offset) and rests `radius` above the surface.
pub fn check_beetle_collision(world: &mut World, this: Entity, check_beetle_collision: Entity, radius: f32, last_pos: Vec4) -> bool {
    check_surface_collision(world, this, check_beetle_collision, false, Some(radius), last_pos)
}

/// `-[KonkeyDongPlatformLogic checkBoulderCollision:radius:lastPos:]` @ 0x1000cd99c
///
/// `checkCollision:lastPos:` for a boulder: the level offset as for the pygmy, plus `radius`.
pub fn check_boulder_collision(world: &mut World, this: Entity, check_boulder_collision: Entity, radius: f32, last_pos: Vec4) -> bool {
    check_surface_collision(world, this, check_boulder_collision, true, Some(radius), last_pos)
}

/// The y of the active level's group (`[[mKonkeyDongPlatformLogicListener
/// konkeyDongLevelDisplayObject] pos]`), read where the original dereferences it.
fn level_display_y_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<KonkeyDongPlatformLogic>(this).konkey_dong_platform_logic_listener;
    let level_display_object = listener.and_then(|listener| kong_statue_logic::konkey_dong_level_display_object(world, listener));
    let level_display_object = level_display_object.expect("KonkeyDongPlatformLogic: no active level group (the original dereferences [nil pos])");
    pg_engine::display::object(world, level_display_object).pos.y
}

/// The shared body of the three collision checks. The surface's y at an x is
/// `((level y + platform y) + tanf(slope) · (x − platform x)) + radius`, where the beetle check
/// has no level term (`with_level_y_pos` false) and the pygmy check no radius term. Every
/// comparison fails on NaN, as the original's branches do.
fn check_surface_collision(world: &mut World, this: Entity, logic: Entity, with_level_y_pos: bool, radius: Option<f32>, last_pos: Vec4) -> bool {
    let logic_display_object = pg_engine::logic::display_object(world, logic);
    let logic_pos = pg_engine::display::object(world, logic_display_object).pos;

    let platform_display_object = pg_engine::logic::display_object(world, this);
    let platform = pg_engine::display::object(world, platform_display_object);
    let platform_pos = platform.pos;
    let angle = slope_angle(platform.rot.z);
    let slope = angle.tan();

    let past_start = platform_pos.x <= logic_pos.x;
    if !past_start {
        return false;
    }
    let collision_width = world.obj::<KonkeyDongPlatformLogic>(this).collision_width;
    let max_x_pos = platform_pos.x + collision_width * angle.cos();
    let before_end = logic_pos.x <= max_x_pos;
    if !before_end {
        return false;
    }

    let base_y_pos = if with_level_y_pos {
        let level_y_pos = level_display_y_pos(world, this);
        level_y_pos + platform_pos.y
    } else {
        platform_pos.y
    };
    let surface_y_pos = surface_y_pos_at(base_y_pos, slope, logic_pos.x - platform_pos.x, radius);
    let at_or_below_surface = logic_pos.y <= surface_y_pos;
    if !at_or_below_surface {
        return false;
    }
    let last_surface_y_pos = surface_y_pos_at(base_y_pos, slope, last_pos.x - platform_pos.x, radius);
    let was_at_or_above_surface = last_surface_y_pos <= last_pos.y;
    if !was_at_or_above_surface {
        return false;
    }

    pg_engine::logic::logic_mut(world, logic).pos_vel.y = 0.0;
    pg_engine::display::object_mut(world, logic_display_object).pos.y = surface_y_pos;
    true
}

/// `(base_y_pos + slope · x_offset) + radius`, the radius term only when the check has one.
fn surface_y_pos_at(base_y_pos: f32, slope: f32, x_offset: f32, radius: Option<f32>) -> f32 {
    let surface_y_pos = base_y_pos + slope * x_offset;
    match radius {
        Some(radius) => surface_y_pos + radius,
        None => surface_y_pos,
    }
}

/// `-[KonkeyDongPlatformLogic onGameFrame:]` @ 0x1000cdb68
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let state = pg_engine::logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let pos = pg_engine::display::object(world, display_object).pos;
    world.obj_mut::<KonkeyDongPlatformLogic>(this).last_pos = pos;
}

/// `-[KonkeyDongPlatformLogic setPlatformParams:::]` @ 0x1000cdc2c
pub fn seq_set_platform_params(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let width = call.float("width");
    world.obj_mut::<KonkeyDongPlatformLogic>(this).collision_width = width;
    ItemResult::Done
}
