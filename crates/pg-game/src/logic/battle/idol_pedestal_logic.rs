//! `IdolPedestalLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::battle::{god_idol_logic, temple_door_logic};
use crate::logic::island_core::sprite_logic;
use pg_engine::animation;
use pg_engine::display;
use pg_engine::logic;
use pg_engine::prefs;
use pg_engine::touch::BcTouch;

/// `IdolPedestalLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IdolPedestalLogic {
    /// `mIdolPedestalLogicListener`: `@"<IdolPedestalLogicListener>"`
    pub idol_pedestal_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mTouchScreenPos`: `{?="x"f"y"f"z"f"w"f}`
    pub touch_screen_pos: Vec4,

    /// `mSpringForce`: `{?="x"f"y"f"z"f"w"f}`
    pub spring_force: Vec4,

    /// `mPedestalIndex`: `i`
    pub pedestal_index: i32,

    /// `mbGodIdolOpenDoor`: `B`
    pub god_idol_open_door: bool,

    /// `mGodIdolSlot`: `i`
    pub god_idol_slot: i32,

    /// `mbGodIdolSlotInit`: `B`
    pub god_idol_slot_init: bool,

    /// `mGodIdolSlotKey`: `@"NSString"`
    pub god_idol_slot_key: Option<String>,

    /// `mTrappedPygmyLogic`: `@"PygmyLogic"`
    pub trapped_pygmy_logic: Option<Entity>,

    /// `mTrappedGodIdolLogic`: `@"GodIdolLogic"`
    pub trapped_god_idol_logic: Option<Entity>,
}

pub const IDOL_PEDESTAL_LOGIC: Class = Class {
    name: "IdolPedestalLogic",
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

/// The animation channel a trapped pygmy is bound to.
const PYGMY_CHANNEL: usize = 1;

/// The animation channel a seated god idol is bound to.
const GOD_IDOL_CHANNEL: usize = 2;

/// `mGodIdolSlot` (and the saved `GodIdolSlot<n>` preference) of a pedestal without an idol.
const NO_GOD_IDOL_SLOT: i32 = -1;

/// Sequence-item handlers implemented by `IdolPedestalLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "playSoundBasedOnPedestalIndex" => seq_play_sound_based_on_pedestal_index(world, this, call),
        "stopSoundBasedOnPedestalIndex" => seq_stop_sound_based_on_pedestal_index(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[IdolPedestalLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&IDOL_PEDESTAL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// The listener (the view controller).
fn pedestal_listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<IdolPedestalLogic>(this).idol_pedestal_logic_listener
}

/// `mGodIdolSlotKey`, built by `setPedestalIndex:` (which `loadGame` sends to every pedestal
/// before anything saves a slot); a nil key would raise in the preferences dictionary.
fn god_idol_slot_key(world: &World, this: Entity) -> String {
    let key = world.obj::<IdolPedestalLogic>(this).god_idol_slot_key.clone();
    key.expect("IdolPedestalLogic: setPedestalIndex: builds the GodIdolSlot key first")
}

/// `[[mIdolPedestalLogicListener templeDoorLogic:self] openDoor]` or `… closeDoor]`.
fn step_temple_door(world: &mut World, this: Entity, open: bool) {
    let Some(listener) = pedestal_listener(world, this) else {
        return;
    };

    let temple_door = crate::controller::temple_door_logic(world, listener, this);
    let Some(temple_door) = temple_door else {
        return;
    };

    if open {
        temple_door_logic::open_door(world, temple_door);
    } else {
        temple_door_logic::close_door(world, temple_door);
    }
}

/// The `play/stopSoundBasedOnPedestalIndex:::` parameter of pedestal `pedestal_index` (1–4:
/// `soundId1` … `soundId4`; other indices are nil).
fn pedestal_sound_id<'a>(call: &'a SequenceCall, pedestal_index: i32) -> Option<&'a str> {
    match pedestal_index {
        1 => call.param("soundId1"),
        2 => call.param("soundId2"),
        3 => call.param("soundId3"),
        4 => call.param("soundId4"),
        _ => None,
    }
}

/// `mTouchScreenPos = [touch pos]`.
fn track_touch(world: &mut World, this: Entity, touch: Entity) {
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    world.obj_mut::<IdolPedestalLogic>(this).touch_screen_pos = touch_pos;
}

/// `-[IdolPedestalLogic initWithDisplayObject:]` @ 0x10013ecb4
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IdolPedestalLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let own_display_object = logic::display_object(world, this);
    {
        let mut display = display::object_mut(world, own_display_object);
        display.col_x_min = -50.0;
        display.col_x_max = 50.0;
        display.col_y_min = 0.0;
        display.col_y_max = 74.0;
    }

    world.obj_mut::<IdolPedestalLogic>(this).god_idol_slot = NO_GOD_IDOL_SLOT;
}

/// `-[IdolPedestalLogic setIdolPedestalLogicListener:]` @ 0x10013ed8c
pub fn set_idol_pedestal_logic_listener(world: &mut World, this: Entity, idol_pedestal_logic_listener: Entity) {
    world.obj_mut::<IdolPedestalLogic>(this).idol_pedestal_logic_listener = Some(idol_pedestal_logic_listener);
}

/// `-[IdolPedestalLogic isAvailable]` @ 0x10013ede8
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let pedestal = world.obj::<IdolPedestalLogic>(this);
    if pedestal.trapped_pygmy_logic.is_some() {
        return false;
    }
    pedestal.trapped_god_idol_logic.is_none()
}

/// `-[IdolPedestalLogic pedestalIndex]` @ 0x10013ee18
pub fn pedestal_index(world: &mut World, this: Entity) -> i32 {
    world.obj::<IdolPedestalLogic>(this).pedestal_index
}

/// `-[IdolPedestalLogic setPedestalIndex:]` @ 0x10013ee28
pub fn set_pedestal_index(world: &mut World, this: Entity, pedestal_index: i32) {
    let key = format!("GodIdolSlot{pedestal_index}");

    let mut pedestal = world.obj_mut::<IdolPedestalLogic>(this);
    pedestal.god_idol_slot_key = Some(key);
    pedestal.pedestal_index = pedestal_index;
}

/// `-[IdolPedestalLogic setGodIdolSlot:]` @ 0x10013ee9c
///
/// Remembers which idol slot stands on the pedestal, also in the `Options` preferences.
pub fn set_god_idol_slot(world: &mut World, this: Entity, god_idol_slot: i32) {
    world.obj_mut::<IdolPedestalLogic>(this).god_idol_slot = god_idol_slot;

    let Some(listener) = pedestal_listener(world, this) else {
        return;
    };
    let option_preferences = crate::controller::option_preferences(world, listener);
    let Some(option_preferences) = option_preferences else {
        return;
    };

    let key = god_idol_slot_key(world, this);
    let saved_slot = world.obj::<IdolPedestalLogic>(this).god_idol_slot;
    prefs::set_preference_as_int(world, option_preferences, &key, saved_slot);
}

/// `-[IdolPedestalLogic fallIntoIdolPedestal:]` @ 0x10013eefc
pub fn fall_into_idol_pedestal(world: &mut World, this: Entity, pygmy_logic: Entity) {
    set_god_idol_slot(world, this, NO_GOD_IDOL_SLOT);
    world.obj_mut::<IdolPedestalLogic>(this).trapped_pygmy_logic = Some(pygmy_logic);

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("IdolPedestalPygmyFall"));
}

/// `-[IdolPedestalLogic ejectFromIdolPedestal:]` @ 0x10013ef9c
pub fn eject_from_idol_pedestal(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    let animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("IdolPedestalEjectPygmy"));
    world.obj_mut::<IdolPedestalLogic>(this).trapped_pygmy_logic = None;
}

/// `-[IdolPedestalLogic godIdolFallIntoIdolPedestal:]` @ 0x10013f004
///
/// Seats the idol, remembers its slot and opens the temple door one step.
pub fn god_idol_fall_into_idol_pedestal(world: &mut World, this: Entity, god_idol_logic: Entity) {
    world.obj_mut::<IdolPedestalLogic>(this).trapped_god_idol_logic = Some(god_idol_logic);

    let animation = logic::animation(world, this);
    let idol_display_object = logic::display_object(world, god_idol_logic);
    animation::set_tweenable(world, animation, Some(idol_display_object), GOD_IDOL_CHANNEL);
    logic::set_behavior(world, this, Some("IdolPedestalGodIdolFall"));
    world.obj_mut::<IdolPedestalLogic>(this).god_idol_open_door = true;

    let trapped_god_idol = world.obj::<IdolPedestalLogic>(this).trapped_god_idol_logic;
    let idol_slot = match trapped_god_idol {
        Some(trapped_god_idol) => god_idol_logic::slot(world, trapped_god_idol),
        None => 0,
    };
    set_god_idol_slot(world, this, idol_slot);

    step_temple_door(world, this, true);
}

/// `-[IdolPedestalLogic ejectGodIdolFromIdolPedestal:]` @ 0x10013f0f8
pub fn eject_god_idol_from_idol_pedestal(world: &mut World, this: Entity, _god_idol_logic: Entity) {
    let animation = logic::animation(world, this);
    animation::clear_tweenable_channel_id_now(world, animation, GOD_IDOL_CHANNEL);
    logic::set_behavior(world, this, Some("IdolPedestalEjectGodIdol"));
    world.obj_mut::<IdolPedestalLogic>(this).trapped_god_idol_logic = None;

    let god_idol_open_door = world.obj::<IdolPedestalLogic>(this).god_idol_open_door;
    if !god_idol_open_door {
        return;
    }

    world.obj_mut::<IdolPedestalLogic>(this).god_idol_open_door = false;
    step_temple_door(world, this, false);
}

/// `-[IdolPedestalLogic godIdolSlot]` @ 0x10013f1ac
///
/// The idol slot standing on the pedestal, loaded once from the `Options` preferences
/// (default −1).
pub fn god_idol_slot(world: &mut World, this: Entity) -> i32 {
    let slot_init = world.obj::<IdolPedestalLogic>(this).god_idol_slot_init;
    if !slot_init {
        world.obj_mut::<IdolPedestalLogic>(this).god_idol_slot_init = true;

        let mut option_preferences = None;
        if let Some(listener) = pedestal_listener(world, this) {
            option_preferences = crate::controller::option_preferences(world, listener);
        }

        // Messages to nil preferences answer 0.
        let mut saved_slot = 0;
        if let Some(option_preferences) = option_preferences {
            let key = god_idol_slot_key(world, this);
            prefs::set_default_preference_as_int(world, option_preferences, &key, NO_GOD_IDOL_SLOT);
            saved_slot = prefs::get_int_preference(world, option_preferences, &key);
        }
        world.obj_mut::<IdolPedestalLogic>(this).god_idol_slot = saved_slot;
    }

    world.obj::<IdolPedestalLogic>(this).god_idol_slot
}

/// `-[IdolPedestalLogic clearGodIdolSlot:]` @ 0x10013f258
pub fn clear_god_idol_slot(world: &mut World, this: Entity, _god_idol_logic: Entity) {
    set_god_idol_slot(world, this, NO_GOD_IDOL_SLOT);
}

/// `-[IdolPedestalLogic enterIsland:]` @ 0x10013f26c
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let active = world.obj::<IdolPedestalLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<IdolPedestalLogic>(this).active = true;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("IdolPedestalEnterIsland"));
}

/// `-[IdolPedestalLogic exitIsland:]` @ 0x10013f2e8
///
/// Ejects a trapped pygmy and a seated idol, closes the door step they opened and deactivates.
pub fn exit_island(world: &mut World, this: Entity, _now: bool) {
    let trapped_pygmy = world.obj::<IdolPedestalLogic>(this).trapped_pygmy_logic;
    if let Some(trapped_pygmy) = trapped_pygmy {
        let animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::eject_from_idol_pedestal(world, trapped_pygmy);
        world.obj_mut::<IdolPedestalLogic>(this).trapped_pygmy_logic = None;
    }

    let trapped_god_idol = world.obj::<IdolPedestalLogic>(this).trapped_god_idol_logic;
    if let Some(trapped_god_idol) = trapped_god_idol {
        let animation = logic::animation(world, this);
        animation::clear_tweenable_channel_id_now(world, animation, GOD_IDOL_CHANNEL);
        god_idol_logic::eject_from_idol_pedestal(world, trapped_god_idol);
        world.obj_mut::<IdolPedestalLogic>(this).trapped_god_idol_logic = None;
    }

    let god_idol_open_door = world.obj::<IdolPedestalLogic>(this).god_idol_open_door;
    if god_idol_open_door {
        world.obj_mut::<IdolPedestalLogic>(this).god_idol_open_door = false;
        step_temple_door(world, this, false);
    }

    let active = world.obj::<IdolPedestalLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<IdolPedestalLogic>(this).active = false;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    logic::set_behavior(world, this, Some("IdolPedestalExitIsland"));
}

/// `-[IdolPedestalLogic exitIslandNow]` @ 0x10013f43c
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[IdolPedestalLogic onGameFrame:]` @ 0x10013f4cc
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[IdolPedestalLogic playSoundBasedOnPedestalIndex:::]` @ 0x10013f4d0
pub fn seq_play_sound_based_on_pedestal_index(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let pedestal_index = world.obj::<IdolPedestalLogic>(this).pedestal_index;

    // `[[BCLibrary getSoundById:nil] play]` does nothing.
    let sound_id = pedestal_sound_id(call, pedestal_index);
    if let Some(sound_id) = sound_id {
        pg_engine::sound::play_sound(world, sound_id);
    }
    ItemResult::Done
}

/// `-[IdolPedestalLogic stopSoundBasedOnPedestalIndex:::]` @ 0x10013f598
pub fn seq_stop_sound_based_on_pedestal_index(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let pedestal_index = world.obj::<IdolPedestalLogic>(this).pedestal_index;

    let sound_id = pedestal_sound_id(call, pedestal_index);
    if let Some(sound_id) = sound_id {
        pg_engine::sound::stop_sound(world, sound_id);
    }
    ItemResult::Done
}

/// `-[IdolPedestalLogic touchBeganInside:allTouches:]` @ 0x10013f660
///
/// A tap on a trapped pygmy makes it scream.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let story_active = match pedestal_listener(world, this) {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    let active_touch = world.obj::<IdolPedestalLogic>(this).active_touch;
    if active_touch.is_some() {
        return;
    }

    world.obj_mut::<IdolPedestalLogic>(this).active_touch = Some(touch);

    let trapped_pygmy = world.obj::<IdolPedestalLogic>(this).trapped_pygmy_logic;
    if trapped_pygmy.is_some() {
        logic::set_behavior(world, this, Some("IdolPedestalPygmyScream"));
    }
}

/// `-[IdolPedestalLogic touchBeganOutside:allTouches:]` @ 0x10013f6e8
pub fn touch_began_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<IdolPedestalLogic>(this).active_touch;
    if active_touch.is_none() {
        track_touch(world, this, touch);
    }
}

/// `-[IdolPedestalLogic touchMovedEntered:allTouches:]` @ 0x10013f738
pub fn touch_moved_entered(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<IdolPedestalLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        track_touch(world, this, touch);
    }
}

/// `-[IdolPedestalLogic touchMovedExited:allTouches:]` @ 0x10013f790
pub fn touch_moved_exited(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<IdolPedestalLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        track_touch(world, this, touch);
    }
}

/// `-[IdolPedestalLogic touchMovedInside:allTouches:]` @ 0x10013f7e8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[IdolPedestalLogic touchMovedOutside:allTouches:]` @ 0x10013f7f8
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<IdolPedestalLogic>(this).active_touch;
    if active_touch.is_none() || active_touch == Some(touch) {
        track_touch(world, this, touch);
    }
}

/// `-[IdolPedestalLogic touchEndedInside:allTouches:]` @ 0x10013f850
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<IdolPedestalLogic>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    let trapped_pygmy = world.obj::<IdolPedestalLogic>(this).trapped_pygmy_logic;
    if trapped_pygmy.is_some() {
        logic::set_behavior(world, this, Some("IdolPedestalPygmyStopScream"));
    }
    world.obj_mut::<IdolPedestalLogic>(this).active_touch = None;
}

/// `-[IdolPedestalLogic touchEndedOutside:allTouches:]` @ 0x10013f8b8
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
