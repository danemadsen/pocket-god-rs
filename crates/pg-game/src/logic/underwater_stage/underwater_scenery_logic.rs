//! `UnderwaterSceneryLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::{display, library, logic};

/// `UnderwaterSceneryLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct UnderwaterSceneryLogic {
    /// `mUnderwaterSceneryType`: `i`
    pub underwater_scenery_type: i32,

    /// `mAlpha`: `f`
    pub alpha: f32,

    /// `mTargetAlpha`: `f`
    pub target_alpha: f32,

    /// `mbTransitionOut`: `B`
    pub transition_out: bool,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,
}

pub const UNDERWATER_SCENERY_LOGIC: Class = Class {
    name: "UnderwaterSceneryLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `mUnderwaterSceneryType` of the school of fish; the two blobs are 1 and 2.
const SCHOOL_OF_FISH: i32 = 0;

/// The width of the screen the scenery drifts across.
const SCREEN_WIDTH: f32 = 480.0;

/// Sequence-item handlers implemented by `UnderwaterSceneryLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "drift" => seq_drift(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `-[UnderwaterSceneryLogic initWithDisplayObject:underwaterSceneryType:]` @ 0x10017c228
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity, underwater_scenery_type: i32) {
    world.entity_mut(this).insert(UnderwaterSceneryLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<UnderwaterSceneryLogic>(this).underwater_scenery_type = underwater_scenery_type;
    refresh_params(world, this);
    world.obj_mut::<UnderwaterSceneryLogic>(this).transition_out = false;

    let own_display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, own_display_object);
}

/// `-[UnderwaterSceneryLogic onGameFrame:]` @ 0x10017c304
///
/// Eases the alpha toward its target, then moves like any sprite.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let alpha = {
        let mut scenery = world.obj_mut::<UnderwaterSceneryLogic>(this);
        scenery.alpha = scenery.alpha + (scenery.target_alpha - scenery.alpha) * 0.5 * dt;
        scenery.alpha
    };
    let own_display_object = logic::display_object(world, this);
    display::object_mut(world, own_display_object).set_alpha(alpha);

    sprite_logic::on_game_frame(world, this, dt);
}

/// `-[UnderwaterSceneryLogic refreshParams]` @ 0x10017c3c0
///
/// A new leftward drift speed: 30–60 for the school of fish, 20–40 for a blob.
pub fn refresh_params(world: &mut World, this: Entity) {
    let underwater_scenery_type = world.obj::<UnderwaterSceneryLogic>(this).underwater_scenery_type;
    let random = world.resource_mut::<Random>().arc4random();

    let tenths = if underwater_scenery_type == SCHOOL_OF_FISH {
        (random % 301) as i32 - 600
    } else {
        (random % 201) as i32 - 400
    };
    logic::logic_mut(world, this).pos_vel.x = tenths as f32 / 10.0;
}

/// `-[UnderwaterSceneryLogic transitionOut]` @ 0x10017c44c
pub fn transition_out(world: &mut World, this: Entity) {
    let underwater_scenery_type = {
        let mut scenery = world.obj_mut::<UnderwaterSceneryLogic>(this);
        scenery.transition_out = true;
        scenery.target_alpha = 0.0;
        scenery.underwater_scenery_type
    };

    let own_display_object = logic::display_object(world, this);
    display::object_mut(world, own_display_object).set_alpha(0.0);

    let behavior = if underwater_scenery_type == SCHOOL_OF_FISH {
        "SchoolOfFishExit"
    } else {
        "BlobExit"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[UnderwaterSceneryLogic transitionIn]` @ 0x10017c4dc
///
/// Fades in from nothing at a random x where the whole collision box is on screen.
pub fn transition_in(world: &mut World, this: Entity) {
    {
        let mut scenery = world.obj_mut::<UnderwaterSceneryLogic>(this);
        scenery.transition_in = true;
        scenery.target_alpha = 1.0;
        scenery.alpha = 0.0;
    }

    let own_display_object = logic::display_object(world, this);
    display::object_mut(world, own_display_object).set_alpha(0.0);
    refresh_params(world, this);

    let (x_scale, col_x_min, col_x_max) = {
        let object = display::object(world, own_display_object);
        (object.scale.x, object.col_x_min, object.col_x_max)
    };

    // In tenths of a pixel: from the x that puts the box's right edge at 0 to the x that puts
    // its left edge at 480, inclusive.
    let min_tenths = trunc((0.0 - x_scale * col_x_max) * 10.0);
    let random = world.resource_mut::<Random>().arc4random();
    let max_tenths = trunc((SCREEN_WIDTH - x_scale * col_x_min) * 10.0).wrapping_add(1);
    let range = max_tenths.wrapping_sub(min_tenths) as u32;
    let offset = if range == 0 { random } else { random % range };
    let x_tenths = offset.wrapping_add(min_tenths as u32) as i32;
    display::object_mut(world, own_display_object).pos.x = x_tenths as f32 / 10.0;

    let marker = library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(own_display_object));
    }

    let underwater_scenery_type = world.obj::<UnderwaterSceneryLogic>(this).underwater_scenery_type;
    let behavior = if underwater_scenery_type == SCHOOL_OF_FISH {
        "SchoolOfFishEnter"
    } else {
        "BlobEnter"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[UnderwaterSceneryLogic drift:::]` @ 0x10017c6ec
///
/// Wraps around the screen: past the right edge it comes back from the left, past the left
/// edge from the right, each time with a new speed. Never completes.
pub fn seq_drift(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let own_display_object = logic::display_object(world, this);
    let min_pos = display::min_pos(world, own_display_object);
    let max_pos = display::max_pos(world, own_display_object);

    if SCREEN_WIDTH < min_pos.x {
        refresh_params(world, this);
        let mut object = display::object_mut(world, own_display_object);
        object.pos.x = 0.0 - object.scale.x * object.col_x_max;
    }

    if max_pos.x < 0.0 {
        refresh_params(world, this);
        let mut object = display::object_mut(world, own_display_object);
        object.pos.x = SCREEN_WIDTH - object.scale.x * object.col_x_min;
    }
    ItemResult::Pending
}

/// `fcvtzs`: the C float → int conversion.
fn trunc(value: f32) -> i32 {
    pg_engine::math::trunc_i32(value)
}
