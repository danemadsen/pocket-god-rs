//! `UnderwaterTempleSymbolLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use pg_engine::logic;

/// `UnderwaterTempleSymbolLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct UnderwaterTempleSymbolLogic {
    /// `mUnderwaterTempleSymbolShapeLogic`: `@"MultiModelLogic"`
    pub underwater_temple_symbol_shape_logic: Option<Entity>,

    /// `mShapeBehaviorId`: `@"NSString"`
    pub shape_behavior_id: Option<String>,

    /// `mShapeGlowBehaviorId`: `@"NSString"`
    pub shape_glow_behavior_id: Option<String>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mUnderwaterTempleMode`: `i`
    pub underwater_temple_mode: i32,
}

pub const UNDERWATER_TEMPLE_SYMBOL_LOGIC: Class = Class {
    name: "UnderwaterTempleSymbolLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `[[UnderwaterTempleSymbolLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&UNDERWATER_TEMPLE_SYMBOL_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[UnderwaterTempleSymbolLogic initWithDisplayObject:]` @ 0x100178058
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(UnderwaterTempleSymbolLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    logic::set_behavior(world, this, Some("UnderwaterTempleSymbolInit"));
}

/// `-[UnderwaterTempleSymbolLogic setSymbolShapeLogic:shapeBehaviorId:shapeGlowBehaviorId:]` @ 0x100178100
pub fn set_symbol_shape_logic(world: &mut World, this: Entity, symbol_shape_logic: Entity, shape_behavior_id: &str, shape_glow_behavior_id: &str) {
    {
        let mut symbol = world.obj_mut::<UnderwaterTempleSymbolLogic>(this);
        symbol.underwater_temple_symbol_shape_logic = Some(symbol_shape_logic);
        symbol.shape_behavior_id = Some(shape_behavior_id.to_owned());
        symbol.shape_glow_behavior_id = Some(shape_glow_behavior_id.to_owned());
    }
    logic::set_behavior(world, symbol_shape_logic, Some(shape_behavior_id));
}

/// `-[UnderwaterTempleSymbolLogic isActive]` @ 0x100178140
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<UnderwaterTempleSymbolLogic>(this).active
}

/// `-[UnderwaterTempleSymbolLogic toggleSymbol]` @ 0x100178150
///
/// Lit, the shape glows; unlit, it goes back to normal.
pub fn toggle_symbol(world: &mut World, this: Entity) {
    let (was_active, shape_logic, shape_behavior_id, shape_glow_behavior_id) = {
        let symbol = world.obj::<UnderwaterTempleSymbolLogic>(this);
        (
            symbol.active,
            symbol.underwater_temple_symbol_shape_logic,
            symbol.shape_behavior_id.clone(),
            symbol.shape_glow_behavior_id.clone(),
        )
    };

    let behavior = if !was_active {
        world.obj_mut::<UnderwaterTempleSymbolLogic>(this).active = true;
        if let Some(shape_logic) = shape_logic {
            logic::set_behavior(world, shape_logic, shape_glow_behavior_id.as_deref());
        }
        "UnderwaterTempleSymbolOn"
    } else {
        world.obj_mut::<UnderwaterTempleSymbolLogic>(this).active = false;
        if let Some(shape_logic) = shape_logic {
            logic::set_behavior(world, shape_logic, shape_behavior_id.as_deref());
        }
        "UnderwaterTempleSymbolOff"
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[UnderwaterTempleSymbolLogic setUnderwaterTempleMode:]` @ 0x1001781fc
///
/// A lit symbol shows the temple's power: on (0), cold, heat, oil, acid or air (1–5).
pub fn set_underwater_temple_mode(world: &mut World, this: Entity, underwater_temple_mode: i32) {
    let active = {
        let mut symbol = world.obj_mut::<UnderwaterTempleSymbolLogic>(this);
        symbol.underwater_temple_mode = underwater_temple_mode;
        symbol.active
    };
    if !active {
        return;
    }

    let behavior = match underwater_temple_mode {
        0 => "UnderwaterTempleSymbolOn",
        1 => "UnderwaterTempleSymbolCold",
        2 => "UnderwaterTempleSymbolHeat",
        3 => "UnderwaterTempleSymbolOil",
        4 => "UnderwaterTempleSymbolAcid",
        5 => "UnderwaterTempleSymbolAir",
        _ => return,
    };
    logic::set_behavior(world, this, Some(behavior));
}
