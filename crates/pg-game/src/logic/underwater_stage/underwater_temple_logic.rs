//! `UnderwaterTempleLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::underwater_stage::underwater_temple_symbol_logic;
use crate::logic_table;
use pg_engine::{display, library, logic};

/// `UnderwaterTempleLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct UnderwaterTempleLogic {
    /// `mUnderwaterTempleLogicListener`: `@"<UnderwaterTempleLogicListener>"`
    pub underwater_temple_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mUnderwaterTempleMode`: `i`
    pub underwater_temple_mode: i32,

    /// `mSymbolLogicArray`: `@"NSMutableArray"`
    pub symbol_logic_array: Vec<Entity>,

    /// `mSymbolButtonLogicArray`: `@"NSMutableArray"`
    pub symbol_button_logic_array: Vec<Entity>,

    /// `mUnderwaterTempleButtonsDisplayGroup`: `@"BCDisplayGroup"`
    pub underwater_temple_buttons_display_group: Option<Entity>,
}

pub const UNDERWATER_TEMPLE_LOGIC: Class = Class {
    name: "UnderwaterTempleLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    on_game_frame,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The temple's nine symbols, rows of three.
pub const SYMBOL_COUNT: usize = 9;

/// A symbol pattern: which of the nine symbols are lit.
pub type SwitchArray = [bool; SYMBOL_COUNT];

/// One symbol as `initWithDisplayObject:` pairs it: the symbol logic, its shape logic and the
/// shape's normal and glowing behaviours.
struct SymbolDef {
    symbol: usize,
    shape: usize,
    shape_behavior_id: &'static str,
    shape_glow_behavior_id: &'static str,
}

const SYMBOL_DEFS: [SymbolDef; SYMBOL_COUNT] = [
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL01,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE01,
        shape_behavior_id: "UnderwaterTempleSymbolShape01",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow01",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL02,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE02,
        shape_behavior_id: "UnderwaterTempleSymbolShape02",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow02",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL03,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE03,
        shape_behavior_id: "UnderwaterTempleSymbolShape03",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow03",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL04,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE04,
        shape_behavior_id: "UnderwaterTempleSymbolShape04",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow04",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL05,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE05,
        shape_behavior_id: "UnderwaterTempleSymbolShape05",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow05",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL06,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE06,
        shape_behavior_id: "UnderwaterTempleSymbolShape06",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow06",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL07,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE07,
        shape_behavior_id: "UnderwaterTempleSymbolShape07",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow07",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL08,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE08,
        shape_behavior_id: "UnderwaterTempleSymbolShape08",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow08",
    },
    SymbolDef {
        symbol: logic_table::UNDERWATER_TEMPLE_SYMBOL09,
        shape: logic_table::UNDERWATER_TEMPLE_SYMBOL_SHAPE09,
        shape_behavior_id: "UnderwaterTempleSymbolShape09",
        shape_glow_behavior_id: "UnderwaterTempleSymbolShapeGlow09",
    },
];

/// The nine symbol buttons, in symbol order.
const SYMBOL_BUTTONS: [usize; SYMBOL_COUNT] = [
    logic_table::SYMBOL01_BUTTON,
    logic_table::SYMBOL02_BUTTON,
    logic_table::SYMBOL03_BUTTON,
    logic_table::SYMBOL04_BUTTON,
    logic_table::SYMBOL05_BUTTON,
    logic_table::SYMBOL06_BUTTON,
    logic_table::SYMBOL07_BUTTON,
    logic_table::SYMBOL08_BUTTON,
    logic_table::SYMBOL09_BUTTON,
];

/// One underwater power: the temple mode it sets, its symbol pattern (`DAT_100602c58 + 9 k`)
/// and the episode event its discovery triggers. Checked in this order.
struct TempleModeDef {
    mode: i32,
    pattern: SwitchArray,
    episode_event: &'static str,
}

const TEMPLE_MODE_DEFS: [TempleModeDef; 5] = [
    TempleModeDef {
        mode: 1,
        pattern: [true, true, true, false, true, false, true, true, true],
        episode_event: "EnableUnderwaterPowerCold",
    },
    TempleModeDef {
        mode: 2,
        pattern: [true, true, false, true, false, false, true, true, true],
        episode_event: "EnableUnderwaterPowerHeat",
    },
    TempleModeDef {
        mode: 3,
        pattern: [true, true, true, true, false, false, true, false, false],
        episode_event: "EnableUnderwaterPowerOil",
    },
    TempleModeDef {
        mode: 4,
        pattern: [true, false, true, true, true, true, true, false, false],
        episode_event: "EnableUnderwaterPowerAcid",
    },
    TempleModeDef {
        mode: 5,
        pattern: [true, true, true, true, true, true, false, true, false],
        episode_event: "EnableUnderwaterPowerAir",
    },
];

/// `[[UnderwaterTempleLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&UNDERWATER_TEMPLE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[UnderwaterTempleLogic initWithDisplayObject:]` @ 0x1000c7cb8
///
/// Pairs each of the nine symbols with its shape and collects the symbol buttons.
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(UnderwaterTempleLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().underwater_temple_logic = Some(this);

    let buttons_display_group = library::display_group(world, "UnderwaterTempleButtonsDisplayGroup");
    world.obj_mut::<UnderwaterTempleLogic>(this).underwater_temple_buttons_display_group = buttons_display_group;

    world.obj_mut::<UnderwaterTempleLogic>(this).symbol_logic_array = Vec::with_capacity(5);
    for symbol_def in &SYMBOL_DEFS {
        let symbol = library::logic(world, symbol_def.symbol);
        let shape = library::logic(world, symbol_def.shape);
        underwater_temple_symbol_logic::set_symbol_shape_logic(
            world,
            symbol,
            shape,
            symbol_def.shape_behavior_id,
            symbol_def.shape_glow_behavior_id,
        );
        world.obj_mut::<UnderwaterTempleLogic>(this).symbol_logic_array.push(symbol);
    }

    world.obj_mut::<UnderwaterTempleLogic>(this).symbol_button_logic_array = Vec::with_capacity(5);
    for button_index in SYMBOL_BUTTONS {
        let button = library::logic(world, button_index);
        world.obj_mut::<UnderwaterTempleLogic>(this).symbol_button_logic_array.push(button);
    }

    logic::set_behavior(world, this, Some("UnderwaterTempleInit"));
}

/// `-[UnderwaterTempleLogic setUnderwaterTempleLogicListener:]` @ 0x1000c8328
pub fn set_underwater_temple_logic_listener(world: &mut World, this: Entity, underwater_temple_logic_listener: Entity) {
    world.obj_mut::<UnderwaterTempleLogic>(this).underwater_temple_logic_listener = Some(underwater_temple_logic_listener);
}

/// `-[UnderwaterTempleLogic underwaterTempleMode]` @ 0x1000c8384
pub fn underwater_temple_mode(world: &mut World, this: Entity) -> i32 {
    world.obj::<UnderwaterTempleLogic>(this).underwater_temple_mode
}

/// `-[UnderwaterTempleLogic onGameFrame:]` @ 0x1000c8394
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[UnderwaterTempleLogic activate]` @ 0x1000c8428
///
/// The symbol buttons appear over the temple.
pub fn activate(world: &mut World, this: Entity) {
    if world.obj::<UnderwaterTempleLogic>(this).active {
        return;
    }
    world.obj_mut::<UnderwaterTempleLogic>(this).active = true;

    let buttons_display_group = world.obj::<UnderwaterTempleLogic>(this).underwater_temple_buttons_display_group;
    let marker = library::display_marker(world, "TsunamiDisplayMarker");
    if let Some(marker) = marker {
        display::insert_before(world, marker, buttons_display_group);
    }

    let own_display_object = logic::display_object(world, this);
    let temple_pos = display::object(world, own_display_object).pos;
    if let Some(buttons_display_group) = buttons_display_group {
        let mut group = display::object_mut(world, buttons_display_group);
        group.pos.x = temple_pos.x;
        group.pos.y = temple_pos.y;
    }
}

/// `-[UnderwaterTempleLogic deactivate]` @ 0x1000c84f4
pub fn deactivate(world: &mut World, this: Entity) {
    if !world.obj::<UnderwaterTempleLogic>(this).active {
        return;
    }
    world.obj_mut::<UnderwaterTempleLogic>(this).active = false;

    let buttons_display_group = world.obj::<UnderwaterTempleLogic>(this).underwater_temple_buttons_display_group;
    if let Some(buttons_display_group) = buttons_display_group {
        display::remove_from_display_group(world, buttons_display_group);
    }
}

/// `-[UnderwaterTempleLogic checkSwitchArray:]` @ 0x1000c8528
///
/// Whether the lit symbols are exactly `check_switch_array`.
pub fn check_switch_array(world: &mut World, this: Entity, check_switch_array: &SwitchArray) -> bool {
    let symbols = world.obj::<UnderwaterTempleLogic>(this).symbol_logic_array.clone();
    for (index, symbol) in symbols.into_iter().enumerate() {
        let symbol_active = underwater_temple_symbol_logic::is_active(world, symbol);
        if symbol_active != check_switch_array[index] {
            return false;
        }
    }
    true
}

/// `-[UnderwaterTempleLogic activateSwitchArray:]` @ 0x1000c8664
///
/// The pattern's symbols take on the temple mode, the others mode 0.
pub fn activate_switch_array(world: &mut World, this: Entity, activate_switch_array: &SwitchArray) {
    let symbols = world.obj::<UnderwaterTempleLogic>(this).symbol_logic_array.clone();
    for (index, symbol) in symbols.into_iter().enumerate() {
        let mode = if activate_switch_array[index] {
            world.obj::<UnderwaterTempleLogic>(this).underwater_temple_mode
        } else {
            0
        };
        underwater_temple_symbol_logic::set_underwater_temple_mode(world, symbol, mode);
    }
}

/// `-[UnderwaterTempleLogic deactivateSwitchArray]` @ 0x1000c87c8
pub fn deactivate_switch_array(world: &mut World, this: Entity) {
    let symbols = world.obj::<UnderwaterTempleLogic>(this).symbol_logic_array.clone();
    for symbol in symbols {
        underwater_temple_symbol_logic::set_underwater_temple_mode(world, symbol, 0);
    }
}

/// `-[UnderwaterTempleLogic onButtonPress:touch:]` @ 0x1000c88d8
///
/// A symbol button toggles its symbol (with its tone); the first power whose pattern the lit
/// symbols match becomes the temple mode, and no match ends the current one.
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let buttons = world.obj::<UnderwaterTempleLogic>(this).symbol_button_logic_array.clone();
    let Some(button_index) = buttons.iter().position(|candidate| *candidate == button) else {
        return;
    };

    let tone_id = format!("TempleTone{}", button_index as i32 + 1);
    pg_engine::sound::play_sound(world, &tone_id);
    trigger_episode_event(world, "ToggleUnderwaterTempleSymbol");

    let symbol = world.obj::<UnderwaterTempleLogic>(this).symbol_logic_array[button_index];
    underwater_temple_symbol_logic::toggle_symbol(world, symbol);

    for mode_def in &TEMPLE_MODE_DEFS {
        if !check_switch_array(world, this, &mode_def.pattern) {
            continue;
        }

        world.obj_mut::<UnderwaterTempleLogic>(this).underwater_temple_mode = mode_def.mode;
        trigger_episode_event(world, mode_def.episode_event);
        notify_mode_changed(world, this);
        activate_switch_array(world, this, &mode_def.pattern);
        return;
    }

    if world.obj::<UnderwaterTempleLogic>(this).underwater_temple_mode == 0 {
        return;
    }
    world.obj_mut::<UnderwaterTempleLogic>(this).underwater_temple_mode = 0;
    notify_mode_changed(world, this);
    deactivate_switch_array(world, this);
}

/// `-[UnderwaterTempleLogic onButtonRelease:touch:]` @ 0x1000c8cfc
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[UnderwaterTempleLogic onButtonReleaseOutside:touch:]` @ 0x1000c8d00
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `+[UnderwaterTempleLogic instance]` @ 0x1000c7cac
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().underwater_temple_logic
}

/// `[[IdolManager instance] triggerEpisodeEvent:event]`.
fn trigger_episode_event(world: &mut World, event: &str) {
    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::trigger_episode_event(world, idol_manager, event);
    }
}

/// `[mUnderwaterTempleLogicListener onUnderwaterTempleModeChanged:mUnderwaterTempleMode]`.
fn notify_mode_changed(world: &mut World, this: Entity) {
    let (listener, mode) = {
        let temple = world.obj::<UnderwaterTempleLogic>(this);
        (temple.underwater_temple_logic_listener, temple.underwater_temple_mode)
    };
    if let Some(listener) = listener {
        crate::controller::on_underwater_temple_mode_changed(world, listener, mode);
    }
}
