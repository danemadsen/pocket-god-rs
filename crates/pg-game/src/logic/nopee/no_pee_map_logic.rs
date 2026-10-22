//! `NoPeeMapLogic` (superclass `SpriteLogic`).

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::nopee::no_pee_logic;
use crate::logic::nopee::no_pee_map_item_logic;
use crate::logic::nopee::no_pee_object_logic::NO_PEE_Z_POS;
use pg_engine::display;
use pg_engine::logic;

/// `NoPeeMapLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct NoPeeMapLogic {
    /// `mNoPeeMapLogicListener`: `@"<NoPeeMapLogicListener>"`
    pub no_pee_map_logic_listener: Option<Entity>,

    /// `mNoPeeMapItemLogicArray`: `@"NSMutableArray"`
    pub no_pee_map_item_logic_array: Vec<Entity>,

    /// `mBreathPercent`: `f`
    pub breath_percent: f32,

    /// `mbTrackBreath`: `B`
    pub track_breath: bool,

    /// `mNoPeeMapBackgroundLogic`: `@"MultiModelLogic"`
    pub no_pee_map_background_logic: Option<Entity>,
}

/// File-scope globals of `NoPeeMapLogic`.
#[derive(Resource, Default, Clone, Debug)]
pub struct NoPeeMapLogicStatics {
    /// `DAT_100821f58`: `CFAbsoluteTimeGetCurrent()` at the last `takeBreath` (never read).
    pub take_breath_time: f64,
}

pub const NO_PEE_MAP_LOGIC: Class = Class {
    name: "NoPeeMapLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    game_frame_order: 60,
    on_game_frame,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// A full breath lasts this many seconds while the pygmy can be hit.
const BREATH_SECONDS: f32 = 15.0;

/// The map's corner sits this far from the screen's top-right corner.
const MAP_X_OFFSET: f32 = -74.0;
const MAP_Y_OFFSET: f32 = -61.0;
const MAP_Y_MARGIN: f32 = -10.0;

/// `[[NoPeeMapLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&NO_PEE_MAP_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[NoPeeMapLogic initWithDisplayObject:]` @ 0x100182b84
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(NoPeeMapLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let background = pg_engine::library::logic(world, crate::logic_table::MAP_BACKGROUND);
    world.obj_mut::<NoPeeMapLogic>(this).no_pee_map_background_logic = Some(background);
}

/// `-[NoPeeMapLogic setNoPeeMapLogicListener:]` @ 0x100182c44
pub fn set_no_pee_map_logic_listener(world: &mut World, this: Entity, no_pee_map_logic_listener: Entity) {
    world.obj_mut::<NoPeeMapLogic>(this).no_pee_map_logic_listener = Some(no_pee_map_logic_listener);
}

/// `-[NoPeeMapLogic addTrackedSpriteLogic:type:]` @ 0x100182ca0
///
/// One `NoPeeMapItem` dot inside the map group; the map's item array is its only owner.
pub fn add_tracked_sprite_logic(world: &mut World, this: Entity, tracked_sprite_logic: Option<Entity>, type_: i32) {
    let model = display::multi_model::spawn(world, Some("NoPeeMapItem"));
    let item = no_pee_map_item_logic::spawn(world, model);
    no_pee_map_item_logic::set_tracked_sprite_logic(world, item, tracked_sprite_logic, type_);
    no_pee_map_item_logic::set_no_pee_map_item_logic_listener(world, item, this);

    {
        let mut model_display = display::object_mut(world, model);
        model_display.pos.x = 0.0;
        model_display.pos.y = 0.0;
        model_display.pos.z = NO_PEE_Z_POS;
    }

    let map_group = logic::display_object(world, this);
    display::group::add_display_object(world, map_group, Some(model));
    world.obj_mut::<NoPeeMapLogic>(this).no_pee_map_item_logic_array.push(item);
}

/// `-[NoPeeMapLogic clearAllTrackedSpriteLogics]` @ 0x100182dec
pub fn clear_all_tracked_sprite_logics(world: &mut World, this: Entity) {
    let items = world.obj::<NoPeeMapLogic>(this).no_pee_map_item_logic_array.clone();
    for item in items {
        logic::shutdown(world, item);
        let item_display_object = logic::display_object(world, item);
        display::remove_from_display_group(world, item_display_object);
    }

    // `removeAllObjects`: the array was each item's only owner, and each item its model's last.
    let removed_items = {
        let mut map = world.obj_mut::<NoPeeMapLogic>(this);
        std::mem::take(&mut map.no_pee_map_item_logic_array)
    };
    for item in removed_items {
        let item_display_object = logic::display_object(world, item);
        logic::dealloc(world, item);
        world.despawn(item_display_object);
    }
}

/// `-[NoPeeMapLogic noPeeLogic]` @ 0x100182f58
pub fn no_pee_logic(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<NoPeeMapLogic>(this).no_pee_map_logic_listener;
    match listener {
        Some(listener) => no_pee_logic::no_pee_logic(world, listener),
        None => None,
    }
}

/// `-[NoPeeMapLogic startTracking]` @ 0x100182f74
pub fn start_tracking(world: &mut World, this: Entity) {
    let mut map = world.obj_mut::<NoPeeMapLogic>(this);
    map.breath_percent = 1.0;
    map.track_breath = false;
}

/// `-[NoPeeMapLogic takeBreath]` @ 0x100182f94
pub fn take_breath(world: &mut World, this: Entity) {
    {
        let mut map = world.obj_mut::<NoPeeMapLogic>(this);
        map.track_breath = true;
        map.breath_percent = 1.0;
    }

    let now = crate::controller::cf_absolute_time_get_current(world);
    world.get_resource_or_insert_with(NoPeeMapLogicStatics::default).take_breath_time = now;
}

/// `-[NoPeeMapLogic onGameFrame:]` @ 0x100182fd8
///
/// Drains the breath while the pygmy can be hit (it runs out after 15 s), shows it on the map
/// background, and keeps the map in the screen's top-right corner.
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let listener = world.obj::<NoPeeMapLogic>(this).no_pee_map_logic_listener;
    let pygmy = match listener {
        Some(listener) => no_pee_logic::no_pee_pygmy_logic(world, listener),
        None => None,
    };

    let track_breath = world.obj::<NoPeeMapLogic>(this).track_breath;
    if track_breath {
        let can_be_hit = match pygmy {
            Some(pygmy) => crate::pygmy::can_be_hit_by_no_pee_object(world, pygmy),
            None => false,
        };
        if can_be_hit {
            let breath_percent = world.obj::<NoPeeMapLogic>(this).breath_percent + dt / -BREATH_SECONDS;
            world.obj_mut::<NoPeeMapLogic>(this).breath_percent = breath_percent;
            if breath_percent < 0.0 {
                world.obj_mut::<NoPeeMapLogic>(this).breath_percent = 0.0;
                if let Some(pygmy) = pygmy {
                    crate::pygmy::no_pee_run_out_of_breath(world, pygmy);
                }
            }
        }
    }

    let background = world.obj::<NoPeeMapLogic>(this).no_pee_map_background_logic;
    if let Some(background) = background {
        let background_animation = logic::animation(world, background);
        let breath_percent = world.obj::<NoPeeMapLogic>(this).breath_percent;
        pg_engine::animation::set_frame_percent(world, background_animation, 1.0 - breath_percent);
    }

    let display_object = logic::display_object(world, this);
    let no_pee = match listener {
        Some(listener) => no_pee_logic::no_pee_logic(world, listener),
        None => None,
    };
    let group_pos = no_pee_logic::world_group_pos(world, no_pee);

    let mut display = display::object_mut(world, display_object);
    display.pos.x = (240.0 - group_pos.x) + MAP_X_OFFSET;
    display.pos.y = (320.0 - group_pos.y) + MAP_Y_OFFSET + MAP_Y_MARGIN;
}
