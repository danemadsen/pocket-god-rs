//! `BonFireLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::coconut_logic::is_island_tipped_over;
use crate::logic::island_core::sprite_logic;
use crate::logic::island_core::wood_logic;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `BonFireLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BonFireLogic {
    /// `mBonFireLogicListener`: `@"<BonFireLogicListener>"`
    pub bon_fire_logic_listener: Option<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mRightWarmingPygmyLogic`: `@"PygmyLogic"`
    pub right_warming_pygmy_logic: Option<Entity>,

    /// `mLeftWarmingPygmyLogic`: `@"PygmyLogic"`
    pub left_warming_pygmy_logic: Option<Entity>,

    /// `mStressLevel`: `f`
    pub stress_level: f32,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbOnFire`: `B`
    pub on_fire: bool,

    /// `mWoodLogicArray`: `@"NSMutableArray"`
    pub wood_logic_array: Vec<Entity>,
}

pub const BON_FIRE_LOGIC: Class = Class {
    name: "BonFireLogic",
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

/// `DAT_10029da1c`: the x offset of the slot of the next log, by the number of logs.
const NEXT_WOOD_X_OFFSETS: [f32; 3] = [-1.0, -3.4, 1.0];

/// `DAT_10029da28`: the y offset of the slot of the next log, by the number of logs.
const NEXT_WOOD_Y_OFFSETS: [f32; 3] = [11.0, 20.9, 22.5];

/// The logs ride these animation channels of the bonfire (`setTweenable:channelId:`).
const LOG_CHANNELS: [usize; 3] = [1, 2, 3];

/// A poke burns the pygmies closer than this horizontally.
const POKE_X_DISTANCE: f32 = 35.0;

/// Sequence-item handlers implemented by `BonFireLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[BonFireLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&BON_FIRE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[BonFireLogic initWithDisplayObject:]` @ 0x10013a630
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(BonFireLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    {
        let mut bon_fire = world.obj_mut::<BonFireLogic>(this);
        bon_fire.bon_fire_logic_listener = None;
        bon_fire.active_touch = None;
        bon_fire.touch_began_inside = false;
        bon_fire.touch_screen_pos = Vec4::new(0.0, 0.0, 0.0, 1.0);
        bon_fire.wood_logic_array = Vec::with_capacity(5);
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[BonFireLogic setBonFireLogicListener:]` @ 0x10013a718
pub fn set_bon_fire_logic_listener(world: &mut World, this: Entity, bon_fire_logic_listener: Entity) {
    world.obj_mut::<BonFireLogic>(this).bon_fire_logic_listener = Some(bon_fire_logic_listener);
}

/// `-[BonFireLogic isLeftWarmingAvailable:]` @ 0x10013a774
pub fn is_left_warming_available(world: &mut World, this: Entity, _left_warming_available: Entity) -> bool {
    let bon_fire = world.obj::<BonFireLogic>(this);
    if bon_fire.left_warming_pygmy_logic.is_some() {
        return false;
    }
    bon_fire.on_fire
}

/// `-[BonFireLogic isRightWarmingAvailable:]` @ 0x10013a7a4
pub fn is_right_warming_available(world: &mut World, this: Entity, _right_warming_available: Entity) -> bool {
    let bon_fire = world.obj::<BonFireLogic>(this);
    if bon_fire.right_warming_pygmy_logic.is_some() {
        return false;
    }
    bon_fire.on_fire
}

/// `-[BonFireLogic stopRightWarming:]` @ 0x10013a7d4
pub fn stop_right_warming(world: &mut World, this: Entity, _right_warming: Entity) {
    world.obj_mut::<BonFireLogic>(this).right_warming_pygmy_logic = None;
}

/// `-[BonFireLogic stopLeftWarming:]` @ 0x10013a7e4
pub fn stop_left_warming(world: &mut World, this: Entity, _left_warming: Entity) {
    world.obj_mut::<BonFireLogic>(this).left_warming_pygmy_logic = None;
}

/// `-[BonFireLogic startRightWarming:]` @ 0x10013a7f4
pub fn start_right_warming(world: &mut World, this: Entity, right_warming: Entity) {
    world.obj_mut::<BonFireLogic>(this).right_warming_pygmy_logic = Some(right_warming);
}

/// `-[BonFireLogic startLeftWarming:]` @ 0x10013a804
pub fn start_left_warming(world: &mut World, this: Entity, left_warming: Entity) {
    world.obj_mut::<BonFireLogic>(this).left_warming_pygmy_logic = Some(left_warming);
}

/// `-[BonFireLogic hasAtLeastOneLog]` @ 0x10013a814
pub fn has_at_least_one_log(world: &mut World, this: Entity) -> bool {
    !world.obj::<BonFireLogic>(this).wood_logic_array.is_empty()
}

/// `-[BonFireLogic isOnFire]` @ 0x10013a848
pub fn is_on_fire(world: &mut World, this: Entity) -> bool {
    world.obj::<BonFireLogic>(this).on_fire
}

/// `-[BonFireLogic nextWoodXOffset]` @ 0x10013a858
pub fn next_wood_x_offset(world: &mut World, this: Entity) -> f32 {
    let count = world.obj::<BonFireLogic>(this).wood_logic_array.len();
    if count < 3 {
        return NEXT_WOOD_X_OFFSETS[count];
    }
    0.0
}

/// `-[BonFireLogic nextWoodYOffset]` @ 0x10013a8a4
pub fn next_wood_y_offset(world: &mut World, this: Entity) -> f32 {
    let count = world.obj::<BonFireLogic>(this).wood_logic_array.len();
    if count < 3 {
        return NEXT_WOOD_Y_OFFSETS[count];
    }
    0.0
}

/// `-[BonFireLogic addWoodLogic:]` @ 0x10013a8f0
pub fn add_wood_logic(world: &mut World, this: Entity, wood_logic: Entity) {
    let count = world.obj::<BonFireLogic>(this).wood_logic_array.len();
    let drop_log_behavior = match count {
        0 => Some("BonFireDropLog1"),
        1 => Some("BonFireDropLog2"),
        2 => Some("BonFireDropLog3"),
        _ => None,
    };
    if let Some(drop_log_behavior) = drop_log_behavior {
        let animation = logic::animation(world, this);
        let wood_display_object = logic::display_object(world, wood_logic);
        let channel = LOG_CHANNELS[count];
        pg_engine::animation::set_tweenable(world, animation, Some(wood_display_object), channel);
        logic::set_behavior(world, this, Some(drop_log_behavior));
    }

    world.obj_mut::<BonFireLogic>(this).wood_logic_array.push(wood_logic);

    let listener = world.obj::<BonFireLogic>(this).bon_fire_logic_listener;
    let fishing_pole = match listener {
        Some(listener) => crate::controller::fishing_pole_logic(world, listener, this),
        None => None,
    };
    if let Some(fishing_pole) = fishing_pole {
        crate::logic::ocean_fishing::fishing_pole_logic::check_proximity_to_bon_fire(world, fishing_pole, this);
    }
}

/// Every warming pygmy jumps away from the fire (`tossFromBonFire`), left one first.
fn toss_warming_pygmies(world: &mut World, this: Entity) {
    let left_warming_pygmy = world.obj::<BonFireLogic>(this).left_warming_pygmy_logic;
    if let Some(left_warming_pygmy) = left_warming_pygmy {
        crate::pygmy::toss_from_bon_fire(world, left_warming_pygmy);
    }

    let right_warming_pygmy = world.obj::<BonFireLogic>(this).right_warming_pygmy_logic;
    if let Some(right_warming_pygmy) = right_warming_pygmy {
        crate::pygmy::toss_from_bon_fire(world, right_warming_pygmy);
    }
}

/// Releases the logs from animation channels 1–3 (`clearTweenableChannelIdNow:`).
fn clear_log_channels(world: &mut World, this: Entity) {
    let animation = logic::animation(world, this);
    for channel in LOG_CHANNELS {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, channel);
    }
}

/// Every log in the pile falls out of the fire (`fallOutOfBonFire`).
fn drop_logs(world: &mut World, this: Entity) {
    let wood_logic_array = world.obj::<BonFireLogic>(this).wood_logic_array.clone();
    for wood in wood_logic_array {
        wood_logic::fall_out_of_bon_fire(world, wood);
    }
}

/// `-[BonFireLogic removeWoodLogic:]` @ 0x10013aa9c
///
/// `wood_logic` is nil when the device is tilted (`onGameFrame:`); `removeObject:nil` removes
/// nothing.
pub fn remove_wood_logic(world: &mut World, this: Entity, wood_logic: Option<Entity>) {
    world.obj_mut::<BonFireLogic>(this).on_fire = false;
    toss_warming_pygmies(world, this);
    clear_log_channels(world, this);

    if let Some(wood_logic) = wood_logic {
        let mut bon_fire = world.obj_mut::<BonFireLogic>(this);
        bon_fire.wood_logic_array.retain(|wood| *wood != wood_logic);
    }
    drop_logs(world, this);
    world.obj_mut::<BonFireLogic>(this).wood_logic_array.clear();

    logic::set_behavior(world, this, Some("BonFireRemoveWood"));
}

/// `-[BonFireLogic hitByLightning]` @ 0x10013ac94
pub fn hit_by_lightning(world: &mut World, this: Entity) -> bool {
    let bon_fire = world.obj::<BonFireLogic>(this);
    let count = bon_fire.wood_logic_array.len();
    if count != 3 || bon_fire.on_fire {
        return false;
    }

    world.obj_mut::<BonFireLogic>(this).on_fire = true;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("BonFireCatchFire"));
    true
}

/// `-[BonFireLogic pickedUpByHurricane:]` @ 0x10013ad38
///
/// Puts the fire out and scatters its logs, but never lets the hurricane take it.
pub fn picked_up_by_hurricane(world: &mut World, this: Entity, _picked_up_by_hurricane: Entity) -> bool {
    let count = world.obj::<BonFireLogic>(this).wood_logic_array.len();
    if count != 0 {
        world.obj_mut::<BonFireLogic>(this).on_fire = false;
        let display_object = logic::display_object(world, this);
        display::object_mut(world, display_object).is_selectable = false;

        clear_log_channels(world, this);
        toss_warming_pygmies(world, this);
        drop_logs(world, this);
        world.obj_mut::<BonFireLogic>(this).wood_logic_array.clear();

        logic::set_behavior(world, this, Some("BonFireRemoveWood"));
    }
    false
}

/// `-[BonFireLogic enterIsland:]` @ 0x10013af54
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let active = world.obj::<BonFireLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<BonFireLogic>(this).active = true;
    let marker = pg_engine::library::display_marker(world, "SmallObjectsMarker");
    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(display_object));
    }
    logic::set_behavior(world, this, Some("BonFireEnterIsland"));
}

/// `-[BonFireLogic exitIsland]` @ 0x10013affc
pub fn exit_island(world: &mut World, this: Entity) {
    let active = world.obj::<BonFireLogic>(this).active;
    if !active {
        return;
    }

    {
        let mut bon_fire = world.obj_mut::<BonFireLogic>(this);
        bon_fire.active = false;
        bon_fire.on_fire = false;
    }
    toss_warming_pygmies(world, this);
    clear_log_channels(world, this);
    world.obj_mut::<BonFireLogic>(this).wood_logic_array.clear();

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("BonFireExitIsland"));
}

/// `-[BonFireLogic exitIslandNow]` @ 0x10013b130
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this);

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[BonFireLogic isAccelEnabled]` @ 0x10013b1f4
pub fn is_accel_enabled(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<BonFireLogic>(this).bon_fire_logic_listener;
    let gravity_enabled = match listener {
        Some(listener) => crate::controller::is_gravity_enabled(world, listener),
        None => false,
    };
    if !gravity_enabled {
        return false;
    }
    sprite_logic::is_accel_enabled(world, this)
}

/// `-[BonFireLogic onGameFrame:]` @ 0x10013b264
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let accel_enabled = is_accel_enabled(world, this);
    if !accel_enabled {
        return;
    }

    let count = world.obj::<BonFireLogic>(this).wood_logic_array.len();
    if count == 0 {
        return;
    }

    // Tipping the island over spills the pile.
    if is_island_tipped_over(world) {
        remove_wood_logic(world, this, None);
    }
}

/// `-[BonFireLogic die:::]` @ 0x10013b338
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<BonFireLogic>(this).active_touch = None;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[BonFireLogic touchBeganInside:allTouches:]` @ 0x10013b378
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let listener = world.obj::<BonFireLogic>(this).bon_fire_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };

    let bon_fire = world.obj::<BonFireLogic>(this);
    let pokeable = bon_fire.active_touch.is_none() && bon_fire.active && bon_fire.on_fire;
    if story_active || !pokeable {
        return;
    }

    logic::set_behavior(world, this, Some("BonFirePoke"));

    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let pygmy_logic_array = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmy_logic_array {
        let pygmy_display_object = logic::display_object(world, pygmy);
        let pygmy_x_pos = display::object(world, pygmy_display_object).pos.x;
        let near = (x_pos - pygmy_x_pos).abs() < POKE_X_DISTANCE;

        let bon_fire = world.obj::<BonFireLogic>(this);
        let left_warming = bon_fire.left_warming_pygmy_logic == Some(pygmy);
        let right_warming = bon_fire.right_warming_pygmy_logic == Some(pygmy);
        if near || left_warming || right_warming {
            crate::pygmy::hit_by_bon_fire(world, pygmy, this);
        }
    }
}

/// `-[BonFireLogic touchBeganOutside:allTouches:]` @ 0x10013b5b4
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BonFireLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let mut bon_fire = world.obj_mut::<BonFireLogic>(this);
    bon_fire.touch_screen_pos = touch_pos;
    bon_fire.touch_began_inside = false;
}

/// `-[BonFireLogic touchMovedEntered:allTouches:]` @ 0x10013b610
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BonFireLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BonFireLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BonFireLogic touchMovedExited:allTouches:]` @ 0x10013b668
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BonFireLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BonFireLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BonFireLogic touchMovedInside:allTouches:]` @ 0x10013b6c0
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[BonFireLogic touchMovedOutside:allTouches:]` @ 0x10013b6d0
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BonFireLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let touch_pos = world.obj::<BcTouch>(touch).pos;
        world.obj_mut::<BonFireLogic>(this).touch_screen_pos = touch_pos;
    }
}

/// `-[BonFireLogic touchEndedInside:allTouches:]` @ 0x10013b728
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<BonFireLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        let mut bon_fire = world.obj_mut::<BonFireLogic>(this);
        bon_fire.touch_began_inside = false;
        bon_fire.active_touch = None;
    }
}

/// `-[BonFireLogic touchEndedOutside:allTouches:]` @ 0x10013b754
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
