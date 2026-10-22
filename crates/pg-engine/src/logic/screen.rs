//! `BCScreenLogic`: modal screens with layers (`docs/re/logic.md` §10).
//!
//! The screen's display object lives in the shared group `ScreenDisplayGroup`, attached on top
//! of the game root. Screens enable and disable their buttons by attaching button display
//! groups (button layers) under that group, since touches only reach the active tree.

use bevy_ecs::prelude::*;

use crate::display;
use crate::logic::multi_model::{self, MULTI_MODEL_LOGIC};
use crate::object::{Class, ObjectWorldExt, isa};
use crate::sequence::{ItemResult, SequenceCall};
use crate::view::{self, BcView};

/// Entries of `mScreenLayerArray`.
pub const SCREEN_LAYER_COUNT: usize = 10;

/// The shared group every screen is shown in (literal 0x100389b50).
const SCREEN_DISPLAY_GROUP_ID: &str = "ScreenDisplayGroup";

/// `ScreenLayer`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScreenLayer {
    pub logic: Option<Entity>,
    pub button_display_group: Option<Entity>,
}

/// `BCScreenLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct BcScreenLogic {
    pub screen_display_group: Option<Entity>,
    pub active_display_group: Option<Entity>,
    pub layers: [ScreenLayer; SCREEN_LAYER_COUNT],
}

pub const BC_SCREEN_LOGIC: Class = Class {
    name: "BCScreenLogic",
    superclass: Some(&MULTI_MODEL_LOGIC),
    sequence_item,
    close_screen_now,
    ..MULTI_MODEL_LOGIC
};

/// `-[BCScreenLogic initWithDisplayObject:]` @ 0x10001cbb0.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    multi_model::init_with_display_object(world, this, display_object);
    let screen_display_group = crate::library::display_group(world, SCREEN_DISPLAY_GROUP_ID);
    let screen = BcScreenLogic {
        screen_display_group,
        ..Default::default()
    };
    world.entity_mut(this).insert(screen);
}

fn screen(world: &World, this: Entity) -> &BcScreenLogic {
    world.obj::<BcScreenLogic>(this)
}

fn screen_mut(world: &mut World, this: Entity) -> Mut<'_, BcScreenLogic> {
    world.obj_mut::<BcScreenLogic>(this)
}

/// `mScreenLayerArray[layer]` for an `intValue` parameter. The original does not check the
/// index (values outside 0–9 reach neighbouring memory); the port panics instead.
fn layer_index(layer: i32) -> usize {
    match usize::try_from(layer) {
        Ok(index) if index < SCREEN_LAYER_COUNT => index,
        _ => panic!("screen layer {layer} is outside mScreenLayerArray[{SCREEN_LAYER_COUNT}]"),
    }
}

/// A group the original reaches through `[group pos]` without a nil check: a nil group is a
/// NULL pointer dereference (a crash) there, a panic here.
fn required_group(group: Option<Entity>, role: &str) -> Entity {
    match group {
        Some(group) => group,
        None => panic!("BCScreenLogic: the {role} is nil (the original dereferences [nil pos])"),
    }
}

/// `-[BCScreenLogic openScreen]` @ 0x10001cc90.
pub fn open_screen(world: &mut World, this: Entity) {
    let active_display_group = world.resource::<BcView>().active_display_group;
    let screen_display_group = crate::library::display_group(world, SCREEN_DISPLAY_GROUP_ID);
    {
        let mut state = screen_mut(world, this);
        state.active_display_group = active_display_group;
        state.screen_display_group = screen_display_group;
    }

    // Appended to the game root, so the screen is drawn (and hit-tested) last.
    if let Some(screen_group) = screen_display_group {
        display::move_to_display_group(world, screen_group, active_display_group);
    }
    let display_object = crate::logic::display_object(world, this);
    display::move_to_display_group(world, display_object, screen_display_group);

    let screen_group = required_group(screen_display_group, "screen display group");
    let mut object = display::object_mut(world, screen_group);
    object.pos.x = 0.0;
    object.pos.y = 0.0;
    object.rot.z = 0.0;
}

/// `-[BCScreenLogic closeScreenNow]` @ 0x10001cd88 (dispatched for subclasses through the
/// class table).
pub fn close_screen_now(world: &mut World, this: Entity) {
    let active_display_group = screen(world, this).active_display_group;
    view::set_active_display_group(world, active_display_group);

    let screen_display_group = screen(world, this).screen_display_group;
    if let Some(screen_group) = screen_display_group {
        display::remove_from_display_group(world, screen_group);
    }
    let display_object = crate::logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);

    for index in 0..SCREEN_LAYER_COUNT {
        let button_display_group = screen(world, this).layers[index].button_display_group;
        let Some(button_group) = button_display_group else {
            continue;
        };
        display::remove_from_display_group(world, button_group);
        screen_mut(world, this).layers[index].button_display_group = None;
    }
}

/// `-[BCScreenLogic setScreenLayer:logic:]` @ 0x10001ce48 (not retained).
pub fn set_screen_layer_logic(world: &mut World, this: Entity, layer: usize, logic: Option<Entity>) {
    screen_mut(world, this).layers[layer].logic = logic;
}

/// `-[BCScreenLogic setScreenLayer:behavior:]` @ 0x10001ce60: drops the layer's button group,
/// then sends `setBehavior:` to self (not to the layer's logic).
pub fn set_screen_layer_behavior(world: &mut World, this: Entity, layer: usize, behavior: Option<&str>) {
    let button_display_group = screen(world, this).layers[layer].button_display_group;
    if let Some(button_group) = button_display_group {
        display::remove_from_display_group(world, button_group);
    }
    screen_mut(world, this).layers[layer].button_display_group = None;
    crate::logic::set_behavior(world, this, behavior);
}

/// `BCScreenLogic` handlers.
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "disableGameLayer" => seq_disable_game_layer(world, this, call),
        "enableGameLayer" => seq_enable_game_layer(world, this, call),
        "setLayerAnimation" => seq_set_layer_animation(world, this, call),
        "enableButtonLayer" => seq_enable_button_layer(world, this, call),
        "disableButtonLayer" => seq_disable_button_layer(world, this, call),
        "closeScreenComplete" => seq_close_screen_complete(world, this, call),
        _ => return multi_model::sequence_item(world, this, call),
    };
    Some(result)
}

/// `-[BCScreenLogic disableGameLayer:::]` @ 0x10001cec4: the screen group becomes the root with
/// the game root's transform; the game world hibernates and is no longer drawn.
pub fn seq_disable_game_layer(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let screen_display_group = screen(world, this).screen_display_group;
    if let Some(screen_group) = screen_display_group {
        display::remove_from_display_group(world, screen_group);
    }

    let screen_group = required_group(screen_display_group, "screen display group");
    let active_display_group = screen(world, this).active_display_group;
    let active_group = required_group(active_display_group, "active display group");
    let (active_pos, active_rot) = {
        let active = display::object(world, active_group);
        (active.pos, active.rot)
    };
    {
        let mut object = display::object_mut(world, screen_group);
        object.pos = active_pos;
        object.rot = active_rot;
    }

    view::set_active_display_group(world, screen_display_group);
    ItemResult::Done
}

/// `-[BCScreenLogic enableGameLayer:::]` @ 0x10001cfa4: the screen group goes back on top of
/// the game root, which becomes the active group again.
pub fn seq_enable_game_layer(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let (screen_display_group, active_display_group) = {
        let state = screen(world, this);
        (state.screen_display_group, state.active_display_group)
    };
    if let Some(screen_group) = screen_display_group {
        display::move_to_display_group(world, screen_group, active_display_group);
    }

    let screen_group = required_group(screen_display_group, "screen display group");
    {
        let mut object = display::object_mut(world, screen_group);
        object.pos.x = 0.0;
        object.pos.y = 0.0;
        object.rot.z = 0.0;
    }

    view::set_active_display_group(world, active_display_group);
    ItemResult::Done
}

/// `-[BCScreenLogic setLayerAnimation:::]` @ 0x10001d054: plays `[1] animation` on the logic of
/// layer `[0] layer` (nothing when the layer has no logic: messages to nil).
pub fn seq_set_layer_animation(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let layer = layer_index(call.int("layer"));
    let animation_id = call.param("animation");
    let layer_logic = screen(world, this).layers[layer].logic;
    if let Some(layer_logic) = layer_logic {
        let player = crate::logic::animation(world, layer_logic);
        crate::animation::play_animation_id(world, player, animation_id);
    }
    ItemResult::Done
}

/// `-[BCScreenLogic enableButtonLayer:::]` @ 0x10001d0d0: attaches display group
/// `[1] displayGroup` as the buttons of layer `[0] layer`, at the layer logic's x/y.
pub fn seq_enable_button_layer(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let layer = layer_index(call.int("layer"));
    let group_id = call.param("displayGroup");

    // The previous group is detached here and replaced below.
    let previous_group = screen(world, this).layers[layer].button_display_group;
    if let Some(previous_group) = previous_group {
        display::remove_from_display_group(world, previous_group);
    }

    let layer_logic = screen(world, this).layers[layer].logic;
    let Some(layer_logic) = layer_logic else {
        panic!("enableButtonLayer: layer {layer} has no logic (the original dereferences [nil pos])");
    };
    let layer_display_object = crate::logic::display_object(world, layer_logic);
    let button_display_group = match group_id {
        Some(group_id) => crate::library::display_group(world, group_id),
        None => None,
    };
    let button_group = required_group(button_display_group, "button display group");

    let layer_pos = display::object(world, layer_display_object).pos;
    {
        let mut object = display::object_mut(world, button_group);
        object.pos.x = layer_pos.x;
        object.pos.y = layer_pos.y;
    }

    screen_mut(world, this).layers[layer].button_display_group = Some(button_group);
    let screen_display_group = screen(world, this).screen_display_group;
    display::move_to_display_group(world, button_group, screen_display_group);
    ItemResult::Done
}

/// `-[BCScreenLogic disableButtonLayer:::]` @ 0x10001d208: detaches the buttons of `[0] layer`.
pub fn seq_disable_button_layer(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let layer = layer_index(call.int("layer"));
    let button_display_group = screen(world, this).layers[layer].button_display_group;
    if let Some(button_group) = button_display_group {
        display::remove_from_display_group(world, button_group);
        screen_mut(world, this).layers[layer].button_display_group = None;
    }
    ItemResult::Done
}

/// `-[BCScreenLogic closeScreenComplete:::]` @ 0x10001d278: `[self closeScreenNow]` (dispatched).
pub fn seq_close_screen_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let class = isa(world, this);
    (class.close_screen_now)(world, this);
    ItemResult::Done
}
