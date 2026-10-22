//! `PocketGodViewController`: methods in this part of the class.
//!
//! The apocalypse event (Xenu starts it, the seven gems back on their pedestals end it), the gems
//! the episodes spawn while it runs, and the seven temple rooms of Apocalypse island (locations
//! 11–17), which replace the island display group while the camera is inside one.

use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::sound;

use crate::logic::apocalypse_locust_closet_xenu::{closet_temple_thetan_meter_logic, xenu_logic, xenu_temple_logic};
use crate::logic::apocalypse_temples::{apocalypse_calendar_logic, apocalypse_gem_logic};
use crate::logic_table;
use crate::managers::idol_manager;
use crate::prelude::*;

use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerApocalypse {
    /// `mApocalypseGemLogicArray`: `@"NSMutableArray"`
    pub apocalypse_gem_logic_array: Vec<Entity>,

    /// `mbApocalypseActive`: `B`
    pub apocalypse_active: bool,
}

/// `-[PocketGodViewController (Game) onApocalypseGemDie:]` @ 0x10016542c
pub fn on_apocalypse_gem_die(world: &mut World, this: Entity, gem_logic: Entity) {
    let mut apocalypse = world.obj_mut::<ControllerApocalypse>(this);
    apocalypse.apocalypse_gem_logic_array.retain(|gem| *gem != gem_logic);
}

/// `-[PocketGodViewController (Game) onApocalypseGemEventTriggered:]` @ 0x10016544c
///
/// During the apocalypse, an episode asks for gem `gem_type` (1 Time … 7 Xenu): its spawn event
/// fires, and the gem (logic #403–409) appears unless it is already out.
pub fn on_apocalypse_gem_event_triggered(world: &mut World, this: Entity, gem_type: i32) {
    let apocalypse_active = world.obj::<ControllerApocalypse>(this).apocalypse_active;
    if !apocalypse_active {
        return;
    }

    let (spawn_event, gem_slot) = match gem_type {
        1 => ("SpawnTimeGem", logic_table::APOCALYPSE_GEM_TIME),
        2 => ("SpawnLocustGem", logic_table::APOCALYPSE_GEM_LOCUST),
        3 => ("SpawnDanceGem", logic_table::APOCALYPSE_GEM_DANCE),
        4 => ("SpawnPlagueGem", logic_table::APOCALYPSE_GEM_PLAGUE),
        5 => ("SpawnPurificationGem", logic_table::APOCALYPSE_GEM_PURIFICATION),
        6 => ("SpawnClosetGem", logic_table::APOCALYPSE_GEM_CLOSET),
        7 => ("SpawnXenuGem", logic_table::APOCALYPSE_GEM_XENU),
        _ => return,
    };
    trigger_episode_event(world, spawn_event);

    let gem = library::logic(world, gem_slot);
    if apocalypse_gem_logic::is_active(world, gem) {
        return;
    }
    apocalypse_gem_logic::activate(world, gem);
    world.obj_mut::<ControllerApocalypse>(this).apocalypse_gem_logic_array.push(gem);
}

/// `-[PocketGodViewController (Game) showApocalypseTempleDisplayGroup]` @ 0x1001680d0
///
/// Entering a temple room: unless its display group is already attached, it takes the island
/// group's place under `IslandRootDisplayMarker`, and the pygmies, thetans, gems and a running
/// locust swarm move into it.
pub fn show_apocalypse_temple_display_group(world: &mut World, this: Entity) {
    let (temple_display_group, temple_drag_display_marker_id) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        match locations.island_location_state {
            11 => (locations.time_temple_display_group, Some("TimeTempleDragDisplayMarker")),
            12 => (locations.locust_temple_display_group, Some("LocustTempleDragDisplayMarker")),
            13 => (locations.dance_temple_display_group, Some("DanceTempleDragDisplayMarker")),
            14 => (locations.plague_temple_display_group, Some("PlagueTempleDragDisplayMarker")),
            15 => (locations.purification_temple_display_group, Some("PurificationTempleDragDisplayMarker")),
            16 => (locations.closet_temple_display_group, Some("ClosetTempleDragDisplayMarker")),
            17 => (locations.xenu_temple_display_group, Some("XenuTempleDragDisplayMarker")),
            _ => (None, None),
        }
    };

    // Outside the temples the group is nil, and `[nil displayGroup]` is nil too: the island
    // group is then detached with nothing in its place.
    let temple_parent = match temple_display_group {
        Some(temple_display_group) => display::object(world, temple_display_group).display_group,
        None => None,
    };
    if temple_parent.is_some() {
        return;
    }

    load_location_texture_atlas(world, this);
    cancel_all_touches(world, this);

    let (island_display_group, island_root_display_marker) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.island_display_group, locations.island_root_display_marker)
    };
    if let Some(island_display_group) = island_display_group {
        display::remove_from_display_group(world, island_display_group);
    }
    if let Some(temple_display_group) = temple_display_group {
        display::move_to_display_group(world, temple_display_group, island_root_display_marker);
    }

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        crate::pygmy::move_to_apocalypse_temple_display_group(world, pygmy);
    }

    drop_apocalypse_thetans_into_temple_room(world, this);
    refresh_apocalypse_gem_display_markers(world, this);

    if is_locust_active(world, this) {
        let drag_display_marker = match temple_drag_display_marker_id {
            Some(temple_drag_display_marker_id) => library::display_marker(world, temple_drag_display_marker_id),
            None => None,
        };
        move_locust_after(world, this, drag_display_marker);
    }
}

/// `-[PocketGodViewController (Game) hideApocalypseTempleDisplayGroup]` @ 0x1001684e8
///
/// Leaving a temple room (the last location): if its display group is attached, the island group
/// comes back under `IslandRootDisplayMarker`, and the pygmies, thetans, gems and a running locust
/// swarm move back out.
pub fn hide_apocalypse_temple_display_group(world: &mut World, this: Entity) {
    let temple_display_group = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        match locations.last_island_location_state {
            11 => locations.time_temple_display_group,
            12 => locations.locust_temple_display_group,
            13 => locations.dance_temple_display_group,
            14 => locations.plague_temple_display_group,
            15 => locations.purification_temple_display_group,
            16 => locations.closet_temple_display_group,
            17 => locations.xenu_temple_display_group,
            _ => None,
        }
    };

    let Some(temple_display_group) = temple_display_group else {
        return;
    };
    let temple_parent = display::object(world, temple_display_group).display_group;
    if temple_parent.is_none() {
        return;
    }

    load_location_texture_atlas(world, this);
    display::remove_from_display_group(world, temple_display_group);

    let (island_display_group, island_root_display_marker) = {
        let locations = world.obj::<ControllerIslandLocations>(this);
        (locations.island_display_group, locations.island_root_display_marker)
    };
    if let Some(island_display_group) = island_display_group {
        display::move_to_display_group(world, island_display_group, island_root_display_marker);
    }

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        crate::pygmy::move_from_apocalypse_temple_display_group(world, pygmy);
    }

    drop_apocalypse_thetans_into_temple_room(world, this);
    refresh_apocalypse_gem_display_markers(world, this);

    if is_locust_active(world, this) {
        let drag_display_marker = library::display_marker(world, "DragDisplayMarker");
        move_locust_after(world, this, drag_display_marker);
    }
}

/// `-[PocketGodViewController (Game) isApocalypseActive]` @ 0x100168884
pub fn is_apocalypse_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerApocalypse>(this).apocalypse_active
}

/// `-[PocketGodViewController (Game) startApocalypse]` @ 0x100168898
///
/// Xenu starts the apocalypse: the sky turns, the thetans spawn and `ApocalypseLoop` replaces the
/// island's background music.
pub fn start_apocalypse(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerApocalypse>(this).apocalypse_active = true;
    world.obj_mut::<ControllerSkyWeather>(this).apocalypse_gradient_target_fade = 1.0;

    let thetan_meter = world.obj::<ControllerWorldRegistry>(this).closet_temple_thetan_meter_logic;
    if let Some(thetan_meter) = thetan_meter {
        closet_temple_thetan_meter_logic::spawn_apocalypse_thetans(world, thetan_meter);
    }

    sound::stop_sound(world, "BackgroundLoop");
    sound::play_sound(world, "ApocalypseLoop");
    trigger_episode_event(world, "StartApocalypse");
}

/// `-[PocketGodViewController (Game) endApocalypse]` @ 0x100168998
///
/// The last gem is on its pedestal: Xenu is banished, his temple wall rebuilt, the calendar's
/// portal closes and the thetans go. The calendar calls `endApocalypseComplete` afterwards.
pub fn end_apocalypse(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerApocalypse>(this).apocalypse_active = false;

    let gems = world.obj::<ControllerApocalypse>(this).apocalypse_gem_logic_array.clone();
    for gem in gems {
        apocalypse_gem_logic::prepare_to_end_apocalypse(world, gem);
    }

    let xenu = world.obj::<ControllerWorldRegistry>(this).xenu_logic;
    if let Some(xenu) = xenu {
        xenu_logic::banish(world, xenu);
    }

    let xenu_temple = world.obj::<ControllerWorldRegistry>(this).xenu_temple_logic;
    if let Some(xenu_temple) = xenu_temple {
        xenu_temple_logic::build_wall_back_up(world, xenu_temple);
    }

    let apocalypse_calendar = world.obj::<ControllerWorldRegistry>(this).apocalypse_calendar_logic;
    if let Some(apocalypse_calendar) = apocalypse_calendar {
        apocalypse_calendar_logic::close_apocalypse_portal(world, apocalypse_calendar);
    }

    let thetan_meter = world.obj::<ControllerWorldRegistry>(this).closet_temple_thetan_meter_logic;
    if let Some(thetan_meter) = thetan_meter {
        closet_temple_thetan_meter_logic::remove_apocalypse_thetans(world, thetan_meter);
    }
}

/// `-[PocketGodViewController (Game) endApocalypseComplete]` @ 0x100168b44
///
/// The sky clears and the island's background music returns.
pub fn end_apocalypse_complete(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerSkyWeather>(this).apocalypse_gradient_target_fade = 0.0;

    sound::stop_sound(world, "ApocalypseLoop");
    sound::play_sound(world, "BackgroundLoop");
    trigger_episode_event(world, "EndApocalypse");
}

/// `-[PocketGodViewController (Game) endApocalypseCompleteNow]` @ 0x100168c0c
///
/// The same body as `endApocalypseComplete`.
pub fn end_apocalypse_complete_now(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerSkyWeather>(this).apocalypse_gradient_target_fade = 0.0;

    sound::stop_sound(world, "ApocalypseLoop");
    sound::play_sound(world, "BackgroundLoop");
    trigger_episode_event(world, "EndApocalypse");
}

/// `-[PocketGodViewController (Game) doesPlagueBodyExist]` @ 0x100168cd4
pub fn does_plague_body_exist(world: &mut World, this: Entity) -> bool {
    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        if crate::pygmy::is_killed_by_plague(world, pygmy) {
            return true;
        }
    }
    false
}

/// `[[IdolManager instance] triggerEpisodeEvent:event]`.
fn trigger_episode_event(world: &mut World, event: &str) {
    let idol_manager = idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        idol_manager::trigger_episode_event(world, idol_manager, event);
    }
}

/// During the apocalypse, `[mClosetTempleThetanMeterLogic
/// dropApocalypseThetansIntoApoclaypseTempleRoom]`.
fn drop_apocalypse_thetans_into_temple_room(world: &mut World, this: Entity) {
    let apocalypse_active = world.obj::<ControllerApocalypse>(this).apocalypse_active;
    if !apocalypse_active {
        return;
    }

    let thetan_meter = world.obj::<ControllerWorldRegistry>(this).closet_temple_thetan_meter_logic;
    if let Some(thetan_meter) = thetan_meter {
        closet_temple_thetan_meter_logic::drop_apocalypse_thetans_into_apoclaypse_temple_room(world, thetan_meter);
    }
}

/// `refreshDisplayMarker` on every gem of `mApocalypseGemLogicArray`.
fn refresh_apocalypse_gem_display_markers(world: &mut World, this: Entity) {
    let gems = world.obj::<ControllerApocalypse>(this).apocalypse_gem_logic_array.clone();
    for gem in gems {
        apocalypse_gem_logic::refresh_display_marker(world, gem);
    }
}

/// `[marker insertAfter:[mLocustLogic displayObject]]`: the locust swarm moves just after the
/// marker (nothing for a nil marker).
fn move_locust_after(world: &mut World, this: Entity, marker: Option<Entity>) {
    let locust = world.obj::<ControllerGodPowers>(this).locust_logic;
    let locust_display_object = match locust {
        Some(locust) => Some(logic::display_object(world, locust)),
        None => None,
    };
    if let Some(marker) = marker {
        display::insert_after(world, marker, locust_display_object);
    }
}
