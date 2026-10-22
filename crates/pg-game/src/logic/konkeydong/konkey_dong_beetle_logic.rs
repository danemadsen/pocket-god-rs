//! `KonkeyDongBeetleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::konkeydong::{kong_statue_logic, konkey_dong_ladder_logic, konkey_dong_platform_logic};

/// `KonkeyDongBeetleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct KonkeyDongBeetleLogic {
    /// `mKonkeyDongBeetleLogicListener`: `@"<KonkeyDongBeetleLogicListener>"`
    pub konkey_dong_beetle_logic_listener: Option<Entity>,

    /// `mStartPos`: `{?="x"f"y"f"z"f"w"f}`
    pub start_pos: Vec4,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mKonkeyDongPlatformLogic`: `@"KonkeyDongPlatformLogic"`
    pub konkey_dong_platform_logic: Option<Entity>,

    /// `mKonkeyDongLadderLogic`: `@"KonkeyDongLadderLogic"`
    pub konkey_dong_ladder_logic: Option<Entity>,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,

    /// `mSkipRollCount`: `i`
    pub skip_roll_count: i32,

    /// `mSkipLadderCount`: `i`
    pub skip_ladder_count: i32,

    /// `mLoopUnlock`: `i`
    pub loop_unlock: i32,

    /// `mbEndLevel`: `B`
    pub end_level: bool,

    /// `mbRolledUp`: `B`
    pub rolled_up: bool,

    /// `mbPlatformSideCollision`: `B`
    pub platform_side_collision: bool,

    /// `mbKonkeyDongPlatformCollision`: `B`
    pub konkey_dong_platform_collision: bool,

    /// `mbMovingUpLadder`: `B`
    pub moving_up_ladder: bool,

    /// `mbUseNextLadder`: `B`
    pub use_next_ladder: bool,
}

pub const KONKEY_DONG_BEETLE_LOGIC: Class = Class {
    name: "KonkeyDongBeetleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"KonkeyDongBeetleIdle"`
const IDLE_BEHAVIOR: &str = "KonkeyDongBeetleIdle";

/// `@"KonkeyDongBeetleLocked"`
const LOCKED_BEHAVIOR: &str = "KonkeyDongBeetleLocked";

/// `@"KonkeyDongBeetleSmashed"`
const SMASHED_BEHAVIOR: &str = "KonkeyDongBeetleSmashed";

/// `@"KonkeyDongBeetleChasePygmy"`
const CHASE_PYGMY_BEHAVIOR: &str = "KonkeyDongBeetleChasePygmy";

/// `@"KonkeyDongBeetleWalk"`
const WALK_BEHAVIOR: &str = "KonkeyDongBeetleWalk";

/// `@"KonkeyDongBeetleRollWalk"`
const ROLL_WALK_BEHAVIOR: &str = "KonkeyDongBeetleRollWalk";

/// `@"KonkeyDongBeetleTurn"`
const TURN_BEHAVIOR: &str = "KonkeyDongBeetleTurn";

/// `@"KonkeyDongBeetleRollTurn"`
const ROLL_TURN_BEHAVIOR: &str = "KonkeyDongBeetleRollTurn";

/// `@"KonkeyDongBeetleUpLadder"`
const UP_LADDER_BEHAVIOR: &str = "KonkeyDongBeetleUpLadder";

/// `@"KonkeyDongBeetleDownLadder"`
const DOWN_LADDER_BEHAVIOR: &str = "KonkeyDongBeetleDownLadder";

/// `@"KonkeyDongBeetleLeaveLadder"`
const LEAVE_LADDER_BEHAVIOR: &str = "KonkeyDongBeetleLeaveLadder";

/// `@"KonkeyDongBeetleRollUp"`
const ROLL_UP_BEHAVIOR: &str = "KonkeyDongBeetleRollUp";

/// `DAT_10029d410`: the ladder speed, indexed by `!mbMovingUpLadder` (up, down).
const LADDER_Y_POS_VEL: [f32; 2] = [100.0, -100.0];

/// The radius the beetle rests above a platform (`checkBeetleCollision:radius:lastPos:`).
const BEETLE_RADIUS: f32 = 16.0;

/// Sequence-item handlers implemented by `KonkeyDongBeetleLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "whichWayDoIGo" => seq_which_way_do_i_go(world, this, call),
        "walk" => seq_walk(world, this, call),
        "rollAfterPygmy" => seq_roll_after_pygmy(world, this, call),
        "smashComplete" => seq_smash_complete(world, this, call),
        "rollUpComplete" => seq_roll_up_complete(world, this, call),
        "startLadder" => seq_start_ladder(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[KonkeyDongBeetleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&KONKEY_DONG_BEETLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[KonkeyDongBeetleLogic initWithDisplayObject:]` @ 0x1000c5bf8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(KonkeyDongBeetleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    let start_pos = pg_engine::display::object(world, display_object).pos;
    world.obj_mut::<KonkeyDongBeetleLogic>(this).start_pos = start_pos;
    pg_engine::logic::set_behavior(world, this, Some(IDLE_BEHAVIOR));
}

/// `-[KonkeyDongBeetleLogic setKonkeyDongBeetleLogicListener:]` @ 0x1000c5ca4
pub fn set_konkey_dong_beetle_logic_listener(world: &mut World, this: Entity, konkey_dong_beetle_logic_listener: Entity) {
    world.obj_mut::<KonkeyDongBeetleLogic>(this).konkey_dong_beetle_logic_listener = Some(konkey_dong_beetle_logic_listener);
}

/// `-[KonkeyDongBeetleLogic isActive]` @ 0x1000c5d00
pub fn is_active(world: &mut World, this: Entity) -> bool {
    if world.obj::<KonkeyDongBeetleLogic>(this).end_level {
        return false;
    }
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object(world, display_object).display_group.is_some()
}

/// `-[KonkeyDongBeetleLogic isRolledUp]` @ 0x1000c5d50
pub fn is_rolled_up(world: &mut World, this: Entity) -> bool {
    world.obj::<KonkeyDongBeetleLogic>(this).rolled_up
}

/// `-[KonkeyDongBeetleLogic setLoopUnlock:]` @ 0x1000c5d60
pub fn set_loop_unlock(world: &mut World, this: Entity, loop_unlock: i32) {
    world.obj_mut::<KonkeyDongBeetleLogic>(this).loop_unlock = loop_unlock;
}

/// `-[KonkeyDongBeetleLogic loopUnlock]` @ 0x1000c5d70
pub fn loop_unlock(world: &mut World, this: Entity) -> i32 {
    world.obj::<KonkeyDongBeetleLogic>(this).loop_unlock
}

/// `-[KonkeyDongBeetleLogic activate:]` @ 0x1000c5d80
///
/// Before its `loopUnlock` loop the beetle stays locked; from then on each level start puts it
/// back at its start position, idle.
pub fn activate(world: &mut World, this: Entity, activate: i32) {
    let loop_unlock = world.obj::<KonkeyDongBeetleLogic>(this).loop_unlock;
    if activate < loop_unlock {
        world.obj_mut::<KonkeyDongBeetleLogic>(this).end_level = true;
        pg_engine::logic::set_behavior(world, this, Some(LOCKED_BEHAVIOR));
        return;
    }

    let start_pos = {
        let mut beetle = world.obj_mut::<KonkeyDongBeetleLogic>(this);
        beetle.rolled_up = false;
        beetle.end_level = false;
        beetle.skip_roll_count = 3;
        beetle.start_pos
    };
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut beetle = world.obj_mut::<KonkeyDongBeetleLogic>(this);
        beetle.konkey_dong_platform_collision = false;
        beetle.konkey_dong_ladder_logic = None;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos = start_pos;
        object.rot.z = 0.0;
    }
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    pg_engine::logic::set_behavior(world, this, Some(IDLE_BEHAVIOR));
}

/// `-[KonkeyDongBeetleLogic deactivate]` @ 0x1000c5e9c
pub fn deactivate(_world: &mut World, _this: Entity) {}

/// `-[KonkeyDongBeetleLogic checkCollision:lastPos:]` @ 0x1000c5ea0
///
/// Whether `logic` is within 16 of the beetle.
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

/// `-[KonkeyDongBeetleLogic endLevel]` @ 0x1000c5f34
pub fn end_level(world: &mut World, this: Entity) {
    world.obj_mut::<KonkeyDongBeetleLogic>(this).end_level = true;
}

/// `-[KonkeyDongBeetleLogic smashBeetle]` @ 0x1000c5f48
pub fn smash_beetle(world: &mut World, this: Entity) {
    world.obj_mut::<KonkeyDongBeetleLogic>(this).end_level = true;
    pg_engine::logic::set_behavior(world, this, Some(SMASHED_BEHAVIOR));
}

/// `-[KonkeyDongBeetleLogic jumpOnBeetle:]` @ 0x1000c5f70
pub fn jump_on_beetle(world: &mut World, this: Entity, jump_on_beetle: Entity) {
    {
        let mut beetle = world.obj_mut::<KonkeyDongBeetleLogic>(this);
        beetle.use_next_ladder = false;
        beetle.skip_ladder_count = 3;
        beetle.pygmy_logic = Some(jump_on_beetle);
    }
    pg_engine::logic::set_behavior(world, this, Some(CHASE_PYGMY_BEHAVIOR));
}

/// `-[KonkeyDongBeetleLogic whichWayDoIGo:::]` @ 0x1000c5fb0
///
/// At a platform end or level edge the beetle heads back inward; otherwise a coin flip picks the
/// direction. Facing that way already it walks, else it turns (rolled up: rolls).
pub fn seq_which_way_do_i_go(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display_object);
    let pos_x = object.pos.x;
    let scale_x = object.scale.x;

    let platform = world.obj::<KonkeyDongBeetleLogic>(this).konkey_dong_platform_logic;
    let platform_min_x_pos = match platform {
        Some(platform) => konkey_dong_platform_logic::platform_min_x_pos(world, platform),
        None => 0.0,
    };
    let left_end_x_pos = platform_min_x_pos + 40.0;
    let platform_max_x_pos = match platform {
        Some(platform) => konkey_dong_platform_logic::platform_max_x_pos(world, platform),
        None => 0.0,
    };
    let right_end_x_pos = platform_max_x_pos + -40.0;

    let random = world.resource_mut::<Random>().arc4random();
    let go_right = if pos_x == left_end_x_pos || pos_x == 0.0 { 1 } else { random & 1 };

    let level_width = active_level_width(world, this);
    let rolled_up = world.obj::<KonkeyDongBeetleLogic>(this).rolled_up;
    let turn = if pos_x == right_end_x_pos || pos_x == level_width || go_right == 0 {
        let facing_left = scale_x < 0.0;
        !facing_left
    } else {
        let facing_right = scale_x > 0.0;
        !facing_right
    };

    let behavior = match (turn, rolled_up) {
        (true, true) => ROLL_TURN_BEHAVIOR,
        (true, false) => TURN_BEHAVIOR,
        (false, true) => ROLL_WALK_BEHAVIOR,
        (false, false) => WALK_BEHAVIOR,
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[KonkeyDongBeetleLogic walk:::]` @ 0x1000c614c
///
/// Walks (rolled up: rolls, twice as fast) the way it faces for a random `[minTime, maxTime]`
/// seconds, or until it meets a platform end or the level edge.
pub fn seq_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let scale_x = pg_engine::display::object(world, display_object).scale.x;

    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;

        let speed = if world.obj::<KonkeyDongBeetleLogic>(this).rolled_up { 100.0 } else { 50.0 };
        pg_engine::logic::logic_mut(world, this).pos_vel.x = scale_x * speed;

        let skip_ladder_count = world.obj::<KonkeyDongBeetleLogic>(this).skip_ladder_count;
        if skip_ladder_count < 1 {
            let random = world.resource_mut::<Random>().arc4random();
            world.obj_mut::<KonkeyDongBeetleLogic>(this).use_next_ladder = (!random & 1) != 0;
        } else {
            world.obj_mut::<KonkeyDongBeetleLogic>(this).skip_ladder_count = skip_ladder_count - 1;
        }
    }

    let platform_side_collision = world.obj::<KonkeyDongBeetleLogic>(this).platform_side_collision;
    let mut state = pg_engine::logic::logic_mut(world, this);
    if platform_side_collision || state.stall_time <= 0.0 {
        state.pos_vel.x = 0.0;
        return ItemResult::Done;
    }
    state.stall_time = state.stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[KonkeyDongBeetleLogic rollAfterPygmy:::]` @ 0x1000c630c
///
/// Rolls toward the pygmy that jumped on it until it hits a platform end or the level edge.
pub fn seq_roll_after_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let pygmy_logic = world.obj::<KonkeyDongBeetleLogic>(this).pygmy_logic;
        let pygmy_logic = pygmy_logic.expect("KonkeyDongBeetleLogic: rollAfterPygmy without a pygmy (the original dereferences [nil pos])");
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy_logic);
        let pygmy_x_pos = pg_engine::display::object(world, pygmy_display_object).pos.x;
        let display_object = pg_engine::logic::display_object(world, this);
        let x_pos = pg_engine::display::object(world, display_object).pos.x;

        let x_pos_vel = if pygmy_x_pos - x_pos < 0.0 { -100.0 } else { 100.0 };
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_pos_vel;
    }

    if !world.obj::<KonkeyDongBeetleLogic>(this).platform_side_collision {
        return ItemResult::Pending;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<KonkeyDongBeetleLogic>(this).rolled_up = false;
    ItemResult::Done
}

/// `-[KonkeyDongBeetleLogic smashComplete:::]` @ 0x1000c640c
pub fn seq_smash_complete(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[KonkeyDongBeetleLogic rollUpComplete:::]` @ 0x1000c6414
///
/// Rolled up next to a ladder, the beetle starts down or up it.
pub fn seq_roll_up_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<KonkeyDongBeetleLogic>(this).rolled_up = true;
    if world.obj::<KonkeyDongBeetleLogic>(this).konkey_dong_ladder_logic.is_none() {
        return ItemResult::Done;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    let behavior = if world.obj::<KonkeyDongBeetleLogic>(this).moving_up_ladder {
        UP_LADDER_BEHAVIOR
    } else {
        DOWN_LADDER_BEHAVIOR
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[KonkeyDongBeetleLogic startLadder:::]` @ 0x1000c64a4
pub fn seq_start_ladder(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let moving_up_ladder = world.obj::<KonkeyDongBeetleLogic>(this).moving_up_ladder;
    let speed_index = usize::from(!moving_up_ladder);
    pg_engine::logic::logic_mut(world, this).pos_vel.y = LADDER_Y_POS_VEL[speed_index];
    ItemResult::Done
}

/// `-[KonkeyDongBeetleLogic checkKonkeyDongCollision:]` @ 0x1000c64e0
///
/// `last_pos` is the beetle's own `mLastPos`. In order: the end of the ladder being used (leave
/// it and stop), a ladder to take when `mbUseNextLadder` is set (take it and stop), then the
/// platform under the beetle.
pub fn check_konkey_dong_collision(world: &mut World, this: Entity, last_pos: Vec4) {
    let listener = world.obj::<KonkeyDongBeetleLogic>(this).konkey_dong_beetle_logic_listener;
    let level_display_object = listener.and_then(|listener| kong_statue_logic::konkey_dong_level_display_object(world, listener));

    world.obj_mut::<KonkeyDongBeetleLogic>(this).konkey_dong_platform_logic = None;

    let ladder = world.obj::<KonkeyDongBeetleLogic>(this).konkey_dong_ladder_logic;
    if let Some(ladder) = ladder {
        let left_ladder = leave_ladder_at_end(world, this, ladder, level_display_object);
        if !left_ladder {
            return;
        }
    }

    if world.obj::<KonkeyDongBeetleLogic>(this).use_next_ladder {
        let ladders = match listener {
            Some(listener) => kong_statue_logic::konkey_dong_ladder_logic_array(world, listener),
            None => Vec::new(),
        };
        for ladder in ladders {
            let took_ladder = take_ladder(world, this, ladder, level_display_object);
            if took_ladder {
                return;
            }
        }
    }

    let platforms = match listener {
        Some(listener) => kong_statue_logic::konkey_dong_platform_logic_array(world, listener),
        None => Vec::new(),
    };
    let mut platform_collision = false;
    for platform in platforms {
        let landed = konkey_dong_platform_logic::check_beetle_collision(world, platform, this, BEETLE_RADIUS, last_pos);
        if landed {
            world.obj_mut::<KonkeyDongBeetleLogic>(this).konkey_dong_platform_logic = Some(platform);
            platform_collision = true;
            break;
        }
    }

    let mut beetle = world.obj_mut::<KonkeyDongBeetleLogic>(this);
    if beetle.konkey_dong_platform_collision != platform_collision {
        beetle.konkey_dong_platform_collision = platform_collision;
    }
}

/// The y of the active level's group, read where the original dereferences
/// `[[mKonkeyDongBeetleLogicListener konkeyDongLevelDisplayObject] pos]`.
fn level_y_pos(world: &World, level_display_object: Option<Entity>) -> f32 {
    let level_display_object = level_display_object.expect("KonkeyDongBeetleLogic: no active level group (the original dereferences [nil pos])");
    pg_engine::display::object(world, level_display_object).pos.y
}

/// The ladder part of `checkKonkeyDongCollision:` while the beetle is on `ladder`: once it is
/// past the ladder's end (16 above the top going up, at most 16 above the foot going down) it
/// steps off at that end. Returns whether it did; if not, the check is over for this frame.
fn leave_ladder_at_end(world: &mut World, this: Entity, ladder: Entity, level_display_object: Option<Entity>) -> bool {
    let display_object = pg_engine::logic::display_object(world, this);
    let ladder_display_object = pg_engine::logic::display_object(world, ladder);
    let ladder_y_pos = pg_engine::display::object(world, ladder_display_object).pos.y;
    let ladder_height = konkey_dong_ladder_logic::ladder_height(world, ladder);

    let level_y_pos = level_y_pos(world, level_display_object);
    let y_pos = pg_engine::display::object(world, display_object).pos.y;
    let moving_up_ladder = world.obj::<KonkeyDongBeetleLogic>(this).moving_up_ladder;
    let foot_y_pos = level_y_pos + ladder_y_pos;
    if moving_up_ladder {
        let top_exit_y_pos = ladder_height + foot_y_pos + 16.0;
        if y_pos < top_exit_y_pos {
            return false;
        }
    } else {
        let bottom_exit_y_pos = foot_y_pos + 16.0;
        if y_pos > bottom_exit_y_pos {
            return false;
        }
    }

    {
        let mut beetle = world.obj_mut::<KonkeyDongBeetleLogic>(this);
        beetle.rolled_up = false;
        beetle.skip_roll_count = 2;
        beetle.skip_ladder_count = 3;
        beetle.use_next_ladder = false;
        beetle.konkey_dong_ladder_logic = None;
        beetle.konkey_dong_platform_collision = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let foot_y_pos = level_y_pos + ladder_y_pos;
    let moving_up_ladder = world.obj::<KonkeyDongBeetleLogic>(this).moving_up_ladder;
    let exit_y_pos = if moving_up_ladder { ladder_height + foot_y_pos } else { foot_y_pos };
    pg_engine::display::object_mut(world, display_object).pos.y = exit_y_pos + 16.0;
    pg_engine::logic::set_behavior(world, this, Some(LEAVE_LADDER_BEHAVIOR));
    true
}

/// One ladder of the `mbUseNextLadder` search in `checkKonkeyDongCollision:`: a passable ladder
/// within 10 in x whose foot (going up) or top (going down) is within 20 of the beetle becomes
/// `mKonkeyDongLadderLogic`; the beetle then lines up with it and rolls up, or (already rolled
/// up) starts along it. Returns whether the search is over.
fn take_ladder(world: &mut World, this: Entity, ladder: Entity, level_display_object: Option<Entity>) -> bool {
    if konkey_dong_ladder_logic::do_not_pass(world, ladder) {
        return false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let ladder_display_object = pg_engine::logic::display_object(world, ladder);
    let ladder_pos = pg_engine::display::object(world, ladder_display_object).pos;
    let ladder_height = konkey_dong_ladder_logic::ladder_height(world, ladder);
    let pos = pg_engine::display::object(world, display_object).pos;

    let x_distance = (ladder_pos.x - pos.x).abs();
    let in_line = x_distance < 10.0;
    if !in_line {
        return false;
    }

    let level_y_pos = level_y_pos(world, level_display_object);
    let foot_y_pos = level_y_pos + ladder_pos.y;
    let foot_distance = (foot_y_pos + 16.0 - pos.y).abs();
    let ladder_logic = if foot_distance < 20.0 {
        world.obj_mut::<KonkeyDongBeetleLogic>(this).moving_up_ladder = true;
        world.obj_mut::<KonkeyDongBeetleLogic>(this).konkey_dong_ladder_logic = Some(ladder);
        Some(ladder)
    } else {
        let top_distance = (ladder_height + foot_y_pos + 16.0 - pos.y).abs();
        if top_distance < 20.0 {
            world.obj_mut::<KonkeyDongBeetleLogic>(this).moving_up_ladder = false;
            world.obj_mut::<KonkeyDongBeetleLogic>(this).konkey_dong_ladder_logic = Some(ladder);
            Some(ladder)
        } else {
            world.obj::<KonkeyDongBeetleLogic>(this).konkey_dong_ladder_logic
        }
    };
    if ladder_logic.is_none() {
        return false;
    }

    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    pg_engine::display::object_mut(world, display_object).pos.x = ladder_pos.x;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    if !world.obj::<KonkeyDongBeetleLogic>(this).rolled_up {
        pg_engine::logic::set_behavior(world, this, Some(ROLL_UP_BEHAVIOR));
        return true;
    }

    pg_engine::display::object_mut(world, display_object).rot.z = 0.0;
    let behavior = if world.obj::<KonkeyDongBeetleLogic>(this).moving_up_ladder {
        UP_LADDER_BEHAVIOR
    } else {
        DOWN_LADDER_BEHAVIOR
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `[[mKonkeyDongBeetleLogicListener konkeyDongLevel] mLevelWidth]` (the float at +0x30).
fn active_level_width(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<KonkeyDongBeetleLogic>(this).konkey_dong_beetle_logic_listener;
    let listener = listener.expect("KonkeyDongBeetleLogic: no listener (the original dereferences [nil konkeyDongLevel])");
    let level_index = kong_statue_logic::konkey_dong_level(world, listener);
    let level_index = level_index.expect("KongStatueLogic::konkeyDongLevel always points into the level array");
    world.obj::<kong_statue_logic::KongStatueLogic>(listener).konkey_dong_level_array[level_index].level_width
}

/// `-[KonkeyDongBeetleLogic onGameFrame:]` @ 0x1000c6ab4
///
/// The beetle's own physics (gravity, motion, rolling spin), kept inside the level and the
/// platform's ends, then its collisions.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = pg_engine::logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;

    let pos = pg_engine::display::object(world, display_object).pos;
    world.obj_mut::<KonkeyDongBeetleLogic>(this).last_pos = pos;
    if world.obj::<KonkeyDongBeetleLogic>(this).end_level {
        return;
    }

    if !world.obj::<SpriteLogic>(this).ignore_gravity {
        let gravity_x = dt * 0.0;
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = gravity_x + state.pos_vel.x;
        state.pos_vel.y = dt * -3000.0 + state.pos_vel.y;
        state.pos_vel.z = gravity_x + state.pos_vel.z;
    }

    let pos_vel = pg_engine::logic::logic(world, this).pos_vel;
    let mut x_pos = {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = object.pos.x + pos_vel.x * dt;
        object.pos.y = object.pos.y + pos_vel.y * dt;
        object.pos.z = object.pos.z + pos_vel.z * dt;
        object.pos.x
    };

    if world.obj::<KonkeyDongBeetleLogic>(this).rolled_up {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.rot.z = object.rot.z + pos_vel.x * -0.1;
        x_pos = object.pos.x;
    }

    world.obj_mut::<KonkeyDongBeetleLogic>(this).platform_side_collision = false;
    if x_pos < 0.0 {
        world.obj_mut::<KonkeyDongBeetleLogic>(this).platform_side_collision = true;
        pg_engine::display::object_mut(world, display_object).pos.x = 0.0;
    }

    let level_width = active_level_width(world, this);
    if pg_engine::display::object(world, display_object).pos.x > level_width {
        world.obj_mut::<KonkeyDongBeetleLogic>(this).platform_side_collision = true;
        pg_engine::display::object_mut(world, display_object).pos.x = level_width;
    }

    let platform = world.obj::<KonkeyDongBeetleLogic>(this).konkey_dong_platform_logic;
    if let Some(platform) = platform {
        let platform_min_x_pos = konkey_dong_platform_logic::platform_min_x_pos(world, platform);
        let left_end_x_pos = platform_min_x_pos + 40.0;
        let platform_max_x_pos = konkey_dong_platform_logic::platform_max_x_pos(world, platform);
        let right_end_x_pos = platform_max_x_pos + -40.0;

        let mut x_pos = pg_engine::display::object(world, display_object).pos.x;
        if x_pos < left_end_x_pos {
            world.obj_mut::<KonkeyDongBeetleLogic>(this).platform_side_collision = true;
            pg_engine::display::object_mut(world, display_object).pos.x = left_end_x_pos;
            x_pos = left_end_x_pos;
        }
        if x_pos > right_end_x_pos {
            world.obj_mut::<KonkeyDongBeetleLogic>(this).platform_side_collision = true;
            pg_engine::display::object_mut(world, display_object).pos.x = right_end_x_pos;
        }
    }

    let last_pos = world.obj::<KonkeyDongBeetleLogic>(this).last_pos;
    check_konkey_dong_collision(world, this, last_pos);
}
