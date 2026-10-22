//! `MonkeyLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ape_monkeys::banana_logic;
use crate::logic::ape_monkeys::banana_tree_logic;
use crate::logic::ape_ritual::ape_torch_logic;
use crate::logic::island_core::coconut_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use pg_engine::display;
use pg_engine::logic;
use pg_engine::math::{length3, normalize3};
use pg_engine::touch::BcTouch;
use pg_engine::view::BcView;

/// `MonkeyLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MonkeyLogic {
    /// `mMonkeyLogicListener`: `@"<MonkeyLogicListener>"`
    pub monkey_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mLastPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_pos: Vec4,

    /// `mTiltState`: `i`
    pub tilt_state: i32,

    /// `mSlideFriction`: `f`
    pub slide_friction: f32,

    /// `mSpringDelta`: `f`
    pub spring_delta: f32,

    /// `mbPlayingCatch`: `B`
    pub playing_catch: bool,

    /// `mbWaitingToThrow`: `B`
    pub waiting_to_throw: bool,

    /// `mbEnableHump`: `B`
    pub enable_hump: bool,

    /// `mbHitByLightning`: `B`
    pub hit_by_lightning: bool,

    /// `mbDancing`: `B`
    pub dancing: bool,

    /// `mbEnableClimb`: `B`
    pub enable_climb: bool,

    /// `mbOnFire`: `B`
    pub on_fire: bool,

    /// `mbInPerch`: `B`
    pub in_perch: bool,

    /// `mbCanFallIntoPerch`: `B`
    pub can_fall_into_perch: bool,

    /// `mEatingBananaLogic`: `@"BananaLogic"`
    pub eating_banana_logic: Option<Entity>,

    /// `mBananaEatingPygmyLogic`: `@"PygmyLogic"`
    pub banana_eating_pygmy_logic: Option<Entity>,

    /// `mThrowingPygmyLogic`: `@"PygmyLogic"`
    pub throwing_pygmy_logic: Option<Entity>,

    /// `mThrowToMonkeyLogic`: `@"MonkeyLogic"`
    pub throw_to_monkey_logic: Option<Entity>,

    /// `mThrowFromMonkeyLogic`: `@"MonkeyLogic"`
    pub throw_from_monkey_logic: Option<Entity>,

    /// `mHumpingPygmyLogic`: `@"PygmyLogic"`
    pub humping_pygmy_logic: Option<Entity>,

    /// `mCharliePygmyLogic`: `@"PygmyLogic"`
    pub charlie_pygmy_logic: Option<Entity>,

    /// `mBananaTreeLogic`: `@"BananaTreeLogic"`
    pub banana_tree_logic: Option<Entity>,

    /// `mApeTorchLogic`: `@"ApeTorchLogic"`
    pub ape_torch_logic: Option<Entity>,

    /// `mHelperDisplayObject`: `@"BCMultiModel"`
    pub helper_display_object: Option<Entity>,

    /// `mHelperLogic`: `@"MultiModelLogic"`
    pub helper_logic: Option<Entity>,

    /// `mbPygmyHasPlague`: `B`
    pub pygmy_has_plague: bool,

    /// `mbOnIsland`: `B`
    pub on_island: bool,

    /// `mbEnableIslandFloorCollision`: `B`
    pub enable_island_floor_collision: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mbEnableIslandSideCollision`: `B`
    pub enable_island_side_collision: bool,

    /// `mbIslandSideCollision`: `B`
    pub island_side_collision: bool,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbDragging`: `B`
    pub dragging: bool,

    /// `mbFalling`: `B`
    pub falling: bool,

    /// `mbApplySpring`: `B`
    pub apply_spring: bool,
}

pub const MONKEY_LOGIC: Class = Class {
    name: "MonkeyLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    perform,
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

/// `mSlideFriction` (`0x3ccccccd`); the monkeys never slide, so nothing reads it.
const SLIDE_FRICTION: f32 = 0.025;

/// The monkeys' island floor (`onGameFrame:`).
const FLOOR_Y: f32 = 80.0;

/// The island's edges that stop a walking monkey (`mbEnableIslandSideCollision`).
const ISLAND_LEFT_X: f32 = 80.0;
const ISLAND_RIGHT_X: f32 = 400.0;

/// Where `enterIsland:` puts a monkey (`0x42a0000042b00000`: x 88, y 80).
const ENTER_ISLAND_X: f32 = 88.0;
const ENTER_ISLAND_Y: f32 = 80.0;

/// Tweenable channels of the monkey's animation: the pygmy it holds, the climbing helper
/// model and the banana it eats.
const PYGMY_CHANNEL: usize = 1;
const HELPER_CHANNEL: usize = 2;
const BANANA_CHANNEL: usize = 3;

/// The spring of `drag:::` (100 / 8, `docs/re/logic-catalog.md` §2.2).
const DRAG_STIFFNESS: f32 = 100.0;
const DRAG_DAMPING: f32 = 8.0;

/// `apocalypseDiseaseType` of the plague: a monkey holding such a pygmy catches it.
const PLAGUE_DISEASE_TYPE: i32 = 1;

/// `personaPersonality` of Charlie, who carries a landing monkey on his back.
const CHARLIE_PERSONALITY: i32 = 1;

/// Sequence-item handlers implemented by `MonkeyLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "checkForPlaguePygmy" => seq_check_for_plague_pygmy(world, this, call),
        "checkForDance" => seq_check_for_dance(world, this, call),
        "resetLastPos" => seq_reset_last_pos(world, this, call),
        "monkeyEnterIslandComplete" => seq_monkey_enter_island_complete(world, this, call),
        "whichWayDoIGo" => seq_which_way_do_i_go(world, this, call),
        "jumpOffTree" => seq_jump_off_tree(world, this, call),
        "jumpOffTorch" => seq_jump_off_torch(world, this, call),
        "finishEatingBanana" => seq_finish_eating_banana(world, this, call),
        "chasePygmy" => seq_chase_pygmy(world, this, call),
        "walk" => seq_walk(world, this, call),
        "idle" => seq_idle(world, this, call),
        "drag" => seq_drag(world, this, call),
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `MonkeyLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animMonkeyHelperBlank" => anim_monkey_helper_blank(world, this),
        "animThrowPygmy" => anim_throw_pygmy(world, this),
        "animPickUpPygmy" => anim_pick_up_pygmy(world, this),
        "animThrowPeel" => anim_throw_peel(world, this),
        "animFallOffIsland" => anim_fall_off_island(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[MonkeyLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&MONKEY_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mMonkeyLogicListener pygmyLogicArray:self]`: the view controller's pygmies (none for a nil
/// listener).
fn listener_pygmy_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::pygmy_logic_array(world, listener, this)
}

/// `[mMonkeyLogicListener apeTorchLogicArray:self]` (none for a nil listener).
fn listener_ape_torch_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let Some(listener) = listener else {
        return Vec::new();
    };
    crate::controller::ape_torch_logic_array(world, listener, this)
}

/// `[mMonkeyLogicListener bananaTreeLogic:self]` (nil for a nil listener).
fn listener_banana_tree_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let listener = listener?;
    crate::controller::banana_tree_logic(world, listener, this)
}

/// `[mMonkeyLogicListener islandCenterXPos]` (0 for a nil listener).
fn listener_island_center_x_pos(world: &mut World, this: Entity) -> f32 {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let Some(listener) = listener else {
        return 0.0;
    };
    crate::controller::island_center_x_pos(world, listener)
}

/// `[mMonkeyLogicListener onMonkeyDragComplete:self]`.
fn notify_monkey_drag_complete(world: &mut World, this: Entity) {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_monkey_drag_complete(world, listener, this);
    }
}

/// Whether nothing keeps the monkey from going back to `MonkeyIdle`: not struck by lightning,
/// not holding a plague pygmy, not touched, and on the island.
fn can_return_to_idle(world: &World, this: Entity) -> bool {
    let monkey = world.obj::<MonkeyLogic>(this);
    !monkey.hit_by_lightning && !monkey.pygmy_has_plague && !monkey.touch_began_inside && monkey.on_island
}

/// `-[MonkeyLogic initWithDisplayObject:]` @ 0x1000ce1d8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(MonkeyLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.tilt_state = 0;
        monkey.slide_friction = SLIDE_FRICTION;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MonkeyLogic>(this).enable_island_floor_collision = true;

    let helper_display_object = display::multi_model::spawn(world, Some("MonkeyHelper"));
    world.obj_mut::<MonkeyLogic>(this).helper_display_object = Some(helper_display_object);
    let helper_logic = logic::multi_model::spawn(world, helper_display_object);
    world.obj_mut::<MonkeyLogic>(this).helper_logic = Some(helper_logic);

    let animation = logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, Some(helper_display_object), HELPER_CHANNEL);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[MonkeyLogic setMonkeyLogicListener:]` @ 0x1000ce324
pub fn set_monkey_logic_listener(world: &mut World, this: Entity, monkey_logic_listener: Entity) {
    world.obj_mut::<MonkeyLogic>(this).monkey_logic_listener = Some(monkey_logic_listener);
}

/// `-[MonkeyLogic isActive]` @ 0x1000ce380
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<MonkeyLogic>(this).active
}

/// `-[MonkeyLogic isWaitingToThrow]` @ 0x1000ce390
pub fn is_waiting_to_throw(world: &mut World, this: Entity) -> bool {
    world.obj::<MonkeyLogic>(this).waiting_to_throw
}

/// `-[MonkeyLogic startPlayingCatch]` @ 0x1000ce3a0
pub fn start_playing_catch(world: &mut World, this: Entity) {
    world.obj_mut::<MonkeyLogic>(this).playing_catch = true;
}

/// `-[MonkeyLogic canChasePygmy]` @ 0x1000ce3b4
pub fn can_chase_pygmy(world: &mut World, this: Entity) -> bool {
    let monkey = world.obj::<MonkeyLogic>(this);
    monkey.on_island
        && !monkey.dancing
        && !monkey.hit_by_lightning
        && !monkey.pygmy_has_plague
        && !monkey.touch_began_inside
        && !monkey.playing_catch
        && monkey.eating_banana_logic.is_none()
}

/// `-[MonkeyLogic canTossPygmy]` @ 0x1000ce434
pub fn can_toss_pygmy(world: &mut World, this: Entity) -> bool {
    let monkey = world.obj::<MonkeyLogic>(this);
    monkey.on_island
        && !monkey.dancing
        && !monkey.hit_by_lightning
        && !monkey.pygmy_has_plague
        && !monkey.touch_began_inside
        && monkey.eating_banana_logic.is_none()
}

/// `-[MonkeyLogic clearThrowToMonkeyLogic]` @ 0x1000ce4a4
pub fn clear_throw_to_monkey_logic(world: &mut World, this: Entity) {
    world.obj_mut::<MonkeyLogic>(this).throw_to_monkey_logic = None;
}

/// `-[MonkeyLogic stopHumpingPygmy]` @ 0x1000ce4b4
pub fn stop_humping_pygmy(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some("MonkeyIdle"));
    world.obj_mut::<MonkeyLogic>(this).humping_pygmy_logic = None;
}

/// `-[MonkeyLogic punchedWhileHumping]` @ 0x1000ce4f4
pub fn punched_while_humping(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let scale_x = display::object(world, display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = scale_x * -400.0;
        state.pos_vel.y = 300.0;
    }

    world.obj_mut::<MonkeyLogic>(this).enable_island_side_collision = false;
    logic::set_behavior(world, this, Some("MonkeyHumpPygmyPunch"));
    world.obj_mut::<MonkeyLogic>(this).humping_pygmy_logic = None;
}

/// `-[MonkeyLogic bootedFromBananaTree]` @ 0x1000ce590
pub fn booted_from_banana_tree(world: &mut World, this: Entity) {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 200.0;
        state.pos_vel.y = 200.0;
    }
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.in_perch = false;
        monkey.banana_tree_logic = None;
    }
    logic::set_behavior(world, this, Some("MonkeyFall"));
}

/// `-[MonkeyLogic bootedFromApeTorch]` @ 0x1000ce5d8
pub fn booted_from_ape_torch(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let scale_x = display::object(world, display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = scale_x * 200.0;
        state.pos_vel.y = 200.0;
    }
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.in_perch = false;
        monkey.ape_torch_logic = None;
    }
    logic::set_behavior(world, this, Some("MonkeyFall"));
}

/// `-[MonkeyLogic bootedFromApeTorchByFire]` @ 0x1000ce670
pub fn booted_from_ape_torch_by_fire(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let scale_x = display::object(world, display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = scale_x * 200.0;
        state.pos_vel.y = 500.0;
    }
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.in_perch = false;
        monkey.ape_torch_logic = None;
    }
    logic::set_behavior(world, this, Some("MonkeyJumpOnFire"));
}

/// `-[MonkeyLogic bootedOffCharliesBack:]` @ 0x1000ce708
pub fn booted_off_charlies_back(world: &mut World, this: Entity, booted_off_charlies_back: Entity) {
    let display_object = logic::display_object(world, this);
    let charlie_display_object = logic::display_object(world, booted_off_charlies_back);
    let charlie_pos = display::object(world, charlie_display_object).pos;
    display::object_mut(world, display_object).pos = charlie_pos;

    let scale_x = display::object(world, display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = scale_x * 200.0;
        state.pos_vel.y = 200.0;
    }

    world.obj_mut::<MonkeyLogic>(this).charlie_pygmy_logic = None;
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("MonkeyFall"));
}

/// `-[MonkeyLogic stopWaitingForPygmyToss]` @ 0x1000ce800
pub fn stop_waiting_for_pygmy_toss(world: &mut World, this: Entity) {
    if can_return_to_idle(world, this) {
        logic::set_behavior(world, this, Some("MonkeyIdle"));
    }
}

/// `-[MonkeyLogic stopInteractingWithObjects]` @ 0x1000ce85c
pub fn stop_interacting_with_objects(world: &mut World, this: Entity) {
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.in_perch = false;
        monkey.waiting_to_throw = false;
        monkey.banana_eating_pygmy_logic = None;
    }

    let eating_banana = world.obj::<MonkeyLogic>(this).eating_banana_logic;
    if let Some(eating_banana) = eating_banana {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, BANANA_CHANNEL);
        banana_logic::drop_while_eating(world, eating_banana);
        world.obj_mut::<MonkeyLogic>(this).eating_banana_logic = None;
    }

    let throwing_pygmy = world.obj::<MonkeyLogic>(this).throwing_pygmy_logic;
    if let Some(throwing_pygmy) = throwing_pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::thrown_by_monkey(world, throwing_pygmy, this, None);
        world.obj_mut::<MonkeyLogic>(this).throwing_pygmy_logic = None;
    }

    let banana_tree = world.obj::<MonkeyLogic>(this).banana_tree_logic;
    if let Some(banana_tree) = banana_tree {
        banana_tree_logic::monkey_jump_off_tree(world, banana_tree, this);
        world.obj_mut::<MonkeyLogic>(this).banana_tree_logic = None;
    }

    let ape_torch = world.obj::<MonkeyLogic>(this).ape_torch_logic;
    if let Some(ape_torch) = ape_torch {
        ape_torch_logic::monkey_jump_off_torch(world, ape_torch, this);
        world.obj_mut::<MonkeyLogic>(this).ape_torch_logic = None;
    }

    let humping_pygmy = world.obj::<MonkeyLogic>(this).humping_pygmy_logic;
    if let Some(humping_pygmy) = humping_pygmy {
        crate::pygmy::unhumped_by_monkey(world, humping_pygmy, this);
        world.obj_mut::<MonkeyLogic>(this).humping_pygmy_logic = None;
    }

    let charlie_pygmy = world.obj::<MonkeyLogic>(this).charlie_pygmy_logic;
    if let Some(charlie_pygmy) = charlie_pygmy {
        crate::pygmy::monkey_booted_off_charlies_back(world, charlie_pygmy, this);
        world.obj_mut::<MonkeyLogic>(this).charlie_pygmy_logic = None;
    }
}

/// `-[MonkeyLogic stopPlayingCatch]` @ 0x1000ce9dc
pub fn stop_playing_catch(world: &mut World, this: Entity) {
    // The game goes on while any pygmy is still playing catch with a monkey.
    let pygmy_logic_array = listener_pygmy_logic_array(world, this);
    for pygmy in pygmy_logic_array {
        let playing_catch = crate::pygmy::playing_catch_with_monkey(world, pygmy);
        if playing_catch {
            return;
        }
    }

    world.obj_mut::<MonkeyLogic>(this).playing_catch = false;

    let waiting_to_throw = world.obj::<MonkeyLogic>(this).waiting_to_throw;
    if !waiting_to_throw {
        return;
    }

    stop_interacting_with_objects(world, this);
    world.obj_mut::<MonkeyLogic>(this).waiting_to_throw = false;
    if can_return_to_idle(world, this) {
        logic::set_behavior(world, this, Some("MonkeyIdle"));
    }
}

/// `-[MonkeyLogic hitByLightning]` @ 0x1000ceb9c
pub fn hit_by_lightning(world: &mut World, this: Entity) -> bool {
    let monkey = world.obj::<MonkeyLogic>(this);
    if monkey.hit_by_lightning || monkey.pygmy_has_plague {
        return false;
    }

    world.obj_mut::<MonkeyLogic>(this).hit_by_lightning = true;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
    stop_interacting_with_objects(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("MonkeyHitByLightning"));
    true
}

/// `-[MonkeyLogic startDancing]` @ 0x1000cec60
pub fn start_dancing(world: &mut World, this: Entity) {
    let monkey = world.obj::<MonkeyLogic>(this);
    let can_dance = !monkey.dancing && monkey.on_island && !monkey.hit_by_lightning && !monkey.pygmy_has_plague;
    if !can_dance {
        return;
    }

    world.obj_mut::<MonkeyLogic>(this).dancing = true;
    stop_interacting_with_objects(world, this);
    logic::set_behavior(world, this, Some("MonkeyStartDancing"));
}

/// `-[MonkeyLogic stopDancing]` @ 0x1000cecfc
pub fn stop_dancing(world: &mut World, this: Entity) {
    let dancing = world.obj::<MonkeyLogic>(this).dancing;
    if !dancing {
        return;
    }

    world.obj_mut::<MonkeyLogic>(this).dancing = false;
    logic::set_behavior(world, this, Some("MonkeyStopDancing"));
}

/// `-[MonkeyLogic pygmyIsEatingBanana:]` @ 0x1000ced2c
pub fn pygmy_is_eating_banana(world: &mut World, this: Entity, pygmy_is_eating_banana: Entity) {
    let display_object = logic::display_object(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_is_eating_banana);
    let x_pos = display::object(world, display_object).pos.x;
    let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
    let offset_x = x_pos - pygmy_x_pos;

    world.obj_mut::<MonkeyLogic>(this).banana_eating_pygmy_logic = Some(pygmy_is_eating_banana);

    // Facing away from the pygmy, the monkey turns first.
    let scale_x = display::object(world, display_object).scale.x;
    let facing_offset = offset_x * scale_x;
    let behavior = if facing_offset > 0.0 { "MonkeyChasePygmyTurn" } else { "MonkeyChasePygmy" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[MonkeyLogic waitForPygmyToss:]` @ 0x1000cee04
pub fn wait_for_pygmy_toss(world: &mut World, this: Entity, wait_for_pygmy_toss: Entity) {
    let display_object = logic::display_object(world, this);
    let thrower_display_object = logic::display_object(world, wait_for_pygmy_toss);
    let x_pos = display::object(world, display_object).pos.x;
    let scale_x = display::object(world, display_object).scale.x;
    let thrower_x_pos = display::object(world, thrower_display_object).pos.x;

    let facing_offset = (x_pos - thrower_x_pos) * scale_x;
    if facing_offset > 0.0 {
        // Original bug, kept: meant to turn the monkey towards the thrower, the facing is
        // stored back unchanged.
        display::object_mut(world, display_object).scale.x = scale_x;
    }

    let waiting_to_throw = world.obj::<MonkeyLogic>(this).waiting_to_throw;
    if waiting_to_throw {
        return;
    }

    world.obj_mut::<MonkeyLogic>(this).waiting_to_throw = true;
    logic::set_behavior(world, this, Some("MonkeyWaitForPygmyToss"));
}

/// `-[MonkeyLogic animMonkeyHelperBlank]` @ 0x1000ceef4
pub fn anim_monkey_helper_blank(world: &mut World, this: Entity) {
    let helper_logic = world.obj::<MonkeyLogic>(this).helper_logic;
    if let Some(helper_logic) = helper_logic {
        logic::set_behavior(world, helper_logic, Some("MonkeyHelperBlank"));
    }
}

/// `-[MonkeyLogic animThrowPygmy]` @ 0x1000cef18
pub fn anim_throw_pygmy(world: &mut World, this: Entity) {
    let throwing_pygmy = world.obj::<MonkeyLogic>(this).throwing_pygmy_logic;
    let Some(throwing_pygmy) = throwing_pygmy else {
        return;
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);

    let throw_to_monkey = world.obj::<MonkeyLogic>(this).throw_to_monkey_logic;
    crate::pygmy::thrown_by_monkey(world, throwing_pygmy, this, throw_to_monkey);

    let throw_to_monkey = world.obj::<MonkeyLogic>(this).throw_to_monkey_logic;
    if throw_to_monkey.is_some() {
        world.obj_mut::<MonkeyLogic>(this).throw_to_monkey_logic = None;
    }
    world.obj_mut::<MonkeyLogic>(this).throwing_pygmy_logic = None;
}

/// `-[MonkeyLogic animPickUpPygmy]` @ 0x1000cefa4
pub fn anim_pick_up_pygmy(world: &mut World, this: Entity) {
    // The pygmy the monkey chased may have left the island meanwhile.
    let pygmy_logic_array = listener_pygmy_logic_array(world, this);
    let banana_eating_pygmy = world.obj::<MonkeyLogic>(this).banana_eating_pygmy_logic;
    let still_present = banana_eating_pygmy.is_some_and(|pygmy| pygmy_logic_array.contains(&pygmy));
    let chased_pygmy = if still_present {
        banana_eating_pygmy
    } else {
        world.obj_mut::<MonkeyLogic>(this).banana_eating_pygmy_logic = None;
        None
    };

    let can_be_chased = match chased_pygmy {
        Some(chased_pygmy) => crate::pygmy::can_be_chased_by_monkey(world, chased_pygmy, this),
        None => false,
    };
    if !can_be_chased {
        return;
    }

    let banana_eating_pygmy = world.obj::<MonkeyLogic>(this).banana_eating_pygmy_logic;
    world.obj_mut::<MonkeyLogic>(this).throwing_pygmy_logic = banana_eating_pygmy;
    if let Some(banana_eating_pygmy) = banana_eating_pygmy {
        crate::pygmy::caught_by_monkey(world, banana_eating_pygmy, this);
    }
    hold_throwing_pygmy(world, this);
    world.obj_mut::<MonkeyLogic>(this).banana_eating_pygmy_logic = None;
}

/// The end of a catch (`animPickUpPygmy`, `checkToCatchPygmy`): the held pygmy is bound to the
/// pygmy channel, and the monkey notes whether it carries the plague.
fn hold_throwing_pygmy(world: &mut World, this: Entity) {
    let animation = logic::animation(world, this);
    let throwing_pygmy = world.obj::<MonkeyLogic>(this).throwing_pygmy_logic;
    let throwing_pygmy_display_object = throwing_pygmy.map(|pygmy| logic::display_object(world, pygmy));
    pg_engine::animation::set_tweenable(world, animation, throwing_pygmy_display_object, PYGMY_CHANNEL);

    let throwing_pygmy = world.obj::<MonkeyLogic>(this).throwing_pygmy_logic;
    let disease_type = match throwing_pygmy {
        Some(throwing_pygmy) => sprite_logic::apocalypse_disease_type(world, throwing_pygmy),
        None => 0,
    };
    world.obj_mut::<MonkeyLogic>(this).pygmy_has_plague = disease_type == PLAGUE_DISEASE_TYPE;
}

/// `-[MonkeyLogic animThrowPeel]` @ 0x1000cf0c4
pub fn anim_throw_peel(world: &mut World, this: Entity) {
    let eating_banana = world.obj::<MonkeyLogic>(this).eating_banana_logic;
    let Some(eating_banana) = eating_banana else {
        return;
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, BANANA_CHANNEL);
    banana_logic::finish_eating_by_monkey(world, eating_banana);
    world.obj_mut::<MonkeyLogic>(this).eating_banana_logic = None;
}

/// `-[MonkeyLogic enterIsland:]` @ 0x1000cf12c
///
/// The startup flag is unused: a monkey always climbs onto the island.
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let active = world.obj::<MonkeyLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<MonkeyLogic>(this).active = true;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.enable_island_side_collision = false;
        monkey.enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;

    // The climb's helper model starts where the monkey does.
    let pos = {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = ENTER_ISLAND_X;
        display.pos.y = ENTER_ISLAND_Y;
        display.pos
    };
    let helper_display_object = world.obj::<MonkeyLogic>(this).helper_display_object;
    if let Some(helper_display_object) = helper_display_object {
        display::object_mut(world, helper_display_object).pos = pos;
    }
    display::object_mut(world, display_object).scale.x = 1.0;
    world.obj_mut::<MonkeyLogic>(this).enable_climb = false;

    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
        display::insert_before(world, marker, helper_display_object);
    }

    logic::set_behavior(world, this, Some("MonkeyEnterIsland"));
    let helper_logic = world.obj::<MonkeyLogic>(this).helper_logic;
    if let Some(helper_logic) = helper_logic {
        logic::set_behavior(world, helper_logic, Some("MonkeyHelperClimb"));
    }
}

/// `-[MonkeyLogic exitIsland:]` @ 0x1000cf30c
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let active = world.obj::<MonkeyLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<MonkeyLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    stop_interacting_with_objects(world, this);

    let dragging = world.obj::<MonkeyLogic>(this).dragging;
    if dragging {
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.falling = true;
            monkey.dragging = false;
            monkey.apply_spring = false;
        }
        notify_monkey_drag_complete(world, this);
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MonkeyLogic>(this).enable_island_floor_collision = false;

    let behavior = if exit_island {
        "MonkeyDie"
    } else {
        let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(display_object));
        }
        "MonkeyFallIgnoreIsland"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[MonkeyLogic exitIslandNow]` @ 0x1000cf474
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[MonkeyLogic findMonkeyToTossPygmyTo]` @ 0x1000cf4ec
pub fn find_monkey_to_toss_pygmy_to(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    world.obj_mut::<MonkeyLogic>(this).throw_to_monkey_logic = None;

    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let monkey_logic_array = match listener {
        Some(listener) => crate::controller::monkey_logic_array(world, listener, this),
        None => Vec::new(),
    };

    // A monkey already waiting for a throw gets the pygmy; otherwise the farthest one.
    let mut farthest_distance = 0.0;
    for other_monkey in monkey_logic_array {
        if other_monkey == this {
            continue;
        }
        let can_toss = can_toss_pygmy(world, other_monkey);
        if !can_toss {
            continue;
        }

        let other_waiting = is_waiting_to_throw(world, other_monkey);
        if other_waiting {
            world.obj_mut::<MonkeyLogic>(this).throw_to_monkey_logic = Some(other_monkey);
            wait_for_pygmy_toss(world, other_monkey, this);
            return;
        }

        let other_display_object = logic::display_object(world, other_monkey);
        let other_x_pos = display::object(world, other_display_object).pos.x;
        let x_pos = display::object(world, display_object).pos.x;
        let distance = (other_x_pos - x_pos).abs();
        if distance > farthest_distance {
            world.obj_mut::<MonkeyLogic>(this).throw_to_monkey_logic = Some(other_monkey);
            farthest_distance = distance;
        }
    }

    let throw_to_monkey = world.obj::<MonkeyLogic>(this).throw_to_monkey_logic;
    if let Some(throw_to_monkey) = throw_to_monkey {
        wait_for_pygmy_toss(world, throw_to_monkey, this);
    }
}

/// `-[MonkeyLogic checkToCatchPygmy]` @ 0x1000cf730
pub fn check_to_catch_pygmy(world: &mut World, this: Entity) {
    let monkey = world.obj::<MonkeyLogic>(this);
    let can_catch = monkey.on_island
        && !monkey.hit_by_lightning
        && !monkey.pygmy_has_plague
        && !monkey.dancing
        && !monkey.on_fire
        && !monkey.in_perch
        && !monkey.touch_began_inside
        && monkey.throwing_pygmy_logic.is_none();
    if !can_catch {
        return;
    }

    let display_object = logic::display_object(world, this);
    let pygmy_logic_array = listener_pygmy_logic_array(world, this);
    for pygmy in pygmy_logic_array {
        let can_be_caught = crate::pygmy::can_be_caught_by_monkey(world, pygmy, this);
        if !can_be_caught {
            continue;
        }

        // Within 20 of a point 16 above the pygmy.
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let pos = display::object(world, display_object).pos;
        let distance_y = ((pos.y + 16.0) - pygmy_pos.y).abs();
        let distance_x = (pos.x - pygmy_pos.x).abs();
        let within_reach = distance_x <= 20.0 && distance_y <= 20.0;
        if !within_reach {
            continue;
        }

        stop_interacting_with_objects(world, this);
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.waiting_to_throw = true;
            monkey.throwing_pygmy_logic = Some(pygmy);
        }
        crate::pygmy::caught_by_monkey(world, pygmy, this);
        hold_throwing_pygmy(world, this);
        find_monkey_to_toss_pygmy_to(world, this);
        logic::set_behavior(world, this, Some("MonkeyCatchPygmy"));
        return;
    }
}

/// `-[MonkeyLogic checkPygmyCollision]` @ 0x1000cfa4c
pub fn check_pygmy_collision(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let falling = world.obj::<MonkeyLogic>(this).falling;
    if !falling {
        return;
    }

    // A monkey dropping onto a pygmy's head rides on its back.
    let pygmy_logic_array = listener_pygmy_logic_array(world, this);
    for pygmy in pygmy_logic_array {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_pos = display::object(world, pygmy_display_object).pos;
        let pos = display::object(world, display_object).pos;
        let distance_x = (pos.x - (pygmy_pos.x + 0.0)).abs();
        let distance_y = (pos.y - (pygmy_pos.y + 31.0)).abs();
        let y_pos_vel = logic::logic(world, this).pos_vel.y;
        let landing_on_head = distance_x < 30.0 && distance_y < 30.0 && y_pos_vel < -20.0;
        if !landing_on_head {
            continue;
        }
        let on_back = crate::pygmy::monkey_on_back(world, pygmy, this);
        if !on_back {
            continue;
        }

        let personality = crate::pygmy::persona_personality(world, pygmy);
        let behavior = if personality == CHARLIE_PERSONALITY {
            {
                let mut state = logic::logic_mut(world, this);
                state.pos_vel.x = 0.0;
                state.pos_vel.y = 0.0;
            }
            {
                let mut monkey = world.obj_mut::<MonkeyLogic>(this);
                monkey.falling = false;
                monkey.enable_climb = false;
            }
            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            {
                let mut monkey = world.obj_mut::<MonkeyLogic>(this);
                monkey.enable_island_side_collision = false;
                monkey.enable_island_floor_collision = false;
            }
            display::object_mut(world, display_object).is_selectable = false;
            world.obj_mut::<MonkeyLogic>(this).charlie_pygmy_logic = Some(pygmy);
            "MonkeyOnPygmyCharlieBack"
        } else {
            "MonkeyOnPygmyBack"
        };
        logic::set_behavior(world, this, Some(behavior));
        return;
    }
}

/// `-[MonkeyLogic checkTorchCollision]` @ 0x1000cfd34
pub fn check_torch_collision(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let monkey = world.obj::<MonkeyLogic>(this);
    let flying = monkey.falling || monkey.dragging;
    if !flying || monkey.on_fire {
        return;
    }

    // A monkey that flies into a lit torch's flame catches fire.
    let ape_torch_logic_array = listener_ape_torch_logic_array(world, this);
    for ape_torch in ape_torch_logic_array {
        let lit = ape_torch_logic::is_lit(world, ape_torch);
        if !lit {
            continue;
        }

        let torch_display_object = logic::display_object(world, ape_torch);
        let torch_pos = display::object(world, torch_display_object).pos;
        let pos = display::object(world, display_object).pos;
        let distance_x = ((pos.x + 0.0) - (torch_pos.x + 0.0)).abs();
        let below_flame = distance_x < 30.0;
        if !below_flame {
            continue;
        }

        // While dragged, the monkey's y is measured 64 lower.
        let dragging = world.obj::<MonkeyLogic>(this).dragging;
        let drag_y_offset = if dragging { -64.0 } else { 0.0 };
        let distance_y = ((drag_y_offset + pos.y) - (torch_pos.y + 90.0)).abs();
        let in_flame = distance_y < 30.0;
        if !in_flame {
            continue;
        }

        if dragging {
            {
                let mut monkey = world.obj_mut::<MonkeyLogic>(this);
                monkey.falling = true;
                monkey.dragging = false;
                monkey.apply_spring = false;
            }
            let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
            if let Some(marker) = marker {
                display::insert_before(world, marker, Some(display_object));
            }
            notify_monkey_drag_complete(world, this);
            {
                let mut monkey = world.obj_mut::<MonkeyLogic>(this);
                monkey.touch_began_inside = false;
                monkey.active_touch = None;
            }
        }

        world.obj_mut::<MonkeyLogic>(this).on_fire = true;
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 300.0;
            state.pos_vel.y = 700.0;
        }
        display::object_mut(world, display_object).rot.z = 0.0;

        // It jumps towards the island's centre.
        let island_center_x_pos = listener_island_center_x_pos(world, this);
        let x_pos = display::object(world, display_object).pos.x;
        if x_pos > island_center_x_pos {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = -state.pos_vel.x;
        }
        logic::set_behavior(world, this, Some("MonkeyJumpOnFire"));
        return;
    }
}

/// `-[MonkeyLogic refreshTiltState]` @ 0x1000d00cc
pub fn refresh_tilt_state(world: &mut World, this: Entity) {
    let ignore_gravity = world.obj::<SpriteLogic>(this).ignore_gravity;
    let monkey = world.obj::<MonkeyLogic>(this);
    let can_fall = (!ignore_gravity || monkey.in_perch) && !monkey.dragging;
    if !can_fall {
        return;
    }

    if !coconut_logic::is_island_tipped_over(world) {
        return;
    }

    world.obj_mut::<MonkeyLogic>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<MonkeyLogic>(this).tilt_state = 0;
    stop_interacting_with_objects(world, this);
    logic::set_behavior(world, this, Some("MonkeyFall"));
}

/// `-[MonkeyLogic undoTiltState]` @ 0x1000d01c0
pub fn undo_tilt_state(world: &mut World, this: Entity) {
    world.obj_mut::<MonkeyLogic>(this).tilt_state = 0;
}

/// `-[MonkeyLogic isAccelEnabled]` @ 0x1000d01d0
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
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

/// `-[MonkeyLogic onGameFrame:]` @ 0x1000d0264
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let display_object = logic::display_object(world, this);

    // Unlike the throwables' tilted gravity, x is zeroed instead of scaled.
    let accel_enabled = is_accel_enabled(world, this);
    if accel_enabled {
        let accel_force = world.resource::<BcView>().accel_force;
        let tilted = Vec4::new(-accel_force.y, accel_force.x, 0.0, 1.0);
        let normalized = normalize3(tilted);
        let gravity_y = normalized.y * 3000.0;
        let gravity_z = normalized.z * 3000.0;
        let gravity_force = Vec4::new(0.0, gravity_y, gravity_z, normalized.w);
        world.obj_mut::<SpriteLogic>(this).gravity_force = gravity_force;
        refresh_tilt_state(world, this);
    } else {
        world.obj_mut::<SpriteLogic>(this).gravity_force = coconut_logic::THROWABLE_GRAVITY_FORCE;
        undo_tilt_state(world, this);
    }

    let sprite = world.obj::<SpriteLogic>(this);
    let gravity_force = sprite.gravity_force;
    let ignore_gravity = sprite.ignore_gravity;
    let monkey = world.obj::<MonkeyLogic>(this);
    let spring_force = monkey.spring_force;
    let apply_spring = monkey.apply_spring;

    let pos_force = coconut_logic::throwable_pos_force(gravity_force, ignore_gravity, spring_force, apply_spring);
    logic::logic_mut(world, this).pos_force = pos_force;
    coconut_logic::integrate_throwable(world, this, pos_force, dt);

    // A walking monkey stops at the island's edges when it crosses one this frame.
    world.obj_mut::<MonkeyLogic>(this).island_side_collision = false;
    let enable_island_side_collision = world.obj::<MonkeyLogic>(this).enable_island_side_collision;
    if enable_island_side_collision {
        let x_pos = display::object(world, display_object).pos.x;
        let last_x_pos = world.obj::<MonkeyLogic>(this).last_pos.x;
        let side_x_pos = if x_pos < ISLAND_LEFT_X && last_x_pos >= ISLAND_LEFT_X {
            Some(ISLAND_LEFT_X)
        } else if x_pos > ISLAND_RIGHT_X && last_x_pos <= ISLAND_RIGHT_X {
            Some(ISLAND_RIGHT_X)
        } else {
            None
        };

        if let Some(side_x_pos) = side_x_pos {
            logic::logic_mut(world, this).pos_vel.x = 0.0;
            display::object_mut(world, display_object).pos.x = side_x_pos;
            world.obj_mut::<MonkeyLogic>(this).island_side_collision = true;
        }
    }

    world.obj_mut::<MonkeyLogic>(this).island_floor_collision = false;
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let mut island_min_x_pos = 0.0;
    let mut island_max_x_pos = 0.0;
    if let Some(listener) = listener {
        island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
        island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
    }

    // Landing: the step crossed the floor above the island.
    let pos = display::object(world, display_object).pos;
    let last_y_pos = world.obj::<MonkeyLogic>(this).last_pos.y;
    let within_island = pos.x >= island_min_x_pos && pos.x <= island_max_x_pos;
    let crossed_floor = last_y_pos >= FLOOR_Y && pos.y < FLOOR_Y;
    let enable_island_floor_collision = world.obj::<MonkeyLogic>(this).enable_island_floor_collision;
    let mut y_pos = pos.y;
    if within_island && crossed_floor && enable_island_floor_collision {
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            if !monkey.on_island {
                monkey.on_island = true;
                monkey.island_floor_collision = true;
                monkey.can_fall_into_perch = true;
            }
            monkey.on_fire = false;
        }
        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        {
            let mut display = display::object_mut(world, display_object);
            display.pos.y = FLOOR_Y;
            display.rot.z = 0.0;
        }
        y_pos = FLOOR_Y;
    } else {
        world.obj_mut::<MonkeyLogic>(this).on_island = false;
    }

    if coconut_logic::is_throwable_out_of_range(y_pos) {
        logic::set_behavior(world, this, Some("MonkeyDie"));
    }

    check_to_catch_pygmy(world, this);
    check_pygmy_collision(world, this);
    check_torch_collision(world, this);

    let pos = display::object(world, display_object).pos;
    world.obj_mut::<MonkeyLogic>(this).last_pos = pos;
}

/// `-[MonkeyLogic checkForPlaguePygmy:::]` @ 0x1000d0728
pub fn seq_check_for_plague_pygmy(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmy_has_plague = world.obj::<MonkeyLogic>(this).pygmy_has_plague;
    if !pygmy_has_plague {
        return ItemResult::Done;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("MonkeyCaughtPlaguePygmy"));
    ItemResult::Replaced
}

/// `-[MonkeyLogic checkForDance:::]` @ 0x1000d07a0
pub fn seq_check_for_dance(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let dance_active = match listener {
        Some(listener) => crate::controller::is_dance_active(world, listener),
        None => false,
    };
    if dance_active {
        start_dancing(world, this);
    }
    ItemResult::Done
}

/// `-[MonkeyLogic resetLastPos:::]` @ 0x1000d07f4
pub fn seq_reset_last_pos(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    world.obj_mut::<MonkeyLogic>(this).last_pos = pos;
    ItemResult::Done
}

/// `-[MonkeyLogic monkeyEnterIslandComplete:::]` @ 0x1000d0844
pub fn seq_monkey_enter_island_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.enable_island_floor_collision = true;
        monkey.enable_island_side_collision = true;
    }

    let helper_display_object = world.obj::<MonkeyLogic>(this).helper_display_object;
    if let Some(helper_display_object) = helper_display_object {
        display::remove_from_display_group(world, helper_display_object);
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    ItemResult::Done
}

/// `-[MonkeyLogic whichWayDoIGo:::]` @ 0x1000d08c0
pub fn seq_which_way_do_i_go(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let random = world.resource_mut::<Random>().arc4random();
    let display = display::object(world, display_object);
    let x_pos = display.pos.x;
    let scale_x = display.scale.x;

    // At an edge the monkey heads back inland (1 = right), elsewhere either way.
    let mut go_right = (random & 1) != 0;
    if x_pos == ISLAND_LEFT_X {
        go_right = true;
    }
    if x_pos == ISLAND_RIGHT_X {
        go_right = false;
    }

    let facing_that_way = if go_right { scale_x > 0.0 } else { scale_x < 0.0 };
    let behavior = if facing_that_way { "MonkeyWalk" } else { "MonkeyTurn" };
    logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[MonkeyLogic checkForBanana]` @ 0x1000d09ac
pub fn check_for_banana(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    let banana = match listener {
        Some(listener) => crate::controller::banana_logic(world, listener, this),
        None => None,
    };
    let Some(banana) = banana else {
        return false;
    };

    let edible = banana_logic::is_edible(world, banana);
    if !edible {
        return false;
    }

    let banana_display_object = logic::display_object(world, banana);
    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let banana_x_pos = display::object(world, banana_display_object).pos.x;
    let within_reach = (x_pos - banana_x_pos).abs() <= 16.0;
    if !within_reach {
        return false;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<MonkeyLogic>(this).eating_banana_logic = Some(banana);

    let animation = logic::animation(world, this);
    pg_engine::animation::set_tweenable(world, animation, Some(banana_display_object), BANANA_CHANNEL);
    banana_logic::start_eating_by_monkey(world, banana, this);
    logic::set_behavior(world, this, Some("MonkeyEatBanana"));
    true
}

/// `-[MonkeyLogic checkForSleepingPygmy]` @ 0x1000d0b14
pub fn check_for_sleeping_pygmy(world: &mut World, this: Entity) -> bool {
    let enable_hump = world.obj::<MonkeyLogic>(this).enable_hump;
    if !enable_hump {
        return false;
    }

    let display_object = logic::display_object(world, this);
    let pygmy_logic_array = listener_pygmy_logic_array(world, this);
    for pygmy in pygmy_logic_array {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
        let x_pos = display::object(world, display_object).pos.x;
        let offset_x = pygmy_x_pos - x_pos;
        let within_reach = offset_x.abs() < 32.0;
        if !within_reach {
            continue;
        }
        let can_be_humped = crate::pygmy::can_be_humped_by_monkey(world, pygmy, this);
        if !can_be_humped {
            continue;
        }

        // Face the pygmy.
        let facing = if offset_x < 0.0 { -1.0 } else { 1.0 };
        display::object_mut(world, display_object).scale.x = facing;
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.enable_hump = false;
            monkey.humping_pygmy_logic = Some(pygmy);
        }
        crate::pygmy::humped_by_monkey(world, pygmy, this);
        logic::set_behavior(world, this, Some("MonkeyHumpPygmy"));
        return true;
    }
    false
}

/// `-[MonkeyLogic checkForBananaTreeDuringFall]` @ 0x1000d0d64
pub fn check_for_banana_tree_during_fall(world: &mut World, this: Entity) -> bool {
    let can_fall_into_perch = world.obj::<MonkeyLogic>(this).can_fall_into_perch;
    if !can_fall_into_perch {
        return false;
    }

    let banana_tree = listener_banana_tree_logic(world, this);
    let Some(banana_tree) = banana_tree else {
        return false;
    };
    let available = banana_tree_logic::is_available_for_monkey(world, banana_tree);
    if !available {
        return false;
    }

    // The perch is 87 above the tree's origin.
    let tree_display_object = logic::display_object(world, banana_tree);
    let display_object = logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    let tree_pos = display::object(world, tree_display_object).pos;
    let distance_y = (pos.y - (tree_pos.y + 87.0)).abs();
    let distance_x = (pos.x - (tree_pos.x + 0.0)).abs();
    let within_reach = distance_x <= 16.0 && distance_y <= 16.0;
    if !within_reach {
        return false;
    }

    let y_pos_vel = logic::logic(world, this).pos_vel.y;
    let moving_down = y_pos_vel < 0.0;
    if !moving_down {
        return false;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.in_perch = true;
        monkey.falling = false;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.enable_island_side_collision = false;
        monkey.enable_island_floor_collision = false;
    }
    display::object_mut(world, display_object).is_selectable = true;
    world.obj_mut::<MonkeyLogic>(this).banana_tree_logic = Some(banana_tree);
    banana_tree_logic::monkey_land(world, banana_tree, this);
    logic::set_behavior(world, this, Some("MonkeyClimbBananaTreeComplete"));
    true
}

/// `-[MonkeyLogic checkForBananaTree]` @ 0x1000d0f2c
pub fn check_for_banana_tree(world: &mut World, this: Entity) -> bool {
    let enable_climb = world.obj::<MonkeyLogic>(this).enable_climb;
    if !enable_climb {
        return false;
    }

    let banana_tree = listener_banana_tree_logic(world, this);
    let Some(banana_tree) = banana_tree else {
        return false;
    };
    let available = banana_tree_logic::is_available_for_monkey(world, banana_tree);
    if !available {
        return false;
    }

    // Only a monkey walking right climbs the tree.
    let tree_display_object = logic::display_object(world, banana_tree);
    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let x_scale = display::object(world, display_object).scale.x;
    let tree_x_pos = display::object(world, tree_display_object).pos.x;
    let within_reach = (x_pos - tree_x_pos).abs() <= 16.0;
    if !within_reach {
        return false;
    }
    let facing_right = x_scale > 0.0;
    if !facing_right {
        return false;
    }

    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    world.obj_mut::<MonkeyLogic>(this).enable_climb = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.enable_island_side_collision = false;
        monkey.enable_island_floor_collision = false;
    }
    display::object_mut(world, display_object).is_selectable = false;
    world.obj_mut::<MonkeyLogic>(this).banana_tree_logic = Some(banana_tree);
    banana_tree_logic::start_monkey_climb(world, banana_tree, this);
    logic::set_behavior(world, this, Some("MonkeyClimbBananaTree"));
    true
}

/// `-[MonkeyLogic bananaTreeClimbComplete]` @ 0x1000d10d4
pub fn banana_tree_climb_complete(world: &mut World, this: Entity) {
    world.obj_mut::<MonkeyLogic>(this).in_perch = true;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("MonkeyClimbBananaTreeComplete"));
}

/// `-[MonkeyLogic jumpOffTree:::]` @ 0x1000d113c
pub fn seq_jump_off_tree(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.enable_island_floor_collision = true;
        monkey.can_fall_into_perch = false;
    }
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 200.0;
        state.pos_vel.y = 200.0;
    }
    world.obj_mut::<MonkeyLogic>(this).in_perch = false;

    let banana_tree = world.obj::<MonkeyLogic>(this).banana_tree_logic;
    if let Some(banana_tree) = banana_tree {
        banana_tree_logic::monkey_jump_off_tree(world, banana_tree, this);
    }
    world.obj_mut::<MonkeyLogic>(this).banana_tree_logic = None;
    ItemResult::Done
}

/// `-[MonkeyLogic checkForApeTorchDuringFall]` @ 0x1000d11d0
pub fn check_for_ape_torch_during_fall(world: &mut World, this: Entity) -> bool {
    let can_fall_into_perch = world.obj::<MonkeyLogic>(this).can_fall_into_perch;
    if !can_fall_into_perch {
        return false;
    }

    let display_object = logic::display_object(world, this);
    let ape_torch_logic_array = listener_ape_torch_logic_array(world, this);
    for ape_torch in ape_torch_logic_array {
        let available = ape_torch_logic::is_available_for_monkey(world, ape_torch);
        if !available {
            continue;
        }

        // The perch is 90 above the torch's origin.
        let torch_display_object = logic::display_object(world, ape_torch);
        let pos = display::object(world, display_object).pos;
        let torch_pos = display::object(world, torch_display_object).pos;
        let distance_y = (pos.y - (torch_pos.y + 90.0)).abs();
        let distance_x = (pos.x - (torch_pos.x + 0.0)).abs();
        let within_reach = distance_x <= 16.0 && distance_y <= 16.0;
        if !within_reach {
            continue;
        }
        let y_pos_vel = logic::logic(world, this).pos_vel.y;
        let moving_down = y_pos_vel < 0.0;
        if !moving_down {
            continue;
        }

        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.in_perch = true;
            monkey.falling = false;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.enable_island_side_collision = false;
            monkey.enable_island_floor_collision = false;
            monkey.ape_torch_logic = Some(ape_torch);
        }
        ape_torch_logic::monkey_land(world, ape_torch, this);
        display::object_mut(world, display_object).is_selectable = true;
        logic::set_behavior(world, this, Some("MonkeyClimbApeTorchComplete"));
        return true;
    }
    false
}

/// `-[MonkeyLogic checkForApeTorch]` @ 0x1000d14a8
pub fn check_for_ape_torch(world: &mut World, this: Entity) -> bool {
    let enable_climb = world.obj::<MonkeyLogic>(this).enable_climb;
    if !enable_climb {
        return false;
    }

    let display_object = logic::display_object(world, this);
    let ape_torch_logic_array = listener_ape_torch_logic_array(world, this);
    for ape_torch in ape_torch_logic_array {
        let available = ape_torch_logic::is_available_for_monkey(world, ape_torch);
        if !available {
            continue;
        }

        let torch_display_object = logic::display_object(world, ape_torch);
        let x_pos = display::object(world, display_object).pos.x;
        let torch_x_pos = display::object(world, torch_display_object).pos.x;
        let x_scale = display::object(world, display_object).scale.x;
        let island_center_x_pos = listener_island_center_x_pos(world, this);

        // Only a monkey walking away from the island's centre climbs a torch.
        let outward = if x_pos < island_center_x_pos { -1.0 } else { 1.0 };
        let within_reach = (x_pos - torch_x_pos).abs() <= 16.0;
        if !within_reach {
            continue;
        }
        let facing_outward = x_scale * outward > 0.0;
        if !facing_outward {
            continue;
        }

        {
            let mut state = logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
        }
        world.obj_mut::<MonkeyLogic>(this).enable_climb = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.enable_island_side_collision = false;
            monkey.enable_island_floor_collision = false;
        }
        display::object_mut(world, display_object).is_selectable = false;
        world.obj_mut::<MonkeyLogic>(this).ape_torch_logic = Some(ape_torch);
        ape_torch_logic::start_monkey_climb(world, ape_torch, this);
        logic::set_behavior(world, this, Some("MonkeyClimbApeTorch"));
        return true;
    }
    false
}

/// `-[MonkeyLogic apeTorchClimbComplete]` @ 0x1000d1790
pub fn ape_torch_climb_complete(world: &mut World, this: Entity) {
    world.obj_mut::<MonkeyLogic>(this).in_perch = true;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("MonkeyClimbApeTorchComplete"));
}

/// `-[MonkeyLogic jumpOffTorch:::]` @ 0x1000d17f8
pub fn seq_jump_off_torch(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.enable_island_floor_collision = true;
        monkey.can_fall_into_perch = false;
    }

    let display_object = logic::display_object(world, this);
    let scale_x = display::object(world, display_object).scale.x;
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = scale_x * 200.0;
        state.pos_vel.y = 200.0;
    }
    world.obj_mut::<MonkeyLogic>(this).in_perch = false;

    let ape_torch = world.obj::<MonkeyLogic>(this).ape_torch_logic;
    if let Some(ape_torch) = ape_torch {
        ape_torch_logic::monkey_jump_off_torch(world, ape_torch, this);
    }
    world.obj_mut::<MonkeyLogic>(this).ape_torch_logic = None;
    ItemResult::Done
}

/// `-[MonkeyLogic finishEatingBanana:::]` @ 0x1000d18c0
pub fn seq_finish_eating_banana(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[MonkeyLogic chasePygmy:::]` @ 0x1000d18c8
pub fn seq_chase_pygmy(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<MonkeyLogic>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let island_side_collision = world.obj::<MonkeyLogic>(this).island_side_collision;
    let banana_eating_pygmy = if island_side_collision {
        None
    } else {
        let pygmy_logic_array = listener_pygmy_logic_array(world, this);
        let banana_eating_pygmy = world.obj::<MonkeyLogic>(this).banana_eating_pygmy_logic;
        banana_eating_pygmy.filter(|pygmy| pygmy_logic_array.contains(pygmy))
    };

    // The chase ends at the island's edge or when the pygmy is gone.
    let Some(banana_eating_pygmy) = banana_eating_pygmy else {
        world.obj_mut::<MonkeyLogic>(this).banana_eating_pygmy_logic = None;
        logic::set_behavior(world, this, Some("MonkeyIdle"));
        return ItemResult::Replaced;
    };

    let can_be_chased = crate::pygmy::can_be_chased_by_monkey(world, banana_eating_pygmy, this);
    if !can_be_chased {
        world.obj_mut::<MonkeyLogic>(this).banana_eating_pygmy_logic = None;
        logic::set_behavior(world, this, Some("MonkeyCancelChasePygmy"));
        return ItemResult::Replaced;
    }

    let display_object = logic::display_object(world, this);
    let pygmy_display_object = logic::display_object(world, banana_eating_pygmy);
    let x_pos = display::object(world, display_object).pos.x;
    let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
    let caught_up = (x_pos - pygmy_x_pos).abs() < 10.0;
    if !caught_up {
        return ItemResult::Pending;
    }

    // Face the monkey the pygmy will be thrown to.
    let throw_to_monkey = world.obj::<MonkeyLogic>(this).throw_to_monkey_logic;
    if let Some(throw_to_monkey) = throw_to_monkey {
        let throw_to_display_object = logic::display_object(world, throw_to_monkey);
        let throw_to_x_pos = display::object(world, throw_to_display_object).pos.x;
        let x_pos = display::object(world, display_object).pos.x;
        let facing = if x_pos < throw_to_x_pos { 1.0 } else { -1.0 };
        display::object_mut(world, display_object).scale.x = facing;
    }
    ItemResult::Done
}

/// `-[MonkeyLogic walk:::]` @ 0x1000d1a9c
pub fn seq_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = logic::random_scaled_time(min_time, max_time, 100.0, random);
        logic::logic_mut(world, this).stall_time = stall_time;
        world.obj_mut::<MonkeyLogic>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    // Something to do on the way ends the walk.
    if check_for_banana(world, this) {
        return ItemResult::Replaced;
    }
    if check_for_banana_tree(world, this) {
        return ItemResult::Replaced;
    }
    if check_for_ape_torch(world, this) {
        return ItemResult::Replaced;
    }
    if check_for_sleeping_pygmy(world, this) {
        return ItemResult::Replaced;
    }

    let island_side_collision = world.obj::<MonkeyLogic>(this).island_side_collision;
    let stall_time = logic::logic(world, this).stall_time;
    let result = if island_side_collision || stall_time <= 0.0 {
        // The next walk may hump a sleeping pygmy (20 %) or climb.
        world.obj_mut::<MonkeyLogic>(this).enable_hump = false;
        let hump_roll = world.resource_mut::<Random>().arc4random_mod(10001);
        let hump_percent = hump_roll as f32 / 100.0;
        if hump_percent < 20.0 {
            world.obj_mut::<MonkeyLogic>(this).enable_hump = true;
        }

        world.obj_mut::<MonkeyLogic>(this).enable_climb = false;
        let climb_roll = world.resource_mut::<Random>().arc4random_mod(10001);
        let climb_percent = climb_roll as f32 / 100.0;
        if climb_percent < 30.0 {
            world.obj_mut::<MonkeyLogic>(this).enable_climb = true;
        }
        // Original bug, kept: the 30 % roll is overwritten, so climbing is always enabled.
        world.obj_mut::<MonkeyLogic>(this).enable_climb = true;

        logic::logic_mut(world, this).pos_vel.x = 0.0;
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let delta_time = call.delta_time;
    logic::logic_mut(world, this).stall_time -= delta_time;
    result
}

/// `-[MonkeyLogic idle:::]` @ 0x1000d1cf0
pub fn seq_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = logic::random_scaled_time(min_time, max_time, 100.0, random);
        logic::logic_mut(world, this).stall_time = stall_time;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let stall_time = logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        return ItemResult::Done;
    }

    let delta_time = call.delta_time;
    logic::logic_mut(world, this).stall_time -= delta_time;
    ItemResult::Pending
}

/// `-[MonkeyLogic drag:::]` @ 0x1000d1e04
pub fn seq_drag(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<MonkeyLogic>(this).enable_island_floor_collision = false;
    }

    let display_object = logic::display_object(world, this);
    let display = display::object(world, display_object);
    let pos = display.pos;
    let display_group = display.display_group;
    let touch_screen_pos = world.obj::<MonkeyLogic>(this).touch_screen_pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };

    // Original quirk, kept: the compiled code subtracts the monkey's own z (not the touch's),
    // so the offset's z is 0.
    let subtracted_z = pos.z;
    let offset_x = pos.x - touch_pos.x;
    let offset_y = pos.y - touch_pos.y;
    let offset_z = pos.z - subtracted_z;
    let offset = Vec4::new(offset_x, offset_y, offset_z, pos.w);

    let length = length3(offset);
    if length != 0.0 {
        if offset.y != 0.0 {
            let z_rot = coconut_logic::throwable_drag_z_rot(offset);
            display::object_mut(world, display_object).rot.z = z_rot;
        }

        let pos_vel = logic::logic(world, this).pos_vel;
        let spring_force = coconut_logic::throwable_spring_force(offset, length, pos_vel, DRAG_STIFFNESS, DRAG_DAMPING);
        world.obj_mut::<MonkeyLogic>(this).spring_force = spring_force;
    }
    ItemResult::Pending
}

/// `-[MonkeyLogic animFallOffIsland]` @ 0x1000d1fb4
pub fn anim_fall_off_island(world: &mut World, this: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    let mut monkey = world.obj_mut::<MonkeyLogic>(this);
    monkey.falling = true;
    monkey.enable_island_floor_collision = false;
}

/// `-[MonkeyLogic fall:::]` @ 0x1000d1fe4
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            if !monkey.island_floor_collision {
                monkey.on_island = false;
            }
            monkey.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<MonkeyLogic>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<MonkeyLogic>(this).enable_island_floor_collision = false;
        }
    }

    // A falling monkey can land in a perch on the way down.
    if check_for_banana_tree_during_fall(world, this) {
        return ItemResult::Replaced;
    }
    if check_for_ape_torch_during_fall(world, this) {
        return ItemResult::Replaced;
    }

    let island_floor_collision = world.obj::<MonkeyLogic>(this).island_floor_collision;
    if !island_floor_collision {
        return ItemResult::Pending;
    }

    let mut monkey = world.obj_mut::<MonkeyLogic>(this);
    monkey.falling = false;
    monkey.enable_island_side_collision = true;
    ItemResult::Done
}

/// `-[MonkeyLogic die:::]` @ 0x1000d20e0
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    let helper_display_object = world.obj::<MonkeyLogic>(this).helper_display_object;
    if let Some(helper_display_object) = helper_display_object {
        display::remove_from_display_group(world, helper_display_object);
    }
    stop_interacting_with_objects(world, this);

    let dragging = world.obj::<MonkeyLogic>(this).dragging;
    if dragging {
        world.obj_mut::<MonkeyLogic>(this).dragging = false;
        notify_monkey_drag_complete(world, this);
    }

    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.active = false;
        monkey.falling = false;
        monkey.apply_spring = false;
        monkey.active_touch = None;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.hit_by_lightning = false;
        monkey.pygmy_has_plague = false;
    }
    ItemResult::Done
}

/// `-[MonkeyLogic touchBeganInside:allTouches:]` @ 0x1000d21f0
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let monkey = world.obj::<MonkeyLogic>(this);
    if monkey.falling || monkey.active_touch.is_some() {
        return;
    }

    stop_interacting_with_objects(world, this);
    world.obj_mut::<MonkeyLogic>(this).active_touch = Some(touch);
    {
        let mut state = logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.touch_began_inside = true;
        monkey.touch_screen_pos = touch_pos;
        monkey.tilt_state = 0;
    }
    logic::set_behavior(world, this, Some("MonkeyTouched"));
}

/// `-[MonkeyLogic touchBeganOutside:allTouches:]` @ 0x1000d22cc
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MonkeyLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut monkey = world.obj_mut::<MonkeyLogic>(this);
    monkey.touch_screen_pos = touch_pos;
    monkey.touch_began_inside = false;
}

/// `-[MonkeyLogic touchMovedEntered:allTouches:]` @ 0x1000d2328
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MonkeyLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<MonkeyLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[MonkeyLogic touchMovedExited:allTouches:]` @ 0x1000d2380
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MonkeyLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<MonkeyLogic>(this).touch_screen_pos = touch_pos;

    // A touch that leaves the monkey drags it.
    let touch_began_inside = world.obj::<MonkeyLogic>(this).touch_began_inside;
    if !touch_began_inside {
        return;
    }

    {
        let mut monkey = world.obj_mut::<MonkeyLogic>(this);
        monkey.dragging = true;
        monkey.apply_spring = true;
        monkey.touch_began_inside = false;
        monkey.enable_island_floor_collision = false;
        monkey.enable_island_side_collision = false;
    }

    let listener = world.obj::<MonkeyLogic>(this).monkey_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_monkey_drag(world, listener, this);
    }
    logic::set_behavior(world, this, Some("MonkeyDrag"));
}

/// `-[MonkeyLogic touchMovedInside:allTouches:]` @ 0x1000d2464
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[MonkeyLogic touchMovedOutside:allTouches:]` @ 0x1000d2474
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MonkeyLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<MonkeyLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[MonkeyLogic touchEndedInside:allTouches:]` @ 0x1000d24cc
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<MonkeyLogic>(this).active_touch;
    if active_touch.is_some() && active_touch != Some(touch) {
        return;
    }

    // A tap drops the monkey.
    let touch_began_inside = world.obj::<MonkeyLogic>(this).touch_began_inside;
    if touch_began_inside {
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.falling = true;
            monkey.touch_began_inside = false;
        }
        logic::set_behavior(world, this, Some("MonkeyFall"));
    }

    // A drag throws it.
    let dragging = world.obj::<MonkeyLogic>(this).dragging;
    if dragging {
        {
            let mut monkey = world.obj_mut::<MonkeyLogic>(this);
            monkey.falling = true;
            monkey.dragging = false;
            monkey.apply_spring = false;
        }
        let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
        let display_object = logic::display_object(world, this);
        if let Some(marker) = marker {
            display::insert_before(world, marker, Some(display_object));
        }
        notify_monkey_drag_complete(world, this);
        logic::set_behavior(world, this, Some("MonkeyFall"));
    }

    let mut monkey = world.obj_mut::<MonkeyLogic>(this);
    monkey.touch_began_inside = false;
    monkey.active_touch = None;
}

/// `-[MonkeyLogic touchEndedOutside:allTouches:]` @ 0x1000d260c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
