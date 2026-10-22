//! `BCLibrary (Logic)`: the game's category on the engine library that creates every logic
//! object of the islands, screens and menus at start-up and connects their listeners.
//! The original hard-codes the table that `assets/data/logic-objects.json` holds
//! (`pg_assets::data::logic_objects`); slot names are in [`crate::logic_table`].
//!
//! Both methods are straight-line generated code (`docs/re/logic.md` §13). The port runs the
//! JSON table in index order, which is the order of the machine code: `createLogicClasses`
//! obtains each entry's display object, creates its logic, stores the slot and runs the
//! entry's start-up steps before the next entry; `initLogicListeners` gives each entry its
//! listener, then binds its tweenables, entry by entry.

use std::sync::OnceLock;

use pg_assets::data::logic_objects::{
    DisplayRef, InitStep, ListenerDef, ListenerTarget, LogicObjectDef, TweenableDef,
};
use pg_engine::library::Library;

use crate::logic::apocalypse_temples::apocalypse_temple_door_logic;
use crate::logic::island_core::sprite_logic;
use crate::logic::misfit_oogajump::island_object_logic;
use crate::prelude::*;

/// `-[BCLibrary(Logic) createLogicClasses]` @ 0x10013feb8
pub fn create_logic_classes(world: &mut World) {
    let defs = logic_object_defs();

    for def in defs {
        create_logic_object(world, def);
    }
}

/// `-[BCLibrary(Logic) initLogicListeners]` @ 0x1001512f8
pub fn init_logic_listeners(world: &mut World) {
    let defs = logic_object_defs();

    for def in defs {
        let logic = pg_engine::library::logic(world, def.index);

        for listener_def in &def.listeners {
            apply_listener(world, logic, listener_def);
        }

        for tweenable_def in &def.tweenables {
            apply_tweenable(world, logic, tweenable_def);
        }
    }
}

/// The table of `assets/data/logic-objects.json`, parsed once.
fn logic_object_defs() -> &'static [LogicObjectDef] {
    static DEFS: OnceLock<Vec<LogicObjectDef>> = OnceLock::new();

    let defs = DEFS.get_or_init(load_logic_object_defs);
    defs.as_slice()
}

fn load_logic_object_defs() -> Vec<LogicObjectDef> {
    match pg_assets::data::logic_objects::load() {
        Ok(defs) => defs,
        Err(error) => panic!("createLogicClasses: {error}"),
    }
}

/// One entry of `createLogicClasses`: the display object, `[[Class alloc] initWithDisplayObject:]`
/// into the slot, then the entry's start-up steps in machine-code order.
fn create_logic_object(world: &mut World, def: &LogicObjectDef) {
    let bound_object = display_ref_object(world, &def.binding);
    let display_object = match bound_object {
        Some(display_object) => display_object,
        None => nil_display_object_stand_in(world),
    };

    let logic = match def.init_selector.as_str() {
        "initWithDisplayObject:" => spawn_logic(world, &def.class, display_object),
        "initWithDisplayObject:displayMarkerId:" => {
            spawn_marker_logic(world, def, display_object)
        }
        other => panic!("createLogicClasses has no initializer {other}"),
    };

    store_logic_slot(world, def.index, logic);

    for step in &def.init_steps {
        run_init_step(world, logic, display_object, step);
    }
}

/// `[[IslandObjectLogic alloc] initWithDisplayObject:display_object displayMarkerId:marker]`
/// (entry 49, marker `MidOceanMarker`).
fn spawn_marker_logic(world: &mut World, def: &LogicObjectDef, display_object: Entity) -> Entity {
    if def.class != "IslandObjectLogic" {
        panic!("createLogicClasses has no initWithDisplayObject:displayMarkerId: for {}", def.class);
    }

    let Some(display_marker_id) = def.display_marker_id.as_deref() else {
        panic!("createLogicClasses entry {} has no display marker id", def.index);
    };

    let class = &island_object_logic::ISLAND_OBJECT_LOGIC;
    let logic = world.spawn(Isa(class)).id();
    island_object_logic::init_with_display_object(world, logic, display_object, display_marker_id);
    logic
}

/// Writes slot `index` of the `getLogic:` table (`DAT_1007e3030 + 8 · index`).
///
/// For 31 entries (the gems, gem pedestals, temple doors, fountains and the
/// `DanceDanceExecutionLogic`) the machine code stores the slot only after their first setters;
/// those setters only write ivars, so storing it right after creation is the same.
fn store_logic_slot(world: &mut World, index: usize, logic: Entity) {
    let mut library = world.resource_mut::<Library>();
    let slots = &mut library.logic_table;

    if slots.len() <= index {
        slots.resize(index + 1, Entity::PLACEHOLDER);
    }
    slots[index] = logic;
}

/// One start-up step of a table entry, sent to the new logic or written through its display
/// object's `pos` / `scale` pointer (only the components the code stores).
fn run_init_step(world: &mut World, logic: Entity, display_object: Entity, step: &InitStep) {
    match step {
        InitStep::SetPos { x, y, z } => {
            let mut object = pg_engine::display::object_mut(world, display_object);

            if let Some(x) = x {
                object.pos.x = *x;
            }
            if let Some(y) = y {
                object.pos.y = *y;
            }
            if let Some(z) = z {
                object.pos.z = *z;
            }
        }
        InitStep::SetScale { x, y, z } => {
            let mut object = pg_engine::display::object_mut(world, display_object);

            if let Some(x) = x {
                object.scale.x = *x;
            }
            if let Some(y) = y {
                object.scale.y = *y;
            }
            if let Some(z) = z {
                object.scale.z = *z;
            }
        }
        InitStep::PlayAnimation { animation_id } => {
            let animation = pg_engine::logic::animation(world, logic);
            pg_engine::animation::play_animation_id(world, animation, Some(animation_id.as_str()));
        }
        InitStep::SetBehavior { sequence_id } => {
            pg_engine::logic::set_behavior(world, logic, Some(sequence_id.as_str()));
        }
        InitStep::SetIslandLocationState { value } => {
            apocalypse_temple_door_logic::set_island_location_state(world, logic, *value);
        }
        InitStep::SetApocalypseDoorType { value } => {
            apocalypse_temple_door_logic::set_apocalypse_door_type(world, logic, *value);
        }
        InitStep::SetAlwaysOpen { value } => {
            apocalypse_temple_door_logic::set_always_open(world, logic, *value);
        }
        InitStep::SetApocalypseGemType { value } => {
            crate::send::set_apocalypse_gem_type(world, Some(logic), *value);
        }
        InitStep::SetApocalypseDiseaseType { value } => {
            crate::send::set_apocalypse_disease_type(world, Some(logic), *value);
        }
        InitStep::SetApocalypseCloudStyleId { value } => {
            sprite_logic::set_apocalypse_cloud_style_id(world, logic, value);
        }
    }
}

/// `[logic set<Name>LogicListener:[<Singleton> instance]]`.
fn apply_listener(world: &mut World, logic: Entity, listener_def: &ListenerDef) {
    let ListenerTarget::Singleton { class, .. } = &listener_def.target;
    let listener = singleton_instance(world, class);

    // `+instance` is never nil here in the original: the logic singletons are earlier table
    // entries, `loadGame` creates the managers before `initLogicListeners`, and every one of these
    // classes' `init`s writes its static. A nil here means the port of that `init` does not store
    // `Some(this)` in `Singletons`, which would silently drop every listener link to it.
    debug_assert!(listener.is_some(), "initLogicListeners: +[{class} instance] is nil");

    // The game's setters take an object, so a nil listener is not sent: the logic keeps the
    // listener it has instead of storing nil, which for a freshly created logic is the same.
    let Some(listener) = listener else {
        return;
    };

    set_listener(world, logic, &listener_def.selector, listener);
}

/// `[[logic animation] setTweenable:target channelId:channel]`.
fn apply_tweenable(world: &mut World, logic: Entity, tweenable_def: &TweenableDef) {
    let animation = pg_engine::logic::animation(world, logic);
    let tweenable = display_ref_object(world, &tweenable_def.target);
    let channel = tweenable_def.channel_id as usize;

    pg_engine::animation::set_tweenable(world, animation, tweenable, channel);
}

/// The display object a table reference names: a new `BCMultiModel`, a display group, an
/// object of a display group (nil when the group has no such child), or the `displayObject`
/// of an earlier table entry.
fn display_ref_object(world: &mut World, display_ref: &DisplayRef) -> Option<Entity> {
    match display_ref {
        DisplayRef::NewModel { class, id } => {
            if class != "BCMultiModel" {
                panic!("createLogicClasses has no display object class {class}");
            }
            let model = pg_engine::display::multi_model::spawn(world, Some(id.as_str()));
            Some(model)
        }
        DisplayRef::DisplayGroup { group } => pg_engine::library::display_group(world, group),
        DisplayRef::GroupObject { group, object } => {
            let display_group = pg_engine::library::display_group(world, group)?;
            pg_engine::display::group::get_display_object(world, display_group, object)
        }
        DisplayRef::LogicDisplayObject { index } => {
            let logic = pg_engine::library::logic(world, *index);
            let display_object = pg_engine::logic::display_object(world, logic);
            Some(display_object)
        }
    }
}

/// The display object given to a logic whose display object is nil: entry 624, `BCButtonLogic`
/// on `OpenFeintButton`, which `CommunityMenuDisplayGroup` does not have (logic.md §3.7).
///
/// PORT: the engine's `initWithDisplayObject:` needs an entity. With nil the original skips
/// `setLogic:`, so the logic and its animation stay awake; an awake object that no display group
/// holds gives the logic the same state, and neither touches nor items can ever reach it.
fn nil_display_object_stand_in(world: &mut World) -> Entity {
    let class = &pg_engine::display::BC_DISPLAY_OBJECT;
    let stand_in = pg_engine::display::spawn(world, class, None);

    pg_engine::display::object_mut(world, stand_in).hibernate = false;
    stand_in
}

/// `+[<class> instance]` for the listener singletons of `initLogicListeners`.
fn singleton_instance(world: &World, class_name: &str) -> Option<Entity> {
    match class_name {
        "ApocalypseTimeTemplePlatformLogic" => crate::logic::apocalypse_temples::apocalypse_time_temple_platform_logic::instance(world),
        "DanceDanceExecutionScreenLogic" => crate::logic::apocalypse_dance::dance_dance_execution_screen_logic::instance(world),
        "DebugScreen" => crate::managers::debug_screen::instance(world),
        "IdolManager" => crate::managers::idol_manager::instance(world),
        "IdolScreenLogic" => crate::logic::idol_screen::idol_screen_logic::instance(world),
        "MapScreen" => crate::managers::map_screen::instance(world),
        "MoronPestsQuestionScreenLogic" => crate::logic::moronpests_quiz::moron_pests_question_screen_logic::instance(world),
        "MoronSnakeLogic" => crate::logic::moronpests_chase::moron_snake_logic::instance(world),
        "NoPeeLogic" => crate::logic::nopee::no_pee_logic::instance(world),
        "PocketGodViewController" => crate::controller::instance(world),
        "TempleDoorLogic" => crate::logic::battle::temple_door_logic::instance(world),
        "UnderwaterTempleLogic" => crate::logic::underwater_stage::underwater_temple_logic::instance(world),
        _ => panic!("initLogicListeners has no singleton {class_name}"),
    }
}

/// `[[<class> alloc] initWithDisplayObject:display_object]` for the classes of the table.
/// (`IslandObjectLogic` uses `initWithDisplayObject:displayMarkerId:`, not covered here.)
pub fn spawn_logic(world: &mut World, class_name: &str, display_object: Entity) -> Entity {
    match class_name {
        "AdBlockerLogic" => crate::logic::idol_ui_widgets::ad_blocker_logic::spawn(world, display_object),
        "ApocalypseCalendarLogic" => crate::logic::apocalypse_temples::apocalypse_calendar_logic::spawn(world, display_object),
        "ApocalypseGemLogic" => crate::logic::apocalypse_temples::apocalypse_gem_logic::spawn(world, display_object),
        "ApocalypseGemPedestalLogic" => crate::logic::apocalypse_temples::apocalypse_gem_pedestal_logic::spawn(world, display_object),
        "ApocalypseRoomFaderLogic" => crate::logic::apocalypse_temples::apocalypse_room_fader_logic::spawn(world, display_object),
        "ApocalypseTempleDoorLogic" => crate::logic::apocalypse_temples::apocalypse_temple_door_logic::spawn(world, display_object),
        "ApocalypseTempleFountainLogic" => crate::logic::apocalypse_temples::apocalypse_temple_fountain_logic::spawn(world, display_object),
        "ApocalypseTimeTempleLeverLogic" => crate::logic::apocalypse_temples::apocalypse_time_temple_lever_logic::spawn(world, display_object),
        "ApocalypseTimeTemplePlatformLogic" => crate::logic::apocalypse_temples::apocalypse_time_temple_platform_logic::spawn(world, display_object),
        "BCButtonLogic" => pg_engine::logic::button::spawn(world, display_object),
        "BattleAlertLogic" => crate::logic::battle::battle_alert_logic::spawn(world, display_object),
        "BattleEnergyBallLogic" => crate::logic::battle::battle_energy_ball_logic::spawn(world, display_object),
        "ClosetTempleClosetDoorLogic" => crate::logic::apocalypse_locust_closet_xenu::closet_temple_closet_door_logic::spawn(world, display_object),
        "ClosetTempleCouchLogic" => crate::logic::apocalypse_locust_closet_xenu::closet_temple_couch_logic::spawn(world, display_object),
        "ClosetTempleThetanMeterLogic" => crate::logic::apocalypse_locust_closet_xenu::closet_temple_thetan_meter_logic::spawn(world, display_object),
        "DanceBalloonBundleLogic" => crate::logic::apocalypse_dance::dance_balloon_bundle_logic::spawn(world, display_object),
        "DanceBalloonLogic" => crate::logic::apocalypse_dance::dance_balloon_logic::spawn(world, display_object),
        "DanceDanceExecutionLogic" => crate::logic::apocalypse_dance::dance_dance_execution_logic::spawn(world, display_object),
        "DanceDanceExecutionScreenLogic" => crate::logic::apocalypse_dance::dance_dance_execution_screen_logic::spawn(world, display_object),
        "DiscoBallLogic" => crate::logic::apocalypse_dance::disco_ball_logic::spawn(world, display_object),
        "DiscoLightsLogic" => crate::logic::apocalypse_dance::disco_lights_logic::spawn(world, display_object),
        "DreamBubbleLogic" => crate::logic::misfit_oogajump::dream_bubble_logic::spawn(world, display_object),
        "EpisodeAlertLogic" => crate::logic::idol_ui_widgets::episode_alert_logic::spawn(world, display_object),
        "FailBoatLogic" => crate::logic::battle::fail_boat_logic::spawn(world, display_object),
        "GoGoCageLogic" => crate::logic::apocalypse_dance::go_go_cage_logic::spawn(world, display_object),
        "GodIdolLogic" => crate::logic::battle::god_idol_logic::spawn(world, display_object),
        "IdolBattleRealmEffectLogic" => crate::logic::idol_ui_widgets::idol_battle_realm_effect_logic::spawn(world, display_object),
        "IdolIconLogic" => crate::logic::idol_ui_widgets::idol_icon_logic::spawn(world, display_object),
        "IdolPedestalLogic" => crate::logic::battle::idol_pedestal_logic::spawn(world, display_object),
        "IdolScreenLogic" => crate::logic::idol_screen::idol_screen_logic::spawn(world, display_object),
        "IdolSelectionIconLogic" => crate::logic::idol_ui_widgets::idol_selection_icon_logic::spawn(world, display_object),
        "JellyfishLogic" => crate::logic::underwater_stage::jellyfish_logic::spawn(world, display_object),
        "LocustQueenLogic" => crate::logic::apocalypse_locust_closet_xenu::locust_queen_logic::spawn(world, display_object),
        "MoronBeeLogic" => crate::logic::moronpests_quiz::moron_bee_logic::spawn(world, display_object),
        "MoronDuckLogic" => crate::logic::moronpests_quiz::moron_duck_logic::spawn(world, display_object),
        "MoronPestsQuestionScreenLogic" => crate::logic::moronpests_quiz::moron_pests_question_screen_logic::spawn(world, display_object),
        "MoronSnakeLogic" => crate::logic::moronpests_chase::moron_snake_logic::spawn(world, display_object),
        "MoronTurtleLogic" => crate::logic::moronpests_chase::moron_turtle_logic::spawn(world, display_object),
        "MultiModelLogic" => pg_engine::logic::multi_model::spawn(world, display_object),
        "NoPeeClamLogic" => crate::logic::nopee::no_pee_clam_logic::spawn(world, display_object),
        "NoPeeFishLogic" => crate::logic::nopee::no_pee_fish_logic::spawn(world, display_object),
        "NoPeeLogic" => crate::logic::nopee::no_pee_logic::spawn(world, display_object),
        "NoPeeMapLogic" => crate::logic::nopee::no_pee_map_logic::spawn(world, display_object),
        "NoPeeMapScreenLogic" => crate::logic::nopee::no_pee_map_screen_logic::spawn(world, display_object),
        "NoPeeMineLogic" => crate::logic::nopee::no_pee_mine_logic::spawn(world, display_object),
        "NoPeeParticleLogic" => crate::logic::nopee::no_pee_particle_logic::spawn(world, display_object),
        "NoPeeSeaBlobLogic" => crate::logic::nopee::no_pee_sea_blob_logic::spawn(world, display_object),
        "NoPeeTreasureLogic" => crate::logic::nopee::no_pee_treasure_logic::spawn(world, display_object),
        "PygmyBottomLogic" => crate::pygmy_extras::pygmy_bottom_logic::spawn(world, display_object),
        "TempleDoorLogic" => crate::logic::battle::temple_door_logic::spawn(world, display_object),
        "UnderwaterTempleLogic" => crate::logic::underwater_stage::underwater_temple_logic::spawn(world, display_object),
        "UnderwaterTempleSymbolLogic" => crate::logic::underwater_stage::underwater_temple_symbol_logic::spawn(world, display_object),
        "WaterWingsLogic" => crate::logic::nopee::water_wings_logic::spawn(world, display_object),
        "XenuLogic" => crate::logic::apocalypse_locust_closet_xenu::xenu_logic::spawn_object(world, display_object),
        "XenuTempleLogic" => crate::logic::apocalypse_locust_closet_xenu::xenu_temple_logic::spawn(world, display_object),
        _ => panic!("createLogicClasses has no class {class_name}"),
    }
}

/// `[logic <selector>listener]` for the listener setters of the table.
pub fn set_listener(world: &mut World, logic: Entity, selector: &str, listener: Entity) {
    match selector {
        "setAdBlockerLogicListener:" => crate::logic::idol_ui_widgets::ad_blocker_logic::set_ad_blocker_logic_listener(world, logic, listener),
        "setApocalypseCalendarLogicListener:" => crate::logic::apocalypse_temples::apocalypse_calendar_logic::set_apocalypse_calendar_logic_listener(world, logic, listener),
        "setApocalypseGemLogicListener:" => crate::logic::apocalypse_temples::apocalypse_gem_logic::set_apocalypse_gem_logic_listener(world, logic, listener),
        "setApocalypseGemPedestalLogicListener:" => crate::logic::apocalypse_temples::apocalypse_gem_pedestal_logic::set_apocalypse_gem_pedestal_logic_listener(world, logic, listener),
        "setApocalypseRoomFaderLogicListener:" => crate::logic::apocalypse_temples::apocalypse_room_fader_logic::set_apocalypse_room_fader_logic_listener(world, logic, listener),
        "setApocalypseTempleDoorLogicListener:" => crate::logic::apocalypse_temples::apocalypse_temple_door_logic::set_apocalypse_temple_door_logic_listener(world, logic, listener),
        "setApocalypseTempleFountainLogicListener:" => crate::logic::apocalypse_temples::apocalypse_temple_fountain_logic::set_apocalypse_temple_fountain_logic_listener(world, logic, listener),
        "setApocalypseTimeTempleLeverLogicListener:" => crate::logic::apocalypse_temples::apocalypse_time_temple_lever_logic::set_apocalypse_time_temple_lever_logic_listener(world, logic, listener),
        "setApocalypseTimeTemplePlatformLogicListener:" => crate::logic::apocalypse_temples::apocalypse_time_temple_platform_logic::set_apocalypse_time_temple_platform_logic_listener(world, logic, listener),
        "setBattleAlertLogicListener:" => crate::logic::battle::battle_alert_logic::set_battle_alert_logic_listener(world, logic, listener),
        "setButtonLogicListener:" => pg_engine::logic::button::set_button_logic_listener(world, logic, Some(listener)),
        "setClosetTempleClosetDoorLogicListener:" => crate::logic::apocalypse_locust_closet_xenu::closet_temple_closet_door_logic::set_closet_temple_closet_door_logic_listener(world, logic, listener),
        "setClosetTempleCouchLogicListener:" => crate::logic::apocalypse_locust_closet_xenu::closet_temple_couch_logic::set_closet_temple_couch_logic_listener(world, logic, listener),
        "setClosetTempleThetanMeterLogicListener:" => crate::logic::apocalypse_locust_closet_xenu::closet_temple_thetan_meter_logic::set_closet_temple_thetan_meter_logic_listener(world, logic, listener),
        "setDanceBalloonBundleLogicListener:" => crate::logic::apocalypse_dance::dance_balloon_bundle_logic::set_dance_balloon_bundle_logic_listener(world, logic, listener),
        "setDanceBalloonLogicListener:" => crate::logic::apocalypse_dance::dance_balloon_logic::set_dance_balloon_logic_listener(world, logic, listener),
        "setDanceDanceExecutionLogicListener:" => crate::logic::apocalypse_dance::dance_dance_execution_logic::set_dance_dance_execution_logic_listener(world, logic, listener),
        "setDanceDanceExecutionScreenLogicListener:" => crate::logic::apocalypse_dance::dance_dance_execution_screen_logic::set_dance_dance_execution_screen_logic_listener(world, logic, listener),
        "setDiscoBallLogicListener:" => crate::logic::apocalypse_dance::disco_ball_logic::set_disco_ball_logic_listener(world, logic, listener),
        "setDiscoLightsLogicListener:" => crate::logic::apocalypse_dance::disco_lights_logic::set_disco_lights_logic_listener(world, logic, listener),
        "setDreamBubbleLogicListener:" => crate::logic::misfit_oogajump::dream_bubble_logic::set_dream_bubble_logic_listener(world, logic, listener),
        "setEpisodeAlertLogicListener:" => crate::logic::idol_ui_widgets::episode_alert_logic::set_episode_alert_logic_listener(world, logic, listener),
        "setFailBoatLogicListener:" => crate::logic::battle::fail_boat_logic::set_fail_boat_logic_listener(world, logic, listener),
        "setGoGoCageLogicListener:" => crate::logic::apocalypse_dance::go_go_cage_logic::set_go_go_cage_logic_listener(world, logic, listener),
        "setGodIdolLogicListener:" => crate::logic::battle::god_idol_logic::set_god_idol_logic_listener(world, logic, listener),
        "setIdolBattleRealmEffectLogicListener:" => crate::logic::idol_ui_widgets::idol_battle_realm_effect_logic::set_idol_battle_realm_effect_logic_listener(world, logic, listener),
        "setIdolIconLogicListener:" => crate::logic::idol_ui_widgets::idol_icon_logic::set_idol_icon_logic_listener(world, logic, listener),
        "setIdolPedestalLogicListener:" => crate::logic::battle::idol_pedestal_logic::set_idol_pedestal_logic_listener(world, logic, listener),
        "setIdolScreenLogicListener:" => crate::logic::idol_screen::idol_screen_logic::set_idol_screen_logic_listener(world, logic, listener),
        "setIdolSelectionIconLogicListener:" => crate::logic::idol_ui_widgets::idol_selection_icon_logic::set_idol_selection_icon_logic_listener(world, logic, listener),
        "setJellyfishLogicListener:" => crate::logic::underwater_stage::jellyfish_logic::set_jellyfish_logic_listener(world, logic, listener),
        "setLocustQueenLogicListener:" => crate::logic::apocalypse_locust_closet_xenu::locust_queen_logic::set_locust_queen_logic_listener(world, logic, listener),
        "setMoronBeeLogicListener:" => crate::logic::moronpests_quiz::moron_bee_logic::set_moron_bee_logic_listener(world, logic, listener),
        "setMoronDuckLogicListener:" => crate::logic::moronpests_quiz::moron_duck_logic::set_moron_duck_logic_listener(world, logic, listener),
        "setMoronPestsQuestionScreenLogicListener:" => crate::logic::moronpests_quiz::moron_pests_question_screen_logic::set_moron_pests_question_screen_logic_listener(world, logic, listener),
        "setMoronSnakeLogicListener:" => crate::logic::moronpests_chase::moron_snake_logic::set_moron_snake_logic_listener(world, logic, listener),
        "setMoronTurtleLogicListener:" => crate::logic::moronpests_chase::moron_turtle_logic::set_moron_turtle_logic_listener(world, logic, listener),
        "setMultiModelLogicListener:" => pg_engine::logic::multi_model::set_multi_model_logic_listener(world, logic, Some(listener)),
        "setNoPeeClamLogicListener:" => crate::logic::nopee::no_pee_clam_logic::set_no_pee_clam_logic_listener(world, logic, listener),
        "setNoPeeFishLogicListener:" => crate::logic::nopee::no_pee_fish_logic::set_no_pee_fish_logic_listener(world, logic, listener),
        "setNoPeeLogicListener:" => crate::logic::nopee::no_pee_logic::set_no_pee_logic_listener(world, logic, listener),
        "setNoPeeMapLogicListener:" => crate::logic::nopee::no_pee_map_logic::set_no_pee_map_logic_listener(world, logic, listener),
        "setNoPeeMapScreenLogicListener:" => crate::logic::nopee::no_pee_map_screen_logic::set_no_pee_map_screen_logic_listener(world, logic, listener),
        "setNoPeeMineLogicListener:" => crate::logic::nopee::no_pee_mine_logic::set_no_pee_mine_logic_listener(world, logic, listener),
        "setNoPeeParticleLogicListener:" => crate::logic::nopee::no_pee_particle_logic::set_no_pee_particle_logic_listener(world, logic, listener),
        "setNoPeeSeaBlobLogicListener:" => crate::logic::nopee::no_pee_sea_blob_logic::set_no_pee_sea_blob_logic_listener(world, logic, listener),
        "setNoPeeTreasureLogicListener:" => crate::logic::nopee::no_pee_treasure_logic::set_no_pee_treasure_logic_listener(world, logic, listener),
        "setPygmyBottomLogicListener:" => crate::pygmy_extras::pygmy_bottom_logic::set_pygmy_bottom_logic_listener(world, logic, listener),
        "setTempleDoorLogicListener:" => crate::logic::battle::temple_door_logic::set_temple_door_logic_listener(world, logic, listener),
        "setUnderwaterTempleLogicListener:" => crate::logic::underwater_stage::underwater_temple_logic::set_underwater_temple_logic_listener(world, logic, listener),
        "setWaterWingsLogicListener:" => crate::logic::nopee::water_wings_logic::set_water_wings_logic_listener(world, logic, listener),
        "setXenuLogicListener:" => crate::logic::apocalypse_locust_closet_xenu::xenu_logic::set_xenu_logic_listener(world, logic, listener),
        "setXenuTempleLogicListener:" => crate::logic::apocalypse_locust_closet_xenu::xenu_temple_logic::set_xenu_temple_logic_listener(world, logic, listener),
        _ => panic!("initLogicListeners has no setter {selector}"),
    }
}
