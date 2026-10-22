//! `OceanLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::display;
use pg_engine::logic;

/// `OceanLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct OceanLogic {
    /// `mbFrontOcean`: `B`
    pub front_ocean: bool,

    /// `mPosVelScale`: `f`
    pub pos_vel_scale: f32,

    /// `mbDragging`: `f`
    pub dragging: f32,
}

pub const OCEAN_LOGIC: Class = Class {
    name: "OceanLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[OceanLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&OCEAN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[OceanLogic initWithDisplayObject:]` @ 0x1000137c8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(OceanLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[OceanLogic setFrontOcean:posVelScale:]` @ 0x100013804
pub fn set_front_ocean(world: &mut World, this: Entity, front_ocean: bool, pos_vel_scale: f32) {
    {
        let mut ocean = world.obj_mut::<OceanLogic>(this);
        ocean.front_ocean = front_ocean;
        ocean.pos_vel_scale = pos_vel_scale;
    }

    let behavior = if front_ocean { "FrontOceanInit" } else { "BehindOceanInit" };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[OceanLogic onGameFrame:]` @ 0x100013884
///
/// Unless dragged, a speed beyond `posVelScale · 20` either way is pulled back by 5 % per frame
/// (in double).
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    if logic::logic(world, this).shutdown {
        return;
    }

    let ocean = world.obj::<OceanLogic>(this);
    if ocean.dragging != 1.0 {
        let max_speed = ocean.pos_vel_scale * 20.0;
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

/// `-[OceanLogic startDragging]` @ 0x100013978
pub fn start_dragging(world: &mut World, this: Entity) {
    logic::logic_mut(world, this).pos_vel.x = 0.0;
    world.obj_mut::<OceanLogic>(this).dragging = 1.0;
}

/// `-[OceanLogic stopDragging:]` @ 0x10001399c
pub fn stop_dragging(world: &mut World, this: Entity, dragging: f32) {
    let pos_vel_scale = {
        let mut ocean = world.obj_mut::<OceanLogic>(this);
        ocean.dragging = 0.0;
        ocean.pos_vel_scale
    };
    logic::logic_mut(world, this).pos_vel.x = pos_vel_scale * dragging;
}

/// `-[OceanLogic applyDragOffset:]` @ 0x1000139cc
pub fn apply_drag_offset(world: &mut World, this: Entity, apply_drag_offset: f32) {
    let display_object = logic::display_object(world, this);
    let mut display = display::object_mut(world, display_object);
    display.pos.x = display.pos.x + apply_drag_offset;
}
