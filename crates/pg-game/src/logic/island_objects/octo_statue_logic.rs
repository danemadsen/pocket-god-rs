//! `OctoStatueLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;

/// `OctoStatueLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct OctoStatueLogic {
    /// `mOctoStatueLogicListener`: `@"<OctoStatueLogicListener>"`
    pub octo_statue_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mTapTouchCount`: `i`
    pub tap_touch_count: i32,

    /// `mTapTouchTime`: `f`
    pub tap_touch_time: f32,
}

pub const OCTO_STATUE_LOGIC: Class = Class {
    name: "OctoStatueLogic",
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

/// Sequence-item handlers implemented by `OctoStatueLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "onActivateComplete" => seq_on_activate_complete(world, this, call),
        "blowUpPygmies" => seq_blow_up_pygmies(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[OctoStatueLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&OCTO_STATUE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[OctoStatueLogic initWithDisplayObject:]` @ 0x100118a9c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(OctoStatueLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);
    pg_engine::logic::set_behavior(world, this, Some("OctoStatueInit"));
}

/// `-[OctoStatueLogic setOctoStatueLogicListener:]` @ 0x100118b1c
pub fn set_octo_statue_logic_listener(world: &mut World, this: Entity, octo_statue_logic_listener: Entity) {
    world.obj_mut::<OctoStatueLogic>(this).octo_statue_logic_listener = Some(octo_statue_logic_listener);
}

/// `-[OctoStatueLogic onGameFrame:]` @ 0x100118b78
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }

    let mut statue = world.obj_mut::<OctoStatueLogic>(this);
    statue.tap_touch_time += dt;
    if 3.0 < statue.tap_touch_time {
        statue.tap_touch_count = 0;
    }
}

/// `-[OctoStatueLogic enterIsland:]` @ 0x100118bbc
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    pg_engine::logic::start_game_frame(world, this);
}

/// `-[OctoStatueLogic exitIsland]` @ 0x100118bcc
pub fn exit_island(world: &mut World, this: Entity) {
    pg_engine::logic::stop_game_frame(world, this);
}

/// `-[OctoStatueLogic onActivateComplete:::]` @ 0x100118c40
pub fn seq_on_activate_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<OctoStatueLogic>(this).active = false;
    ItemResult::Done
}

/// `-[OctoStatueLogic blowUpPygmies:::]` @ 0x100118c54
pub fn seq_blow_up_pygmies(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<OctoStatueLogic>(this).octo_statue_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        crate::pygmy::explode_from_loving_pocket_god(world, pygmy);
    }
    ItemResult::Done
}

/// `-[OctoStatueLogic checkForStatueTapping:]` @ 0x100118d7c
pub fn check_for_statue_tapping(world: &mut World, this: Entity, _check_for_statue_tapping: Entity) {
    let tap_touch_count = {
        let mut statue = world.obj_mut::<OctoStatueLogic>(this);
        if statue.active || statue.tap_touch_time == 0.0 {
            return;
        }
        statue.tap_touch_time = 0.0;
        statue.tap_touch_count += 1;
        statue.tap_touch_count
    };

    if tap_touch_count == 3 {
        world.obj_mut::<OctoStatueLogic>(this).active = true;
        pg_engine::logic::set_behavior(world, this, Some("OctoStatueActivate"));
        return;
    }
    pg_engine::logic::set_behavior(world, this, Some("OctoStatueTouch"));
}

/// `-[OctoStatueLogic touchBeganInside:allTouches:]` @ 0x100118dfc
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<OctoStatueLogic>(this).octo_statue_logic_listener;
    if let Some(listener) = listener {
        if crate::controller::is_story_active(world, listener) {
            return;
        }
    }
    check_for_statue_tapping(world, this, touch);
}

/// `-[OctoStatueLogic touchBeganOutside:allTouches:]` @ 0x100118e5c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OctoStatueLogic touchMovedEntered:allTouches:]` @ 0x100118e60
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OctoStatueLogic touchMovedExited:allTouches:]` @ 0x100118e64
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OctoStatueLogic touchMovedInside:allTouches:]` @ 0x100118e68
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[OctoStatueLogic touchMovedOutside:allTouches:]` @ 0x100118e78
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OctoStatueLogic touchEndedInside:allTouches:]` @ 0x100118e7c
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[OctoStatueLogic touchEndedOutside:allTouches:]` @ 0x100118e80
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
