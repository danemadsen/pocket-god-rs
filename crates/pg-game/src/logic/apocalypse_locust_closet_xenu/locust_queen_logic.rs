//! `LocustQueenLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;

use crate::logic::apocalypse_locust_closet_xenu::locust_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic_table;

/// `LocustQueenLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LocustQueenLogic {
    /// `mLocustQueenLogicListener`: `@"<LocustQueenLogicListener>"`
    pub locust_queen_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mLocustQueenBellyLogic`: `@"MultiModelLogic"`
    pub locust_queen_belly_logic: Option<Entity>,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,

    /// `mLocustEmbryoCount`: `i`
    pub locust_embryo_count: i32,

    /// `mbTouchBeganInside`: `B`
    pub touch_began_inside: bool,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,
}

pub const LOCUST_QUEEN_LOGIC: Class = Class {
    name: "LocustQueenLogic",
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

/// Sequence-item handlers implemented by `LocustQueenLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "eatPygmyComplete" => seq_eat_pygmy_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `LocustQueenLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animBirthLocust" => anim_birth_locust(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[LocustQueenLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&LOCUST_QUEEN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mLocustQueenLogicListener locustLogic:self]`.
fn listener_locust_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<LocustQueenLogic>(this).locust_queen_logic_listener;
    match listener {
        Some(listener) => crate::controller::locust_logic(world, listener, this),
        None => None,
    }
}

/// `[[mLocustQueenLogicListener locustLogic:self] canCreateNewLocusts]` (NO for nil).
fn can_create_new_locusts(world: &mut World, this: Entity) -> bool {
    let locust = listener_locust_logic(world, this);
    match locust {
        Some(locust) => locust_logic::can_create_new_locusts(world, locust),
        None => false,
    }
}

/// `[mLocustQueenBellyLogic setBehavior:behavior]`.
fn set_belly_behavior(world: &mut World, this: Entity, behavior: &str) {
    let belly = world.obj::<LocustQueenLogic>(this).locust_queen_belly_logic;
    if let Some(belly) = belly {
        logic::set_behavior(world, belly, Some(behavior));
    }
}

/// `-[LocustQueenLogic initWithDisplayObject:]` @ 0x1001998ac
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(LocustQueenLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().locust_queen_logic = Some(this);

    let belly = library::logic(world, logic_table::LOCUST_QUEEN_BELLY);
    world.obj_mut::<LocustQueenLogic>(this).locust_queen_belly_logic = Some(belly);
    logic::stop_game_frame(world, this);
}

/// `-[LocustQueenLogic setLocustQueenLogicListener:]` @ 0x100199950
pub fn set_locust_queen_logic_listener(world: &mut World, this: Entity, locust_queen_logic_listener: Entity) {
    world.obj_mut::<LocustQueenLogic>(this).locust_queen_logic_listener = Some(locust_queen_logic_listener);
}

/// `-[LocustQueenLogic isAvailable]` @ 0x1001999ac
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let queen = world.obj::<LocustQueenLogic>(this);
    if queen.pygmy_logic.is_some() {
        return false;
    }
    !queen.touch_began_inside
}

/// `-[LocustQueenLogic fallIntoLocustQueenArms:]` @ 0x1001999dc
pub fn fall_into_locust_queen_arms(world: &mut World, this: Entity, fall_into_locust_queen_arms: Entity) {
    world.obj_mut::<LocustQueenLogic>(this).pygmy_logic = Some(fall_into_locust_queen_arms);

    let animation = logic::animation(world, this);
    let pygmy_display_object = logic::display_object(world, fall_into_locust_queen_arms);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), 1);

    logic::set_behavior(world, this, Some("LocustQueenPygmyFall"));
}

/// `-[LocustQueenLogic rescuedFromLocustQueen:]` @ 0x100199a5c
pub fn rescued_from_locust_queen(world: &mut World, this: Entity, _rescued_from_locust_queen: Entity) {
    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

    logic::set_behavior(world, this, Some("LocustQueenEjectPygmy"));
    world.obj_mut::<LocustQueenLogic>(this).pygmy_logic = None;
}

/// `-[LocustQueenLogic enterIsland:]` @ 0x100199ac4
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    if world.obj::<LocustQueenLogic>(this).active {
        return;
    }
    world.obj_mut::<LocustQueenLogic>(this).active = true;

    let listener = world.obj::<LocustQueenLogic>(this).locust_queen_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_locust_queen_visible(world, listener, this);
    }
    logic::set_behavior(world, this, Some("LocustQueenEnterIsland"));

    let locust_embryo_count = world.obj::<LocustQueenLogic>(this).locust_embryo_count;
    let belly_behavior = if locust_embryo_count < 1 {
        "LocustQueenBellyEnterIsland"
    } else {
        "LocustQueenBellyPregnant"
    };
    set_belly_behavior(world, this, belly_behavior);
}

/// `-[LocustQueenLogic exitIsland:]` @ 0x100199b7c
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    if !world.obj::<LocustQueenLogic>(this).active {
        return;
    }
    world.obj_mut::<LocustQueenLogic>(this).active = false;

    let pygmy = world.obj::<LocustQueenLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        let animation = logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

        crate::pygmy::rescued_from_locust_queen(world, pygmy);
        world.obj_mut::<LocustQueenLogic>(this).pygmy_logic = None;
    }
    logic::set_behavior(world, this, Some("LocustQueenExitIsland"));
}

/// `-[LocustQueenLogic exitIslandNow]` @ 0x100199c1c
pub fn exit_island_now(world: &mut World, this: Entity) {
    exit_island(world, this, true);
}

/// `-[LocustQueenLogic onGameFrame:]` @ 0x100199c94
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[LocustQueenLogic animBirthLocust]` @ 0x100199c98
pub fn anim_birth_locust(world: &mut World, this: Entity) {
    {
        let mut queen = world.obj_mut::<LocustQueenLogic>(this);
        queen.locust_embryo_count = queen.locust_embryo_count - 1;
    }

    let display_object = logic::display_object(world, this);
    let mut birth_pos = display::object(world, display_object).pos;
    birth_pos.x = birth_pos.x + -108.0;
    birth_pos.y = birth_pos.y + 140.0;

    let locust = listener_locust_logic(world, this);
    if let Some(locust) = locust {
        locust_logic::add_locust(world, locust, birth_pos);
    }

    let locust_embryo_count = world.obj::<LocustQueenLogic>(this).locust_embryo_count;
    if locust_embryo_count != 0 && can_create_new_locusts(world, this) {
        return;
    }

    set_belly_behavior(world, this, "LocustQueenBellyEmpty");
    logic::set_behavior(world, this, Some("LocustQueenBirthLocustsComplete"));
}

/// `-[LocustQueenLogic eatPygmyComplete:::]` @ 0x100199dc8
pub fn seq_eat_pygmy_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    if world.obj::<LocustQueenLogic>(this).pygmy_logic.is_none() {
        return ItemResult::Done;
    }

    if can_create_new_locusts(world, this) {
        {
            let mut queen = world.obj_mut::<LocustQueenLogic>(this);
            queen.locust_embryo_count = queen.locust_embryo_count + 20;
        }
        set_belly_behavior(world, this, "LocustQueenBellyPregnant");
    }

    let animation = logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 1);

    let pygmy = world.obj::<LocustQueenLogic>(this).pygmy_logic;
    if let Some(pygmy) = pygmy {
        crate::pygmy::eaten_by_locust_queen_complete(world, pygmy, this);
    }
    world.obj_mut::<LocustQueenLogic>(this).pygmy_logic = None;
    ItemResult::Done
}

/// `-[LocustQueenLogic touchBeganInside:allTouches:]` @ 0x100199ea4
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    {
        let queen = world.obj::<LocustQueenLogic>(this);
        if queen.touch_began_inside || queen.pygmy_logic.is_some() {
            return;
        }
    }

    let locust_embryo_count = {
        let mut queen = world.obj_mut::<LocustQueenLogic>(this);
        queen.active_touch = Some(touch);
        queen.touch_began_inside = true;
        queen.locust_embryo_count
    };

    let behavior = if locust_embryo_count > 0 && can_create_new_locusts(world, this) {
        "LocustQueenStartBirthLocusts"
    } else {
        "LocustQueenAttemptBirthLocusts"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[LocustQueenLogic touchBeganOutside:allTouches:]` @ 0x100199f78
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[LocustQueenLogic touchMovedEntered:allTouches:]` @ 0x100199f7c
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[LocustQueenLogic touchMovedExited:allTouches:]` @ 0x100199f80
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[LocustQueenLogic touchMovedInside:allTouches:]` @ 0x100199f84
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[LocustQueenLogic touchMovedOutside:allTouches:]` @ 0x100199f94
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[LocustQueenLogic touchEndedInside:allTouches:]` @ 0x100199f98
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    {
        let queen = world.obj::<LocustQueenLogic>(this);
        if !queen.touch_began_inside || queen.active_touch != Some(touch) {
            return;
        }
    }

    {
        let mut queen = world.obj_mut::<LocustQueenLogic>(this);
        queen.active_touch = None;
        queen.touch_began_inside = false;
    }
    logic::set_behavior(world, this, Some("LocustQueenStopBirthLocusts"));
}

/// `-[LocustQueenLogic touchEndedOutside:allTouches:]` @ 0x100199fe0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// `+[LocustQueenLogic instance]` @ 0x1001998a0
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().locust_queen_logic
}
