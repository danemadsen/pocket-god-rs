//! `KonkeyDongBoulderLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::konkeydong::{kong_statue_logic, konkey_dong_ladder_logic, konkey_dong_platform_logic};
use pg_engine::display::model::SPRITE_Z;

/// `KonkeyDongBoulderLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct KonkeyDongBoulderLogic {
    /// `mKonkeyDongBoulderLogicListener`: `@"<KonkeyDongBoulderLogicListener>"`
    pub konkey_dong_boulder_logic_listener: Option<Entity>,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mKonkeyDongLadderLogic`: `@"KonkeyDongLadderLogic"`
    pub konkey_dong_ladder_logic: Option<Entity>,

    /// `mDifficulty`: `f`
    pub difficulty: f32,

    /// `mbKonkeyDongPlatformCollision`: `B`
    pub konkey_dong_platform_collision: bool,

    /// `mbKonkeyDongFalling`: `B`
    pub konkey_dong_falling: bool,

    /// `mbUseNextLadder`: `B`
    pub use_next_ladder: bool,

    /// `mbEndLevel`: `B`
    pub end_level: bool,
}

pub const KONKEY_DONG_BOULDER_LOGIC: Class = Class {
    name: "KonkeyDongBoulderLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"KonkeyDongBoulderInit"`
const INIT_BEHAVIOR: &str = "KonkeyDongBoulderInit";

/// `@"KonkeyDongBoulderCrack"`
const CRACK_BEHAVIOR: &str = "KonkeyDongBoulderCrack";

/// `@"KonkeyDongBoulderDownLadder"`
const DOWN_LADDER_BEHAVIOR: &str = "KonkeyDongBoulderDownLadder";

/// The radius the boulder rests above a platform (`checkBoulderCollision:radius:lastPos:`).
const BOULDER_RADIUS: f32 = 16.0;

/// Sequence-item handlers implemented by `KonkeyDongBoulderLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "crackComplete" => seq_crack_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[KonkeyDongBoulderLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&KONKEY_DONG_BOULDER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[KonkeyDongBoulderLogic initWithDisplayObject:]` @ 0x10017e9d4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(KonkeyDongBoulderLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
}

/// `-[KonkeyDongBoulderLogic setKonkeyDongBoulderLogicListener:]` @ 0x10017ea50
pub fn set_konkey_dong_boulder_logic_listener(world: &mut World, this: Entity, konkey_dong_boulder_logic_listener: Entity) {
    world.obj_mut::<KonkeyDongBoulderLogic>(this).konkey_dong_boulder_logic_listener = Some(konkey_dong_boulder_logic_listener);
}

/// `-[KonkeyDongBoulderLogic isActive]` @ 0x10017eaac
pub fn is_active(world: &mut World, this: Entity) -> bool {
    if world.obj::<KonkeyDongBoulderLogic>(this).end_level {
        return false;
    }
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object(world, display_object).display_group.is_some()
}

/// `-[KonkeyDongBoulderLogic activateAtXPos:yPos:loopCount:]` @ 0x10017eafc
///
/// Each completed loop of the levels makes boulders roll 15% faster.
pub fn activate_at_x_pos(world: &mut World, this: Entity, activate_at_x_pos: f32, y_pos: f32, loop_count: i32) {
    {
        let mut boulder = world.obj_mut::<KonkeyDongBoulderLogic>(this);
        boulder.end_level = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let random = world.resource_mut::<Random>().arc4random();
    {
        let mut boulder = world.obj_mut::<KonkeyDongBoulderLogic>(this);
        boulder.konkey_dong_falling = false;
        boulder.konkey_dong_platform_collision = false;
        boulder.use_next_ladder = (!random & 1) != 0;
        boulder.konkey_dong_ladder_logic = None;
        boulder.difficulty = loop_count as f32 * 0.15 + 1.0;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = activate_at_x_pos;
        object.pos.y = y_pos;
        object.pos.z = SPRITE_Z;
    }
    pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
}

/// `-[KonkeyDongBoulderLogic deactivate]` @ 0x10017ebf0
pub fn deactivate(_world: &mut World, _this: Entity) {}

/// `-[KonkeyDongBoulderLogic checkCollision:lastPos:]` @ 0x10017ebf4
///
/// Whether `logic` is within 16 of the boulder.
pub fn check_collision(world: &mut World, this: Entity, check_collision: Entity, _last_pos: Vec4) -> bool {
    let logic_display_object = pg_engine::logic::display_object(world, check_collision);
    let logic_pos = pg_engine::display::object(world, logic_display_object).pos;
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display_object).pos;

    let x_offset = logic_pos.x - pos.x;
    let y_offset = logic_pos.y - pos.y;
    let distance = (x_offset * x_offset + y_offset * y_offset).sqrt();
    distance < 16.0
}

/// `-[KonkeyDongBoulderLogic endLevel]` @ 0x10017ec88
pub fn end_level(world: &mut World, this: Entity) {
    world.obj_mut::<KonkeyDongBoulderLogic>(this).end_level = true;
}

/// `-[KonkeyDongBoulderLogic crackBoulder]` @ 0x10017ec9c
pub fn crack_boulder(world: &mut World, this: Entity) {
    world.obj_mut::<KonkeyDongBoulderLogic>(this).end_level = true;
    pg_engine::logic::set_behavior(world, this, Some(CRACK_BEHAVIOR));
}

/// `-[KonkeyDongBoulderLogic crackComplete:::]` @ 0x10017ecc4
pub fn seq_crack_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[KonkeyDongBoulderLogic checkKonkeyDongCollision:]` @ 0x10017ecf8
///
/// `last_pos` is the boulder's own `mLastPos`. In order: the foot of the ladder it is rolling
/// down (land and stop), a ladder to drop down (when `mbUseNextLadder` is set or the ladder forces
/// drops; take it and stop), then the platform under it, which sets its rolling speed.
pub fn check_konkey_dong_collision(world: &mut World, this: Entity, last_pos: Vec4) {
    let display_object = pg_engine::logic::display_object(world, this);
    let listener = world.obj::<KonkeyDongBoulderLogic>(this).konkey_dong_boulder_logic_listener;
    let level_display_object = listener.and_then(|listener| kong_statue_logic::konkey_dong_level_display_object(world, listener));

    let ladder = world.obj::<KonkeyDongBoulderLogic>(this).konkey_dong_ladder_logic;
    if let Some(ladder) = ladder {
        let ladder_display_object = pg_engine::logic::display_object(world, ladder);
        let ladder_y_pos = pg_engine::display::object(world, ladder_display_object).pos.y;
        let y_pos = pg_engine::display::object(world, display_object).pos.y;
        let level_y_pos = level_y_pos(world, level_display_object);
        let foot_y_pos = level_y_pos + ladder_y_pos + 16.0;
        if y_pos > foot_y_pos {
            return;
        }

        {
            let mut boulder = world.obj_mut::<KonkeyDongBoulderLogic>(this);
            boulder.konkey_dong_ladder_logic = None;
            boulder.konkey_dong_platform_collision = false;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        {
            let mut state = pg_engine::logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        let landing_y_pos = level_y_pos + ladder_y_pos + 16.0;
        pg_engine::display::object_mut(world, display_object).pos.y = landing_y_pos;
        pg_engine::logic::set_behavior(world, this, Some(INIT_BEHAVIOR));
    }

    let ladders = match listener {
        Some(listener) => kong_statue_logic::konkey_dong_ladder_logic_array(world, listener),
        None => Vec::new(),
    };
    for ladder in ladders {
        let took_ladder = drop_down_ladder(world, this, ladder, level_display_object);
        if took_ladder {
            return;
        }
    }

    let platforms = match listener {
        Some(listener) => kong_statue_logic::konkey_dong_platform_logic_array(world, listener),
        None => Vec::new(),
    };
    let mut platform_collision = false;
    for platform in platforms {
        let landed = konkey_dong_platform_logic::check_boulder_collision(world, platform, this, BOULDER_RADIUS, last_pos);
        if !landed {
            continue;
        }
        platform_collision = true;
        if world.obj::<KonkeyDongBoulderLogic>(this).konkey_dong_platform_collision {
            break;
        }

        let boulder_x_pos_vel = konkey_dong_platform_logic::boulder_x_pos_vel(world, platform);
        let difficulty = world.obj::<KonkeyDongBoulderLogic>(this).difficulty;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = boulder_x_pos_vel * difficulty;
        if konkey_dong_platform_logic::flip_boulder_x_pos_vel(world, platform) {
            let random = world.resource_mut::<Random>().arc4random();
            if random & 1 == 0 {
                let mut state = pg_engine::logic::logic_mut(world, this);
                state.pos_vel.x = -state.pos_vel.x;
            }
        }
        break;
    }

    let platform_collision_changed = world.obj::<KonkeyDongBoulderLogic>(this).konkey_dong_platform_collision != platform_collision;
    if !platform_collision_changed {
        return;
    }
    world.obj_mut::<KonkeyDongBoulderLogic>(this).konkey_dong_platform_collision = platform_collision;
    if platform_collision {
        world.obj_mut::<KonkeyDongBoulderLogic>(this).konkey_dong_falling = false;
        let random = world.resource_mut::<Random>().arc4random();
        world.obj_mut::<KonkeyDongBoulderLogic>(this).use_next_ladder = (!random & 1) != 0;
    } else {
        let mut boulder = world.obj_mut::<KonkeyDongBoulderLogic>(this);
        if !boulder.konkey_dong_falling {
            boulder.konkey_dong_falling = true;
        }
    }
}

/// The y of the active level's group, read where the original dereferences
/// `[[mKonkeyDongBoulderLogicListener konkeyDongLevelDisplayObject] pos]`.
fn level_y_pos(world: &World, level_display_object: Option<Entity>) -> f32 {
    let level_display_object = level_display_object.expect("KonkeyDongBoulderLogic: no active level group (the original dereferences [nil pos])");
    pg_engine::display::object(world, level_display_object).pos.y
}

/// One ladder of the search in `checkKonkeyDongCollision:`: when the boulder may use it
/// (`mbUseNextLadder`, or the ladder forces drops) and sits on its top (within 10 in x and 20 in
/// y), it starts rolling down it. Returns whether it did.
fn drop_down_ladder(world: &mut World, this: Entity, ladder: Entity, level_display_object: Option<Entity>) -> bool {
    let use_next_ladder = world.obj::<KonkeyDongBoulderLogic>(this).use_next_ladder;
    if !use_next_ladder && !konkey_dong_ladder_logic::force_boulder_drop(world, ladder) {
        return false;
    }

    let ladder_display_object = pg_engine::logic::display_object(world, ladder);
    let ladder_pos = pg_engine::display::object(world, ladder_display_object).pos;
    let ladder_height = konkey_dong_ladder_logic::ladder_height(world, ladder);
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display_object).pos;
    let level_y_pos = level_y_pos(world, level_display_object);

    let x_distance = (ladder_pos.x - pos.x).abs();
    let top_y_pos = ladder_height + (level_y_pos + ladder_pos.y) + 16.0;
    let y_distance = (top_y_pos - pos.y).abs();
    let on_top = x_distance < 10.0 && y_distance < 20.0;
    if !on_top {
        return false;
    }

    world.obj_mut::<KonkeyDongBoulderLogic>(this).konkey_dong_ladder_logic = Some(ladder);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = -100.0;
    }
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = ladder_pos.x;
        object.rot.z = 0.0;
    }
    pg_engine::logic::set_behavior(world, this, Some(DOWN_LADDER_BEHAVIOR));
    true
}

/// `-[KonkeyDongBoulderLogic onGameFrame:]` @ 0x10017f21c
///
/// The boulder's own physics (gravity, motion, rolling spin) kept inside the level, its
/// collisions, and its exit once it falls below the bottom of the level group.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = pg_engine::logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let pos = pg_engine::display::object(world, display_object).pos;
    world.obj_mut::<KonkeyDongBoulderLogic>(this).last_pos = pos;

    if !world.obj::<KonkeyDongBoulderLogic>(this).end_level {
        if !world.obj::<SpriteLogic>(this).ignore_gravity {
            let gravity_x = dt * 0.0;
            let mut state = pg_engine::logic::logic_mut(world, this);
            state.pos_vel.x = gravity_x + state.pos_vel.x;
            state.pos_vel.y = dt * -3000.0 + state.pos_vel.y;
            state.pos_vel.z = gravity_x + state.pos_vel.z;
        }

        let pos_vel = pg_engine::logic::logic(world, this).pos_vel;
        {
            let mut object = pg_engine::display::object_mut(world, display_object);
            object.pos.x = object.pos.x + pos_vel.x * dt;
            object.pos.y = object.pos.y + pos_vel.y * dt;
            object.pos.z = object.pos.z + pos_vel.z * dt;
            object.rot.z = object.rot.z + pos_vel.x * -0.1;
            if object.pos.x < 0.0 {
                object.pos.x = 0.0;
            }
        }

        let level_width = active_level_width(world, this);
        {
            let mut object = pg_engine::display::object_mut(world, display_object);
            if object.pos.x > level_width {
                object.pos.x = level_width;
            }
        }

        let last_pos = world.obj::<KonkeyDongBoulderLogic>(this).last_pos;
        check_konkey_dong_collision(world, this, last_pos);
    }

    let display_group = pg_engine::display::object(world, display_object).display_group;
    let Some(display_group) = display_group else {
        return;
    };
    let group_y_pos = pg_engine::display::object(world, display_group).pos.y;
    let y_pos = pg_engine::display::object(world, display_object).pos.y;
    if group_y_pos + y_pos < -16.0 {
        pg_engine::display::remove_from_display_group(world, display_object);
    }
}

/// `[[mKonkeyDongBoulderLogicListener konkeyDongLevel] mLevelWidth]` (the float at +0x30).
fn active_level_width(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<KonkeyDongBoulderLogic>(this).konkey_dong_boulder_logic_listener;
    let listener = listener.expect("KonkeyDongBoulderLogic: no listener (the original dereferences [nil konkeyDongLevel])");
    let level_index = kong_statue_logic::konkey_dong_level(world, listener);
    let level_index = level_index.expect("KongStatueLogic::konkeyDongLevel always points into the level array");
    world.obj::<kong_statue_logic::KongStatueLogic>(listener).konkey_dong_level_array[level_index].level_width
}
