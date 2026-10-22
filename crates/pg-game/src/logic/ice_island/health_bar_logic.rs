//! `HealthBarLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;

/// `HealthBarLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct HealthBarLogic {
    /// `mHealthBarLogicListener`: `@"<HealthBarLogicListener>"`
    pub health_bar_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mTargetXPos`: `f`
    pub target_x_pos: f32,

    /// `mTargetYPos`: `f`
    pub target_y_pos: f32,

    /// `mTargetHealthPercent`: `f`
    pub target_health_percent: f32,

    /// `mHealthPercent`: `f`
    pub health_percent: f32,
}

pub const HEALTH_BAR_LOGIC: Class = Class {
    name: "HealthBarLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The bar's y position while it is hidden above the screen.
const HIDDEN_Y_POS: f32 = -30.0;

/// The bar's y position while it is shown.
const SHOWN_Y_POS: f32 = 7.0;

/// Sequence-item handlers implemented by `HealthBarLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "showHealthBar" => seq_show_health_bar(world, this, call),
        "hideHealthBar" => seq_hide_health_bar(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[HealthBarLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&HEALTH_BAR_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[HealthBarLogic initWithDisplayObject:]` @ 0x1000d6d4c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(HealthBarLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<HealthBarLogic>(this).target_y_pos = HIDDEN_Y_POS;

    let own_display_object = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, own_display_object).pos.y = HIDDEN_Y_POS;
    pg_engine::display::remove_from_display_group(world, own_display_object);
}

/// `-[HealthBarLogic setHealthBarLogicListener:]` @ 0x1000d6de4
pub fn set_health_bar_logic_listener(world: &mut World, this: Entity, health_bar_logic_listener: Entity) {
    world.obj_mut::<HealthBarLogic>(this).health_bar_logic_listener = Some(health_bar_logic_listener);
}

/// `-[HealthBarLogic activate]` @ 0x1000d6e40
pub fn activate(world: &mut World, this: Entity) {
    {
        let mut health_bar = world.obj_mut::<HealthBarLogic>(this);
        health_bar.active = true;
        health_bar.health_percent = 0.0;
        health_bar.target_health_percent = 1.0;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let marker = pg_engine::library::display_marker(world, "MenuDisplayMarker");
    if let Some(marker) = marker {
        pg_engine::display::insert_before(world, marker, Some(display_object));
    }
}

/// `-[HealthBarLogic deactivate]` @ 0x1000d6ed4
pub fn deactivate(world: &mut World, this: Entity) {
    world.obj_mut::<HealthBarLogic>(this).active = false;
}

/// `-[HealthBarLogic setHealthPercent:]` @ 0x1000d6ee4
pub fn set_health_percent(world: &mut World, this: Entity, health_percent: f32) {
    let mut health_bar = world.obj_mut::<HealthBarLogic>(this);
    health_bar.target_health_percent = if health_percent < 0.0 { 0.0 } else { health_percent };
    if 1.0 < health_bar.target_health_percent {
        health_bar.target_health_percent = 1.0;
    }
}

/// `-[HealthBarLogic onGameFrame:]` @ 0x1000d6f18
///
/// Only while a boss is the listener: eases the bar towards its target height, slides it in
/// (with the boss's enter animation) or out as the boss wants it shown, and eases the shown
/// health, which drives the bar animation's frame.
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    if pg_engine::logic::logic(world, this).shutdown {
        return;
    }
    if world.obj::<HealthBarLogic>(this).health_bar_logic_listener.is_none() {
        return;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let target_y_pos = world.obj::<HealthBarLogic>(this).target_y_pos;
    {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.y = object.pos.y + (target_y_pos - object.pos.y) * 0.5;
    }

    // The original asks the listener a second time before sliding the bar in.
    let active = world.obj::<HealthBarLogic>(this).active;
    let visible = active && listener_wants_bar_visible(world, this);
    let target_y_pos = world.obj::<HealthBarLogic>(this).target_y_pos;
    if !visible && target_y_pos != HIDDEN_Y_POS {
        world.obj_mut::<HealthBarLogic>(this).target_y_pos = HIDDEN_Y_POS;
        pg_engine::logic::set_behavior(world, this, Some("HealthBarExit"));
    } else {
        let active = world.obj::<HealthBarLogic>(this).active;
        let visible = active && listener_wants_bar_visible(world, this);
        let target_y_pos = world.obj::<HealthBarLogic>(this).target_y_pos;
        if visible && target_y_pos != SHOWN_Y_POS {
            world.obj_mut::<HealthBarLogic>(this).target_y_pos = SHOWN_Y_POS;

            let listener = world.obj::<HealthBarLogic>(this).health_bar_logic_listener;
            let health_bar_type = crate::send::health_bar_type(world, listener);
            let behavior = match health_bar_type {
                2 => Some("HealthBarIceMonsterEnter"),
                1 => Some("HealthBarSpiderEnter"),
                0 => Some("HealthBarTRexEnter"),
                _ => None,
            };
            if let Some(behavior) = behavior {
                pg_engine::logic::set_behavior(world, this, Some(behavior));
            }
        }
    }

    let health_percent = {
        let mut health_bar = world.obj_mut::<HealthBarLogic>(this);
        health_bar.health_percent =
            health_bar.health_percent + (health_bar.target_health_percent - health_bar.health_percent) * 0.5;
        health_bar.health_percent
    };
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::set_frame_percent(world, animation, health_percent);
}

/// `[mHealthBarLogicListener shouldHealthBarBeVisible:self]`.
fn listener_wants_bar_visible(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<HealthBarLogic>(this).health_bar_logic_listener;
    crate::send::should_health_bar_be_visible(world, listener, this)
}

/// `-[HealthBarLogic showHealthBar:::]` @ 0x1000d7114
pub fn seq_show_health_bar(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<HealthBarLogic>(this).target_y_pos = SHOWN_Y_POS;
    ItemResult::Done
}

/// `-[HealthBarLogic hideHealthBar:::]` @ 0x1000d712c
///
/// Waits until the bar is off screen; an inactive bar then drops its boss and leaves the
/// display.
pub fn seq_hide_health_bar(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let y_pos = pg_engine::display::object(world, display_object).pos.y;
    if y_pos != HIDDEN_Y_POS {
        return ItemResult::Pending;
    }

    let active = world.obj::<HealthBarLogic>(this).active;
    if !active {
        world.obj_mut::<HealthBarLogic>(this).health_bar_logic_listener = None;
        pg_engine::display::remove_from_display_group(world, display_object);
    }
    ItemResult::Done
}

/// `-[HealthBarLogic setTargetXPos:yPos:]` @ 0x1000d7234
pub fn set_target_x_pos(world: &mut World, this: Entity, target_x_pos: f32, y_pos: f32) {
    let mut health_bar = world.obj_mut::<HealthBarLogic>(this);
    health_bar.target_x_pos = target_x_pos;
    health_bar.target_y_pos = y_pos;
}
