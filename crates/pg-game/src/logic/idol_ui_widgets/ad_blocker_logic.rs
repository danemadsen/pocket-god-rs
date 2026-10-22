//! `AdBlockerLogic` (superclass `SpriteLogic`).

use pg_engine::display;
use pg_engine::logic;

use crate::logic::island_core::sprite_logic;
use crate::managers::map_screen;
use crate::prelude::*;

/// `AdBlockerLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct AdBlockerLogic {
    /// `mAdBlockerLogicListener`: `@"<AdBlockerLogicListener>"`
    pub ad_blocker_logic_listener: Option<Entity>,
}

pub const AD_BLOCKER_LOGIC: Class = Class {
    name: "AdBlockerLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"AdBlockerEnter"`
const AD_BLOCKER_ENTER: &str = "AdBlockerEnter";

/// Sequence-item handlers implemented by `AdBlockerLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "waitForChartboostAdHide" => seq_wait_for_chartboost_ad_hide(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[AdBlockerLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&AD_BLOCKER_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[AdBlockerLogic initWithDisplayObject:]` @ 0x1001a9564
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(AdBlockerLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[AdBlockerLogic setAdBlockerLogicListener:]` @ 0x1001a95a0
pub fn set_ad_blocker_logic_listener(world: &mut World, this: Entity, ad_blocker_logic_listener: Entity) {
    world.obj_mut::<AdBlockerLogic>(this).ad_blocker_logic_listener = Some(ad_blocker_logic_listener);
}

/// `-[AdBlockerLogic activate]` @ 0x1001a9660
pub fn activate(world: &mut World, this: Entity) {
    logic::set_behavior(world, this, Some(AD_BLOCKER_ENTER));

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.is_selectable = true;
    object.hidden = false;
}

/// `-[AdBlockerLogic deactivate]` @ 0x1001a96c8
pub fn deactivate(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.is_selectable = false;
    object.hidden = true;
}

/// `-[AdBlockerLogic waitForChartboostAdHide:::]` @ 0x1001a9718
///
/// Waits until the listener (always the `MapScreen`, wired by `initLogicListeners`) no longer
/// shows an advert.
pub fn seq_wait_for_chartboost_ad_hide(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<AdBlockerLogic>(this).ad_blocker_logic_listener;
    let ad_visible = match listener {
        Some(listener) => map_screen::is_chartboost_ad_visible(world, listener),
        None => false,
    };
    if ad_visible {
        return ItemResult::Pending;
    }

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).is_selectable = false;
    ItemResult::Done
}
