//! `StormCloudLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::display::multi_model;
use pg_engine::display::tile_model;
use pg_engine::logic;

/// `StormCloudLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct StormCloudLogic {
    /// `mbFrontStormCloud`: `B`
    pub front_storm_cloud: bool,

    /// `mPosVelScale`: `f`
    pub pos_vel_scale: f32,

    /// `mbDragging`: `f`
    pub dragging: f32,

    /// `mbTransitionOut`: `B`
    pub transition_out: bool,

    /// `mbTransitionIn`: `B`
    pub transition_in: bool,
}

pub const STORM_CLOUD_LOGIC: Class = Class {
    name: "StormCloudLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `StormCloudLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "startTransitionIn" => seq_start_transition_in(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[StormCloudLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&STORM_CLOUD_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[StormCloudLogic initWithDisplayObject:]` @ 0x1001a83e0
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(StormCloudLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    tile_model::reset_transition_invisible(world, display_object);
    display::remove_from_display_group(world, display_object);
}

/// `-[StormCloudLogic setFrontStormCloud:posVelScale:]` @ 0x1001a8464
pub fn set_front_storm_cloud(world: &mut World, this: Entity, front_storm_cloud: bool, pos_vel_scale: f32) {
    {
        let mut storm_cloud = world.obj_mut::<StormCloudLogic>(this);
        storm_cloud.front_storm_cloud = front_storm_cloud;
        storm_cloud.pos_vel_scale = pos_vel_scale;
    }

    let behavior = if front_storm_cloud { "FrontStormCloudInit" } else { "BehindStormCloudInit" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[StormCloudLogic onGameFrame:]` @ 0x1001a84e4
///
/// A transition moves the tile model's edge by the speed (per frame) until it reports done; a
/// finished transition-out leaves the display tree. Otherwise, unless dragged, a speed beyond
/// `posVelScale · 20` either way is pulled back by 5 % per frame (in double).
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let state = logic::logic(world, this);
    if state.shutdown {
        return;
    }
    let display_object = state.display_object;
    let vel_x = state.pos_vel.x;

    let storm_cloud = world.obj::<StormCloudLogic>(this);
    let transition_in = storm_cloud.transition_in;
    let transition_out = storm_cloud.transition_out;
    let dragging = storm_cloud.dragging;
    let pos_vel_scale = storm_cloud.pos_vel_scale;

    if transition_in {
        let finished = if vel_x > 0.0 {
            tile_model::transition_in_left(world, display_object, vel_x)
        } else {
            tile_model::transition_in_right(world, display_object, vel_x)
        };
        if finished {
            world.obj_mut::<StormCloudLogic>(this).transition_in = false;
        }
    } else if transition_out {
        let finished = if vel_x > 0.0 {
            tile_model::transition_out_left(world, display_object, vel_x)
        } else {
            tile_model::transition_out_right(world, display_object, vel_x)
        };
        if finished {
            world.obj_mut::<StormCloudLogic>(this).transition_out = false;
            display::remove_from_display_group(world, display_object);
        }
    } else if dragging != 1.0 {
        let max_speed = pos_vel_scale * 20.0;
        let mut state = logic::logic_mut(world, this);

        let mut vel_x = state.pos_vel.x;
        if max_speed < vel_x {
            let step = f64::from(max_speed - vel_x) * 0.05;
            vel_x = (f64::from(vel_x) + step) as f32;
            state.pos_vel.x = vel_x;
        }

        let min_speed = -max_speed;
        if vel_x < min_speed {
            let step = f64::from(min_speed - vel_x) * 0.05;
            state.pos_vel.x = (f64::from(vel_x) + step) as f32;
        }
    }

    sprite_logic::on_game_frame(world, this, dt);
}

/// `-[StormCloudLogic enableDisplayObject]` @ 0x1001a86c0
pub fn enable_display_object(world: &mut World, this: Entity) {
    let front_storm_cloud = world.obj::<StormCloudLogic>(this).front_storm_cloud;
    let marker_id = if front_storm_cloud { "CloudFrontDisplayMarker" } else { "CloudBehindDisplayMarker" };
    let marker = pg_engine::library::display_marker(world, marker_id);

    let display_object = logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
}

/// `-[StormCloudLogic startVisible]` @ 0x1001a8744
pub fn start_visible(world: &mut World, this: Entity) {
    let pos_vel_scale = world.obj::<StormCloudLogic>(this).pos_vel_scale;
    logic::logic_mut(world, this).pos_vel.x = pos_vel_scale * 20.0;

    let display_object = logic::display_object(world, this);
    tile_model::reset_transition_visible(world, display_object);
    enable_display_object(world, this);
}

/// `-[StormCloudLogic startDragging]` @ 0x1001a87b4
pub fn start_dragging(world: &mut World, this: Entity) {
    logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<StormCloudLogic>(this).dragging = 1.0;
}

/// `-[StormCloudLogic stopDragging:]` @ 0x1001a87d8
pub fn stop_dragging(world: &mut World, this: Entity, dragging: f32) {
    let pos_vel_scale = {
        let mut storm_cloud = world.obj_mut::<StormCloudLogic>(this);
        storm_cloud.dragging = 0.0;
        storm_cloud.pos_vel_scale
    };
    logic::logic_mut(world, this).pos_vel.x = pos_vel_scale * dragging;
}

/// `-[StormCloudLogic applyDragOffset:]` @ 0x1001a8808
pub fn apply_drag_offset(world: &mut World, this: Entity, apply_drag_offset: f32) {
    let display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, display_object);
    display.pos.x = display.pos.x + apply_drag_offset;
}

/// `-[StormCloudLogic transitionOut:]` @ 0x1001a8850
pub fn transition_out(world: &mut World, this: Entity, transition_out: f32) {
    let pos_vel_scale = {
        let mut storm_cloud = world.obj_mut::<StormCloudLogic>(this);
        storm_cloud.transition_out = true;
        storm_cloud.transition_in = false;
        storm_cloud.pos_vel_scale
    };
    logic::logic_mut(world, this).pos_vel.x = pos_vel_scale * transition_out;

    let display_object = logic::display_object(world, this);
    if transition_out > 0.0 {
        tile_model::start_transition_out_left(world, display_object);
    } else {
        tile_model::start_transition_out_right(world, display_object);
    }
}

/// `-[StormCloudLogic transitionIn:]` @ 0x1001a88c4
pub fn transition_in(world: &mut World, this: Entity, transition_in: f32) {
    let pos_vel_scale = world.obj::<StormCloudLogic>(this).pos_vel_scale;
    logic::logic_mut(world, this).pos_vel.x = pos_vel_scale * transition_in;

    enable_display_object(world, this);

    let front_storm_cloud = world.obj::<StormCloudLogic>(this).front_storm_cloud;
    let behavior = if front_storm_cloud { "FrontStormCloudTransitionIn" } else { "BehindStormCloudTransitionIn" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[StormCloudLogic startTransitionIn:::]` @ 0x1001a8944
///
/// Waits for the tile model's frame, then starts its transition from the side the band moves
/// in from.
pub fn seq_start_transition_in(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = logic::display_object(world, this);
    let frame = multi_model::binary_frame_def(world, display_object);
    if frame.is_none() {
        return ItemResult::Pending;
    }

    {
        let mut storm_cloud = world.obj_mut::<StormCloudLogic>(this);
        storm_cloud.transition_in = true;
        storm_cloud.transition_out = false;
    }

    let vel_x = logic::logic(world, this).pos_vel.x;
    if vel_x > 0.0 {
        tile_model::start_transition_in_left(world, display_object);
    } else {
        tile_model::start_transition_in_right(world, display_object);
    }
    ItemResult::Done
}
