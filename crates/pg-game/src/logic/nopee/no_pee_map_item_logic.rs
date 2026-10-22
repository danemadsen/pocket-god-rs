//! `NoPeeMapItemLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_map_logic;
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeMapItemLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeMapItemLogic {
    /// `mNoPeeMapItemLogicListener`: `@"<NoPeeMapItemLogicListener>"`
    pub no_pee_map_item_logic_listener: Option<Entity>,

    /// `mNoPeeMapItemType`: `i`
    pub no_pee_map_item_type: i32,

    /// `mTrackedSpriteLogic`: `@"SpriteLogic"`
    pub tracked_sprite_logic: Option<Entity>,
}

pub const NO_PEE_MAP_ITEM_LOGIC: Class = Class {
    name: "NoPeeMapItemLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    game_frame_order: 60,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// The map scales the world, horizontally around the screen centre, by 148/3000 about x 76, and
/// vertically by 50/1000 from y 10.
const WORLD_WIDTH: f32 = 3000.0;
const MAP_WIDTH: f32 = 148.0;
const MAP_CENTER_X_POS: f32 = 76.0;
const WORLD_HEIGHT: f32 = 1000.0;
const MAP_HEIGHT: f32 = 50.0;
const MAP_BOTTOM_Y_POS: f32 = 10.0;

/// `[[NoPeeMapItemLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_MAP_ITEM_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeMapItemLogic initWithDisplayObject:]` @ 0x10017f714
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeMapItemLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);
}

/// `-[NoPeeMapItemLogic setNoPeeMapItemLogicListener:]` @ 0x10017f750
pub fn set_no_pee_map_item_logic_listener(world: &mut World, this: Entity, no_pee_map_item_logic_listener: Entity) {
    world.obj_mut::<NoPeeMapItemLogic>(this).no_pee_map_item_logic_listener = Some(no_pee_map_item_logic_listener);
}

/// `-[NoPeeMapItemLogic setTrackedSpriteLogic:type:]` @ 0x10017f7b4
///
/// Types: 0 pygmy, 1 treasure, 2 obstacle, 3 blob.
pub fn set_tracked_sprite_logic(world: &mut World, this: Entity, tracked_sprite_logic: Option<Entity>, type_: i32) {
    {
        let mut item = world.obj_mut::<NoPeeMapItemLogic>(this);
        item.tracked_sprite_logic = tracked_sprite_logic;
        item.no_pee_map_item_type = type_;
    }

    let behavior = match type_ {
        0 => "NoPeeMapItemTypePygmy",
        1 => "NoPeeMapItemTypeTreasure",
        2 => "NoPeeMapItemTypeObstacle",
        3 => "NoPeeMapItemTypeBlob",
        _ => return,
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[NoPeeMapItemLogic onGameFrame:]` @ 0x10017f860
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let tracked = world.obj::<NoPeeMapItemLogic>(this).tracked_sprite_logic;
    let Some(tracked) = tracked else {
        return;
    };

    let display_object = logic::display_object(world, this);
    let tracked_display_object = logic::display_object(world, tracked);
    let tracked_pos = display::object(world, tracked_display_object).pos;

    let listener = world.obj::<NoPeeMapItemLogic>(this).no_pee_map_item_logic_listener;
    let no_pee = match listener {
        Some(listener) => no_pee_map_logic::no_pee_logic(world, listener),
        None => None,
    };
    let group_pos = no_pee_logic::world_group_pos(world, no_pee);

    {
        let center_x_pos = 240.0 - group_pos.x;
        let mut display = display::object_mut(world, display_object);
        display.pos.x = ((tracked_pos.x - center_x_pos) * MAP_WIDTH) / WORLD_WIDTH + MAP_CENTER_X_POS;
        display.pos.y = (tracked_pos.y * MAP_HEIGHT) / WORLD_HEIGHT + MAP_BOTTOM_Y_POS;
    }

    let tracks_world_object = is_kind_of(world, Some(tracked), "NoPeeObjectLogic");
    if !tracks_world_object {
        return;
    }

    let hide = crate::send::should_hide_no_pee_map_item(world, Some(tracked));
    display::object_mut(world, display_object).hidden = hide;
}
