//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::island_core::sprite_logic::SpriteLogic;
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::view::BcView;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyPygmyBehaviourItems {
    /// `mTumblePos`: `{?="x"f"y"f"z"f"w"f}`
    pub tumble_pos: Vec4,

    /// `mNextTurnStallTime`: `f`
    pub next_turn_stall_time: f32,

    /// `mBurningMagnifyingGlassLogic`: `@"MagnifyingGlassLogic"`
    pub burning_magnifying_glass_logic: Option<Entity>,

    /// `mbHitIceMonster`: `B`
    pub hit_ice_monster: bool,

    /// `mAshPileLogic`: `@"BCLogic"`
    pub ash_pile_logic: Option<Entity>,

    /// `mbEnableSlide`: `B`
    pub enable_slide: bool,

    /// `mbIslandFloorCollision`: `B`
    pub island_floor_collision: bool,

    /// `mStartFallYPos`: `f`
    pub start_fall_y_pos: f32,

    /// `mbBrainsEaten`: `B`
    pub brains_eaten: bool,

    /// `mbMouthFilledByLocusts`: `B`
    pub mouth_filled_by_locusts: bool,

    /// `mbTellClosetToGrabNewTom`: `B`
    pub tell_closet_to_grab_new_tom: bool,

    /// `mbHadHeartAttack`: `B`
    pub had_heart_attack: bool,

    /// `mbIglooCannon`: `B`
    pub igloo_cannon: bool,

    /// `mbTornApartByApeAltar`: `B`
    pub torn_apart_by_ape_altar: bool,

    /// `mbFartherThanIceMonster`: `B`
    pub farther_than_ice_monster: bool,

    /// `mbDescending`: `B`
    pub descending: bool,

    /// `mbHitVolcano`: `B`
    pub hit_volcano: bool,

    /// `mbScreamDuringFall`: `B`
    pub scream_during_fall: bool,
}

/// `PygmyLogic`'s file-scope globals used by this part.
#[derive(Resource, Default)]
pub struct PygmyPygmyBehaviourItemsStatics {
    /// `DAT_100816200`: the dialog dictionary of `checkPygmyDialog:::` (dialog id → the
    /// `CFAbsoluteTimeGetCurrent()` it started at). `initWithDisplayObject:` creates it once per
    /// process; this resource is created on first use instead.
    pub dialog_start_times: HashMap<String, f64>,

    /// The game time [`absolute_time_get_current`] read last.
    pub clock_last_view_time: f32,

    /// The game time that passed before the last render-loop restart.
    pub clock_elapsed_before_restart: f64,
}

/// What [`absolute_time_get_current`] reads at game time zero. The original's clock (seconds
/// since 2001) is never near zero, which `checkPygmyDialog:::` relies on when its dictionary is
/// empty (the newest start time is then 0).
const ABSOLUTE_TIME_ORIGIN: f64 = 1.0e9;

/// `checkForTigerBlood` is first (Charlie only); then these interrupt `walk:::`, in order.
const WALK_INTERACTIONS: [fn(&mut World, Entity) -> bool; 11] = [
    check_for_coconut,
    check_for_igloo,
    check_for_wood,
    check_for_plague_body,
    check_for_out_house,
    check_for_bon_fire,
    check_for_fish,
    check_for_bird,
    check_for_banana,
    check_for_fishing_pole,
    check_for_shark,
];

/// The containers that can catch a pygmy during `fall:::`, in the order they are asked.
const FALL_CATCHERS: [fn(&mut World, Entity) -> bool; 20] = [
    check_to_absorb_pygmy_during_fall,
    check_for_kong_statue_during_fall,
    check_for_ape_wench_during_fall,
    check_for_ape_bongo_during_fall,
    check_for_ape_altar_during_fall,
    check_for_go_go_cage_during_fall,
    check_for_dance_dance_execution_during_fall,
    check_for_igloo_during_fall,
    check_for_zombie_grave_during_fall,
    check_for_idol_pedestal_during_fall,
    check_for_temple_door_during_fall,
    check_for_apocalypse_temple_door_during_fall,
    check_for_apocalypse_time_temple_platform_during_fall,
    check_for_locust_queen_during_fall,
    check_for_apocalypse_temple_fountain_during_fall,
    check_for_closet_temple_thetan_meter_during_fall,
    check_for_closet_temple_couch_during_fall,
    check_for_closet_temple_closet_door_during_fall,
    check_for_tom_pygmy_during_fall,
    check_for_xenu_during_fall,
];

/// `mPygmyLogicListener`: always the view controller (docs/re/pygmy.md §23).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `[listener isOceanEnabled]` (NO for a nil listener).
fn is_ocean_enabled(world: &mut World, listener: Option<Entity>) -> bool {
    match listener {
        Some(listener) => crate::controller::is_ocean_enabled(world, listener),
        None => false,
    }
}

/// `[listener isEruptionActive]` (NO for a nil listener).
fn is_eruption_active(world: &mut World, listener: Option<Entity>) -> bool {
    match listener {
        Some(listener) => crate::controller::is_eruption_active(world, listener),
        None => false,
    }
}

/// `[listener isIceMonsterEnabled]` (NO for a nil listener).
fn is_ice_monster_enabled(world: &mut World, listener: Option<Entity>) -> bool {
    match listener {
        Some(listener) => crate::controller::is_ice_monster_enabled(world, listener),
        None => false,
    }
}

/// `[listener iceMonsterLogic:self]` (nil for a nil listener).
fn ice_monster_of(world: &mut World, listener: Option<Entity>, this: Entity) -> Option<Entity> {
    match listener {
        Some(listener) => crate::controller::ice_monster_logic(world, listener, this),
        None => None,
    }
}

/// `[listener onPygmySacrificeEvent:sacrificeId]`.
fn notify_sacrifice_event(world: &mut World, listener: Option<Entity>, sacrifice_id: &str) {
    if let Some(listener) = listener {
        crate::controller::on_pygmy_sacrifice_event(world, listener, sacrifice_id);
    }
}

/// `CFAbsoluteTimeGetCurrent()`.
// PORT: the engine has no wall clock (and `std::time` panics on the web), so this reads the game
// time, carried across render-loop restarts and offset by `ABSOLUTE_TIME_ORIGIN`. Unlike the
// original's clock it stands still while the game is paused.
fn absolute_time_get_current(world: &mut World) -> f64 {
    let view_time = match world.get_resource::<BcView>() {
        Some(view) => view.time,
        None => 0.0,
    };

    let mut statics = world.get_resource_or_insert_with(PygmyPygmyBehaviourItemsStatics::default);
    if view_time < statics.clock_last_view_time {
        // `startRenderLoop` restarted the game time.
        let elapsed = f64::from(statics.clock_last_view_time);
        statics.clock_elapsed_before_restart += elapsed;
    }
    statics.clock_last_view_time = view_time;

    let elapsed = statics.clock_elapsed_before_restart + f64::from(view_time);
    ABSOLUTE_TIME_ORIGIN + elapsed
}

/// The parameter for disease `D` of a selection item: `purification` when D is 2, `plague` when
/// it is 1, nil otherwise (docs/re/pygmy.md §7.4).
fn disease_variant<'a>(call: &'a SequenceCall, disease: i32, purification: &str, plague: &str) -> Option<&'a str> {
    match disease {
        2 => call.param(purification),
        1 => call.param(plague),
        _ => None,
    }
}

/// The sequence id shared by `pushSequenceBasedOnPersonality:::` and
/// `runSequenceBasedOnPersonality:::`: the pygmy type's variant, then the disease's, then the
/// personality's, then `defaultId`.
fn sequence_based_on_personality<'a>(world: &World, this: Entity, call: &'a SequenceCall) -> Option<&'a str> {
    let (pygmy_type, personality) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.time_temple_pygmy_type, core.persona_personality)
    };
    let disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;

    let mut sequence_id = match pygmy_type {
        2 => {
            let variant = disease_variant(call, disease, "babyPurificationId", "babyPlagueId");
            variant.or(call.param("babyId"))
        }
        1 => {
            let variant = disease_variant(call, disease, "oldFartPurificationId", "oldFartPlagueId");
            variant.or(call.param("oldFartId"))
        }
        3 => {
            let variant = disease_variant(call, disease, "gingerPurificationId", "gingerPlagueId");
            variant.or(call.param("gingerId"))
        }
        4 => {
            let variant = disease_variant(call, disease, "fastPurificationId", "fastPlagueId");
            variant.or(call.param("fastId"))
        }
        _ => None,
    };
    if sequence_id.is_none() {
        sequence_id = disease_variant(call, disease, "purificationId", "plagueId");
    }
    if sequence_id.is_none() {
        sequence_id = match personality {
            2 => call.param("tomId"),
            1 => call.param("charlieId"),
            _ => None,
        };
    }
    if sequence_id.is_none() {
        sequence_id = call.param("defaultId");
    }
    sequence_id
}

/// The ice monster's hit window of `tumble:::`: within 50 of its x, within `radius` of
/// `collision_y` on screen, and between 100 and 300 in front of it.
fn is_inside_ice_monster_window(pos: Vec4, monster_pos: Vec4, screen_y: f32, collision_y: f32, radius: f32) -> bool {
    let x_distance = (pos.x - monster_pos.x).abs();
    if !(x_distance < 50.0) {
        return false;
    }

    let y_distance = (screen_y - collision_y).abs();
    if !(y_distance < radius) {
        return false;
    }

    let far_z = monster_pos.z + 300.0;
    let near_z = monster_pos.z + 100.0;
    pos.z < far_z && pos.z > near_z
}

/// `-[PygmyLogic triggerActiveSacrifice]` @ 0x100128c8c
pub fn trigger_active_sacrifice(world: &mut World, this: Entity) {
    let (listener, sacrifice_id) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.pygmy_logic_listener, core.sacrifice_id.clone())
    };
    let Some(sacrifice_id) = sacrifice_id else {
        return;
    };

    notify_sacrifice_event(world, listener, &sacrifice_id);

    let current_sacrifice_id = world.obj::<PygmyPygmyCoreFrame>(this).sacrifice_id.clone();
    if let Some(listener) = listener {
        crate::controller::on_trigger_sacrifice(world, listener, this, current_sacrifice_id.as_deref());
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("DroppedDrown".to_owned());
}

/// `-[PygmyLogic disableLabels:::]` @ 0x10012ea44
pub fn seq_disable_labels(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).disable_labels = true;
    ItemResult::Done
}

/// `-[PygmyLogic flipXScale:::]` @ 0x10012ea5c
pub fn seq_flip_x_scale(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.scale.x = -object.scale.x;
    ItemResult::Done
}

/// `-[PygmyLogic idle:::]` @ 0x10012eb58
pub fn seq_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    if personality == 1 && check_for_tiger_blood(world, this) {
        return ItemResult::Replaced;
    }
    if check_for_shark(world, this) {
        return ItemResult::Replaced;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        return ItemResult::Done;
    }
    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[PygmyLogic die:::]` @ 0x10012ecb8
pub fn seq_die(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);

    let tumble = world.obj::<PygmyPygmyCoreFrame>(this).tumble;
    if tumble {
        if let Some(listener) = listener {
            crate::controller::on_pygmy_tumble_complete(world, listener, this);
        }
    }
    let earthquake = world.obj::<PygmyPygmyCoreFrame>(this).earthquake;
    if earthquake {
        if let Some(listener) = listener {
            crate::controller::on_pygmy_earthquake_stop(world, listener, this);
        }
    }
    let konkey_dong = world.obj::<PygmyPygmyTouchDrag>(this).konkey_dong;
    if konkey_dong {
        let kong_statue = match listener {
            Some(listener) => crate::controller::kong_statue_logic(world, listener, this),
            None => None,
        };
        if let Some(kong_statue) = kong_statue {
            crate::logic::konkeydong::kong_statue_logic::stop_konkey_dong(world, kong_statue);
        }
    }

    crate::logic::island_core::sprite_logic::cure_disease(world, this);
    stop_warming_by_bon_fire(world, this);

    release_containers_on_death(world, this, listener);
    stop_activities_on_death(world, this);

    let shark = match listener {
        Some(listener) => crate::controller::shark_logic(world, listener, this),
        None => None,
    };
    if let Some(shark) = shark {
        crate::logic::ocean_creatures::shark_logic::cant_catch_pygmy(world, shark, this);
    }

    resolve_death_sacrifice(world, this, listener);

    let dragging = world.obj::<PygmyPygmyCoreFrame>(this).dragging;
    if dragging {
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            core.dragging = false;
            core.apply_spring = false;
        }
        let display_object = pg_engine::logic::display_object(world, this);
        display::object_mut(world, display_object).rot.z = 0.0;
        if let Some(listener) = listener {
            crate::controller::on_pygmy_drag_complete(world, listener, this);
        }
    }
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.touch_began_inside = false;
        core.active_touch = None;
    }

    if can_become_ghost(world, this, listener) {
        become_ghost(world, this, listener);
        return ItemResult::Replaced;
    }

    die_for_good(world, this, listener);
    ItemResult::Done
}

/// `die:::` step 3: every container, catcher and carrier lets go of the pygmy (each reference is
/// cleared after the call, except the ice hole's).
fn release_containers_on_death(world: &mut World, this: Entity, listener: Option<Entity>) {
    let waiting_out_house = world.obj::<PygmyIslandLife>(this).waiting_out_house_logic;
    if let Some(out_house) = waiting_out_house {
        crate::logic::island_objects::out_house_logic::stop_waiting_to_crap(world, out_house, this);
        world.obj_mut::<PygmyIslandLife>(this).waiting_out_house_logic = None;
    }

    let zombie_grave = world.obj::<PygmyContainersAndCatchers>(this).zombie_grave_logic;
    if let Some(zombie_grave) = zombie_grave {
        crate::logic::monsters::zombie_grave_logic::eject_from_grave(world, zombie_grave, this);
        world.obj_mut::<PygmyContainersAndCatchers>(this).zombie_grave_logic = None;
    }

    let idol_pedestal = world.obj::<PygmyContainersAndCatchers>(this).idol_pedestal_logic;
    if let Some(idol_pedestal) = idol_pedestal {
        crate::logic::battle::idol_pedestal_logic::eject_from_idol_pedestal(world, idol_pedestal, this);
        world.obj_mut::<PygmyContainersAndCatchers>(this).idol_pedestal_logic = None;
    }

    let temple_door = world.obj::<PygmyContainersAndCatchers>(this).temple_door_logic;
    if let Some(temple_door) = temple_door {
        crate::logic::battle::temple_door_logic::eject_from_temple_door(world, temple_door, this);
        world.obj_mut::<PygmyContainersAndCatchers>(this).temple_door_logic = None;
    }

    let apocalypse_temple_door = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic;
    if let Some(apocalypse_temple_door) = apocalypse_temple_door {
        crate::logic::apocalypse_temples::apocalypse_temple_door_logic::eject_from_apocalypse_temple_door(
            world,
            apocalypse_temple_door,
            this,
        );
        world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_door_logic = None;
    }

    let time_temple_platform = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_time_temple_platform_logic;
    if let Some(time_temple_platform) = time_temple_platform {
        crate::logic::apocalypse_temples::apocalypse_time_temple_platform_logic::eject_from_apocalypse_time_temple_platform(
            world,
            time_temple_platform,
            this,
        );
        world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_time_temple_platform_logic = None;
    }

    let locust_queen = world.obj::<PygmyContainersAndCatchers>(this).locust_queen_logic;
    if let Some(locust_queen) = locust_queen {
        crate::logic::apocalypse_locust_closet_xenu::locust_queen_logic::rescued_from_locust_queen(world, locust_queen, this);
        world.obj_mut::<PygmyContainersAndCatchers>(this).locust_queen_logic = None;
    }

    let fountain = world.obj::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic;
    if let Some(fountain) = fountain {
        crate::logic::apocalypse_temples::apocalypse_temple_fountain_logic::eject_from_apocalypse_temple_fountain(
            world, fountain, this,
        );
        world.obj_mut::<PygmyContainersAndCatchers>(this).apocalypse_temple_fountain_logic = None;
    }

    let thetan = world.obj::<PygmyApocalypse>(this).thetan_logic;
    if let Some(thetan) = thetan {
        crate::logic::apocalypse_locust_closet_xenu::thetan_logic::stop_possessing_pygmy(world, thetan, this);
        world.obj_mut::<PygmyApocalypse>(this).thetan_logic = None;
    }

    let thetan_attacker = world.obj::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic;
    if let Some(thetan_attacker) = thetan_attacker {
        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 16);
        stop_thetan_brawl_by_pygmy(world, thetan_attacker, this);
        world.obj_mut::<PygmyApocalypse>(this).thetan_attacker_pygmy_logic = None;
    }

    let thetan_defender = world.obj::<PygmyApocalypse>(this).thetan_defender_pygmy_logic;
    if let Some(thetan_defender) = thetan_defender {
        stop_thetan_brawl_by_pygmy(world, thetan_defender, this);
        world.obj_mut::<PygmyApocalypse>(this).thetan_defender_pygmy_logic = None;
    }

    let thetan_meter = world.obj::<PygmyApocalypse>(this).closet_temple_thetan_meter_logic;
    if let Some(thetan_meter) = thetan_meter {
        crate::logic::apocalypse_locust_closet_xenu::closet_temple_thetan_meter_logic::eject_from_closet_temple_thetan_meter(
            world,
            thetan_meter,
            this,
        );
        world.obj_mut::<PygmyApocalypse>(this).closet_temple_thetan_meter_logic = None;
    }

    let couch = world.obj::<PygmyApocalypse>(this).closet_temple_couch_logic;
    if let Some(couch) = couch {
        crate::logic::apocalypse_locust_closet_xenu::closet_temple_couch_logic::eject_from_closet_temple_couch(
            world, couch, this,
        );
        world.obj_mut::<PygmyApocalypse>(this).closet_temple_couch_logic = None;
    }

    let closet_door = world.obj::<PygmyApocalypse>(this).closet_temple_closet_door_logic;
    if let Some(closet_door) = closet_door {
        crate::logic::apocalypse_locust_closet_xenu::closet_temple_closet_door_logic::eject_from_closet_temple_closet_door(
            world,
            closet_door,
            this,
        );
        world.obj_mut::<PygmyApocalypse>(this).closet_temple_closet_door_logic = None;
    }

    let xenu = world.obj::<PygmyApocalypse>(this).xenu_logic;
    if let Some(xenu) = xenu {
        crate::logic::apocalypse_locust_closet_xenu::xenu_logic::eject_from_xenu(world, xenu, this);
        world.obj_mut::<PygmyApocalypse>(this).xenu_logic = None;
    }

    let tom = world.obj::<PygmyApocalypse>(this).tom_pygmy_logic;
    if let Some(tom) = tom {
        eject_from_toms_arms_with_arg(world, tom, this);
        world.obj_mut::<PygmyApocalypse>(this).tom_pygmy_logic = None;
    }

    let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
    if let Some(out_house) = crapping_out_house {
        crate::logic::island_objects::out_house_logic::stop_crapping(world, out_house, this);
        world.obj_mut::<PygmyIslandLife>(this).crapping_out_house_logic = None;
    }

    let magnifying_glass = world.obj::<PygmyPygmyBehaviourItems>(this).burning_magnifying_glass_logic;
    if let Some(magnifying_glass) = magnifying_glass {
        crate::logic::island_anthill::magnifying_glass_logic::on_pygmy_die(world, magnifying_glass, this);
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).burning_magnifying_glass_logic = None;
    }

    release_ash_pile(world, this);

    let fishing_pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if let Some(fishing_pole) = fishing_pole {
        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 2);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 3);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, 4);
        crate::logic::ocean_fishing::fishing_pole_logic::die(world, fishing_pole);
        world.obj_mut::<PygmyIslandLife>(this).fishing_pole_logic = None;
    }

    let ghost = world.obj::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic;
    if let Some(ghost) = ghost {
        cancel_ghost_hold(world, ghost, this);
        world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_pygmy_logic = None;
    }

    let ghost_touch = world.obj::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic;
    if let Some(ghost_touch) = ghost_touch {
        cancel_ghost_touch_dead(world, ghost_touch, this);
        world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_touch_pygmy_logic = None;
    }

    let jellyfish = world.obj::<PygmyUnderwaterNopee>(this).jellyfish_logic;
    if let Some(jellyfish) = jellyfish {
        crate::logic::underwater_stage::jellyfish_logic::cancel_jellyfish_attack(world, jellyfish, this);
        world.obj_mut::<PygmyUnderwaterNopee>(this).jellyfish_logic = None;
    }

    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        world.obj_mut::<SpriteLogic>(this).picked_up_by_hurricane_logic = None;
    }

    let trapped_in_spider_web = world.obj::<PygmyPygmyCoreFrame>(this).trapped_in_spider_web;
    if trapped_in_spider_web {
        let spider_tree = match listener {
            Some(listener) => crate::controller::spider_tree_logic(world, listener, this),
            None => None,
        };
        if let Some(spider_tree) = spider_tree {
            crate::logic::monsters::spider_tree_logic::eject_from_web(world, spider_tree, this);
        }
    }

    let sucked_into_drain = world.obj::<PygmyPygmyCoreFrame>(this).sucked_into_drain;
    if sucked_into_drain {
        let tile_drain = match listener {
            Some(listener) => crate::controller::tile_drain_logic(world, listener, this),
            None => None,
        };
        if let Some(tile_drain) = tile_drain {
            crate::logic::underwater_paindrain::tile_drain_logic::escape_from_drain(world, tile_drain, this);
        }
    }

    let kong_statue = world.obj::<PygmyPygmyCoreFrame>(this).kong_statue_logic;
    if let Some(kong_statue) = kong_statue {
        crate::logic::konkeydong::kong_statue_logic::remove_pygmy_from_kong_statue(world, kong_statue, this);
        world.obj_mut::<PygmyPygmyCoreFrame>(this).kong_statue_logic = None;
    }

    let ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).ape_wench_logic;
    if let Some(ape_wench) = ape_wench {
        crate::logic::ape_ritual::ape_wench_logic::remove_pygmy_from_ape_wench(world, ape_wench, this);
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_wench_logic = None;
    }

    let ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).ape_bongo_logic;
    if let Some(ape_bongo) = ape_bongo {
        crate::logic::ape_ritual::ape_bongo_logic::remove_pygmy_from_ape_bongo(world, ape_bongo, this);
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_bongo_logic = None;
    }

    let ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).ape_altar_logic;
    if let Some(ape_altar) = ape_altar {
        crate::logic::ape_ritual::ape_altar_logic::remove_pygmy_from_ape_altar(world, ape_altar, this);
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_altar_logic = None;
    }

    let go_go_cage = world.obj::<PygmyContainersAndCatchers>(this).go_go_cage_logic;
    if let Some(go_go_cage) = go_go_cage {
        crate::logic::apocalypse_dance::go_go_cage_logic::remove_pygmy_from_go_go_cage(world, go_go_cage, this);
        world.obj_mut::<PygmyContainersAndCatchers>(this).go_go_cage_logic = None;
    }

    let dance_dance_execution = world.obj::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic;
    if let Some(dance_dance_execution) = dance_dance_execution {
        crate::logic::apocalypse_dance::dance_dance_execution_logic::remove_pygmy_from_dance_dance_execution(
            world,
            dance_dance_execution,
            this,
        );
        world.obj_mut::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic = None;
    }

    let igloo = world.obj::<PygmyContainersAndCatchers>(this).igloo_logic;
    if let Some(igloo) = igloo {
        crate::logic::ice_island::igloo_logic::remove_pygmy_from_igloo(world, igloo, this);
        world.obj_mut::<PygmyContainersAndCatchers>(this).igloo_logic = None;
    }

    let ice_hole = world.obj::<PygmyIslandHazards>(this).ice_hole_logic;
    if let Some(ice_hole) = ice_hole {
        crate::logic::ice_island::ice_hole_logic::remove_pygmy_from_ice_hole(world, ice_hole, this);
    }

    let carrying_bird = world.obj::<PygmyIslandHazards>(this).carrying_bird_logic;
    if let Some(carrying_bird) = carrying_bird {
        crate::logic::ocean_birds::bird_logic::drop_pygmy_logic(world, carrying_bird, this);
        world.obj_mut::<PygmyIslandHazards>(this).carrying_bird_logic = None;
    }
}

/// The ash pile of `explode:::` leaves the display tree; this was the last owner of its logic.
fn release_ash_pile(world: &mut World, this: Entity) {
    let ash_pile_display_object = world.obj::<PygmyPygmyCoreFrame>(this).ash_pile_display_object;
    let Some(ash_pile_display_object) = ash_pile_display_object else {
        return;
    };

    display::remove_from_display_group(world, ash_pile_display_object);
    let ash_pile_logic = world.obj::<PygmyPygmyBehaviourItems>(this).ash_pile_logic;
    if let Some(ash_pile_logic) = ash_pile_logic {
        pg_engine::logic::shutdown(world, ash_pile_logic);
        pg_engine::logic::dealloc(world, ash_pile_logic);
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).ash_pile_display_object = None;
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).ash_pile_logic = None;
}

/// `die:::` step 4: the stop list (docs/re/pygmy.md §1.3, SDIE), in the original's order.
fn stop_activities_on_death(world: &mut World, this: Entity) {
    stop_sleeping(world, this);
    stop_being_eaten(world, this);
    stop_being_eaten_by_piranha(world, this);
    stop_holding_chum(world, this);
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_throwing_spear(world, this);
    stop_being_trapped_in_zombie_grave(world, this);
    stop_being_trapped_in_idol_pedestal(world, this);
    stop_being_inside_temple_door(world, this);
    stop_being_inside_apocalypse_temple_door(world, this);
    stop_being_on_top_apocalypse_time_temple_platform(world, this);
    stop_being_in_apocalypse_temple_fountain(world, this);
    stop_thetan_brawl(world, this);
    stop_being_on_closet_temple_thetan_meter(world, this);
    stop_being_on_closet_temple_couch(world, this);
    stop_being_in_closet_temple_closet_door(world, this);
    stop_slamming_closet_temple_closet_door(world, this);
    stop_being_held_by_xenu(world, this);
    stop_being_held_in_toms_arms(world, this);
    stop_being_possessed_by_thetan(world, this);
    stop_holding_baby_pygmy(world, this);
    stop_being_eaten_by_locust_queen(world, this);
    stop_being_thrown_by_locusts(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_fishing(world, this);
    stop_waiting_to_crap(world, this);
    stop_playing_catch_with_monkey(world, this);
    close_word_balloon_now(world, this);
    stop_charlie_mischief(world, this);
}

/// `die:::` step 5: what the death is recorded as, and the sacrifice's episode event.
fn resolve_death_sacrifice(world: &mut World, this: Entity, listener: Option<Entity>) {
    let invalid_sacrifice = world.obj::<PygmyPygmyCoreFrame>(this).invalid_sacrifice;
    if invalid_sacrifice {
        let sacrifice_id = world.obj::<PygmyPygmyCoreFrame>(this).sacrifice_id.clone();
        if let Some(sacrifice_id) = sacrifice_id {
            notify_sacrifice_event(world, listener, &sacrifice_id);
        }
        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = None;
    }

    let (doodle_jumping, is_zombie, cut_in_half_by_charlie, held_by_xenu, current_sacrifice_id) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (
            core.doodle_jumping,
            core.is_zombie,
            core.cut_in_half_by_charlie,
            core.held_by_xenu,
            core.sacrifice_id.clone(),
        )
    };
    let (brains_eaten, had_heart_attack) = {
        let items = world.obj::<PygmyPygmyBehaviourItems>(this);
        (items.brains_eaten, items.had_heart_attack)
    };

    let override_id = if doodle_jumping {
        Some("Flattened")
    } else if is_zombie {
        if brains_eaten {
            Some("ZombieExplodeBrainsEaten")
        } else {
            Some("ZombieExplode")
        }
    } else if cut_in_half_by_charlie {
        if held_by_xenu {
            Some("XenuPuppetShow")
        } else {
            Some("CutInHalfByCharlie")
        }
    } else if had_heart_attack {
        Some("OldFartPygmyHeartAttack")
    } else {
        None
    };

    let sacrifice_id = match override_id {
        Some(override_id) => {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(override_id.to_owned());
            Some(override_id.to_owned())
        }
        None => current_sacrifice_id,
    };
    if let Some(sacrifice_id) = sacrifice_id {
        notify_sacrifice_event(world, listener, &sacrifice_id);
    }
}

/// `die:::` step 7: whether the dead pygmy comes back as a ghost.
fn can_become_ghost(world: &mut World, this: Entity, listener: Option<Entity>) -> bool {
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    let (the_runs, konkey_dong, no_pee) = {
        let touch_drag = world.obj::<PygmyPygmyTouchDrag>(this);
        (touch_drag.the_runs, touch_drag.konkey_dong, touch_drag.no_pee)
    };
    let excluded = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        core.is_ghost
            || story_mode
            || core.carried_away_by_ghost
            || core.invalid_sacrifice
            || core.skewered
            || core.caught_by_bird
            || core.remove_from_story_mode
            || the_runs
            || konkey_dong
            || no_pee
    };
    if excluded {
        return false;
    }

    match listener {
        Some(listener) => crate::controller::are_ghosts_enabled(world, listener),
        None => false,
    }
}

/// `die:::` step 7: the pygmy turns into a ghost and keeps its sacrifice id.
fn become_ghost(world: &mut World, this: Entity, listener: Option<Entity>) {
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.is_ghost = true;
        core.disable_labels = false;
        core.enable_island_floor_collision = false;
        core.enable_island_side_collision = false;
    }
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).enable_slide = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).killed_by_locusts = false;
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).mouth_filled_by_locusts = false;
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.mutated_by_locusts = false;
        core.drag_label_x_offset = 0.0;
        core.drag_label_y_offset = -20.0;
    }
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.z = SPRITE_Z;
        object.rot.z = 0.0;
        object.set_alpha(1.0);
        object.is_selectable = true;
    }
    world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_path_vector = Vec::new();

    let drag_marker = match listener {
        Some(listener) => crate::controller::pygmy_drag_display_marker(world, listener),
        None => None,
    };
    if let Some(drag_marker) = drag_marker {
        display::insert_before(world, drag_marker, Some(display_object));
    }
    update_label_layer(world, this);

    if let Some(listener) = listener {
        crate::controller::on_pygmy_gift_given(world, listener, this, "BecomeGhost");
    }
    pg_engine::logic::set_behavior(world, this, Some("PygmyGhostInit"));
}

/// `die:::` step 8: the real death, reported to the controller.
fn die_for_good(world: &mut World, this: Entity, listener: Option<Entity>) {
    let banshee_scream_started = world.obj::<PygmyGhostZombieVampire>(this).ghost_banshee_scream_started;
    if banshee_scream_started {
        world.obj_mut::<PygmyGhostZombieVampire>(this).ghost_banshee_scream_started = false;

        // Fast enumeration forbids mutation, so iterating a snapshot behaves the same.
        let pygmies = match listener {
            Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
            None => Vec::new(),
        };
        for pygmy in pygmies {
            stop_hold_ears_from_banshee_scream(world, pygmy, this);
        }
        pg_engine::logic::set_behavior(world, this, Some("PygmyGhostStopScream"));
    }

    let (name_text, name_background) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.name_text, core.name_background)
    };
    let name_text_group = match name_text {
        Some(name_text) => display::object(world, name_text).display_group,
        None => None,
    };
    if name_text_group.is_some() {
        if let Some(name_background) = name_background {
            display::remove_from_display_group(world, name_background);
        }
        if let Some(name_text) = name_text {
            display::remove_from_display_group(world, name_text);
        }
    }

    // Read before the controller shuts this pygmy down and drops it.
    let sacrifice_id = world.obj::<PygmyPygmyCoreFrame>(this).sacrifice_id.clone();
    let tell_closet_to_grab_new_tom = world.obj::<PygmyPygmyBehaviourItems>(this).tell_closet_to_grab_new_tom;

    if let Some(listener) = listener {
        crate::controller::on_pygmy_die(world, listener, this, sacrifice_id.as_deref());
    }

    if tell_closet_to_grab_new_tom {
        let closet_door = match listener {
            Some(listener) => crate::controller::closet_temple_closet_door_logic(world, listener, this),
            None => None,
        };
        if let Some(closet_door) = closet_door {
            crate::logic::apocalypse_locust_closet_xenu::closet_temple_closet_door_logic::grab_new_tom(world, closet_door);
        }
    }
}

/// `-[PygmyLogic explode:::]` @ 0x10012fcc0
pub fn seq_explode(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_pygmy_explode(world, listener, this);
    }

    let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;
    let leaves_ash_pile = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        !picked_up_by_hurricane
            && !core.kidnapped_by_konkey_dong
            && !core.in_kong_statue
            && !core.in_ape_wench
            && !core.in_ape_bongo
            && !core.in_ape_altar
            && !core.in_go_go_cage
            && !core.in_dance_dance_execution
            && core.hit_by_lightning
    };
    if leaves_ash_pile {
        let ash_pile_display_object = display::multi_model::spawn(world, Some("AshPile"));
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ash_pile_display_object = Some(ash_pile_display_object);
        let ash_pile_logic = pg_engine::logic::multi_model::spawn(world, ash_pile_display_object);
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).ash_pile_logic = Some(ash_pile_logic);
        pg_engine::logic::set_behavior(world, ash_pile_logic, Some("AshPileInit"));

        let pygmy_marker = match listener {
            Some(listener) => crate::controller::pygmy_display_marker(world, listener),
            None => None,
        };
        if let Some(pygmy_marker) = pygmy_marker {
            display::insert_before(world, pygmy_marker, Some(ash_pile_display_object));
        }

        let display_object = pg_engine::logic::display_object(world, this);
        let (pos, origin) = {
            let object = display::object(world, display_object);
            (object.pos, object.origin_pos)
        };
        let mut ash_pile = display::object_mut(world, ash_pile_display_object);
        ash_pile.pos.x = pos.x;
        ash_pile.pos.y = pos.y;
        ash_pile.pos.z = pos.z;
        ash_pile.origin_pos.x = origin.x;
        ash_pile.origin_pos.y = origin.y;
    }

    let waiting_out_house = world.obj::<PygmyIslandLife>(this).waiting_out_house_logic;
    if let Some(out_house) = waiting_out_house {
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        crate::logic::island_objects::out_house_logic::stop_waiting_to_crap(world, out_house, this);
        world.obj_mut::<PygmyIslandLife>(this).waiting_out_house_logic = None;
    }

    let magnifying_glass = world.obj::<PygmyPygmyBehaviourItems>(this).burning_magnifying_glass_logic;
    if let Some(magnifying_glass) = magnifying_glass {
        crate::logic::island_anthill::magnifying_glass_logic::on_pygmy_die(world, magnifying_glass, this);
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).burning_magnifying_glass_logic = None;
    }
    ItemResult::Done
}

/// `-[PygmyLogic checkPygmyDialog:::]` @ 0x10012ff78
pub fn seq_check_pygmy_dialog(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let dialog_id = call.param("id");
    let now = absolute_time_get_current(world);

    let blocked = {
        let mut statics = world.get_resource_or_insert_with(PygmyPygmyBehaviourItemsStatics::default);

        // Forget dialogs older than 15 s and find the newest one still listed.
        let mut newest_start_time = 0.0_f64;
        statics.dialog_start_times.retain(|_, start_time| {
            let elapsed = now - *start_time;
            if elapsed > 15.0 {
                return false;
            }
            if *start_time > newest_start_time {
                newest_start_time = *start_time;
            }
            true
        });

        let started_recently = now - newest_start_time < 3.0;
        let still_listed = match dialog_id {
            Some(dialog_id) => statics.dialog_start_times.contains_key(dialog_id),
            None => false,
        };
        let blocked = started_recently || still_listed;

        // `setObject:forKey:` with a nil key would raise; every shipped item names its dialog.
        if !blocked {
            if let Some(dialog_id) = dialog_id {
                statics.dialog_start_times.insert(dialog_id.to_owned(), now);
            }
        }
        blocked
    };

    if blocked {
        let alternate_sequence_id = call.param("alternateSequenceId");
        pg_engine::logic::set_behavior(world, this, alternate_sequence_id);
        return ItemResult::Replaced;
    }
    ItemResult::Done
}

/// `-[PygmyLogic resetVel:::]` @ 0x1001301c8
pub fn seq_reset_vel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut state = pg_engine::logic::logic_mut(world, this);
    state.pos_vel.x = 0.0;
    state.pos_vel.y = 0.0;
    state.pos_vel.z = 0.0;
    ItemResult::Done
}

/// `-[PygmyLogic walk:::]` @ 0x1001301e8
pub fn seq_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;

    if call.first_time {
        let direction = call.param("direction");
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        let coin = world.resource_mut::<Random>().arc4random();

        let goes_right = match direction {
            Some(direction) => direction == "Right",
            None => {
                let (island_min_x_pos, island_max_x_pos) = match listener {
                    Some(listener) => {
                        let island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
                        let island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
                        (island_min_x_pos, island_max_x_pos)
                    }
                    None => (0.0, 0.0),
                };
                if x_pos == island_max_x_pos {
                    false
                } else {
                    coin & 1 != 0 || x_pos == island_min_x_pos
                }
            }
        };

        let (x_scale, x_vel) = if goes_right { (1.0, 50.0) } else { (-1.0, -50.0) };
        display::object_mut(world, display_object).scale.x = x_scale;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if !story_mode {
        let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
        let tiger_blood = personality == 1 && check_for_tiger_blood(world, this);
        if tiger_blood || WALK_INTERACTIONS.iter().any(|check| check(world, this)) {
            return ItemResult::Replaced;
        }
    }

    let island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let mut result = if island_side_collision {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    if stall_time <= 0.0 {
        if is_next_to_burning_bon_fire(world, this, listener, x_pos) {
            // Never stop right next to a burning fire: walk on.
            let min_time = call.float("minTime");
            let max_time = call.float("maxTime");
            let random = world.resource_mut::<Random>().arc4random();
            let extra_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
            pg_engine::logic::logic_mut(world, this).stall_time = extra_time + stall_time;
            return result;
        }
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        result = ItemResult::Done;
    }

    let mut state = pg_engine::logic::logic_mut(world, this);
    state.stall_time -= call.delta_time;
    result
}

/// `walk:::`'s timeout test: the bonfire is enabled and burning and `x_pos` is within 35 of it.
fn is_next_to_burning_bon_fire(world: &mut World, this: Entity, listener: Option<Entity>, x_pos: f32) -> bool {
    let Some(listener) = listener else {
        return false;
    };
    if !crate::controller::is_bon_fire_enabled(world, listener) {
        return false;
    }
    let Some(bon_fire) = crate::controller::bon_fire_logic(world, listener, this) else {
        return false;
    };

    let bon_fire_display_object = pg_engine::logic::display_object(world, bon_fire);
    if !crate::logic::island_core::bon_fire_logic::is_on_fire(world, bon_fire) {
        return false;
    }
    let bon_fire_x_pos = display::object(world, bon_fire_display_object).pos.x;
    bon_fire_x_pos - 35.0 < x_pos && x_pos < bon_fire_x_pos + 35.0
}

/// `-[PygmyLogic run:::]` @ 0x1001306bc
pub fn seq_run(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);
    let x_pos = display::object(world, display_object).pos.x;

    if call.first_time {
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        let listener = listener_of(world, this);
        let island_center_x_pos = match listener {
            Some(listener) => crate::controller::island_center_x_pos(world, listener),
            None => 0.0,
        };
        let (x_scale, x_vel) = if x_pos > island_center_x_pos {
            (-1.0, -100.0)
        } else {
            (1.0, 100.0)
        };
        display::object_mut(world, display_object).scale.x = x_scale;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let mut result = if island_side_collision {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    let mut state = pg_engine::logic::logic_mut(world, this);
    if state.stall_time <= 0.0 {
        state.pos_vel.x = 0.0;
        result = ItemResult::Done;
    }
    state.stall_time -= call.delta_time;
    result
}

/// `-[PygmyLogic panicRun:::]` @ 0x10013089c
pub fn seq_panic_run(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let display_object = pg_engine::logic::display_object(world, this);

    if call.first_time {
        let total_time = call.float("totalTime");
        pg_engine::logic::logic_mut(world, this).stall_time = total_time;
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let next_turn_stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).next_turn_stall_time = next_turn_stall_time;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

        let x_pos = display::object(world, display_object).pos.x;
        let listener = listener_of(world, this);
        let island_center_x_pos = match listener {
            Some(listener) => crate::controller::island_center_x_pos(world, listener),
            None => 0.0,
        };
        let (x_scale, x_vel) = if x_pos > island_center_x_pos {
            (-1.0, -200.0)
        } else {
            (1.0, 200.0)
        };
        display::object_mut(world, display_object).scale.x = x_scale;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    let island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let next_turn_stall_time = world.obj::<PygmyPygmyBehaviourItems>(this).next_turn_stall_time;
    if island_side_collision || next_turn_stall_time <= 0.0 {
        {
            let mut object = display::object_mut(world, display_object);
            object.scale.x = -object.scale.x;
        }
        {
            let mut state = pg_engine::logic::logic_mut(world, this);
            state.pos_vel.x = -state.pos_vel.x;
        }
        let min_time = call.float("minTime");
        let max_time = call.float("maxTime");
        let random = world.resource_mut::<Random>().arc4random();
        let next_turn_stall_time = pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random);
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).next_turn_stall_time = next_turn_stall_time;
    }

    let result = {
        let mut state = pg_engine::logic::logic_mut(world, this);
        let result = if state.stall_time <= 0.0 {
            state.pos_vel.x = 0.0;
            ItemResult::Done
        } else {
            ItemResult::Pending
        };
        state.stall_time -= call.delta_time;
        result
    };
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).next_turn_stall_time -= call.delta_time;
    result
}

/// `-[PygmyLogic slide:::]` @ 0x100130b94
pub fn seq_slide(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);

    if call.first_time {
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            if !core.cooked && !core.killed_by_plague && !core.cut_in_half_by_charlie && core.time_temple_pygmy_type == 0 {
                core.sacrifice_id = Some("SlideDrown".to_owned());
            }
            core.enable_island_side_collision = false;
            core.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    }

    let (screen_side_collision, is_zombie) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.screen_side_collision, core.is_zombie)
    };
    if screen_side_collision {
        if is_zombie {
            zombie_explode(world, this);
            return ItemResult::Replaced;
        }
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            if !core.cooked && !core.killed_by_plague && !core.cut_in_half_by_charlie && core.time_temple_pygmy_type == 0 {
                core.sacrifice_id = Some("SlideDrown".to_owned());
            }
        }
        pg_engine::logic::set_behavior(world, this, Some("PygmyFallIgnoreIsland"));
        return ItemResult::Replaced;
    }

    let eruption_active = is_eruption_active(world, listener);
    let picks_up_in_eruption = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        eruption_active
            && !core.is_zombie
            && !core.cooked
            && !core.killed_by_plague
            && !core.cut_in_half_by_charlie
            && core.time_temple_pygmy_type == 0
    };
    if picks_up_in_eruption {
        let tilt_state = world.obj::<PygmyPygmyCoreFrame>(this).tilt_state;
        if tilt_state == 4 || tilt_state == 3 {
            stop_sleeping(world, this);
            pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
            let display_object = pg_engine::logic::display_object(world, this);
            let x_scale = display::object(world, display_object).scale.x;
            let backward = if tilt_state == 4 { x_scale > 0.0 } else { x_scale < 0.0 };
            let behavior = if backward { "PygmyPickupBackward" } else { "PygmyPickupForward" };
            // The item goes on to the ocean test below.
            pg_engine::logic::set_behavior(world, this, Some(behavior));
        }
    }

    let ocean_enabled = is_ocean_enabled(world, listener);
    let display_object = pg_engine::logic::display_object(world, this);
    let y_pos = display::object(world, display_object).pos.y;
    if !ocean_enabled || !(y_pos < 16.0) {
        return ItemResult::Pending;
    }

    stop_sleeping(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).drowning = true;
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }

    let behavior = {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        if core.cooked {
            "PygmyCookedSplash"
        } else if core.killed_by_plague {
            "PygmyPlagueBodySplash"
        } else if core.cut_in_half_by_charlie {
            "HalfPygmySplash"
        } else if core.time_temple_pygmy_type != 0 {
            "TemplePygmySplash"
        } else {
            core.sacrifice_id = Some("SlideDrown".to_owned());
            "PygmySplash"
        }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[PygmyLogic fall:::]` @ 0x100130fb8
pub fn seq_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let display_object = pg_engine::logic::display_object(world, this);

    if call.first_time {
        let island_floor_collision = world.obj::<PygmyPygmyBehaviourItems>(this).island_floor_collision;
        if !island_floor_collision {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).on_island = false;
        }
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            core.enable_island_side_collision = false;
            core.enable_island_floor_collision = true;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
        let y_pos = display::object(world, display_object).pos.y;
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).start_fall_y_pos = y_pos;
        if call.bool("ignoreIsland") {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;
        }
    }

    // Behind the island plane the water line is the island's height.
    let z_pos = display::object(world, display_object).pos.z;
    let ocean_level = if z_pos < SPRITE_Z {
        world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos
    } else {
        16.0
    };

    let beheaded_by_squid = world.obj::<PygmyPygmyCoreFrame>(this).beheaded_by_squid;
    let torn_apart_by_ape_altar = world.obj::<PygmyPygmyBehaviourItems>(this).torn_apart_by_ape_altar;
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    let mut caught = false;
    if !beheaded_by_squid && !torn_apart_by_ape_altar && !story_mode {
        caught = FALL_CATCHERS.iter().any(|check| check(world, this));
    }
    if !caught {
        let cooked = world.obj::<PygmyPygmyCoreFrame>(this).cooked;
        caught = cooked && check_for_ice_hole_during_fall(world, this);
    }
    if caught {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
        return ItemResult::Replaced;
    }

    let island_floor_collision = world.obj::<PygmyPygmyBehaviourItems>(this).island_floor_collision;
    if island_floor_collision {
        return land_from_fall(world, this, listener, display_object);
    }

    let screams = {
        let y_pos = display::object(world, display_object).pos.y;
        let y_vel = pg_engine::logic::logic(world, this).pos_vel.y;
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        let items = world.obj::<PygmyPygmyBehaviourItems>(this);
        let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
        y_pos < core.island_y_pos
            && y_vel < 0.0
            && !items.scream_during_fall
            && !core.cooked
            && !core.killed_by_plague
            && !core.beheaded_by_squid
            && !items.torn_apart_by_ape_altar
            && !story_mode
    };
    if screams && !is_ocean_enabled(world, listener) {
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).scream_during_fall = true;
        pg_engine::logic::set_behavior(world, this, Some("PygmyFallScream"));
        return ItemResult::Replaced;
    }

    let ocean_enabled = is_ocean_enabled(world, listener);
    let y_pos = display::object(world, display_object).pos.y;
    if !(ocean_enabled && y_pos < ocean_level) {
        return ItemResult::Pending;
    }
    splash_from_fall(world, this, listener, display_object, ocean_level)
}

/// `fall:::` on the island floor: the landing.
fn land_from_fall(world: &mut World, this: Entity, listener: Option<Entity>, display_object: Entity) -> ItemResult {
    let doodle_jumping = world.obj::<PygmyPygmyCoreFrame>(this).doodle_jumping;
    if doodle_jumping {
        if let Some(listener) = listener {
            crate::controller::on_pygmy_stop_doodle_jump(world, listener, this);
        }
    }
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    display::object_mut(world, display_object).rot.z = 0.0;

    let (pygmy_type, killed_by_plague) = {
        let core = world.obj::<PygmyPygmyCoreFrame>(this);
        (core.time_temple_pygmy_type, core.killed_by_plague)
    };
    if pygmy_type != 1 || killed_by_plague {
        return ItemResult::Done;
    }

    // The old fart only stays on his feet after a short drop.
    let start_fall_y_pos = world.obj::<PygmyPygmyBehaviourItems>(this).start_fall_y_pos;
    let y_pos = display::object(world, display_object).pos.y;
    let lands_on_feet = start_fall_y_pos - y_pos < 50.0;
    world.obj_mut::<PygmyApocalypse>(this).temple_old_pygmy_upright = lands_on_feet;
    let behavior = if lands_on_feet {
        "TempleOldPygmyLandOnFeet"
    } else {
        "TempleOldPygmyFallDown"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `fall:::` below the water line: the splash variant of the pygmy's state.
fn splash_from_fall(
    world: &mut World,
    this: Entity,
    listener: Option<Entity>,
    display_object: Entity,
    ocean_level: f32,
) -> ItemResult {
    stop_sleeping(world, this);
    let doodle_jumping = world.obj::<PygmyPygmyCoreFrame>(this).doodle_jumping;
    if doodle_jumping {
        if let Some(listener) = listener {
            crate::controller::on_pygmy_stop_doodle_jump(world, listener, this);
        }
    }
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).drowning = true;
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
    }
    display::object_mut(world, display_object).pos.y = ocean_level;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    display::object_mut(world, display_object).rot.z = 0.0;

    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if story_mode {
        return ItemResult::Done;
    }
    let is_zombie = world.obj::<PygmyPygmyCoreFrame>(this).is_zombie;
    if is_zombie {
        zombie_explode(world, this);
        return ItemResult::Replaced;
    }

    let had_heart_attack = world.obj::<PygmyPygmyBehaviourItems>(this).had_heart_attack;
    let behavior = {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        if had_heart_attack {
            core.sacrifice_id = Some("OldFartPygmyHeartAttack".to_owned());
            "TempleOldPygmyHeartAttackSplash"
        } else if core.cooked {
            "PygmyCookedSplash"
        } else if core.killed_by_plague {
            if core.dropped_by_hurricane {
                "PygmyPlagueBodyHurricaneSplash"
            } else if core.killed_by_plague_disappear {
                "PygmyPlagueBodySplashDisappear"
            } else {
                "PygmyPlagueBodySplash"
            }
        } else if core.cut_in_half_by_charlie {
            "HalfPygmySplash"
        } else if core.time_temple_pygmy_type != 0 {
            "TemplePygmySplash"
        } else if core.ate_burnt_food {
            "PygmyChokeSplash"
        } else if core.beheaded_by_squid {
            "PygmyBeheadedSplash"
        } else if core.ate_sword_fish {
            "PygmyAteSwordFishSplash"
        } else if core.hit_by_icicle {
            "PygmyHitByIcicleSplash"
        } else if core.died_from_thetan_brawl {
            "PygmyThetanBrawlSplash"
        } else {
            "PygmySplash"
        }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `-[PygmyLogic tumble:::]` @ 0x1001317ec
pub fn seq_tumble(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);

    if call.first_time {
        let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
        if pygmy_type == 4 {
            if let Some(idol_manager) = crate::managers::idol_manager::instance(world) {
                crate::managers::idol_manager::trigger_episode_event(world, idol_manager, "FastPygmyFlick");
            }
        }
        if let Some(listener) = listener {
            crate::controller::on_pygmy_tumble(world, listener, this);
        }
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            core.enable_island_side_collision = false;
            core.enable_island_floor_collision = false;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.falling = true;
        core.tumble = true;
        return ItemResult::Pending;
    }

    let display_object = pg_engine::logic::display_object(world, this);

    let ice_monster_enabled = is_ice_monster_enabled(world, listener);
    let hit_ice_monster = world.obj::<PygmyPygmyBehaviourItems>(this).hit_ice_monster;
    if ice_monster_enabled && !hit_ice_monster {
        let ice_monster = ice_monster_of(world, listener, this);
        if let Some(ice_monster) = ice_monster {
            if let Some(result) = tumble_into_ice_monster(world, this, display_object, ice_monster) {
                return result;
            }
        }
    }

    // The pygmy flies past the ice monster: from now on it is drawn behind it.
    let farther_than_ice_monster = world.obj::<PygmyPygmyBehaviourItems>(this).farther_than_ice_monster;
    if !farther_than_ice_monster && is_ice_monster_enabled(world, listener) {
        let ice_monster = ice_monster_of(world, listener, this);
        if let Some(ice_monster) = ice_monster {
            let ice_monster_display_object = pg_engine::logic::display_object(world, ice_monster);
            let ice_monster_z_pos = display::object(world, ice_monster_display_object).pos.z;
            let z_pos = display::object(world, display_object).pos.z;
            if z_pos < ice_monster_z_pos + 100.0 {
                world.obj_mut::<PygmyPygmyBehaviourItems>(this).farther_than_ice_monster = true;
                display::insert_before(world, ice_monster_display_object, Some(display_object));
                update_label_layer(world, this);
            }
        }
    }

    // At the apex of the flight the pygmy may head into the volcano.
    let descending = world.obj::<PygmyPygmyBehaviourItems>(this).descending;
    let y_vel = pg_engine::logic::logic(world, this).pos_vel.y;
    if !descending && y_vel <= 0.0 {
        let pos = display::object(world, display_object).pos;
        let tumble_pos = world.obj::<PygmyPygmyBehaviourItems>(this).tumble_pos;
        let mut layer_marker = match listener {
            Some(listener) => crate::controller::pygmy_island_display_marker(world, listener),
            None => None,
        };

        let volcano_enabled = match listener {
            Some(listener) => crate::controller::is_volcano_enabled(world, listener),
            None => false,
        };
        if volcano_enabled {
            let x_ahead = pos.x + (pos.x - tumble_pos.x);
            let eruption_active = is_eruption_active(world, listener);
            let z_ahead = pos.z + (pos.z - tumble_pos.z);
            if x_ahead > 230.0 && !eruption_active && z_ahead < -700.0 {
                {
                    let mut object = display::object_mut(world, display_object);
                    object.pos.x = 270.0;
                    object.pos.z = -1200.0;
                    if object.pos.y < 1000.0 {
                        object.pos.y = 1000.0;
                    }
                }
                {
                    let mut state = pg_engine::logic::logic_mut(world, this);
                    state.pos_vel.x = 0.0;
                    state.pos_vel.z = 0.0;
                }
                world.obj_mut::<PygmyPygmyBehaviourItems>(this).hit_volcano = true;
                world.obj_mut::<PygmyPygmyCoreFrame>(this).disable_labels = true;
                layer_marker = match listener {
                    Some(listener) => crate::controller::pygmy_cloud_behind_display_marker(world, listener),
                    None => None,
                };
            }
        }

        world.obj_mut::<PygmyPygmyBehaviourItems>(this).descending = true;
        if let Some(layer_marker) = layer_marker {
            display::insert_before(world, layer_marker, Some(display_object));
        }
        update_label_layer(world, this);

        let igloo_cannon = world.obj::<PygmyPygmyBehaviourItems>(this).igloo_cannon;
        if igloo_cannon {
            pg_engine::logic::set_behavior(world, this, Some("PygmyLeaveIglooTopExplodeFall"));
            return ItemResult::Replaced;
        }
    }

    let hit_volcano = world.obj::<PygmyPygmyBehaviourItems>(this).hit_volcano;
    if hit_volcano && is_eruption_active(world, listener) {
        world.obj_mut::<PygmyPygmyBehaviourItems>(this).hit_volcano = false;
    }

    if !is_ocean_enabled(world, listener) {
        return ItemResult::Pending;
    }
    let hit_volcano = world.obj::<PygmyPygmyBehaviourItems>(this).hit_volcano;
    let y_pos = display::object(world, display_object).pos.y;
    let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
    let water_level = if hit_volcano { island_y_pos + 100.0 } else { island_y_pos };
    if !(y_pos <= water_level) {
        return ItemResult::Pending;
    }

    stop_sleeping(world, this);
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).drowning = true;
    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
        state.pos_vel.z = 0.0;
    }
    let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.y = island_y_pos;
        object.rot.z = 0.0;
    }

    let (hit_volcano, igloo_cannon, hit_ice_monster) = {
        let items = world.obj::<PygmyPygmyBehaviourItems>(this);
        (items.hit_volcano, items.igloo_cannon, items.hit_ice_monster)
    };
    if hit_volcano {
        let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
        display::object_mut(world, display_object).pos.y = island_y_pos + 100.0;
        {
            let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
            let sacrifice_id = if core.time_temple_pygmy_type == 4 {
                "FastPygmyHitVolano"
            } else {
                "ThrownInVolcano"
            };
            core.sacrifice_id = Some(sacrifice_id.to_owned());
        }
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        if let Some(listener) = listener {
            crate::controller::on_volcano_hit(world, listener, this);
        }
        return ItemResult::Replaced;
    }

    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if story_mode {
        return ItemResult::Done;
    }
    let behavior = {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        let sacrifice_id = if igloo_cannon {
            "IglooCannon"
        } else if hit_ice_monster {
            "BounceOffIceMonster"
        } else {
            "FlickDrown"
        };
        core.sacrifice_id = Some(sacrifice_id.to_owned());
        if core.time_temple_pygmy_type == 0 {
            "PygmySplash"
        } else {
            "TemplePygmySplash"
        }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
    ItemResult::Replaced
}

/// `tumble:::` against the ice monster: its mouth first (when open), then its body. `None`
/// when the pygmy misses it.
fn tumble_into_ice_monster(
    world: &mut World,
    this: Entity,
    display_object: Entity,
    ice_monster: Entity,
) -> Option<ItemResult> {
    use crate::logic::ice_island::ice_monster_logic;

    let ice_monster_display_object = pg_engine::logic::display_object(world, ice_monster);
    let pos = display::object(world, display_object).pos;
    let screen_y_pos = (SPRITE_Z / pos.z) * pos.y;

    let mouth_active = ice_monster_logic::is_mouth_collision_active(world, ice_monster);
    if mouth_active {
        let mouth_y_pos = ice_monster_logic::screen_mouth_collision_y_pos(world, ice_monster);
        let mouth_radius = ice_monster_logic::screen_mouth_collision_radius(world, ice_monster);
        let pos = display::object(world, display_object).pos;
        let ice_monster_pos = display::object(world, ice_monster_display_object).pos;
        if is_inside_ice_monster_window(pos, ice_monster_pos, screen_y_pos, mouth_y_pos, mouth_radius) {
            world.obj_mut::<PygmyPygmyBehaviourItems>(this).hit_ice_monster = true;
            ice_monster_logic::hit_by_pygmy_in_mouth(world, ice_monster, this);
            world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("EatenByIceMonster".to_owned());
            pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
            return Some(ItemResult::Replaced);
        }
    }

    let body_y_pos = ice_monster_logic::screen_collision_y_pos(world, ice_monster);
    let body_radius = ice_monster_logic::screen_collision_radius(world, ice_monster);
    let pos = display::object(world, display_object).pos;
    let ice_monster_pos = display::object(world, ice_monster_display_object).pos;
    if !is_inside_ice_monster_window(pos, ice_monster_pos, screen_y_pos, body_y_pos, body_radius) {
        return None;
    }

    world.obj_mut::<PygmyPygmyBehaviourItems>(this).hit_ice_monster = true;
    let igloo_cannon = world.obj::<PygmyPygmyBehaviourItems>(this).igloo_cannon;
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let on_fire = igloo_cannon || pygmy_type != 4;
    ice_monster_logic::hit_by_pygmy(world, ice_monster, this, on_fire);

    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    if pygmy_type == 4 {
        {
            let mut state = pg_engine::logic::logic_mut(world, this);
            state.pos_vel.x = 0.0;
            state.pos_vel.y = 0.0;
            state.pos_vel.z = 0.0;
        }
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("FastPygmyHitIceMonster".to_owned());
        pg_engine::logic::set_behavior(world, this, Some("TempleFastPygmyExplodeOnIceMonster"));
        return Some(ItemResult::Replaced);
    }

    let recharging = ice_monster_logic::is_recharging(world, ice_monster);
    if !recharging {
        // Bounced off the monster.
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.y = 1000.0;
        state.pos_vel.z *= -0.5;
        return Some(ItemResult::Pending);
    }

    {
        let mut state = pg_engine::logic::logic_mut(world, this);
        state.pos_vel.x = 0.0;
        state.pos_vel.y = 0.0;
        state.pos_vel.z = 0.0;
    }
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some("ExplodeOnIceMonster".to_owned());
    pg_engine::logic::set_behavior(world, this, Some("PygmyExplodeOnIceMonster"));
    Some(ItemResult::Replaced)
}

/// `-[PygmyLogic setOrigin:::]` @ 0x1001326a4
pub fn seq_set_origin(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let x_pos = call.float("xPos");
    let y_pos = call.float("yPos");
    let z_pos = call.float("zPos");
    let display_object = pg_engine::logic::display_object(world, this);

    if !call.bool("inPlace") {
        let mut object = display::object_mut(world, display_object);
        object.origin_pos.x = x_pos;
        object.origin_pos.y = y_pos;
        object.origin_pos.z = z_pos;
        return ItemResult::Done;
    }

    // In place: the sprite stays where it is, so the spring's last position moves with it.
    let (scale, origin) = {
        let object = display::object(world, display_object);
        (object.scale, object.origin_pos)
    };
    let x_shift = x_pos - origin.x;
    let y_shift = y_pos - origin.y;
    let z_shift = z_pos - origin.z;
    {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.last_pos.x -= x_shift * scale.x;
        core.last_pos.y -= y_shift * scale.y;
        core.last_pos.z -= z_shift * scale.z;
    }
    display::set_origin_x_pos_in_place(world, display_object, x_pos);
    display::set_origin_y_pos_in_place(world, display_object, y_pos);
    display::set_origin_z_pos_in_place(world, display_object, z_pos);
    ItemResult::Done
}

/// `-[PygmyLogic setAnimationBasedOnPygmyType:::]` @ 0x100132898
pub fn seq_set_animation_based_on_pygmy_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let disease = world.obj::<SpriteLogic>(this).apocalypse_disease_type;

    let mut animation_id = match pygmy_type {
        1 => {
            let variant = disease_variant(call, disease, "oldFartPurification", "oldFartPlague");
            variant.or(call.param("oldFart"))
        }
        2 => {
            let variant = disease_variant(call, disease, "babyPurification", "babyPlague");
            variant.or(call.param("baby"))
        }
        3 => {
            let variant = disease_variant(call, disease, "gingerPurification", "gingerPlague");
            variant.or(call.param("ginger"))
        }
        4 => {
            let variant = disease_variant(call, disease, "fastPurification", "fastPlague");
            variant.or(call.param("fast"))
        }
        _ => None,
    };
    if animation_id.is_none() {
        let variant = disease_variant(call, disease, "purification", "plague");
        animation_id = variant.or(call.param("normal"));
    }

    // Played even when nil.
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::play_animation_id(world, animation, animation_id);
    ItemResult::Done
}

/// `-[PygmyLogic pushSequenceBasedOnPersonality:::]` @ 0x100132a64
pub fn seq_push_sequence_based_on_personality(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let Some(sequence_id) = sequence_based_on_personality(world, this, call) else {
        return ItemResult::Done;
    };
    let sequence_def = pg_engine::library::sequence_def(world, sequence_id);
    pg_engine::sequence::push_sequence_def(world, call.sequence, sequence_def);
    ItemResult::Done
}

/// `-[PygmyLogic runSequenceBasedOnPersonality:::]` @ 0x100132c9c
pub fn seq_run_sequence_based_on_personality(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let Some(sequence_id) = sequence_based_on_personality(world, this, call) else {
        return ItemResult::Done;
    };
    pg_engine::logic::set_behavior(world, this, Some(sequence_id));
    ItemResult::Done
}

/// `-[PygmyLogic onMultiModelEvent:eventId:]` @ 0x100132e94
pub fn on_multi_model_event(world: &mut World, this: Entity, _logic: Entity, event_id: &str) {
    if event_id == "WordBalloonCloseComplete" {
        close_word_balloon_now(world, this);
        return;
    }
    if event_id != "TheLightExit" {
        return;
    }

    let (light_display_object, light_logic) = {
        let ghost = world.obj::<PygmyGhostZombieVampire>(this);
        (ghost.the_light_display_object, ghost.the_light_logic)
    };
    if let Some(light_display_object) = light_display_object {
        display::remove_from_display_group(world, light_display_object);
    }
    if let Some(light_logic) = light_logic {
        // The pygmy held the last reference: the light's logic is deallocated.
        pg_engine::logic::shutdown(world, light_logic);
        pg_engine::logic::dealloc(world, light_logic);
    }
    {
        let mut ghost = world.obj_mut::<PygmyGhostZombieVampire>(this);
        ghost.the_light_display_object = None;
        ghost.the_light_logic = None;
    }

    let go_into_the_light = world.obj::<PygmyGhostZombieVampire>(this).go_into_the_light;
    if go_into_the_light {
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
        let listener = listener_of(world, this);
        if let Some(listener) = listener {
            crate::controller::on_pygmy_gift_given(world, listener, this, "IntoTheLight");
        }
    }
}
