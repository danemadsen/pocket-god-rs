//! `IcicleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::ice_island::ice_hole_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};

/// `IcicleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IcicleLogic {
    /// `mIcicleLogicListener`: `@"<IcicleLogicListener>"`
    pub icicle_logic_listener: Option<Entity>,
}

pub const ICICLE_LOGIC: Class = Class {
    name: "IcicleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `IcicleLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[IcicleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&ICICLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[IcicleLogic initWithDisplayObject:]` @ 0x1000ca14c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IcicleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    pg_engine::logic::set_behavior(world, this, Some("IcicleFall"));
}

/// `-[IcicleLogic setIcicleLogicListener:]` @ 0x1000ca1c8
pub fn set_icicle_logic_listener(world: &mut World, this: Entity, icicle_logic_listener: Entity) {
    world.obj_mut::<IcicleLogic>(this).icicle_logic_listener = Some(icicle_logic_listener);
}

/// `-[IcicleLogic fall:::]` @ 0x1000ca288
///
/// Keeps the icicle inside the 480-wide screen, breaks it on the first pygmy within 64 units
/// that takes the hit, and completes once it lands on the island (y 86).
pub fn seq_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let min_pos = pg_engine::display::min_pos(world, display_object);
    let max_pos = pg_engine::display::max_pos(world, display_object);

    if min_pos.x < 0.0 {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = (object.pos.x - min_pos.x) + 0.0;
    }
    if 480.0 < max_pos.x {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = 480.0 - (max_pos.x - object.pos.x);
    }

    let icicle_pos = pg_engine::display::object(world, display_object).pos;
    let listener = world.obj::<IcicleLogic>(this).icicle_logic_listener;
    let pygmies = match listener {
        Some(ice_hole) => ice_hole_logic::pygmy_logic_array(world, ice_hole, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_pos = pg_engine::display::object(world, pygmy_display_object).pos;
        let delta_x = icicle_pos.x - pygmy_pos.x;
        let delta_y = icicle_pos.y - pygmy_pos.y;
        let in_range = delta_x * delta_x + delta_y * delta_y < 4096.0;
        if in_range && crate::pygmy::hit_by_icicle(world, pygmy, this) {
            pg_engine::logic::set_behavior(world, this, Some("IcicleDie"));
            break;
        }
    }

    let listener = world.obj::<IcicleLogic>(this).icicle_logic_listener;
    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(ice_hole) = listener {
        island_min_x = ice_hole_logic::island_min_x_pos(world, ice_hole);
        island_max_x = ice_hole_logic::island_max_x_pos(world, ice_hole);
    }

    let pos = pg_engine::display::object(world, display_object).pos;
    let landed = island_min_x <= pos.x && pos.x <= island_max_x && pos.y < 86.0;
    if !landed {
        return ItemResult::Pending;
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = 0.0;
        logic.pos_vel.y = 0.0;
    }
    let mut object = pg_engine::display::object_mut(world, display_object);
    object.rot.z = 0.0;
    object.pos.y = 86.0;
    ItemResult::Done
}

/// `-[IcicleLogic die:::]` @ 0x1000ca614
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<IcicleLogic>(this).icicle_logic_listener;
    if let Some(ice_hole) = listener {
        ice_hole_logic::on_icicle_die(world, ice_hole, this);
    }
    ItemResult::Done
}
