//! `ApocalypseTempleDoorLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_temples::apocalypse_room_fader_logic;
use pg_engine::logic;

/// `ApocalypseTempleDoorLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseTempleDoorLogic {
    /// `mApocalypseTempleDoorLogicListener`: `@"<ApocalypseTempleDoorLogicListener>"`
    pub apocalypse_temple_door_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbOpen`: `B`
    pub open: bool,

    /// `mApocalypseDoorType`: `i`
    pub apocalypse_door_type: i32,

    /// `mIslandLocationState`: `i`
    pub island_location_state: i32,

    /// `mbAlwaysOpen`: `B`
    pub always_open: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const APOCALYPSE_TEMPLE_DOOR_LOGIC: Class = Class {
    name: "ApocalypseTempleDoorLogic",
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

/// The animation channel the pygmy falling through the door is bound to.
const PYGMY_CHANNEL: usize = 1;

/// Sequence-item handlers implemented by `ApocalypseTempleDoorLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onPygmyFallInsideComplete" => seq_on_pygmy_fall_inside_complete(world, this, call),
        "setAnimationBasedOnPygmyType" => seq_set_animation_based_on_pygmy_type(world, this, call),
        "setAnimationBasedOnDoorType" => seq_set_animation_based_on_door_type(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApocalypseTempleDoorLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_TEMPLE_DOOR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ApocalypseTempleDoorLogic initWithDisplayObject:]` @ 0x100135e30
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseTempleDoorLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
}

/// `-[ApocalypseTempleDoorLogic setApocalypseTempleDoorLogicListener:]` @ 0x100135e94
pub fn set_apocalypse_temple_door_logic_listener(world: &mut World, this: Entity, apocalypse_temple_door_logic_listener: Entity) {
    world.obj_mut::<ApocalypseTempleDoorLogic>(this).apocalypse_temple_door_logic_listener = Some(apocalypse_temple_door_logic_listener);
}

/// `-[ApocalypseTempleDoorLogic setIslandLocationState:]` @ 0x100135ef0
pub fn set_island_location_state(world: &mut World, this: Entity, island_location_state: i32) {
    world.obj_mut::<ApocalypseTempleDoorLogic>(this).island_location_state = island_location_state;
}

/// `-[ApocalypseTempleDoorLogic setApocalypseDoorType:]` @ 0x100135f00
pub fn set_apocalypse_door_type(world: &mut World, this: Entity, apocalypse_door_type: i32) {
    world.obj_mut::<ApocalypseTempleDoorLogic>(this).apocalypse_door_type = apocalypse_door_type;
}

/// `-[ApocalypseTempleDoorLogic setAlwaysOpen:]` @ 0x100135f10
pub fn set_always_open(world: &mut World, this: Entity, always_open: bool) {
    world.obj_mut::<ApocalypseTempleDoorLogic>(this).always_open = always_open;
}

/// `-[ApocalypseTempleDoorLogic isAvailable]` @ 0x100135f20
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let door = world.obj::<ApocalypseTempleDoorLogic>(this);
    if door.pygmy_logic.is_some() {
        return false;
    }
    door.open || door.always_open
}

/// `-[ApocalypseTempleDoorLogic fallIntoApocalypseTempleDoor:]` @ 0x100135f68
pub fn fall_into_apocalypse_temple_door(world: &mut World, this: Entity, pygmy: Entity) {
    world.obj_mut::<ApocalypseTempleDoorLogic>(this).pygmy_logic = Some(pygmy);

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);

    let door_type = world.obj::<ApocalypseTempleDoorLogic>(this).apocalypse_door_type;
    let behavior = match door_type {
        0 => "ApocalypseTempleDoorPygmyFallEntrance",
        1 => "ApocalypseTempleDoorPygmyFallPortal",
        2 => "ApocalypseTempleDoorPygmyFallTimeTempleSideLeft",
        3 => "ApocalypseTempleDoorPygmyFallTimeTempleSideRight",
        4 => "ApocalypseTempleDoorPygmyFallLocustTempleSideLeft",
        5 => "ApocalypseTempleDoorPygmyFallLocustTempleSideRight",
        6 => "ApocalypseTempleDoorPygmyFallDanceTempleSideLeft",
        7 => "ApocalypseTempleDoorPygmyFallDanceTempleSideRight",
        8 => "ApocalypseTempleDoorPygmyFallPlagueTempleSideLeft",
        9 => "ApocalypseTempleDoorPygmyFallPlagueTempleSideRight",
        10 => "ApocalypseTempleDoorPygmyFallPurificationTempleSideLeft",
        11 => "ApocalypseTempleDoorPygmyFallPurificationTempleSideRight",
        12 => "ApocalypseTempleDoorPygmyFallClosetTempleSideLeft",
        13 => "ApocalypseTempleDoorPygmyFallXenuTempleSideLeft",
        _ => return,
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApocalypseTempleDoorLogic ejectFromApocalypseTempleDoor:]` @ 0x10013618c
pub fn eject_from_apocalypse_temple_door(world: &mut World, this: Entity, _pygmy: Entity) {
    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApocalypseTempleDoorEjectPygmy"));
    world.obj_mut::<ApocalypseTempleDoorLogic>(this).pygmy_logic = None;
}

/// `-[ApocalypseTempleDoorLogic enterIsland:]` @ 0x1001361f4
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let active = world.obj::<ApocalypseTempleDoorLogic>(this).active;
    if active {
        return;
    }

    {
        let mut door = world.obj_mut::<ApocalypseTempleDoorLogic>(this);
        door.active = true;
        door.open = false;
    }
    logic::set_behavior(world, this, Some("ApocalypseTempleDoorEnterIsland"));
}

/// `-[ApocalypseTempleDoorLogic exitIsland:]` @ 0x100136234
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let active = world.obj::<ApocalypseTempleDoorLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<ApocalypseTempleDoorLogic>(this).active = false;

    let pygmy = world.obj::<ApocalypseTempleDoorLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::eject_from_apocalypse_temple_door(world, pygmy);
        world.obj_mut::<ApocalypseTempleDoorLogic>(this).pygmy_logic = None;
    }
    logic::set_behavior(world, this, Some("ApocalypseTempleDoorExitIsland"));
}

/// `-[ApocalypseTempleDoorLogic exitIslandNow]` @ 0x1001362d4
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ApocalypseTempleDoorLogic onGameFrame:]` @ 0x10013634c
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ApocalypseTempleDoorLogic onPygmyFallInsideComplete:::]` @ 0x100136350
pub fn seq_on_pygmy_fall_inside_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    // The original also asks the listener for `islandLocationState` and ignores the answer.
    let island_location_state = world.obj::<ApocalypseTempleDoorLogic>(this).island_location_state;
    let room_fader = pg_engine::library::logic(world, crate::logic_table::APOCALYPSE_ROOM_FADER);
    apocalypse_room_fader_logic::activate(world, room_fader, island_location_state);

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApocalypseTempleDoorEjectPygmy"));
    world.obj_mut::<ApocalypseTempleDoorLogic>(this).pygmy_logic = None;
    ItemResult::Done
}

/// The `setAnimationBasedOnPygmyType:::` parameter for the pygmy held by a time-temple door or
/// platform: `normal`, or by `timeTemplePygmyType` 1–4 `oldFart`, `baby`, `ginger`, `fast`
/// (other types keep `normal`). Shared with `ApocalypseTimeTemplePlatformLogic`.
pub(crate) fn pygmy_type_animation_id<'a>(world: &mut World, call: &'a SequenceCall, pygmy: Option<Entity>) -> Option<&'a str> {
    let normal = call.param("normal");
    let Some(pygmy) = pygmy else {
        return normal;
    };

    let time_temple_pygmy_type = crate::pygmy::time_temple_pygmy_type(world, pygmy);
    match time_temple_pygmy_type {
        0 => normal,
        1 => call.param("oldFart"),
        2 => call.param("baby"),
        3 => call.param("ginger"),
        4 => call.param("fast"),
        _ => normal,
    }
}

/// `-[ApocalypseTempleDoorLogic setAnimationBasedOnPygmyType:::]` @ 0x100136420
pub fn seq_set_animation_based_on_pygmy_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let pygmy = world.obj::<ApocalypseTempleDoorLogic>(this).pygmy_logic;
    let animation_id = pygmy_type_animation_id(world, call, pygmy);

    let animation = logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// `-[ApocalypseTempleDoorLogic setAnimationBasedOnDoorType:::]` @ 0x100136504
pub fn seq_set_animation_based_on_door_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let door_type = world.obj::<ApocalypseTempleDoorLogic>(this).apocalypse_door_type;
    let animation_id = match door_type {
        0 => call.param("entrance"),
        1 => call.param("portal"),
        2 => call.param("timeSideLeft"),
        3 => call.param("timeSideRight"),
        4 => call.param("locustSideLeft"),
        5 => call.param("locustSideRight"),
        6 => call.param("danceSideLeft"),
        7 => call.param("danceSideRight"),
        8 => call.param("plagueSideLeft"),
        9 => call.param("plagueSideRight"),
        10 => call.param("purificationSideLeft"),
        11 => call.param("purificationSideRight"),
        12 => call.param("closetSideLeft"),
        13 => call.param("xenuSideLeft"),
        _ => None,
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// `-[ApocalypseTempleDoorLogic touchBeganInside:allTouches:]` @ 0x100136644
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let door = world.obj::<ApocalypseTempleDoorLogic>(this);
    if door.always_open {
        return;
    }

    let listener = door.apocalypse_temple_door_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    let open = world.obj::<ApocalypseTempleDoorLogic>(this).open;
    let behavior = if open {
        world.obj_mut::<ApocalypseTempleDoorLogic>(this).open = false;
        "ApocalypseTempleDoorClose"
    } else {
        world.obj_mut::<ApocalypseTempleDoorLogic>(this).open = true;
        "ApocalypseTempleDoorOpen"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApocalypseTempleDoorLogic touchBeganOutside:allTouches:]` @ 0x1001366e8
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTempleDoorLogic touchMovedEntered:allTouches:]` @ 0x1001366ec
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTempleDoorLogic touchMovedExited:allTouches:]` @ 0x1001366f0
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTempleDoorLogic touchMovedInside:allTouches:]` @ 0x1001366f4
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[ApocalypseTempleDoorLogic touchMovedOutside:allTouches:]` @ 0x100136704
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTempleDoorLogic touchEndedInside:allTouches:]` @ 0x100136708
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[ApocalypseTempleDoorLogic touchEndedOutside:allTouches:]` @ 0x10013670c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
