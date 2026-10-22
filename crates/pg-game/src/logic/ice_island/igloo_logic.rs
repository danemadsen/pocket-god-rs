//! `IglooLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::ocean_fishing::fish_logic;

/// `IglooLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IglooLogic {
    /// `mIglooLogicListener`: `@"<IglooLogicListener>"`
    pub igloo_logic_listener: Option<Entity>,

    /// `mbInTransition`: `B`
    pub in_transition: bool,

    /// `mRestingPygmyLogic`: `@"PygmyLogic"`
    pub resting_pygmy_logic: Option<Entity>,

    /// `mLeavingPygmyLogic`: `@"PygmyLogic"`
    pub leaving_pygmy_logic: Option<Entity>,

    /// `mFishLogic`: `@"FishLogic"`
    pub fish_logic: Option<Entity>,

    /// `mKnockTouchTime`: `f`
    pub knock_touch_time: f32,

    /// `mKnockTouchCount`: `i`
    pub knock_touch_count: i32,
}

pub const IGLOO_LOGIC: Class = Class {
    name: "IglooLogic",
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

/// Tweenable channel of the pygmy resting in the igloo.
const RESTING_PYGMY_CHANNEL: usize = 1;

/// Tweenable channel of the pygmy a newcomer pushes out.
const LEAVING_PYGMY_CHANNEL: usize = 2;

/// Sequence-item handlers implemented by `IglooLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "startTransition" => seq_start_transition(world, this, call),
        "endTransition" => seq_end_transition(world, this, call),
        "checkForFishDinner" => seq_check_for_fish_dinner(world, this, call),
        "checkForFishOnly" => seq_check_for_fish_only(world, this, call),
        "eatFishComplete" => seq_eat_fish_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `IglooLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animPygmyExitTopExplode" => anim_pygmy_exit_top_explode(world, this),
        "animPygmyExitDoorExplode" => anim_pygmy_exit_door_explode(world, this),
        "animPygmyCrawlOut" => anim_pygmy_crawl_out(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[IglooLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&IGLOO_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[IglooLogic initWithDisplayObject:]` @ 0x1000c33d0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IglooLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(String::from("Igloo"));
    pg_engine::logic::set_behavior(world, this, Some("IglooExitIsland"));
}

/// `-[IglooLogic setIglooLogicListener:]` @ 0x1000c3468
pub fn set_igloo_logic_listener(world: &mut World, this: Entity, igloo_logic_listener: Entity) {
    world.obj_mut::<IglooLogic>(this).igloo_logic_listener = Some(igloo_logic_listener);
}

/// `-[IglooLogic isPygmyCrawlAvailable:]` @ 0x1000c34c4
pub fn is_pygmy_crawl_available(world: &mut World, this: Entity, _pygmy_crawl_available: Entity) -> bool {
    let igloo = world.obj::<IglooLogic>(this);
    igloo.fish_logic.is_some() && igloo.resting_pygmy_logic.is_none() && !igloo.in_transition
}

/// `-[IglooLogic isPygmyFallAvailable:]` @ 0x1000c3504
pub fn is_pygmy_fall_available(world: &mut World, this: Entity, _pygmy_fall_available: Entity) -> bool {
    !world.obj::<IglooLogic>(this).in_transition
}

/// `-[IglooLogic isFishFallAvailable:]` @ 0x1000c351c
pub fn is_fish_fall_available(world: &mut World, this: Entity, _fish_fall_available: Entity) -> bool {
    !world.obj::<IglooLogic>(this).in_transition
}

/// `-[IglooLogic pygmyCrawlIntoIgloo:]` @ 0x1000c3534
pub fn pygmy_crawl_into_igloo(world: &mut World, this: Entity, pygmy: Entity) {
    world.obj_mut::<IglooLogic>(this).resting_pygmy_logic = Some(pygmy);

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), RESTING_PYGMY_CHANNEL);

    pg_engine::logic::set_behavior(world, this, Some("IglooPygmyCrawlInside"));
}

/// `-[IglooLogic pygmyFallIntoIgloo:]` @ 0x1000c35b4
///
/// A pygmy falling in on top of a resting one becomes the resting pygmy and pushes the old
/// one out.
pub fn pygmy_fall_into_igloo(world: &mut World, this: Entity, pygmy: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    let previous_pygmy = world.obj::<IglooLogic>(this).resting_pygmy_logic;

    let behavior = match previous_pygmy {
        None => {
            world.obj_mut::<IglooLogic>(this).resting_pygmy_logic = Some(pygmy);
            let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
            pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), RESTING_PYGMY_CHANNEL);
            "IglooPygmyFallInside"
        }
        Some(previous_pygmy) => {
            world.obj_mut::<IglooLogic>(this).leaving_pygmy_logic = Some(previous_pygmy);
            let previous_display_object = pg_engine::logic::display_object(world, previous_pygmy);
            pg_engine::animation::set_tweenable(world, animation, Some(previous_display_object), LEAVING_PYGMY_CHANNEL);

            world.obj_mut::<IglooLogic>(this).resting_pygmy_logic = Some(pygmy);
            let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
            pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), RESTING_PYGMY_CHANNEL);

            let igloo = world.obj::<IglooLogic>(this);
            if igloo.resting_pygmy_logic.is_none() || igloo.fish_logic.is_none() {
                "IglooMakeRoomForNewPygmy"
            } else {
                "IglooMakeRoomForNewPygmyCooking"
            }
        }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[IglooLogic fishFallIntoIgloo:]` @ 0x1000c3718
pub fn fish_fall_into_igloo(world: &mut World, this: Entity, fish: Entity) {
    world.obj_mut::<IglooLogic>(this).fish_logic = Some(fish);
    pg_engine::logic::set_behavior(world, this, Some("IglooFishFallInside"));
}

/// `-[IglooLogic enterIsland:]` @ 0x1000c373c
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    pg_engine::logic::start_game_frame(world, this);
    pg_engine::logic::set_behavior(world, this, Some("IglooEnterIsland"));
}

/// `-[IglooLogic removePygmyFromIgloo:]` @ 0x1000c3780
pub fn remove_pygmy_from_igloo(world: &mut World, this: Entity, pygmy: Entity) {
    let animation = pg_engine::logic::animation(world, this);

    let resting_pygmy = world.obj::<IglooLogic>(this).resting_pygmy_logic;
    if resting_pygmy == Some(pygmy) {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, RESTING_PYGMY_CHANNEL);
        world.obj_mut::<IglooLogic>(this).resting_pygmy_logic = None;

        let fish = world.obj::<IglooLogic>(this).fish_logic;
        let behavior = if fish.is_none() { "IglooEnterIsland" } else { "IglooFishOnly" };
        pg_engine::logic::set_behavior(world, this, Some(behavior));
    }

    let leaving_pygmy = world.obj::<IglooLogic>(this).leaving_pygmy_logic;
    if leaving_pygmy != Some(pygmy) {
        return;
    }
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, LEAVING_PYGMY_CHANNEL);
    world.obj_mut::<IglooLogic>(this).leaving_pygmy_logic = None;
    pg_engine::logic::set_behavior(world, this, Some("IglooEnterIsland"));
}

/// `-[IglooLogic exitIsland:]` @ 0x1000c3880
///
/// Tosses out whoever is inside; `exitIsland` set also stops the igloo's game frame.
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let animation = pg_engine::logic::animation(world, this);

    let resting_pygmy = world.obj::<IglooLogic>(this).resting_pygmy_logic;
    if let Some(resting_pygmy) = resting_pygmy {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, RESTING_PYGMY_CHANNEL);
        crate::pygmy::toss_from_igloo(world, resting_pygmy);
        world.obj_mut::<IglooLogic>(this).resting_pygmy_logic = None;
    }

    let leaving_pygmy = world.obj::<IglooLogic>(this).leaving_pygmy_logic;
    if let Some(leaving_pygmy) = leaving_pygmy {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, LEAVING_PYGMY_CHANNEL);
        crate::pygmy::toss_from_igloo(world, leaving_pygmy);
        world.obj_mut::<IglooLogic>(this).leaving_pygmy_logic = None;
    }

    let fish = world.obj::<IglooLogic>(this).fish_logic;
    if let Some(fish) = fish {
        fish_logic::toss_from_igloo(world, fish);
        world.obj_mut::<IglooLogic>(this).fish_logic = None;
    }

    if exit_island {
        pg_engine::logic::stop_game_frame(world, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("IglooExitIsland"));
}

/// `-[IglooLogic onGameFrame:]` @ 0x1000c39f8
///
/// Knocks more than a second apart start the count again.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let mut igloo = world.obj_mut::<IglooLogic>(this);
    igloo.knock_touch_time = igloo.knock_touch_time + dt;
    if 1.0 < igloo.knock_touch_time {
        igloo.knock_touch_count = 0;
    }
}

/// `-[IglooLogic startTransition:::]` @ 0x1000c3a3c
pub fn seq_start_transition(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<IglooLogic>(this).in_transition = true;
    ItemResult::Done
}

/// `-[IglooLogic endTransition:::]` @ 0x1000c3a54
pub fn seq_end_transition(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<IglooLogic>(this).in_transition = false;
    ItemResult::Done
}

/// `-[IglooLogic checkForFishDinner:::]` @ 0x1000c3a68
pub fn seq_check_for_fish_dinner(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let igloo = world.obj::<IglooLogic>(this);
    let dinner_ready = igloo.resting_pygmy_logic.is_some() && igloo.fish_logic.is_some();
    if dinner_ready {
        pg_engine::logic::set_behavior(world, this, Some("IglooEatFish"));
    }
    ItemResult::Done
}

/// `-[IglooLogic checkForFishOnly:::]` @ 0x1000c3ab4
pub fn seq_check_for_fish_only(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let igloo = world.obj::<IglooLogic>(this);
    let fish_only = igloo.resting_pygmy_logic.is_none() && igloo.fish_logic.is_some();
    if fish_only {
        pg_engine::logic::set_behavior(world, this, Some("IglooFishOnly"));
    }
    ItemResult::Done
}

/// `-[IglooLogic eatFishComplete:::]` @ 0x1000c3b00
pub fn seq_eat_fish_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let fish = world.obj::<IglooLogic>(this).fish_logic;
    if let Some(fish) = fish {
        fish_logic::finish_eating_in_igloo(world, fish);
    }
    world.obj_mut::<IglooLogic>(this).fish_logic = None;
    ItemResult::Done
}

/// `-[IglooLogic animPygmyExitTopExplode]` @ 0x1000c3b40
pub fn anim_pygmy_exit_top_explode(world: &mut World, this: Entity) {
    let resting_pygmy = world.obj::<IglooLogic>(this).resting_pygmy_logic;
    let Some(resting_pygmy) = resting_pygmy else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, RESTING_PYGMY_CHANNEL);
    crate::pygmy::leave_igloo_top_explode(world, resting_pygmy);
    world.obj_mut::<IglooLogic>(this).resting_pygmy_logic = None;
}

/// `-[IglooLogic animPygmyExitDoorExplode]` @ 0x1000c3ba8
pub fn anim_pygmy_exit_door_explode(world: &mut World, this: Entity) {
    let leaving_pygmy = world.obj::<IglooLogic>(this).leaving_pygmy_logic;
    let Some(leaving_pygmy) = leaving_pygmy else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, LEAVING_PYGMY_CHANNEL);
    crate::pygmy::leave_igloo_door_explode(world, leaving_pygmy);
    world.obj_mut::<IglooLogic>(this).leaving_pygmy_logic = None;
}

/// `-[IglooLogic animPygmyCrawlOut]` @ 0x1000c3c10
pub fn anim_pygmy_crawl_out(world: &mut World, this: Entity) {
    let resting_pygmy = world.obj::<IglooLogic>(this).resting_pygmy_logic;
    let Some(resting_pygmy) = resting_pygmy else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, RESTING_PYGMY_CHANNEL);
    crate::pygmy::leave_igloo_door_crawl(world, resting_pygmy);
    world.obj_mut::<IglooLogic>(this).resting_pygmy_logic = None;
}

/// `-[IglooLogic checkForDoorKnocking:]` @ 0x1000c3c78
///
/// With a pygmy inside, a knock counts unless it comes in the same frame as the last one
/// (the knock time is still 0); the third one in a row makes the pygmy crawl out.
pub fn check_for_door_knocking(world: &mut World, this: Entity, _touch: Entity) {
    let (in_transition, resting_pygmy) = {
        let igloo = world.obj::<IglooLogic>(this);
        (igloo.in_transition, igloo.resting_pygmy_logic)
    };
    if in_transition {
        return;
    }

    if resting_pygmy.is_some() {
        let knock_touch_count = {
            let mut igloo = world.obj_mut::<IglooLogic>(this);
            if igloo.knock_touch_time == 0.0 {
                return;
            }
            igloo.knock_touch_time = 0.0;
            igloo.knock_touch_count = igloo.knock_touch_count.wrapping_add(1);
            igloo.knock_touch_count
        };
        if knock_touch_count == 3 {
            pg_engine::logic::set_behavior(world, this, Some("IglooPygmyCrawlOutNoEat"));
            return;
        }
    }
    pg_engine::logic::set_behavior(world, this, Some("IglooKnock"));
}

/// `-[IglooLogic touchBeganInside:allTouches:]` @ 0x1000c3d00
///
/// Tapping an igloo with a pygmy and a fish inside fires the pygmy out like a cannon; any
/// other tap is a knock.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let (resting_pygmy, fish) = {
        let igloo = world.obj::<IglooLogic>(this);
        (igloo.resting_pygmy_logic, igloo.fish_logic)
    };
    if resting_pygmy.is_some() && fish.is_some() {
        pg_engine::logic::set_behavior(world, this, Some("IglooPygmyCannon"));

        let fish = world.obj::<IglooLogic>(this).fish_logic;
        if let Some(fish) = fish {
            fish_logic::finish_eating_in_igloo(world, fish);
        }
        world.obj_mut::<IglooLogic>(this).fish_logic = None;
        return;
    }

    check_for_door_knocking(world, this, touch);
}

/// `-[IglooLogic touchBeganOutside:allTouches:]` @ 0x1000c3d8c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IglooLogic touchMovedEntered:allTouches:]` @ 0x1000c3d90
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IglooLogic touchMovedExited:allTouches:]` @ 0x1000c3d94
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IglooLogic touchMovedInside:allTouches:]` @ 0x1000c3d98
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[IglooLogic touchMovedOutside:allTouches:]` @ 0x1000c3da8
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IglooLogic touchEndedInside:allTouches:]` @ 0x1000c3dac
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IglooLogic touchEndedOutside:allTouches:]` @ 0x1000c3db0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
