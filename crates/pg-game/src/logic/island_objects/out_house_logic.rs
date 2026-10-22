//! `OutHouseLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};

/// `OutHouseLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct OutHouseLogic {
    /// `mOutHouseLogicListener`: `@"<OutHouseLogicListener>"`
    pub out_house_logic_listener: Option<Entity>,

    /// `mbFinishingUp`: `B`
    pub finishing_up: bool,

    /// `mCrappingPygmyLogic`: `@"PygmyLogic"`
    pub crapping_pygmy_logic: Option<Entity>,

    /// `mWaitingPygmyLogic`: `@"PygmyLogic"`
    pub waiting_pygmy_logic: Option<Entity>,

    /// `mKnockTouchTime`: `f`
    pub knock_touch_time: f32,

    /// `mKnockTouchCount`: `i`
    pub knock_touch_count: i32,
}

pub const OUT_HOUSE_LOGIC: Class = Class {
    name: "OutHouseLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    set_behavior,
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

/// Sequence-item handlers implemented by `OutHouseLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "finishCrapping" => seq_finish_crapping(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[OutHouseLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&OUT_HOUSE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[OutHouseLogic initWithDisplayObject:]` @ 0x100022b94
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(OutHouseLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);
    world.obj_mut::<SpriteLogic>(this).apocalypse_cloud_style_id = Some(String::from("OutHouse"));
    pg_engine::logic::set_behavior(world, this, Some("OutHouseInit"));
}

/// `-[OutHouseLogic setBehavior:]` @ 0x100022c2c
///
/// Only logs (dropped) before `[super setBehavior:]`.
pub fn set_behavior(world: &mut World, this: Entity, behavior: Option<&str>) {
    pg_engine::logic::set_behavior_base(world, this, behavior);
}

/// `-[OutHouseLogic setOutHouseLogicListener:]` @ 0x100022ca8
pub fn set_out_house_logic_listener(world: &mut World, this: Entity, out_house_logic_listener: Entity) {
    world.obj_mut::<OutHouseLogic>(this).out_house_logic_listener = Some(out_house_logic_listener);
}

/// `-[OutHouseLogic isCrappingAvailable:]` @ 0x100022d04
pub fn is_crapping_available(world: &mut World, this: Entity, crapping_available: Entity) -> bool {
    let out_house = world.obj::<OutHouseLogic>(this);
    if out_house.crapping_pygmy_logic.is_some() {
        return false;
    }
    out_house.waiting_pygmy_logic.is_none() || out_house.waiting_pygmy_logic == Some(crapping_available)
}

/// `-[OutHouseLogic isWaitingAvailable:]` @ 0x100022d40
pub fn is_waiting_available(world: &mut World, this: Entity, _waiting_available: Entity) -> bool {
    let out_house = world.obj::<OutHouseLogic>(this);
    if out_house.crapping_pygmy_logic.is_some() {
        return out_house.waiting_pygmy_logic.is_none();
    }
    false
}

/// `-[OutHouseLogic takeCrap:]` @ 0x100022d70
pub fn take_crap(world: &mut World, this: Entity, take_crap: Entity) {
    world.obj_mut::<OutHouseLogic>(this).crapping_pygmy_logic = Some(take_crap);

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, take_crap);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), 1);

    pg_engine::logic::set_behavior(world, this, Some("OutHouseTakeCrap"));
}

/// `-[OutHouseLogic waitToCrap:]` @ 0x100022df0
pub fn wait_to_crap(world: &mut World, this: Entity, wait_to_crap: Entity) {
    world.obj_mut::<OutHouseLogic>(this).waiting_pygmy_logic = Some(wait_to_crap);
}

/// `-[OutHouseLogic stopWaitingToCrap:]` @ 0x100022e00
pub fn stop_waiting_to_crap(world: &mut World, this: Entity, _waiting_to_crap: Entity) {
    world.obj_mut::<OutHouseLogic>(this).waiting_pygmy_logic = None;
}

/// `-[OutHouseLogic stopCrapping:]` @ 0x100022e10
pub fn stop_crapping(world: &mut World, this: Entity, crapping: Entity) {
    if world.obj::<OutHouseLogic>(this).crapping_pygmy_logic != Some(crapping) {
        return;
    }

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);
    world.obj_mut::<OutHouseLogic>(this).crapping_pygmy_logic = None;
    pg_engine::logic::set_behavior(world, this, Some("OutHouseEnterIsland"));
}

/// `-[OutHouseLogic enterIsland:]` @ 0x100022e8c
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    pg_engine::logic::start_game_frame(world, this);
    pg_engine::logic::set_behavior(world, this, Some("OutHouseEnterIsland"));
}

/// `-[OutHouseLogic exitIsland:]` @ 0x100022ed0
pub fn exit_island(world: &mut World, this: Entity, exit_island: bool) {
    let waiting = world.obj::<OutHouseLogic>(this).waiting_pygmy_logic;
    if let Some(waiting) = waiting {
        crate::pygmy::toss_from_out_house(world, waiting);
        world.obj_mut::<OutHouseLogic>(this).waiting_pygmy_logic = None;
    }

    if world.obj::<OutHouseLogic>(this).crapping_pygmy_logic.is_some() {
        world.obj_mut::<OutHouseLogic>(this).finishing_up = false;

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

        let crapping = world.obj::<OutHouseLogic>(this).crapping_pygmy_logic;
        if let Some(crapping) = crapping {
            crate::pygmy::toss_from_out_house(world, crapping);
        }
        world.obj_mut::<OutHouseLogic>(this).crapping_pygmy_logic = None;
    }

    if exit_island {
        pg_engine::logic::stop_game_frame(world, this);
    }
    pg_engine::logic::set_behavior(world, this, Some("OutHouseExitIsland"));
}

/// `-[OutHouseLogic onGameFrame:]` @ 0x100023008
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let mut out_house = world.obj_mut::<OutHouseLogic>(this);
    out_house.knock_touch_time += dt;
    if 1.0 < out_house.knock_touch_time {
        out_house.knock_touch_count = 0;
    }
}

/// `-[OutHouseLogic finishCrapping:::]` @ 0x10002304c
pub fn seq_finish_crapping(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if world.obj::<OutHouseLogic>(this).crapping_pygmy_logic.is_some() {
        world.obj_mut::<OutHouseLogic>(this).finishing_up = false;

        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id(world, animation, 1);

        let crapping = world.obj::<OutHouseLogic>(this).crapping_pygmy_logic;
        if let Some(crapping) = crapping {
            crate::pygmy::finish_crapping(world, crapping);
        }
        world.obj_mut::<OutHouseLogic>(this).crapping_pygmy_logic = None;
    }
    ItemResult::Done
}

/// `-[OutHouseLogic checkForDoorKnocking:]` @ 0x1000230c4
pub fn check_for_door_knocking(world: &mut World, this: Entity, _check_for_door_knocking: Entity) {
    let (finishing_up, crapping, knock_touch_time) = {
        let out_house = world.obj::<OutHouseLogic>(this);
        (out_house.finishing_up, out_house.crapping_pygmy_logic, out_house.knock_touch_time)
    };
    if finishing_up {
        return;
    }

    if crapping.is_some() {
        if knock_touch_time == 0.0 {
            return;
        }

        let knock_touch_count = {
            let mut out_house = world.obj_mut::<OutHouseLogic>(this);
            out_house.knock_touch_time = 0.0;
            out_house.knock_touch_count += 1;
            out_house.knock_touch_count
        };
        if knock_touch_count == 3 {
            world.obj_mut::<OutHouseLogic>(this).finishing_up = true;
            pg_engine::logic::set_behavior(world, this, Some("OutHouseFinishCrapping"));
            return;
        }
    }
    pg_engine::logic::set_behavior(world, this, Some("OutHouseKnock"));
}

/// `-[OutHouseLogic touchBeganInside:allTouches:]` @ 0x100023154
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    check_for_door_knocking(world, this, touch);
}

/// `-[OutHouseLogic touchBeganOutside:allTouches:]` @ 0x100023164
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OutHouseLogic touchMovedEntered:allTouches:]` @ 0x100023168
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OutHouseLogic touchMovedExited:allTouches:]` @ 0x10002316c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OutHouseLogic touchMovedInside:allTouches:]` @ 0x100023170
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[OutHouseLogic touchMovedOutside:allTouches:]` @ 0x100023180
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OutHouseLogic touchEndedInside:allTouches:]` @ 0x100023184
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OutHouseLogic touchEndedOutside:allTouches:]` @ 0x100023188
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
