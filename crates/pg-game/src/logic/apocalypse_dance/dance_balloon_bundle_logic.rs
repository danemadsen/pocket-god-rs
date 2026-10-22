//! `DanceBalloonBundleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::apocalypse_dance::dance_balloon_logic;
use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;
use pg_engine::touch::BcTouch;

/// `DanceBalloonBundleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DanceBalloonBundleLogic {
    /// `mDanceBalloonBundleLogicListener`: `@"<DanceBalloonBundleLogicListener>"`
    pub dance_balloon_bundle_logic_listener: Option<Entity>,
}

pub const DANCE_BALLOON_BUNDLE_LOGIC: Class = Class {
    name: "DanceBalloonBundleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
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

/// `[[DanceBalloonBundleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DANCE_BALLOON_BUNDLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DanceBalloonBundleLogic initWithDisplayObject:]` @ 0x1000c9e60
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DanceBalloonBundleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::stop_game_frame(world, this);
}

/// `-[DanceBalloonBundleLogic setDanceBalloonBundleLogicListener:]` @ 0x1000c9ec4
pub fn set_dance_balloon_bundle_logic_listener(world: &mut World, this: Entity, dance_balloon_bundle_logic_listener: Entity) {
    let listener = Some(dance_balloon_bundle_logic_listener);
    world.obj_mut::<DanceBalloonBundleLogic>(this).dance_balloon_bundle_logic_listener = listener;
}

/// `-[DanceBalloonBundleLogic enterIsland:]` @ 0x1000c9f20
pub fn enter_island(world: &mut World, this: Entity, _enter_island: bool) {
    logic::set_behavior(world, this, Some("DanceBalloonBundleEnterIsland"));
}

/// `-[DanceBalloonBundleLogic exitIsland:]` @ 0x1000c9f38
pub fn exit_island(world: &mut World, this: Entity, _exit_island: bool) {
    logic::set_behavior(world, this, Some("DanceBalloonBundleExitIsland"));
}

/// `-[DanceBalloonBundleLogic touchBeganInside:allTouches:]` @ 0x1000c9fb4
///
/// Outside a story, a tap pulls the balloon out of the bundle under the finger (unless it is
/// already out) and hands it the touch, so it is dragged at once.
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let listener = world.obj::<DanceBalloonBundleLogic>(this).dance_balloon_bundle_logic_listener;
    let story_active = match listener {
        Some(listener) => crate::controller::is_story_active(world, listener),
        None => false,
    };
    if story_active {
        return;
    }

    let dance_balloon = match listener {
        Some(listener) => crate::controller::dance_balloon_logic(world, listener, this),
        None => None,
    };
    let dance_balloon_active = match dance_balloon {
        Some(dance_balloon) => dance_balloon_logic::is_active(world, dance_balloon),
        None => false,
    };
    if dance_balloon_active {
        return;
    }
    let Some(dance_balloon) = dance_balloon else {
        // PORT: the original writes the position of a nil balloon (a NULL pointer) and crashes;
        // the Dance temple always has its balloon.
        return;
    };

    let balloon_display_object = logic::display_object(world, dance_balloon);
    let display_object = logic::display_object(world, this);
    let display_group = display::object(world, display_object).display_group;
    let touch_screen_pos = world.obj::<BcTouch>(touch).pos;
    let touch_pos = match display_group {
        Some(display_group) => display::view_to_display_object(world, display_group, touch_screen_pos),
        None => Vec4::ZERO,
    };
    {
        let mut balloon_display = display::object_mut(world, balloon_display_object);
        balloon_display.pos.x = touch_pos.x;
        balloon_display.pos.y = touch_pos.y;
        balloon_display.pos.z = SPRITE_Z;
    }

    dance_balloon_logic::enter_island(world, dance_balloon);
    let balloon_class = isa(world, dance_balloon);
    (balloon_class.touch_began_inside)(world, dance_balloon, touch);
}

/// `-[DanceBalloonBundleLogic touchBeganOutside:allTouches:]` @ 0x1000ca118
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DanceBalloonBundleLogic touchMovedEntered:allTouches:]` @ 0x1000ca11c
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DanceBalloonBundleLogic touchMovedExited:allTouches:]` @ 0x1000ca120
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DanceBalloonBundleLogic touchMovedInside:allTouches:]` @ 0x1000ca124
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[DanceBalloonBundleLogic touchMovedOutside:allTouches:]` @ 0x1000ca134
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DanceBalloonBundleLogic touchEndedInside:allTouches:]` @ 0x1000ca138
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DanceBalloonBundleLogic touchEndedOutside:allTouches:]` @ 0x1000ca13c
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
