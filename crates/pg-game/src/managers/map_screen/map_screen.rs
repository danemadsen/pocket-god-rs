//! `MapScreen`: methods in this part of the class.

use pg_engine::display;
use pg_engine::library::{self, Library};
use pg_engine::sequence;

use crate::controller;
use crate::logic::idol_ui_widgets::{ad_blocker_logic, map_paper_logic};
use crate::logic_table;
use crate::prelude::*;
use super::*;

/// `@"MapScreen"` (`0x1003ca770`).
const CLASS_ID: &str = "MapScreen";

/// `@"MapContentDisplayGroup"` (`0x1003ca790`): the island art and buttons on the map paper.
const MAP_CONTENT_DISPLAY_GROUP: &str = "MapContentDisplayGroup";

/// `@"OpenMapScreen"` (`0x1003ca7b0`).
const OPEN_MAP_SCREEN_SEQUENCE: &str = "OpenMapScreen";

/// `@"ClickMapScreenButton"` (`0x1003ca7d0`).
const CLICK_MAP_SCREEN_BUTTON_SEQUENCE: &str = "ClickMapScreenButton";

/// `@"CloseMapScreen"` (`0x1003ca7f0`).
const CLOSE_MAP_SCREEN_SEQUENCE: &str = "CloseMapScreen";

/// `@"MapTextureAtlas0"`: the map's atlas image.
const MAP_TEXTURE_ATLAS: &str = "MapTextureAtlas0";

/// The atlas slot every location atlas goes into (the def at `0x10077b790`).
const SCREEN_ATLAS_ID: &str = "screenAtlas0";

// `mIslandLocationState` values the map buttons choose (`controller.md` §4.1).
const LOCATION_SAND_ISLAND: i32 = 0;
const LOCATION_ROCK_ISLAND: i32 = 1;
const LOCATION_UNDERWATER: i32 = 2;
const LOCATION_GRAVEYARD_ISLAND: i32 = 3;
const LOCATION_ICE_ISLAND: i32 = 4;
const LOCATION_APE_ISLAND: i32 = 5;
const LOCATION_MISFIT_ISLAND: i32 = 6;
const LOCATION_GOD_ISLAND: i32 = 7;
const LOCATION_FAIL_BOAT: i32 = 8;
const LOCATION_UNDERWATER_CORAL_REEF: i32 = 9;
const LOCATION_APOCALYPSE_ISLAND: i32 = 10;

/// `-[MapScreen init]` @ 0x1000c0a18
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(MapScreen::default());

    world.get_resource_or_insert_with(Singletons::default).map_screen = Some(this);

    let map_sequence = sequence::spawn(world, None, Some(this));

    let close_button_logic = library::logic(world, logic_table::MAP_CONTENT_CLOSE_BUTTON);
    let ad_blocker = library::logic(world, logic_table::AD_BLOCKER);
    let sand_island_button_logic = library::logic(world, logic_table::SAND_ISLAND_BUTTON);
    let rock_island_button_logic = library::logic(world, logic_table::ROCK_ISLAND_BUTTON);
    let graveyard_island_button_logic = library::logic(world, logic_table::GRAVEYARD_ISLAND_BUTTON);
    let ice_island_button_logic = library::logic(world, logic_table::ICE_ISLAND_BUTTON);
    let ape_island_button_logic = library::logic(world, logic_table::APE_ISLAND_BUTTON);
    let misfit_island_button_logic = library::logic(world, logic_table::MISFIT_ISLAND_BUTTON);
    let god_island_button_logic = library::logic(world, logic_table::MAP_CONTENT_GOD_ISLAND_BUTTON);
    let apocalypse_island_button_logic = library::logic(world, logic_table::APOCALYPSE_ISLAND_BUTTON);
    let underwater_button_logic = library::logic(world, logic_table::UNDERWATER_BUTTON);
    let underwater_coral_reef_button_logic = library::logic(world, logic_table::UNDERWATER_CORAL_REEF_BUTTON);

    // `mMapScreenFailBoatButtonLogic` is never assigned and stays nil.
    let mut map_screen = world.obj_mut::<MapScreen>(this);
    map_screen.sequence = Some(map_sequence);
    map_screen.map_screen_close_button_logic = Some(close_button_logic);
    map_screen.map_screen_ad_blocker_logic = Some(ad_blocker);
    map_screen.map_screen_sand_island_button_logic = Some(sand_island_button_logic);
    map_screen.map_screen_rock_island_button_logic = Some(rock_island_button_logic);
    map_screen.map_screen_graveyard_island_button_logic = Some(graveyard_island_button_logic);
    map_screen.map_screen_ice_island_button_logic = Some(ice_island_button_logic);
    map_screen.map_screen_ape_island_button_logic = Some(ape_island_button_logic);
    map_screen.map_screen_misfit_island_button_logic = Some(misfit_island_button_logic);
    map_screen.map_screen_god_island_button_logic = Some(god_island_button_logic);
    map_screen.map_screen_apocalypse_island_button_logic = Some(apocalypse_island_button_logic);
    map_screen.map_screen_underwater_button_logic = Some(underwater_button_logic);
    map_screen.map_screen_underwater_coral_reef_button_logic = Some(underwater_coral_reef_button_logic);
}

/// `-[MapScreen id]` @ 0x1000c0c80
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(CLASS_ID.to_owned())
}

/// `-[MapScreen setMapScreenListener:]` @ 0x1000c0c8c
pub fn set_map_screen_listener(world: &mut World, this: Entity, map_screen_listener: Entity) {
    world.obj_mut::<MapScreen>(this).map_screen_listener = Some(map_screen_listener);
}

/// `-[MapScreen dealloc]` @ 0x1000c0ce8
///
/// Only the controller's own `dealloc` releases the map screen, so this never runs in the game.
pub fn dealloc(world: &mut World, this: Entity) {
    let map_sequence = world.obj::<MapScreen>(this).sequence;
    if let Some(map_sequence) = map_sequence {
        sequence::stop(world, map_sequence);
        world.despawn(map_sequence);
    }

    world.despawn(this);
}

/// `-[MapScreen openMapScreen]` @ 0x1000c0d78
pub fn open_map_screen(world: &mut World, this: Entity) {
    world.obj_mut::<MapScreen>(this).map_screen_active = true;

    let map_content_display_group = library::display_group(world, MAP_CONTENT_DISPLAY_GROUP);
    set_sequence_def(world, this, OPEN_MAP_SCREEN_SEQUENCE);

    // The misfit button comes from the logic table again, not from its ivar.
    let misfit_island_button_logic = library::logic(world, logic_table::MISFIT_ISLAND_BUTTON);
    let misfit_island_button = pg_engine::logic::display_object(world, misfit_island_button_logic);

    let listener = world.obj::<MapScreen>(this).map_screen_listener;
    let misfit_island_found = match listener {
        Some(listener) => controller::has_misfit_island_been_found(world, listener),
        None => false,
    };
    {
        let mut button_object = display::object_mut(world, misfit_island_button);
        button_object.hidden = !misfit_island_found;
        button_object.is_selectable = misfit_island_found;
    }

    let map_paper = match listener {
        Some(listener) => controller::map_paper_logic(world, listener),
        None => None,
    };
    if let Some(map_paper) = map_paper {
        map_paper_logic::set_map_paper_logic_listener(world, map_paper, this);
        map_paper_logic::activate(world, map_paper);
    }

    if let Some(map_content_display_group) = map_content_display_group {
        display::object_mut(world, map_content_display_group).hidden = true;
    }

    let show_chartboost_ad = match listener {
        Some(listener) => controller::show_chartboost_ad(world, listener),
        None => false,
    };
    let ad_blocker = world.obj::<MapScreen>(this).map_screen_ad_blocker_logic;
    if let Some(ad_blocker) = ad_blocker {
        if show_chartboost_ad {
            ad_blocker_logic::activate(world, ad_blocker);
        } else {
            ad_blocker_logic::deactivate(world, ad_blocker);
        }
    }
}

/// `-[MapScreen isChartboostAdVisible]` @ 0x1000c0f70
pub fn is_chartboost_ad_visible(world: &mut World, this: Entity) -> bool {
    let listener = world.obj::<MapScreen>(this).map_screen_listener;
    match listener {
        Some(listener) => controller::is_chartboost_ad_visible(world, listener),
        None => false,
    }
}

/// `-[MapScreen closeMapScreen]` @ 0x1000c0f8c
pub fn close_map_screen(world: &mut World, this: Entity) {
    world.obj_mut::<MapScreen>(this).map_screen_active = false;
    deactivate_map_paper(world, this);
    set_sequence_def(world, this, CLICK_MAP_SCREEN_BUTTON_SEQUENCE);

    let mut map_screen = world.obj_mut::<MapScreen>(this);
    map_screen.cancel_location_state_change = true;
    map_screen.island_location_state = 0;
}

/// `-[MapScreen onMapPaperLogicShowMap]` @ 0x1000c1050
pub fn on_map_paper_logic_show_map(world: &mut World, _this: Entity) {
    load_map_texture_atlas(world);

    let map_content_display_group = library::display_group(world, MAP_CONTENT_DISPLAY_GROUP);
    if let Some(map_content_display_group) = map_content_display_group {
        display::object_mut(world, map_content_display_group).hidden = false;
    }
}

/// `-[MapScreen onMapPaperLogicHideMap]` @ 0x1000c10b8
pub fn on_map_paper_logic_hide_map(world: &mut World, this: Entity) {
    let listener = world.obj::<MapScreen>(this).map_screen_listener;
    if let Some(listener) = listener {
        controller::on_map_screen_closed(world, listener);
    }

    let current_location = match listener {
        Some(listener) => controller::island_location_state(world, listener),
        None => 0,
    };

    let (target_location, cancelled) = {
        let map_screen = world.obj::<MapScreen>(this);
        (map_screen.island_location_state, map_screen.cancel_location_state_change)
    };

    // A change between two islands keeps the map's atlas slot for the new island to fill. Staying,
    // a cancelled change, or anything involving the underwater locations reloads the current
    // location's atlases first.
    let changes_island = current_location != target_location
        && current_location != LOCATION_UNDERWATER
        && current_location != LOCATION_UNDERWATER_CORAL_REEF
        && target_location != LOCATION_UNDERWATER
        && target_location != LOCATION_UNDERWATER_CORAL_REEF
        && !cancelled;

    let mut change_location_state = true;
    if !changes_island {
        if let Some(listener) = listener {
            controller::on_load_location_texture_atlas(world, listener);
        }
        let cancelled = world.obj::<MapScreen>(this).cancel_location_state_change;
        change_location_state = !cancelled;
    }

    if change_location_state {
        let target_location = world.obj::<MapScreen>(this).island_location_state;
        if current_location != target_location {
            if let Some(listener) = listener {
                controller::on_map_screen_change_island_location_state(world, listener, target_location);
            }
        }
    }

    let map_content_display_group = library::display_group(world, MAP_CONTENT_DISPLAY_GROUP);
    if let Some(map_content_display_group) = map_content_display_group {
        display::object_mut(world, map_content_display_group).hidden = true;
    }
}

/// `-[MapScreen onMapPaperLogicAnimComplete]` @ 0x1000c11c4
pub fn on_map_paper_logic_anim_complete(world: &mut World, this: Entity) {
    set_sequence_def(world, this, CLOSE_MAP_SCREEN_SEQUENCE);
}

/// `-[MapScreen onMapCloseComplete:::]` @ 0x1000c122c
pub fn seq_on_map_close_complete(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[MapScreen changeLocation:]` @ 0x1000c1234
pub fn change_location(world: &mut World, this: Entity, change_location: i32) {
    world.obj_mut::<MapScreen>(this).map_screen_active = false;
    deactivate_map_paper(world, this);
    set_sequence_def(world, this, CLICK_MAP_SCREEN_BUTTON_SEQUENCE);

    let mut map_screen = world.obj_mut::<MapScreen>(this);
    map_screen.cancel_location_state_change = false;
    map_screen.island_location_state = change_location;
}

/// `-[MapScreen onButtonPress:touch:]` @ 0x1000c1300
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let map_screen = world.obj::<MapScreen>(this).clone();
    if !map_screen.map_screen_active {
        return;
    }

    let pressed = Some(button);

    if map_screen.map_screen_close_button_logic == pressed {
        // The body of `closeMapScreen`, written out again in the original.
        world.obj_mut::<MapScreen>(this).map_screen_active = false;
        deactivate_map_paper(world, this);
        set_sequence_def(world, this, CLICK_MAP_SCREEN_BUTTON_SEQUENCE);

        let mut map_screen = world.obj_mut::<MapScreen>(this);
        map_screen.cancel_location_state_change = true;
        map_screen.island_location_state = 0;
        return;
    }

    let location = if map_screen.map_screen_sand_island_button_logic == pressed {
        LOCATION_SAND_ISLAND
    } else if map_screen.map_screen_rock_island_button_logic == pressed {
        LOCATION_ROCK_ISLAND
    } else if map_screen.map_screen_graveyard_island_button_logic == pressed {
        LOCATION_GRAVEYARD_ISLAND
    } else if map_screen.map_screen_ice_island_button_logic == pressed {
        LOCATION_ICE_ISLAND
    } else if map_screen.map_screen_ape_island_button_logic == pressed {
        LOCATION_APE_ISLAND
    } else {
        // These four are separate tests that fall through to the underwater ones. The fail boat
        // button ivar is nil, so it never matches.
        if map_screen.map_screen_misfit_island_button_logic == pressed {
            change_location(world, this, LOCATION_MISFIT_ISLAND);
        }
        if map_screen.map_screen_god_island_button_logic == pressed {
            change_location(world, this, LOCATION_GOD_ISLAND);
        }
        if map_screen.map_screen_fail_boat_button_logic == pressed {
            change_location(world, this, LOCATION_FAIL_BOAT);
        }
        if map_screen.map_screen_apocalypse_island_button_logic == pressed {
            change_location(world, this, LOCATION_APOCALYPSE_ISLAND);
        }

        if map_screen.map_screen_underwater_button_logic == pressed {
            LOCATION_UNDERWATER
        } else if map_screen.map_screen_underwater_coral_reef_button_logic == pressed {
            LOCATION_UNDERWATER_CORAL_REEF
        } else {
            return;
        }
    };
    change_location(world, this, location);
}

/// `-[MapScreen onButtonRelease:touch:]` @ 0x1000c15b8
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[MapScreen onButtonReleaseOutside:touch:]` @ 0x1000c15bc
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `+[MapScreen instance]` @ 0x1000c0a0c
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().map_screen
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:sequence_id]]`.
fn set_sequence_def(world: &mut World, this: Entity, sequence_id: &str) {
    let map_sequence = world.obj::<MapScreen>(this).sequence;
    let sequence_def = library::sequence_def(world, sequence_id);
    if let Some(map_sequence) = map_sequence {
        sequence::set_sequence_def(world, map_sequence, sequence_def);
    }
}

/// `[[mMapScreenListener mapPaperLogic] deactivate]`: rolls the map paper up.
fn deactivate_map_paper(world: &mut World, this: Entity) {
    let listener = world.obj::<MapScreen>(this).map_screen_listener;
    let Some(listener) = listener else {
        return;
    };

    let map_paper = controller::map_paper_logic(world, listener);
    if let Some(map_paper) = map_paper {
        map_paper_logic::deactivate(world, map_paper);
    }
}

/// `FUN_100038d68(@"MapTextureAtlas0", @"raw", 0x10077b790)`: the map into the location atlas slot.
fn load_map_texture_atlas(world: &mut World) {
    let screen_atlas = world.resource::<Library>().atlas_id(SCREEN_ATLAS_ID);
    let Some(screen_atlas) = screen_atlas else {
        panic!("onMapPaperLogicShowMap: there is no atlas slot {SCREEN_ATLAS_ID}");
    };
    pg_engine::texture::load_atlas(world, screen_atlas, MAP_TEXTURE_ATLAS);
}
