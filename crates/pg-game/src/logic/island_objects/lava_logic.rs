//! `LavaLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::island_objects::volcano_logic;

/// `LavaLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct LavaLogic {
    /// `mLavaLogicListener`: `@"<LavaLogicListener>"`
    pub lava_logic_listener: Option<Entity>,
}

pub const LAVA_LOGIC: Class = Class {
    name: "LavaLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `LavaLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "fall" => seq_fall(world, this, call),
        "die" => seq_die(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[LavaLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&LAVA_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[LavaLogic initWithDisplayObject:]` @ 0x10000d14c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(LavaLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    pg_engine::logic::set_behavior(world, this, Some("LavaFall"));
}

/// `-[LavaLogic setLavaLogicListener:]` @ 0x10000d1c8
pub fn set_lava_logic_listener(world: &mut World, this: Entity, lava_logic_listener: Entity) {
    world.obj_mut::<LavaLogic>(this).lava_logic_listener = Some(lava_logic_listener);
}

/// `-[LavaLogic fall:::]` @ 0x10000d288
pub fn seq_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let min_pos = pg_engine::display::min_pos(world, display_object);
    let max_pos = pg_engine::display::max_pos(world, display_object);

    // Keeps the blob inside the 480-wide screen.
    if min_pos.x < 0.0 {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = (object.pos.x - min_pos.x) + 0.0;
    }
    if 480.0 < max_pos.x {
        let mut object = pg_engine::display::object_mut(world, display_object);
        object.pos.x = 480.0 - (max_pos.x - object.pos.x);
    }

    let lava_pos = pg_engine::display::object(world, display_object).pos;
    let listener = world.obj::<LavaLogic>(this).lava_logic_listener;
    let pygmies = match listener {
        Some(volcano) => volcano_logic::pygmy_logic_array(world, volcano, this),
        None => Vec::new(),
    };
    for pygmy in pygmies {
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_pos = pg_engine::display::object(world, pygmy_display_object).pos;
        let delta_x = lava_pos.x - pygmy_pos.x;
        let delta_y = lava_pos.y - pygmy_pos.y;
        if delta_x * delta_x + delta_y * delta_y < 4096.0 {
            crate::pygmy::hit_by_lava(world, pygmy, this);
        }
    }

    let mut island_min_x = 0.0;
    let mut island_max_x = 0.0;
    if let Some(volcano) = listener {
        island_min_x = volcano_logic::island_min_x_pos(world, volcano);
        island_max_x = volcano_logic::island_max_x_pos(world, volcano);
    }

    let pos = pg_engine::display::object(world, display_object).pos;
    let landed = island_min_x <= pos.x && pos.x <= island_max_x && pos.y < 76.0;
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
    object.pos.y = 76.0;
    ItemResult::Done
}

/// `-[LavaLogic die:::]` @ 0x10000d5f0
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<LavaLogic>(this).lava_logic_listener;
    if let Some(volcano) = listener {
        volcano_logic::on_lava_die(world, volcano, this);
    }
    ItemResult::Done
}
