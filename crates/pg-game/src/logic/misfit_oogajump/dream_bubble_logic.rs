//! `DreamBubbleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `DreamBubbleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct DreamBubbleLogic {
    /// `mDreamBubbleLogicListener`: `@"<DreamBubbleLogicListener>"`
    pub dream_bubble_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbInTransition`: `B`
    pub in_transition: bool,

    /// `mPygmyLogic`: `@"PygmyLogic"`
    pub pygmy_logic: Option<Entity>,
}

pub const DREAM_BUBBLE_LOGIC: Class = Class {
    name: "DreamBubbleLogic",
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

/// The island location the bubble leads to: Misfit island.
const MISFIT_ISLAND_LOCATION_STATE: i32 = 6;

/// Sequence-item handlers implemented by `DreamBubbleLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "die" => seq_die(world, this, call),
        "transitionComplete" => seq_transition_complete(world, this, call),
        "deactivateComplete" => seq_deactivate_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `DreamBubbleLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animChangeLocations" => anim_change_locations(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[DreamBubbleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&DREAM_BUBBLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[DreamBubbleLogic initWithDisplayObject:]` @ 0x10019f1b8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(DreamBubbleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, display_object);
    display.is_selectable = true;
    display.col_x_min = 0.0;
    display.col_x_max = 60.0;
    display.col_y_min = 40.0;
    display.col_y_max = 90.0;
}

/// `-[DreamBubbleLogic setDreamBubbleLogicListener:]` @ 0x10019f298
pub fn set_dream_bubble_logic_listener(world: &mut World, this: Entity, dream_bubble_logic_listener: Entity) {
    world.obj_mut::<DreamBubbleLogic>(this).dream_bubble_logic_listener = Some(dream_bubble_logic_listener);
}

/// `-[DreamBubbleLogic isActive]` @ 0x10019f358
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<DreamBubbleLogic>(this).active
}

/// `[[pygmyLogic displayObject] pos]`.
fn pygmy_pos(world: &World, pygmy_logic: Entity) -> Vec4 {
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    display::object(world, pygmy_display_object).pos
}

/// `-[DreamBubbleLogic onGameFrame:]` @ 0x10019f368
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    let pygmy_logic = world.obj::<DreamBubbleLogic>(this).pygmy_logic;
    if shutdown {
        return;
    }

    // The bubble stays on its dreaming pygmy.
    let Some(pygmy_logic) = pygmy_logic else {
        return;
    };

    let pos = pygmy_pos(world, pygmy_logic);
    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos = pos;
}

/// `-[DreamBubbleLogic die:::]` @ 0x10019f3fc
pub fn seq_die(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[DreamBubbleLogic activate:]` @ 0x10019f404
pub fn activate(world: &mut World, this: Entity, pygmy_logic: Entity) {
    {
        let mut bubble = world.obj_mut::<DreamBubbleLogic>(this);
        bubble.active = true;
        bubble.in_transition = false;
        bubble.pygmy_logic = Some(pygmy_logic);
    }

    let display_object = logic::display_object(world, this);
    let pygmy_display_object = logic::display_object(world, pygmy_logic);
    display::insert_before(world, pygmy_display_object, Some(display_object));

    let pos = pygmy_pos(world, pygmy_logic);
    display::object_mut(world, display_object).pos = pos;

    logic::set_behavior(world, this, Some("DreamBubbleMisfitIslandActivate"));
}

/// `-[DreamBubbleLogic deactivate]` @ 0x10019f4e0
pub fn deactivate(world: &mut World, this: Entity) {
    let bubble = world.obj::<DreamBubbleLogic>(this);
    if !bubble.active || bubble.pygmy_logic.is_none() {
        return;
    }

    world.obj_mut::<DreamBubbleLogic>(this).pygmy_logic = None;
    logic::set_behavior(world, this, Some("DreamBubbleMisfitIslandDeactivate"));
}

/// `-[DreamBubbleLogic animChangeLocations]` @ 0x10019f520
pub fn anim_change_locations(world: &mut World, this: Entity) {
    let listener = world.obj::<DreamBubbleLogic>(this).dream_bubble_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_dream_bubble_change_island_location_state(world, listener, MISFIT_ISLAND_LOCATION_STATE);
    }
}

/// `-[DreamBubbleLogic transitionComplete:::]` @ 0x10019f540
pub fn seq_transition_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    {
        let mut bubble = world.obj_mut::<DreamBubbleLogic>(this);
        bubble.in_transition = false;
        bubble.active = false;
    }

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[DreamBubbleLogic deactivateComplete:::]` @ 0x10019f58c
pub fn seq_deactivate_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<DreamBubbleLogic>(this).active = false;

    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
    ItemResult::Done
}

/// `-[DreamBubbleLogic touchBeganInside:allTouches:]` @ 0x10019f5cc
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let bubble = world.obj::<DreamBubbleLogic>(this);
    let active = bubble.active;
    let in_transition = bubble.in_transition;
    let listener = bubble.dream_bubble_logic_listener;
    if !active || in_transition {
        return;
    }

    // `[nil islandLocationState]` is 0.
    let island_location_state = match listener {
        Some(listener) => crate::controller::island_location_state(world, listener),
        None => 0,
    };
    if island_location_state == MISFIT_ISLAND_LOCATION_STATE {
        return;
    }

    world.obj_mut::<DreamBubbleLogic>(this).in_transition = true;

    let pygmy_logic = world.obj::<DreamBubbleLogic>(this).pygmy_logic;
    if let Some(pygmy_logic) = pygmy_logic {
        crate::pygmy::dream_bubble_selected(world, pygmy_logic);
    }
    world.obj_mut::<DreamBubbleLogic>(this).pygmy_logic = None;

    let display_object = logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "MapDisplayMarker");
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }

    // One 8-byte store: x and y both become 0.
    {
        let mut display = display::object_mut(world, display_object);
        display.pos.x = 0.0;
        display.pos.y = 0.0;
    }

    logic::set_behavior(world, this, Some("DreamBubbleSelected"));
}

/// `-[DreamBubbleLogic touchBeganOutside:allTouches:]` @ 0x10019f6e0
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DreamBubbleLogic touchMovedEntered:allTouches:]` @ 0x10019f6e4
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DreamBubbleLogic touchMovedExited:allTouches:]` @ 0x10019f6e8
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DreamBubbleLogic touchMovedInside:allTouches:]` @ 0x10019f6ec
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[DreamBubbleLogic touchMovedOutside:allTouches:]` @ 0x10019f6fc
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DreamBubbleLogic touchEndedInside:allTouches:]` @ 0x10019f700
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[DreamBubbleLogic touchEndedOutside:allTouches:]` @ 0x10019f704
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
