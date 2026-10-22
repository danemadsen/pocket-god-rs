//! `ApocalypseTimeTemplePlatformLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_temples::apocalypse_temple_door_logic;
use pg_engine::display;
use pg_engine::logic;

/// `ApocalypseTimeTemplePlatformLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseTimeTemplePlatformLogic {
    /// `mApocalypseTimeTemplePlatformLogicListener`: `@"<ApocalypseTimeTemplePlatformLogicListener>"`
    pub apocalypse_time_temple_platform_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mMovePercent`: `f`
    pub move_percent: f32,

    /// `mbMovePlatform`: `B`
    pub move_platform: bool,

    /// `mTargetXPos`: `f`
    pub target_x_pos: f32,

    /// `mActiveStage`: `i`
    pub active_stage: i32,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const APOCALYPSE_TIME_TEMPLE_PLATFORM_LOGIC: Class = Class {
    name: "ApocalypseTimeTemplePlatformLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The x of the first of the four stages and the distance between stages.
const FIRST_STAGE_X_POS: f32 = 145.0;
const STAGE_SPACING: f32 = 66.666664;

/// The x of the last stage (`FIRST_STAGE_X_POS + 3 · STAGE_SPACING`).
const LAST_STAGE_X_POS: f32 = 345.0;

/// How far a fully pulled lever moves the platform per second.
const MOVE_SPEED: f32 = 100.0;

/// The animation channel the pygmy on the platform is bound to.
const PYGMY_CHANNEL: usize = 1;

/// Sequence-item handlers implemented by `ApocalypseTimeTemplePlatformLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "setAnimationBasedOnPygmyType" => seq_set_animation_based_on_pygmy_type(world, this, call),
        "onTransformComplete" => seq_on_transform_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApocalypseTimeTemplePlatformLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_TIME_TEMPLE_PLATFORM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[ApocalypseTimeTemplePlatformLogic initWithDisplayObject:]` @ 0x100182350
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseTimeTemplePlatformLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().apocalypse_time_temple_platform_logic = Some(this);
    logic::stop_game_frame(world, this);
}

/// `-[ApocalypseTimeTemplePlatformLogic setApocalypseTimeTemplePlatformLogicListener:]` @ 0x1001823bc
pub fn set_apocalypse_time_temple_platform_logic_listener(world: &mut World, this: Entity, apocalypse_time_temple_platform_logic_listener: Entity) {
    world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this).apocalypse_time_temple_platform_logic_listener = Some(apocalypse_time_temple_platform_logic_listener);
}

/// `-[ApocalypseTimeTemplePlatformLogic isAvailable]` @ 0x100182418
pub fn is_available(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseTimeTemplePlatformLogic>(this).pygmy_logic.is_none()
}

/// `-[ApocalypseTimeTemplePlatformLogic movePlatform:]` @ 0x100182430
pub fn move_platform(world: &mut World, this: Entity, move_platform: f32) {
    let mut platform = world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this);
    platform.move_platform = true;
    platform.move_percent = move_platform;
}

/// `-[ApocalypseTimeTemplePlatformLogic stopMovingPlatform]` @ 0x100182450
pub fn stop_moving_platform(world: &mut World, this: Entity) {
    world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this).move_platform = false;

    // Snap to the nearest stage (`floorf`, not a truncating cast).
    let display_object = logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;
    let stage = ((x_pos + -FIRST_STAGE_X_POS) / STAGE_SPACING + 0.5).floor();

    let mut platform = world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this);
    platform.active_stage = stage as i32;
    platform.target_x_pos = stage * STAGE_SPACING + FIRST_STAGE_X_POS;
}

/// `-[ApocalypseTimeTemplePlatformLogic fallOntoApocalypseTimeTemplePlatform:]` @ 0x1001824f0
pub fn fall_onto_apocalypse_time_temple_platform(world: &mut World, this: Entity, pygmy: Entity) {
    world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this).pygmy_logic = Some(pygmy);

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApocalypseTimeTemplePlatformPygmyFall"));
}

/// `-[ApocalypseTimeTemplePlatformLogic ejectFromApocalypseTimeTemplePlatform:]` @ 0x100182570
pub fn eject_from_apocalypse_time_temple_platform(world: &mut World, this: Entity, _pygmy: Entity) {
    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
    logic::set_behavior(world, this, Some("ApocalypseTimeTemplePlatformEjectPygmy"));
    world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this).pygmy_logic = None;
}

/// `-[ApocalypseTimeTemplePlatformLogic enterIsland:]` @ 0x1001825d8
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let active = world.obj::<ApocalypseTimeTemplePlatformLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this).active = true;
    stop_moving_platform(world, this);
    logic::set_behavior(world, this, Some("ApocalypseTimeTemplePlatformEnterIsland"));
}

/// `-[ApocalypseTimeTemplePlatformLogic exitIsland:]` @ 0x100182644
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let active = world.obj::<ApocalypseTimeTemplePlatformLogic>(this).active;
    if !active {
        return;
    }

    world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this).active = false;

    let pygmy = world.obj::<ApocalypseTimeTemplePlatformLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, PYGMY_CHANNEL);
        crate::pygmy::eject_from_apocalypse_time_temple_platform(world, pygmy);
        world.obj_mut::<ApocalypseTimeTemplePlatformLogic>(this).pygmy_logic = None;
    }
    logic::set_behavior(world, this, Some("ApocalypseTimeTemplePlatformExitIsland"));
}

/// `-[ApocalypseTimeTemplePlatformLogic exitIslandNow]` @ 0x1001826e4
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ApocalypseTimeTemplePlatformLogic onGameFrame:]` @ 0x10018275c
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let platform = world.obj::<ApocalypseTimeTemplePlatformLogic>(this);
    let move_platform = platform.move_platform;
    let move_percent = platform.move_percent;
    let target_x_pos = platform.target_x_pos;
    let pygmy = platform.pygmy_logic;

    let display_object = logic::display_object(world, this);
    let x_pos = {
        let mut display = display::object_mut(world, display_object);
        if move_platform {
            let moved_x_pos = display.pos.x + move_percent * MOVE_SPEED * dt;
            display.pos.x = if moved_x_pos < FIRST_STAGE_X_POS { FIRST_STAGE_X_POS } else { moved_x_pos };
            if display.pos.x > LAST_STAGE_X_POS {
                display.pos.x = LAST_STAGE_X_POS;
            }
        } else {
            display.pos.x = display.pos.x + (target_x_pos - display.pos.x) * 0.5;
        }
        display.pos.x
    };

    if let Some(pygmy) = pygmy {
        let pygmy_display_object = logic::display_object(world, pygmy);
        display::object_mut(world, pygmy_display_object).pos.x = x_pos;
    }
}

/// `-[ApocalypseTimeTemplePlatformLogic setAnimationBasedOnPygmyType:::]` @ 0x100182884
pub fn seq_set_animation_based_on_pygmy_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let pygmy = world.obj::<ApocalypseTimeTemplePlatformLogic>(this).pygmy_logic;
    let animation_id = apocalypse_temple_door_logic::pygmy_type_animation_id(world, call, pygmy);

    let animation = logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// `-[ApocalypseTimeTemplePlatformLogic onTransformComplete:::]` @ 0x100182968
pub fn seq_on_transform_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let platform = world.obj::<ApocalypseTimeTemplePlatformLogic>(this);
    let Some(pygmy) = platform.pygmy_logic else {
        return ItemResult::Done;
    };
    let active_stage = platform.active_stage;

    let (episode_event, time_temple_pygmy_type) = match active_stage {
        0 => (Some("CreateOldFartPygmy"), 1),
        1 => (Some("CreateBabyPygmy"), 2),
        2 => (Some("CreateGingerPygmy"), 3),
        3 => (Some("CreateFastPygmy"), 4),
        _ => (None, 1),
    };
    if let Some(episode_event) = episode_event {
        let idol_manager = crate::managers::idol_manager::instance(world);
        if let Some(idol_manager) = idol_manager {
            crate::managers::idol_manager::trigger_episode_event(world, idol_manager, episode_event);
        }
    }

    // Transforming into the type the pygmy already has turns it back to normal.
    let current_type = crate::pygmy::time_temple_pygmy_type(world, pygmy);
    let new_type = if time_temple_pygmy_type == current_type { 0 } else { time_temple_pygmy_type };
    crate::pygmy::set_time_temple_pygmy_type(world, pygmy, new_type);
    ItemResult::Done
}

/// `-[ApocalypseTimeTemplePlatformLogic onButtonPress:touch:]` @ 0x100182af4
pub fn on_button_press(world: &mut World, this: Entity, _button: Entity, _touch: Entity) {
    let platform = world.obj::<ApocalypseTimeTemplePlatformLogic>(this);
    let listener = platform.apocalypse_time_temple_platform_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    let pygmy = world.obj::<ApocalypseTimeTemplePlatformLogic>(this).pygmy_logic;
    let behavior = match pygmy {
        Some(_) => "ApocalypseTimeTemplePlatformEnergyPygmy",
        None => "ApocalypseTimeTemplePlatformEnergy",
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApocalypseTimeTemplePlatformLogic onButtonRelease:touch:]` @ 0x100182b70
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[ApocalypseTimeTemplePlatformLogic onButtonReleaseOutside:touch:]` @ 0x100182b74
pub fn on_button_release_outside(world: &mut World, this: Entity, button: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.on_button_release)(world, this, button, touch);
}

/// `+[ApocalypseTimeTemplePlatformLogic instance]` @ 0x100182344
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().apocalypse_time_temple_platform_logic
}
