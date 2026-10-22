//! `IceMonsterLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ice_island::{health_bar_logic, snow_ball_logic};
use crate::logic::island_core::sprite_logic;

/// `IceMonsterLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IceMonsterLogic {
    /// `mIceMonsterLogicListener`: `@"<IceMonsterLogicListener>"`
    pub ice_monster_logic_listener: Option<Entity>,

    /// `mStartXPos`: `f`
    pub start_x_pos: f32,

    /// `mHealth`: `f`
    pub health: f32,

    /// `mbMouthCollisionActive`: `B`
    pub mouth_collision_active: bool,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbInTransition`: `B`
    pub in_transition: bool,

    /// `mbDormant`: `B`
    pub dormant: bool,

    /// `mRechargeCount`: `i`
    pub recharge_count: i32,

    /// `mRechargeHitCount`: `i`
    pub recharge_hit_count: i32,

    /// `mStopRechargeXPos`: `f`
    pub stop_recharge_x_pos: f32,

    /// `mbIsRecharging`: `B`
    pub is_recharging: bool,

    /// `mThrowSnowBallCount`: `i`
    pub throw_snow_ball_count: i32,

    /// `mSnowBallLogicArray`: `@"NSMutableArray"`
    pub snow_ball_logic_array: Vec<Entity>,
}

pub const ICE_MONSTER_LOGIC: Class = Class {
    name: "IceMonsterLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Full health; the health bar shows `mHealth / 100`.
const MAX_HEALTH: f32 = 100.0;

/// The monster walks between these x positions.
const MIN_WALK_X_POS: f32 = 50.0;
const MAX_WALK_X_POS: f32 = 430.0;

/// The middle of the screen, which the monster faces when it stops to throw or recharge.
const SCREEN_CENTER_X_POS: f32 = 240.0;

/// Sequence-item handlers implemented by `IceMonsterLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "startTransition" => seq_start_transition(world, this, call),
        "endTransition" => seq_end_transition(world, this, call),
        "resetPosition" => seq_reset_position(world, this, call),
        "enableMouthCollision" => seq_enable_mouth_collision(world, this, call),
        "disableMouthCollision" => seq_disable_mouth_collision(world, this, call),
        "chooseIdleBehavior" => seq_choose_idle_behavior(world, this, call),
        "chooseRechargeBehavior" => seq_choose_recharge_behavior(world, this, call),
        "walk" => seq_walk(world, this, call),
        "rechargeWalk" => seq_recharge_walk(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `IceMonsterLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animThrowLeft" => anim_throw_left(world, this),
        "animThrowRight" => anim_throw_right(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[IceMonsterLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ICE_MONSTER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[IceMonsterLogic initWithDisplayObject:]` @ 0x1001831f8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IceMonsterLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, own_display_object);

    let start_x_pos = pg_engine::display::object(world, own_display_object).pos.x;
    let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
    ice_monster.start_x_pos = start_x_pos;
    ice_monster.snow_ball_logic_array = Vec::with_capacity(5);
}

/// `-[IceMonsterLogic setIceMonsterLogicListener:]` @ 0x1001832d8
pub fn set_ice_monster_logic_listener(world: &mut World, this: Entity, ice_monster_logic_listener: Entity) {
    world.obj_mut::<IceMonsterLogic>(this).ice_monster_logic_listener = Some(ice_monster_logic_listener);
}

/// `-[IceMonsterLogic enterIsland:]` @ 0x100183334
///
/// The monster comes back dormant at its start position (`IceMonsterStartEnter` when the
/// island is shown at start-up).
pub fn enter_island(world: &mut World, this: Entity, enter_island: bool) {
    let start_x_pos = {
        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        if ice_monster.active {
            return;
        }
        ice_monster.active = true;
        ice_monster.dormant = true;
        ice_monster.mouth_collision_active = false;
        ice_monster.recharge_count = 0;
        ice_monster.recharge_hit_count = 0;
        ice_monster.is_recharging = false;
        ice_monster.throw_snow_ball_count = 0;
        ice_monster.start_x_pos
    };

    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).pos.x = start_x_pos;

    let marker = pg_engine::library::display_marker(world, "MidOceanMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_after(world, marker, Some(display_object));
    }

    let behavior = if enter_island { "IceMonsterStartEnter" } else { "IceMonsterEnter" };
    pg_engine::logic::set_behavior(world, this, Some(behavior));

    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_ice_monster_visible(world, listener, this);
    }
}

/// `-[IceMonsterLogic exitIsland:]` @ 0x100183488
///
/// Calls off the battle and plays the exit; with `exitIsland` set the monster and its
/// snowballs go at once.
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    {
        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        if !ice_monster.active {
            return;
        }
        ice_monster.active = false;
    }

    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_ice_monster_cancel_battle(world, listener, this);
    }
    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::deactivate(world, health_bar);
    }

    if !exit_island {
        pg_engine::logic::set_behavior(world, this, Some("IceMonsterExit"));
        return;
    }

    loop {
        let first_snow_ball = world.obj::<IceMonsterLogic>(this).snow_ball_logic_array.first().copied();
        let Some(snow_ball) = first_snow_ball else {
            break;
        };
        remove_snow_ball(world, this, snow_ball);
    }
    pg_engine::logic::set_behavior(world, this, Some("IceMonsterDie"));
}

/// `-[IceMonsterLogic islandMinXPos]` @ 0x1001836a0
pub fn island_min_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_min_x_pos(world, listener)
}

/// `-[IceMonsterLogic islandMaxXPos]` @ 0x1001836bc
pub fn island_max_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_max_x_pos(world, listener)
}

/// `-[IceMonsterLogic pygmyLogicArray:]` @ 0x1001836d8
///
/// Forwards to the listener with `self` as the argument (the caller's is ignored).
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `-[IceMonsterLogic healthBarType]` @ 0x1001836fc
pub fn health_bar_type(_world: &mut World, _this: Entity) -> i32 {
    2
}

/// `-[IceMonsterLogic shouldHealthBarBeVisible:]` @ 0x100183704
pub fn should_health_bar_be_visible(_world: &mut World, _this: Entity, _should_health_bar_be_visible: Entity) -> bool {
    true
}

/// `-[IceMonsterLogic onSnowBallDie:]` @ 0x10018370c
pub fn on_snow_ball_die(world: &mut World, this: Entity, snow_ball: Entity) {
    remove_snow_ball(world, this, snow_ball);
}

/// `[[snowBall displayObject] removeFromDisplayGroup]; [snowBall shutdown];
/// [mSnowBallLogicArray removeObject:snowBall]`, as `onSnowBallDie:` and `exitIsland:` write it.
///
/// The array held the snowball's last reference once `shutdown` made its animation drop its
/// own: the snowball is deallocated here.
fn remove_snow_ball(world: &mut World, this: Entity, snow_ball: Entity) {
    let snow_ball_display_object = pg_engine::logic::display_object(world, snow_ball);
    pg_engine::display::remove_from_display_group(world, snow_ball_display_object);
    pg_engine::logic::shutdown(world, snow_ball);

    {
        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        ice_monster.snow_ball_logic_array.retain(|&thrown| thrown != snow_ball);
    }
    pg_engine::logic::dealloc(world, snow_ball);
}

/// `-[IceMonsterLogic onSnowBallHitPygmy:]` @ 0x100183780
///
/// Reports the monster itself, not the snowball.
pub fn on_snow_ball_hit_pygmy(world: &mut World, this: Entity, _snow_ball: Entity) {
    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_snow_ball_hit_pygmy(world, listener, this);
    }
}

/// `-[IceMonsterLogic onSnowBallMissPygmy:]` @ 0x1001837a4
///
/// Reports the monster itself, not the snowball.
pub fn on_snow_ball_miss_pygmy(world: &mut World, this: Entity, _snow_ball: Entity) {
    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_snow_ball_miss_pygmy(world, listener, this);
    }
}

/// `-[IceMonsterLogic onGameFrame:]` @ 0x1001837c8
///
/// A recharging monster regains 0.005 health per frame.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }
    let (is_recharging, health) = {
        let ice_monster = world.obj::<IceMonsterLogic>(this);
        (ice_monster.is_recharging, ice_monster.health)
    };
    if !is_recharging || !(health < MAX_HEALTH) {
        return;
    }

    world.obj_mut::<IceMonsterLogic>(this).health = health + 0.005;
    update_health_bar(world, this);
}

/// `[[mIceMonsterLogicListener healthBarLogic:self] setHealthPercent:mHealth / 100]`.
fn update_health_bar(world: &mut World, this: Entity) {
    let health_bar = listener_health_bar(world, this);
    let Some(health_bar) = health_bar else {
        return;
    };
    let health = world.obj::<IceMonsterLogic>(this).health;
    health_bar_logic::set_health_percent(world, health_bar, health / MAX_HEALTH);
}

/// `[mIceMonsterLogicListener healthBarLogic:self]`.
fn listener_health_bar(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener?;
    crate::controller::health_bar_logic(world, listener, this)
}

/// `-[IceMonsterLogic isRecharging]` @ 0x100183880
pub fn is_recharging(world: &mut World, this: Entity) -> bool {
    world.obj::<IceMonsterLogic>(this).is_recharging
}

/// `-[IceMonsterLogic isDormant]` @ 0x100183890
pub fn is_dormant(world: &mut World, this: Entity) -> bool {
    world.obj::<IceMonsterLogic>(this).dormant
}

/// `-[IceMonsterLogic screenCollisionYPos]` @ 0x1001838a0
///
/// The body's centre on screen: 15 units above the origin while dormant, 60 when awake.
pub fn screen_collision_y_pos(world: &mut World, this: Entity) -> f32 {
    let display_object = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display_object);
    let depth_factor = pg_engine::display::model::depth_factor(object.pos.z);

    let dormant = world.obj::<IceMonsterLogic>(this).dormant;
    let body_height = if dormant { 15.0 } else { 60.0 };
    depth_factor * (object.pos.y + body_height * object.scale.y)
}

/// `-[IceMonsterLogic screenCollisionRadius]` @ 0x10018393c
pub fn screen_collision_radius(world: &mut World, this: Entity) -> f32 {
    let display_object = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display_object);
    let depth_factor = pg_engine::display::model::depth_factor(object.pos.z);

    let dormant = world.obj::<IceMonsterLogic>(this).dormant;
    let body_radius = if dormant { 15.0 } else { 50.0 };
    depth_factor * (object.scale.y * body_radius)
}

/// `-[IceMonsterLogic isMouthCollisionActive]` @ 0x1001839d4
pub fn is_mouth_collision_active(world: &mut World, this: Entity) -> bool {
    world.obj::<IceMonsterLogic>(this).mouth_collision_active
}

/// `-[IceMonsterLogic screenMouthCollisionYPos]` @ 0x1001839e4
pub fn screen_mouth_collision_y_pos(world: &mut World, this: Entity) -> f32 {
    let display_object = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display_object);
    let depth_factor = pg_engine::display::model::depth_factor(object.pos.z);
    depth_factor * (object.pos.y + object.scale.y * 60.0)
}

/// `-[IceMonsterLogic screenMouthCollisionRadius]` @ 0x100183a68
pub fn screen_mouth_collision_radius(world: &mut World, this: Entity) -> f32 {
    let display_object = pg_engine::logic::display_object(world, this);
    let object = pg_engine::display::object(world, display_object);
    let depth_factor = pg_engine::display::model::depth_factor(object.pos.z);
    depth_factor * (object.scale.y * 30.0)
}

/// `-[IceMonsterLogic doDamage:]` @ 0x100183ae4
///
/// At zero health the monster is beaten: dormant again, the battle and its bar end and the
/// `ConqueredIceMonster` achievement is awarded.
pub fn do_damage(world: &mut World, this: Entity, damage: f32) {
    {
        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        ice_monster.health = ice_monster.health - damage;
    }
    update_health_bar(world, this);

    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_ice_monster_hit_by_pygmy(world, listener, this);
    }

    let health = world.obj::<IceMonsterLogic>(this).health;
    let killed = health <= 0.0;
    if !killed {
        return;
    }

    {
        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        ice_monster.dormant = true;
        ice_monster.mouth_collision_active = false;
        ice_monster.recharge_count = 0;
        ice_monster.recharge_hit_count = 0;
        ice_monster.is_recharging = false;
        ice_monster.throw_snow_ball_count = 0;
    }

    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_ice_monster_killed(world, listener, this);
    }
    let health_bar = listener_health_bar(world, this);
    if let Some(health_bar) = health_bar {
        health_bar_logic::deactivate(world, health_bar);
    }
    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_achievement(world, listener, "ConqueredIceMonster", -1, true);
    }
    pg_engine::logic::set_behavior(world, this, Some("IceMonsterHitAndKilled"));
}

/// `-[IceMonsterLogic hitByPygmy:onFire:]` @ 0x100183c54
///
/// The first pygmy wakes the monster and starts the battle. Afterwards a hit is blocked for 5
/// damage (10 on fire) unless the monster is recharging, when it takes 10 (15 on fire) and the
/// second such hit makes it dizzy.
pub fn hit_by_pygmy(world: &mut World, this: Entity, _pygmy: Entity, fire: bool) {
    let (in_transition, dormant) = {
        let ice_monster = world.obj::<IceMonsterLogic>(this);
        (ice_monster.in_transition, ice_monster.dormant)
    };
    if in_transition {
        return;
    }

    if dormant {
        {
            let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
            ice_monster.dormant = false;
            ice_monster.health = MAX_HEALTH;
        }

        let health_bar = listener_health_bar(world, this);
        if let Some(health_bar) = health_bar {
            health_bar_logic::set_health_bar_logic_listener(world, health_bar, this);
            let health = world.obj::<IceMonsterLogic>(this).health;
            health_bar_logic::set_health_percent(world, health_bar, health / MAX_HEALTH);
            health_bar_logic::activate(world, health_bar);
        }

        let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
        if let Some(listener) = listener {
            crate::controller::on_ice_monster_start_battle(world, listener, this);
        }
        pg_engine::logic::set_behavior(world, this, Some("IceMonsterRise"));
        return;
    }

    let is_recharging = world.obj::<IceMonsterLogic>(this).is_recharging;
    let damage = if !is_recharging {
        pg_engine::logic::set_behavior(world, this, Some("IceMonsterBlockPygmy"));
        if fire { 10.0 } else { 5.0 }
    } else {
        let recharge_hit_count = world.obj::<IceMonsterLogic>(this).recharge_hit_count;
        let behavior = if recharge_hit_count == 1 {
            "IceMonsterHitByPygmyDizzy"
        } else {
            "IceMonsterHitByPygmy"
        };
        pg_engine::logic::set_behavior(world, this, Some(behavior));

        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        ice_monster.recharge_hit_count = ice_monster.recharge_hit_count.wrapping_add(1);
        if fire { 15.0 } else { 10.0 }
    };
    do_damage(world, this, damage);
}

/// `-[IceMonsterLogic hitByPygmyInMouth:]` @ 0x100183e1c
pub fn hit_by_pygmy_in_mouth(world: &mut World, this: Entity, _pygmy: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("IceMonsterSwallowPygmy"));
    do_damage(world, this, 20.0);
}

/// `-[IceMonsterLogic startTransition:::]` @ 0x100183e64
pub fn seq_start_transition(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<IceMonsterLogic>(this).in_transition = true;
    ItemResult::Done
}

/// `-[IceMonsterLogic endTransition:::]` @ 0x100183e7c
pub fn seq_end_transition(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<IceMonsterLogic>(this).in_transition = false;
    ItemResult::Done
}

/// `-[IceMonsterLogic resetPosition:::]` @ 0x100183e90
pub fn seq_reset_position(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let start_x_pos = world.obj::<IceMonsterLogic>(this).start_x_pos;
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display_object).pos.x = start_x_pos;
    ItemResult::Done
}

/// `-[IceMonsterLogic enableMouthCollision:::]` @ 0x100183ee0
pub fn seq_enable_mouth_collision(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<IceMonsterLogic>(this).mouth_collision_active = true;
    ItemResult::Done
}

/// `-[IceMonsterLogic disableMouthCollision:::]` @ 0x100183ef8
pub fn seq_disable_mouth_collision(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<IceMonsterLogic>(this).mouth_collision_active = false;
    ItemResult::Done
}

/// `-[IceMonsterLogic chooseIdleBehavior:::]` @ 0x100183f0c
///
/// After two volleys the monster recharges; otherwise it throws snowballs (one draw in two) or
/// walks, turning first when it does not face the chosen way (inward at the walk limits,
/// random in between).
pub fn seq_choose_idle_behavior(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let throw_snow_ball_count = world.obj::<IceMonsterLogic>(this).throw_snow_ball_count;

    let behavior = if throw_snow_ball_count == 2 {
        {
            let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
            ice_monster.is_recharging = true;
            ice_monster.recharge_hit_count = 0;
            ice_monster.throw_snow_ball_count = 0;
        }
        face_screen_center(world, display_object);
        "IceMonsterRechargeStart"
    } else {
        let throw_random = world.resource_mut::<Random>().arc4random();
        if throw_random & 1 != 0 {
            {
                let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
                ice_monster.throw_snow_ball_count = ice_monster.throw_snow_ball_count.wrapping_add(1);
            }
            face_screen_center(world, display_object);
            "IceMonsterThrowSnowBalls"
        } else {
            let direction_random = world.resource_mut::<Random>().arc4random();
            let mut object = pg_engine::display::object_mut(world, display_object);
            let x_pos = object.pos.x;
            let walk_right = if x_pos <= MIN_WALK_X_POS {
                object.pos.x = MIN_WALK_X_POS;
                true
            } else if MAX_WALK_X_POS <= x_pos {
                object.pos.x = MAX_WALK_X_POS;
                false
            } else {
                direction_random & 1 != 0
            };

            let facing_right = 0.0 < object.scale.x;
            let facing_left = object.scale.x < 0.0;
            let facing_walk_direction = if walk_right { facing_right } else { facing_left };
            if facing_walk_direction {
                "IceMonsterWalk"
            } else {
                "IceMonsterWalkTurn"
            }
        }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// Turns the monster towards the middle of the screen (`scale.x` negative on the right half).
fn face_screen_center(world: &mut World, display_object: Entity) {
    let mut object = pg_engine::display::object_mut(world, display_object);
    let x_scale = object.scale.x.abs();
    object.scale.x = x_scale;
    if SCREEN_CENTER_X_POS < object.pos.x {
        object.scale.x = -x_scale;
    }
}

/// `-[IceMonsterLogic chooseRechargeBehavior:::]` @ 0x100184140
///
/// The fifth recharge ends the recharging; before that the monster recharges again, turning
/// first when it faces away from the middle of the screen.
pub fn seq_choose_recharge_behavior(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let recharge_count = {
        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        ice_monster.recharge_count = ice_monster.recharge_count.wrapping_add(1);
        ice_monster.recharge_count
    };

    let behavior = if recharge_count == 5 {
        let mut ice_monster = world.obj_mut::<IceMonsterLogic>(this);
        ice_monster.is_recharging = false;
        ice_monster.recharge_count = 0;
        "IceMonsterRechargeExit"
    } else {
        let display_object = pg_engine::logic::display_object(world, this);
        let mut object = pg_engine::display::object_mut(world, display_object);
        let x_pos = object.pos.x;
        if x_pos <= MIN_WALK_X_POS {
            object.pos.x = MIN_WALK_X_POS;
        } else if MAX_WALK_X_POS <= x_pos {
            object.pos.x = MAX_WALK_X_POS;
        }

        // The side is decided by the position before the clamp.
        let facing_left = object.scale.x < 0.0;
        let facing_right = 0.0 < object.scale.x;
        let facing_center = if SCREEN_CENTER_X_POS < x_pos { facing_left } else { facing_right };
        if facing_center {
            "IceMonsterRecharge"
        } else {
            "IceMonsterRechargeTurn"
        }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[IceMonsterLogic throwSnowBallXOffset:yOffset:range:]` @ 0x100184274
///
/// Throws a snowball from the hand offset at a random x (0.01 steps) within `range` of the
/// first pygmy (or of the middle of the screen without pygmies).
pub fn throw_snow_ball_x_offset(world: &mut World, this: Entity, x_offset: f32, y_offset: f32, range: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    let pos = pg_engine::display::object(world, display_object).pos;

    let snow_ball_model = pg_engine::display::multi_model::spawn(world, Some("SnowBall"));
    {
        let mut object = pg_engine::display::object_mut(world, snow_ball_model);
        object.pos.x = pos.x + x_offset;
        object.pos.y = pos.y + y_offset;
        object.pos.z = pos.z;
    }
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_before(world, marker, Some(snow_ball_model));
    }

    let snow_ball = snow_ball_logic::spawn(world, snow_ball_model);
    snow_ball_logic::set_snow_ball_logic_listener(world, snow_ball, this);

    let listener = world.obj::<IceMonsterLogic>(this).ice_monster_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    let (min_target_x, max_target_x) = if pygmies.is_empty() {
        (SCREEN_CENTER_X_POS - range, range + SCREEN_CENTER_X_POS)
    } else {
        let first_pygmy_display_object = pg_engine::logic::display_object(world, pygmies[0]);
        let pygmy_x_pos = pg_engine::display::object(world, first_pygmy_display_object).pos.x;

        let mut min_target_x = pygmy_x_pos - range;
        let mut max_target_x = pygmy_x_pos + range;
        if min_target_x < 20.0 {
            min_target_x = min_target_x + (20.0 - min_target_x);
            // Original bug, kept: the shift is taken after the minimum moved, so it adds ~0.
            max_target_x = max_target_x + (20.0 - min_target_x);
        }
        if 460.0 < max_target_x {
            let overshoot = max_target_x - 460.0;
            min_target_x = min_target_x - overshoot;
            max_target_x = max_target_x - overshoot;
        }
        (min_target_x, max_target_x)
    };

    let random = world.resource_mut::<Random>().arc4random();
    let target_x = pg_engine::logic::random_scaled_time(min_target_x, max_target_x, 100.0, random);

    let throw_x_pos = pg_engine::display::object(world, display_object).pos.x + x_offset;
    {
        let mut snow_ball_state = pg_engine::logic::logic_mut(world, snow_ball);
        snow_ball_state.pos_vel.x = target_x - throw_x_pos;
        snow_ball_state.pos_vel.y = 1500.0;
        snow_ball_state.pos_vel.z = 500.0;
    }
    world.obj_mut::<IceMonsterLogic>(this).snow_ball_logic_array.push(snow_ball);
}

/// `-[IceMonsterLogic animThrowLeft]` @ 0x100184590
pub fn anim_throw_left(world: &mut World, this: Entity) {
    throw_snow_ball_x_offset(world, this, -85.0, 90.0, 100.0);
}

/// `-[IceMonsterLogic animThrowRight]` @ 0x1001845b8
pub fn anim_throw_right(world: &mut World, this: Entity) {
    throw_snow_ball_x_offset(world, this, 85.0, 90.0, 0.0);
}

/// `-[IceMonsterLogic walk:::]` @ 0x1001845dc
///
/// Walks for a random `[minTime, maxTime]` (0.01 s steps) or until the monster leaves its walk
/// range.
pub fn seq_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    let x_pos = pg_engine::display::object(world, display_object).pos.x;
    let out_of_range = x_pos < MIN_WALK_X_POS || MAX_WALK_X_POS < x_pos;
    let result = if stall_time <= 0.0 || out_of_range {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let mut state = pg_engine::logic::logic_mut(world, this);
    state.stall_time = state.stall_time - call.delta_time;
    result
}

/// `-[IceMonsterLogic rechargeWalk:::]` @ 0x100184738
///
/// Regains 0.01 health per frame. On the fourth recharge the walk ends at a random x in
/// [140, 340] (0.01 steps); it also ends when the monster leaves its walk range.
pub fn seq_recharge_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);

    let recharge_count = world.obj::<IceMonsterLogic>(this).recharge_count;
    if call.first_time && recharge_count == 4 {
        let random = world.resource_mut::<Random>().arc4random();
        let stop_x_hundredths = (random % 20001 + 14000) as i32;
        world.obj_mut::<IceMonsterLogic>(this).stop_recharge_x_pos = stop_x_hundredths as f32 / 100.0;
    }

    let health = world.obj::<IceMonsterLogic>(this).health;
    if health < MAX_HEALTH {
        world.obj_mut::<IceMonsterLogic>(this).health = health + 0.01;
        update_health_bar(world, this);
    }

    let object = pg_engine::display::object(world, display_object);
    let x_pos = object.pos.x;
    let x_scale = object.scale.x;
    let (recharge_count, stop_x_pos) = {
        let ice_monster = world.obj::<IceMonsterLogic>(this);
        (ice_monster.recharge_count, ice_monster.stop_recharge_x_pos)
    };

    let walked_left_past_stop = x_pos < stop_x_pos && x_scale < 0.0;
    let walked_right_past_stop = stop_x_pos < x_pos && 0.0 < x_scale;
    let reached_stop = recharge_count == 4 && (walked_left_past_stop || walked_right_past_stop);
    let out_of_range = x_pos < MIN_WALK_X_POS || MAX_WALK_X_POS < x_pos;
    if out_of_range || reached_stop {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[IceMonsterLogic die:::]` @ 0x1001848ec
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::remove_from_display_group(world, display_object);
    ItemResult::Done
}
