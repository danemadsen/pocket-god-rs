//! `ApocalypseGemPedestalLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_temples::apocalypse_gem_logic;
use pg_engine::display;
use pg_engine::logic;

/// `ApocalypseGemPedestalLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ApocalypseGemPedestalLogic {
    /// `mApocalypseGemPedestalLogicListener`: `@"<ApocalypseGemPedestalLogicListener>"`
    pub apocalypse_gem_pedestal_logic_listener: Option<Entity>,

    /// `mApocalypseGemType`: `i`
    pub apocalypse_gem_type: i32,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbUnlightingTorch`: `B`
    pub unlighting_torch: bool,

    /// `mApocalypseGemLogic`: `@"ApocalypseGemLogic"`
    pub apocalypse_gem_logic: Option<Entity>,
}

pub const APOCALYPSE_GEM_PEDESTAL_LOGIC: Class = Class {
    name: "ApocalypseGemPedestalLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The animation channel the seated gem is bound to.
const GEM_CHANNEL: usize = 2;

/// Sequence-item handlers implemented by `ApocalypseGemPedestalLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "unlightTorchComplete" => seq_unlight_torch_complete(world, this, call),
        "setAnimationBasedOnGemType" => seq_set_animation_based_on_gem_type(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[ApocalypseGemPedestalLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&APOCALYPSE_GEM_PEDESTAL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mApocalypseGemPedestalLogicListener isApocalypseActive]` (NO for a nil listener).
fn is_apocalypse_active(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_pedestal_logic_listener;
    match listener {
        Some(listener) => crate::controller::is_apocalypse_active(world, listener),
        None => false,
    }
}

/// Lets go of the seated gem: `[mAnimation clearTweenableChannelIdNow:2]`,
/// `[mApocalypseGemLogic ejectFromApocalypseGemPedestal]`, `mApocalypseGemLogic = nil`
/// (`unlightTorchNow`, `exitIsland:`, `unlightTorchComplete:::`).
fn release_apocalypse_gem(world: &mut World, this: Entity) {
    let gem = world.obj::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_logic;
    let Some(gem) = gem else {
        return;
    };

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, GEM_CHANNEL);
    apocalypse_gem_logic::eject_from_apocalypse_gem_pedestal(world, gem);
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_logic = None;
}

/// `-[ApocalypseGemPedestalLogic initWithDisplayObject:]` @ 0x100069150
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(ApocalypseGemPedestalLogic::default());
    crate::logic::island_core::sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
}

/// `-[ApocalypseGemPedestalLogic setApocalypseGemPedestalLogicListener:]` @ 0x1000691b4
pub fn set_apocalypse_gem_pedestal_logic_listener(world: &mut World, this: Entity, apocalypse_gem_pedestal_logic_listener: Entity) {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_pedestal_logic_listener = Some(apocalypse_gem_pedestal_logic_listener);
}

/// `-[ApocalypseGemPedestalLogic setApocalypseGemType:]` @ 0x100069210
pub fn set_apocalypse_gem_type(world: &mut World, this: Entity, apocalypse_gem_type: i32) {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_type = apocalypse_gem_type;
}

/// `-[ApocalypseGemPedestalLogic apocalypseGemType]` @ 0x100069220
pub fn apocalypse_gem_type(world: &mut World, this: Entity) -> i32 {
    world.obj::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_type
}

/// `-[ApocalypseGemPedestalLogic isAvailable]` @ 0x100069230
pub fn is_available(world: &mut World, this: Entity) -> bool {
    world.obj::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_logic.is_none()
}

/// `-[ApocalypseGemPedestalLogic lightTorch]` @ 0x100069248
pub fn light_torch(world: &mut World, this: Entity) {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).unlighting_torch = false;
    logic::set_behavior(world, this, Some("ApocalypseGemPedestalLightTorch"));
}

/// `-[ApocalypseGemPedestalLogic elevateGem]` @ 0x10006926c
pub fn elevate_gem(world: &mut World, this: Entity) {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).unlighting_torch = true;
    logic::set_behavior(world, this, Some("ApocalypseGemPedestalElevateGem"));
}

/// `-[ApocalypseGemPedestalLogic unlightTorch]` @ 0x100069294
pub fn unlight_torch(world: &mut World, this: Entity) {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).unlighting_torch = true;
    logic::set_behavior(world, this, Some("ApocalypseGemPedestalUnlightTorch"));
}

/// `-[ApocalypseGemPedestalLogic unlightTorchNow]` @ 0x1000692bc
pub fn unlight_torch_now(world: &mut World, this: Entity) {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).unlighting_torch = false;
    release_apocalypse_gem(world, this);
    logic::set_behavior(world, this, Some("ApocalypseGemPedestalEnterIsland"));
}

/// `-[ApocalypseGemPedestalLogic startTorchLit]` @ 0x100069348
pub fn start_torch_lit(world: &mut World, this: Entity) {
    let apocalypse_active = is_apocalypse_active(world, this);

    let pedestal = world.obj::<ApocalypseGemPedestalLogic>(this);
    let gem = pedestal.apocalypse_gem_logic;
    let pedestal_gem_type = pedestal.apocalypse_gem_type;

    let behavior = if !apocalypse_active {
        "ApocalypseGemPedestalTorchLit"
    } else if let Some(gem) = gem {
        let gem_type = apocalypse_gem_logic::apocalypse_gem_type(world, gem);
        if gem_type == pedestal_gem_type {
            "ApocalypseGemPedestalApocalypseGemFallMatch"
        } else {
            "ApocalypseGemPedestalApocalypseGemFall"
        }
    } else {
        "ApocalypseGemPedestalTorchLitApocalypse"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[ApocalypseGemPedestalLogic hasCorrectGem]` @ 0x10006940c
pub fn has_correct_gem(world: &mut World, this: Entity) -> bool {
    let pedestal = world.obj::<ApocalypseGemPedestalLogic>(this);
    let gem = pedestal.apocalypse_gem_logic;
    let pedestal_gem_type = pedestal.apocalypse_gem_type;

    let Some(gem) = gem else {
        return false;
    };
    let gem_type = apocalypse_gem_logic::apocalypse_gem_type(world, gem);
    gem_type == pedestal_gem_type
}

/// `-[ApocalypseGemPedestalLogic apocalypseGemFallIntoApocalypseGemPedestal:]` @ 0x100069464
pub fn apocalypse_gem_fall_into_apocalypse_gem_pedestal(world: &mut World, this: Entity, gem: Entity) {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_logic = Some(gem);

    let animation = logic::animation(world, this);
    let gem_display_object = logic::display_object(world, gem);
    pg_engine::animation::set_tweenable(world, animation, Some(gem_display_object), GEM_CHANNEL);

    let gem_type = apocalypse_gem_logic::apocalypse_gem_type(world, gem);
    let pedestal_gem_type = world.obj::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_type;
    let behavior = if gem_type != pedestal_gem_type {
        "ApocalypseGemPedestalApocalypseGemFall"
    } else {
        let episode_event = match gem_type {
            1 => Some("TimeGemInPedestal"),
            2 => Some("LocustGemInPedestal"),
            3 => Some("DanceGemInPedestal"),
            4 => Some("PlagueGemInPedestal"),
            5 => Some("PurificationGemInPedestal"),
            6 => Some("ClosetGemInPedestal"),
            7 => Some("XenuGemInPedestal"),
            _ => None,
        };
        if let Some(episode_event) = episode_event {
            let idol_manager = crate::managers::idol_manager::instance(world);
            if let Some(idol_manager) = idol_manager {
                crate::managers::idol_manager::trigger_episode_event(world, idol_manager, episode_event);
            }
        }
        "ApocalypseGemPedestalApocalypseGemFallMatch"
    };
    logic::set_behavior(world, this, Some(behavior));

    // Every pedestal holding its own gem ends the apocalypse.
    let listener = world.obj::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_pedestal_logic_listener;
    let Some(listener) = listener else {
        return;
    };
    let pedestals = crate::controller::apocalypse_gem_pedestal_logic_array(world, listener, this);
    for pedestal in pedestals {
        let correct = has_correct_gem(world, pedestal);
        if !correct {
            return;
        }
    }
    crate::controller::end_apocalypse(world, listener);
}

/// `-[ApocalypseGemPedestalLogic ejectApocalypseGemFromApocalypseGemPedestal:]` @ 0x1000697e0
pub fn eject_apocalypse_gem_from_apocalypse_gem_pedestal(world: &mut World, this: Entity, _gem: Entity) {
    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, GEM_CHANNEL);

    let apocalypse_active = is_apocalypse_active(world, this);
    let behavior = if apocalypse_active {
        "ApocalypseGemPedestalEjectApocalypseGemApocalypse"
    } else {
        "ApocalypseGemPedestalEjectApocalypseGem"
    };
    logic::set_behavior(world, this, Some(behavior));

    world.obj_mut::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_logic = None;
}

/// `-[ApocalypseGemPedestalLogic enterIsland:]` @ 0x100069874
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    let active = world.obj::<ApocalypseGemPedestalLogic>(this).active;
    if active {
        return;
    }

    world.obj_mut::<ApocalypseGemPedestalLogic>(this).active = true;
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = true;
    logic::set_behavior(world, this, Some("ApocalypseGemPedestalEnterIsland"));
}

/// `-[ApocalypseGemPedestalLogic exitIsland:]` @ 0x1000698f0
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    let active = world.obj::<ApocalypseGemPedestalLogic>(this).active;
    if active {
        world.obj_mut::<ApocalypseGemPedestalLogic>(this).active = false;
        let display_object = logic::display_object(world, this);
        display::object_mut(world, display_object).is_selectable = false;
        logic::set_behavior(world, this, Some("ApocalypseGemPedestalExitIsland"));
    }

    let unlighting_torch = world.obj::<ApocalypseGemPedestalLogic>(this).unlighting_torch;
    if unlighting_torch {
        world.obj_mut::<ApocalypseGemPedestalLogic>(this).unlighting_torch = false;
        release_apocalypse_gem(world, this);
    }
}

/// `-[ApocalypseGemPedestalLogic exitIslandNow]` @ 0x1000699c0
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[ApocalypseGemPedestalLogic onGameFrame:]` @ 0x100069a38
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[ApocalypseGemPedestalLogic unlightTorchComplete:::]` @ 0x100069a3c
pub fn seq_unlight_torch_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ApocalypseGemPedestalLogic>(this).unlighting_torch = false;
    release_apocalypse_gem(world, this);
    ItemResult::Done
}

/// `-[ApocalypseGemPedestalLogic setAnimationBasedOnGemType:::]` @ 0x100069ab4
pub fn seq_set_animation_based_on_gem_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let gem_type = world.obj::<ApocalypseGemPedestalLogic>(this).apocalypse_gem_type;
    let animation_id = apocalypse_gem_logic::gem_type_animation_id(call, gem_type);

    let animation = logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}
