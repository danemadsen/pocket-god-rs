//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use pg_engine::display;
use pg_engine::library::Library;

use crate::logic::island_core::sprite_logic::SpriteLogic;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyCharlieStory {
    /// `mCharlieCuttingPygmyLogic`: `@"PygmyLogic"`
    pub charlie_cutting_pygmy_logic: Option<Entity>,

    /// `mCharliePygmyLogic`: `@"PygmyLogic"`
    pub charlie_pygmy_logic: Option<Entity>,

    /// `mCharlieOnBackMonkeyLogic`: `@"MonkeyLogic"`
    pub charlie_on_back_monkey_logic: Option<Entity>,

    /// `mbAbsorbedByHalfPygmy`: `B`
    pub absorbed_by_half_pygmy: bool,

    /// `mbStoryMode`: `B`
    pub story_mode: bool,

    /// `mbStoryModeReady`: `B`
    pub story_mode_ready: bool,

    /// `mbDieInStory`: `B`
    pub die_in_story: bool,

    /// `mbDieInStoryResume`: `B`
    pub die_in_story_resume: bool,

    /// `mbDieInStoryComplete`: `B`
    pub die_in_story_complete: bool,

    /// `mWordBalloonLogic`: `@"MultiModelLogic"`
    pub word_balloon_logic: Option<Entity>,
}

/// `mPersonaPersonality` of Charlie.
const PERSONALITY_CHARLIE: i32 = 1;

/// The animation channel the food a pygmy eats is bound to.
const FOOD_CHANNEL: usize = 14;

/// Id of the `BCMultiModel` of the word balloon.
const WORD_BALLOON_ID: &str = "WordBalloon";

/// The library's dynamic text the word balloon shows.
const DIALOGUE_LINE_TEXT_ID: &str = "DialogueLine1";

/// Wrap width of the word balloon's text.
const DIALOGUE_LINE_WRAP_WIDTH: f32 = 104.0;

/// `mPygmyLogicListener`: always the view controller (docs/re/pygmy.md §23).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `[mPygmyLogicListener pygmyLogicArray:self]` (a nil listener answers no pygmies).
fn pygmy_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = listener_of(world, this);
    match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

/// `mPosVel.x = mPosVel.y = 0` (z and w keep their values).
fn stop_moving(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `[mDisplayObject setZRot:0]`.
fn reset_z_rot(world: &mut World, this: Entity) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).rot.z = 0.0;
}

/// `[mDisplayObject setXScale:x_scale]`.
fn set_x_scale(world: &mut World, this: Entity, x_scale: f32) {
    let display_object = pg_engine::logic::display_object(world, this);
    display::object_mut(world, display_object).scale.x = x_scale;
}

/// `[[logic displayObject] xPos]`.
fn x_pos_of(world: &World, logic: Entity) -> f32 {
    let display_object = pg_engine::logic::display_object(world, logic);
    display::object(world, display_object).pos.x
}

/// `mSacrificeId = sacrifice_id`: what the coming death will be recorded as.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_string());
}

/// The guard of `hitByLightningByWarlock` (the same as `hitByLightning`'s): the busy set FREE
/// of `docs/re/pygmy.md` §1.3 without the hurricane flags, the containers and `mbIsZombie`.
/// Pure reads, so the order of the original's checks does not matter.
fn is_busy_for_lightning(core: &PygmyPygmyCoreFrame) -> bool {
    core.touch_began_inside
        || core.remove_from_story_mode
        || core.cut_in_half_by_charlie
        || core.time_temple_pygmy_type != 0
        || core.dropped_by_hurricane
        || core.hit_by_lightning
        || core.died_from_thetan_brawl
        || core.catch_shark
        || core.eaten_by_piranha
        || core.sucked_into_drain
        || core.pain_drain
        || core.eaten_by_t_rex
        || core.eaten_by_spider
        || core.hit_by_snow_ball
        || core.igloo_roll
        || core.sun_frozen
        || core.sun_burnt
        || core.kidnapped_by_konkey_dong
        || core.resting_in_igloo
        || core.crapping
        || core.trapped_in_spider_web
        || core.trapped_in_zombie_grave
        || core.trapped_in_idol_pedestal
        || core.inside_temple_door
        || core.inside_apocalypse_temple_door
        || core.on_top_apocalypse_time_temple_platform
        || core.eaten_by_locust_queen
        || core.in_apocalypse_temple_fountain
        || core.thetan_brawl
        || core.on_closet_temple_thetan_meter
        || core.on_closet_temple_couch
        || core.in_closet_temple_closet_door
        || core.slamming_closet_temple_closet_door
        || core.held_by_xenu
        || core.held_in_toms_arms
        || core.holding_baby_pygmy
        || core.thrown_by_locusts
        || core.killed_by_locusts
        || core.killed_by_purification
        || core.dance_dance_executed
        || core.battle_of_the_gods
        || core.zombie_explode
        || core.is_ghost
        || core.touched_by_ghost
        || core.possessed_by_thetan
        || core.caught_by_monkey
        || core.monkey_on_back
        || core.eaten_by_moron_snake
        || core.wearing_water_wings
        || core.hold_ears_from_banshee_scream
        || core.explode_from_banshee_scream
        || core.skewered
        || core.underwater
        || core.explode_from_not_crapping
        || core.explode_from_vampire_sun
        || core.explode_from_vampire_ginger
        || core.explode_from_loving_pocket_god
        || core.dragging
        || core.crushed
        || core.caught_by_bird
        || core.falling
        || core.in_crack_hole
        || core.doodle_jumping
        || core.swarmed
        || core.burnt_by_magnifying_glass
        || core.tumble
        || core.on_fire_from_lava
        || core.on_fire_from_bon_fire
        || core.ate_burnt_food
        || core.ate_sword_fish
        || core.hit_by_icicle
        || core.cooked
        || core.killed_by_plague
        || core.drowning
}

/// The containers of FREE: the Kong statue, the ape wench, bongo and altar, the go-go cage and
/// the Dance Dance Execution.
fn is_in_container(core: &PygmyPygmyCoreFrame) -> bool {
    core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
}

/// The whole busy set FREE.
fn is_busy(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let sprite = world.obj::<SpriteLogic>(this);
    let held_by_hurricane = sprite.blown_by_hurricane || sprite.picked_up_by_hurricane;

    is_busy_for_lightning(core) || held_by_hurricane || is_in_container(core) || core.is_zombie
}

/// FREE +{mbSharkOnHead} with `mTiltState == 0`: the guard of `canBeAbsorbedByHalfPygmy:` and
/// of Charlie's smacks, which the other two Charlie guards extend.
fn is_free_upright_without_shark_on_head(world: &World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let tilt_state = core.tilt_state;
    let shark_on_head = core.shark_on_head;

    tilt_state == 0 && !shark_on_head && !is_busy(world, this)
}

/// The stop list S13 (`docs/re/pygmy.md` §1.3), in the original's order.
fn stop_activities(world: &mut World, this: Entity) {
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// The stop list S14: S13 with `stopBeingEaten` after `stopSleeping`.
fn stop_activities_and_being_eaten(world: &mut World, this: Entity) {
    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_being_eaten(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// The stop list S34 of `removeFromStoryMode`, in the original's order.
fn stop_activities_for_story_removal(world: &mut World, this: Entity) {
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
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);
}

/// The random duration `[minTime, maxTime]` of the timed items (one `arc4random()`, in
/// hundredths of a second).
fn random_item_time(world: &mut World, call: &SequenceCall) -> f32 {
    let min_time = call.float("minTime");
    let max_time = call.float("maxTime");
    let random = world.resource_mut::<Random>().arc4random();
    pg_engine::logic::random_scaled_time(min_time, max_time, 100.0, random)
}

/// The direction rule of the timed walks (`docs/re/pygmy.md` §1.2): left at the island's right
/// end, or on a zero coin anywhere but its left end.
fn walks_left(world: &mut World, this: Entity, coin: u32) -> bool {
    let x_pos = x_pos_of(world, this);
    let listener = listener_of(world, this);

    // A nil listener answers 0 for both ends.
    let (island_min_x_pos, island_max_x_pos) = match listener {
        Some(listener) => {
            let island_min_x_pos = crate::controller::island_min_x_pos(world, listener);
            let island_max_x_pos = crate::controller::island_max_x_pos(world, listener);
            (island_min_x_pos, island_max_x_pos)
        }
        None => (0.0, 0.0),
    };

    x_pos == island_max_x_pos || (coin & 1 == 0 && x_pos != island_min_x_pos)
}

/// The first frame of `halfPygmyWalk:::` and `charlieMonkeyWalk:::`: a random walk time, island
/// side collisions and gravity on, and the facing by the walk rule. Returns whether the pygmy
/// walks left.
fn start_random_walk(world: &mut World, this: Entity, call: &SequenceCall) -> bool {
    let stall_time = random_item_time(world, call);
    pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_side_collision = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let coin = world.resource_mut::<Random>().arc4random();
    let left = walks_left(world, this, coin);
    let x_scale = if left { -1.0 } else { 1.0 };
    set_x_scale(world, this, x_scale);

    left
}

/// Every frame of the two walks: done, and stopped, at an island side or once the time is up;
/// the time counts down either way.
fn continue_random_walk(world: &mut World, this: Entity, call: &SequenceCall) -> ItemResult {
    let island_side_collision = world.obj::<PygmyPygmyCoreFrame>(this).island_side_collision;
    let stall_time = pg_engine::logic::logic(world, this).stall_time;

    let result = if island_side_collision || stall_time <= 0.0 {
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// `halfPygmyIdle:::` and `charlieMonkeyIdle:::`: wait a random `[minTime, maxTime]`; the time
/// counts down even on the frame it runs out.
fn stall_for_random_time(world: &mut World, this: Entity, call: &SequenceCall) -> ItemResult {
    if call.first_time {
        let stall_time = random_item_time(world, call);
        pg_engine::logic::logic_mut(world, this).stall_time = stall_time;
    }

    let stall_time = pg_engine::logic::logic(world, this).stall_time;
    let result = if stall_time <= 0.0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    };

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    result
}

/// The body `charlieSmackBoulder:` and `charlieSmackTRexEgg:` share: guard FREE +{mbSharkOnHead}
/// while upright, the stop list S13, then Charlie stops, faces `target` and plays `behavior`.
fn charlie_smack(world: &mut World, this: Entity, target: Entity, behavior: &str) -> bool {
    if !is_free_upright_without_shark_on_head(world, this) {
        return false;
    }

    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);
    reset_z_rot(world, this);

    let x_pos = x_pos_of(world, this);
    let target_x_pos = x_pos_of(world, target);
    let x_scale = if target_x_pos < x_pos { -1.0 } else { 1.0 };
    set_x_scale(world, this, x_scale);

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Charlie) monkeyBootedOffCharliesBack:]` @ 0x1000580f4
pub fn monkey_booted_off_charlies_back(world: &mut World, this: Entity, _monkey_logic: Entity) {
    world.obj_mut::<PygmyCharlieStory>(this).charlie_on_back_monkey_logic = None;
    pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
}

/// `-[PygmyLogic (Charlie) isCutInHalf]` @ 0x10005811c
pub fn is_cut_in_half(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).cut_in_half_by_charlie
}

/// `-[PygmyLogic (Charlie) stopCharlieMischief]` @ 0x100058130
///
/// Breaks every Charlie link: the victim tells its Charlie, Charlie tells its victim, and the
/// monkey on Charlie's back is booted off.
pub fn stop_charlie_mischief(world: &mut World, this: Entity) {
    let charlie = world.obj::<PygmyCharlieStory>(this).charlie_pygmy_logic;
    if let Some(charlie) = charlie {
        stop_charlie_mischief_with_arg(world, charlie, this);
        world.obj_mut::<PygmyCharlieStory>(this).charlie_pygmy_logic = None;
    }

    let cutting_pygmy = world.obj::<PygmyCharlieStory>(this).charlie_cutting_pygmy_logic;
    if let Some(cutting_pygmy) = cutting_pygmy {
        stop_cut_in_half_by_charlie(world, cutting_pygmy, this);
        world.obj_mut::<PygmyCharlieStory>(this).charlie_cutting_pygmy_logic = None;
    }

    let monkey = world.obj::<PygmyCharlieStory>(this).charlie_on_back_monkey_logic;
    if let Some(monkey) = monkey {
        crate::logic::ape_monkeys::monkey_logic::booted_off_charlies_back(world, monkey, this);
        world.obj_mut::<PygmyCharlieStory>(this).charlie_on_back_monkey_logic = None;
    }
}

/// `-[PygmyLogic (Charlie) hitByLightningByWarlock]` @ 0x1000581d0
///
/// `hitByLightning` with its own sacrifice, without the earthquake reset, and with only the
/// behaviour (not the sacrifice) changing in a hurricane.
pub fn hit_by_lightning_by_warlock(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let is_live_zombie = core.is_zombie && !core.is_ghost;
    let busy = is_busy_for_lightning(core);

    if is_live_zombie {
        zombie_explode(world, this);
        return true;
    }

    if busy {
        return false;
    }

    stop_activities_and_being_eaten(world, this);

    // The hurricane forgets the pygmy without calling back `droppedByHurricane:`, so
    // `mbPickedUpByHurricane` stays set.
    let hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane_logic;
    if let Some(hurricane) = hurricane {
        crate::logic::sky_powers::hurricane_logic::drop_sprite_logic(world, hurricane, this);
        world.obj_mut::<SpriteLogic>(this).picked_up_by_hurricane_logic = None;

        let listener = listener_of(world, this);
        if let Some(listener) = listener {
            crate::controller::on_pygmy_hit_by_lightning_in_hurricane(world, listener, this);
        }
    }

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.hit_by_lightning = true;
    core.tilt_state = 0;

    stop_moving(world, this);
    reset_z_rot(world, this);
    set_sacrifice_id(world, this, "ZappedByWarlockAssassin");

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let in_kong_statue = core.in_kong_statue;
    let in_ape_container = core.in_ape_wench || core.in_ape_bongo || core.in_ape_altar;

    let behavior = if in_kong_statue {
        let kong_statue = world.obj::<PygmyPygmyCoreFrame>(this).kong_statue_logic;
        if let Some(kong_statue) = kong_statue {
            crate::logic::konkeydong::kong_statue_logic::remove_pygmy_from_kong_statue(world, kong_statue, this);
        }
        world.obj_mut::<PygmyPygmyCoreFrame>(this).kong_statue_logic = None;

        "PygmyHitByLightningInKongStatue"
    } else if in_ape_container {
        // Bongo, wench, altar: each flag is read again after the previous removal.
        let in_ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_bongo;
        if in_ape_bongo {
            let ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).ape_bongo_logic;
            if let Some(ape_bongo) = ape_bongo {
                crate::logic::ape_ritual::ape_bongo_logic::remove_pygmy_from_ape_bongo(world, ape_bongo, this);
            }
            world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_bongo_logic = None;
        }

        let in_ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_wench;
        if in_ape_wench {
            let ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).ape_wench_logic;
            if let Some(ape_wench) = ape_wench {
                crate::logic::ape_ritual::ape_wench_logic::remove_pygmy_from_ape_wench(world, ape_wench, this);
            }
            world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_wench_logic = None;
        }

        let in_ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_altar;
        if in_ape_altar {
            let ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).ape_altar_logic;
            if let Some(ape_altar) = ape_altar {
                crate::logic::ape_ritual::ape_altar_logic::remove_pygmy_from_ape_altar(world, ape_altar, this);
            }
            world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_altar_logic = None;
        }

        "PygmyHitByLightningInApeSomething"
    } else {
        let picked_up_by_hurricane = world.obj::<SpriteLogic>(this).picked_up_by_hurricane;

        if picked_up_by_hurricane {
            "PygmyHitByLightningInHurricane"
        } else {
            "PygmyHitByLightning"
        }
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
    true
}

/// `-[PygmyLogic (Charlie) canBeCutInHalfByCharlie:]` @ 0x100058b80
///
/// FREE +{mbBitByBat, mbSleeping, mbSharkOnHead} while upright.
pub fn can_be_cut_in_half_by_charlie(world: &mut World, this: Entity, _charlie: Entity) -> bool {
    let bit_by_bat = world.obj::<PygmyGhostZombieVampire>(this).bit_by_bat;
    let sleeping = world.obj::<PygmyPygmyCoreFrame>(this).sleeping;

    !bit_by_bat && !sleeping && is_free_upright_without_shark_on_head(world, this)
}

/// `-[PygmyLogic (Charlie) cutInHalfByCharlie:]` @ 0x10005925c
pub fn cut_in_half_by_charlie(world: &mut World, this: Entity, charlie: Entity) {
    stop_activities(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);
    reset_z_rot(world, this);

    world.obj_mut::<PygmyCharlieStory>(this).charlie_pygmy_logic = Some(charlie);

    // `[[listener pygmyDisplayMarker] insertBefore:mDisplayObject]`.
    let listener = listener_of(world, this);
    let marker = match listener {
        Some(listener) => crate::controller::pygmy_display_marker(world, listener),
        None => None,
    };
    if let Some(marker) = marker {
        let display_object = pg_engine::logic::display_object(world, this);
        display::insert_before(world, marker, Some(display_object));
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyCutInHalfByCharlie"));
}

/// `-[PygmyLogic (Charlie) cutInHalfByCharlieComplete:]` @ 0x100059428
///
/// The legs fly off as the half-pygmy bottom; from now on the pygmy is a half pygmy.
pub fn cut_in_half_by_charlie_complete(world: &mut World, this: Entity, _charlie: Entity) {
    let explode_from_loving_pocket_god = world.obj::<PygmyPygmyCoreFrame>(this).explode_from_loving_pocket_god;
    if explode_from_loving_pocket_god {
        return;
    }

    let x_pos = x_pos_of(world, this);

    // The original reads Charlie's position through `mCharliePygmyLogic`, which every caller
    // has set; a nil Charlie would crash it (a NULL `pos`), here it reads as x = 0.
    let charlie = world.obj::<PygmyCharlieStory>(this).charlie_pygmy_logic;
    let charlie_x_pos = match charlie {
        Some(charlie) => x_pos_of(world, charlie),
        None => 0.0,
    };

    let x_vel = if x_pos < charlie_x_pos { -200.0 } else { 100.0 };
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = 700.0;

    let pygmy_bottom = pg_engine::library::logic(world, crate::logic_table::PYGMY_BOTTOM);
    crate::pygmy_extras::pygmy_bottom_logic::toss_bottom(world, pygmy_bottom, this);

    world.obj_mut::<PygmyGhostZombieVampire>(this).bit_by_bat = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<PygmyCharlieStory>(this).charlie_pygmy_logic = None;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).cut_in_half_by_charlie = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyCutInHalfByCharlieComplete"));
}

/// `-[PygmyLogic (Charlie) animCutPygmyInHalf]` @ 0x100059588
pub fn anim_cut_pygmy_in_half(world: &mut World, this: Entity) {
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if story_mode {
        return;
    }

    let cutting_pygmy = world.obj::<PygmyCharlieStory>(this).charlie_cutting_pygmy_logic;
    if let Some(cutting_pygmy) = cutting_pygmy {
        cut_in_half_by_charlie_complete(world, cutting_pygmy, this);
        world.obj_mut::<PygmyCharlieStory>(this).charlie_cutting_pygmy_logic = None;
    }
}

/// `-[PygmyLogic (Charlie) animCharlieLaunchShark]` @ 0x1000595e4
pub fn anim_charlie_launch_shark(world: &mut World, this: Entity) {
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if story_mode {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).shark_on_head = false;

    let shark = world.obj::<PygmyIslandLife>(this).eating_shark_logic;
    if let Some(shark) = shark {
        crate::logic::ocean_creatures::shark_logic::launch_off_charlie(world, shark);
    }
    world.obj_mut::<PygmyIslandLife>(this).eating_shark_logic = None;
}

/// `-[PygmyLogic (Charlie) animCharlieThrowFood]` @ 0x100059648
pub fn anim_charlie_throw_food(world: &mut World, this: Entity) {
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if story_mode {
        return;
    }

    let coconut = world.obj::<PygmyIslandLife>(this).eating_coconut_logic;
    if let Some(coconut) = coconut {
        crate::logic::island_core::coconut_logic::thrown_by_charlie(world, coconut, this);
        world.obj_mut::<PygmyIslandLife>(this).eating_coconut_logic = None;
    }

    let fish = world.obj::<PygmyIslandLife>(this).eating_fish_logic;
    if let Some(fish) = fish {
        crate::logic::ocean_fishing::fish_logic::thrown_by_charlie(world, fish, this);
        world.obj_mut::<PygmyIslandLife>(this).eating_fish_logic = None;
    }

    let bird = world.obj::<PygmyIslandLife>(this).eating_bird_logic;
    if let Some(bird) = bird {
        crate::logic::ocean_birds::bird_logic::thrown_by_charlie(world, bird, this);
        world.obj_mut::<PygmyIslandLife>(this).eating_bird_logic = None;
    }

    let banana = world.obj::<PygmyIslandLife>(this).eating_banana_logic;
    if let Some(banana) = banana {
        crate::logic::ape_monkeys::banana_logic::thrown_by_charlie(world, banana, this);
        world.obj_mut::<PygmyIslandLife>(this).eating_banana_logic = None;
    }

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FOOD_CHANNEL);
}

/// `-[PygmyLogic (Charlie) animWarlockBurnVampire]` @ 0x100059754
///
/// Burns the first vampire in the pygmy list, and skips the explosion that follows.
pub fn anim_warlock_burn_vampire(world: &mut World, this: Entity) {
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if story_mode {
        return;
    }

    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        if pygmy == this {
            continue;
        }

        let burnable = can_be_burnt_by_warlock(world, pygmy, this);
        if burnable {
            world.obj_mut::<PygmyIslandHazards>(this).warlock_skip_next_explosion = true;
            burnt_by_warlock(world, pygmy, this);
            return;
        }
    }
}

/// `-[PygmyLogic (Charlie) animWarlockStrike]` @ 0x1000598dc
///
/// Zaps the first pygmy the lightning can hit; the controller draws the bolt to it.
pub fn anim_warlock_strike(world: &mut World, this: Entity) {
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    let skip_next_explosion = world.obj::<PygmyIslandHazards>(this).warlock_skip_next_explosion;
    if story_mode || skip_next_explosion {
        return;
    }

    let pygmies = pygmy_logic_array(world, this);
    for pygmy in pygmies {
        if pygmy == this {
            continue;
        }

        let hit = hit_by_lightning_by_warlock(world, pygmy);
        if hit {
            world.obj_mut::<PygmyIslandHazards>(this).warlock_skip_next_explosion = true;

            let listener = listener_of(world, this);
            if let Some(listener) = listener {
                crate::controller::hit_pygmy_with_warlock_lightning(world, listener, pygmy);
            }
            return;
        }
    }
}

/// `-[PygmyLogic (Charlie) checkForTigerBlood]` @ 0x100059a70
///
/// Charlie cuts the first pygmy standing about 40 units in front of him in half.
pub fn check_for_tiger_blood(world: &mut World, this: Entity) -> bool {
    let display_object = pg_engine::logic::display_object(world, this);
    let pygmies = pygmy_logic_array(world, this);

    for pygmy in pygmies {
        if pygmy == this {
            continue;
        }

        let can_be_cut = can_be_cut_in_half_by_charlie(world, pygmy, this);
        if !can_be_cut {
            continue;
        }

        let object = display::object(world, display_object);
        let x_pos = object.pos.x;
        let x_scale = object.scale.x;
        let victim_x_pos = x_pos_of(world, pygmy);

        let reach_x_pos = x_pos + x_scale * 40.0;
        let distance = (reach_x_pos - victim_x_pos).abs();
        if distance < 20.0 {
            let x_scale = if victim_x_pos < x_pos { -1.0 } else { 1.0 };
            display::object_mut(world, display_object).scale.x = x_scale;

            stop_moving(world, this);
            world.obj_mut::<PygmyCharlieStory>(this).charlie_cutting_pygmy_logic = Some(pygmy);

            pg_engine::logic::set_behavior(world, this, Some("PygmyCharlieCutPygmyInHalf"));
            cut_in_half_by_charlie(world, pygmy, this);
            return true;
        }
    }

    false
}

/// `-[PygmyLogic (Charlie) canBeBurntByWarlock:]` @ 0x100059d20
///
/// Only vampires (`mbBitByBat`), FREE +{mbSharkOnHead} while upright.
pub fn can_be_burnt_by_warlock(world: &mut World, this: Entity, _warlock: Entity) -> bool {
    let bit_by_bat = world.obj::<PygmyGhostZombieVampire>(this).bit_by_bat;

    bit_by_bat && is_free_upright_without_shark_on_head(world, this)
}

/// `-[PygmyLogic (Charlie) burntByWarlock:]` @ 0x10005a3e8
pub fn burnt_by_warlock(world: &mut World, this: Entity, _warlock: Entity) {
    stop_activities(world, this);

    set_sacrifice_id(world, this, "BurntByWarlockAssassin");
    pg_engine::logic::set_behavior(world, this, Some("PygmyVampireBurntByWarlock"));
}

/// `-[PygmyLogic (Charlie) canBeAbsorbedByHalfPygmy:]` @ 0x10005a534
pub fn can_be_absorbed_by_half_pygmy(world: &mut World, this: Entity, _half_pygmy: Entity) -> bool {
    is_free_upright_without_shark_on_head(world, this)
}

/// `-[PygmyLogic (Charlie) absorbedByHalfPygmy:]` @ 0x10005abe8
pub fn absorbed_by_half_pygmy(world: &mut World, this: Entity, _half_pygmy: Entity) {
    stop_activities(world, this);
    stop_moving(world, this);

    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    let behavior = if personality == PERSONALITY_CHARLIE {
        "PygmyCharlieAbsorbedByHalfPygmy"
    } else {
        world.obj_mut::<PygmyCharlieStory>(this).absorbed_by_half_pygmy = true;
        set_sacrifice_id(world, this, "AbsorbedByHalfPygmy");
        "PygmyAbsorbedByHalfPygmy"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Charlie) checkToAbsorbPygmyDuringFall]` @ 0x10005ad8c
///
/// A falling half pygmy (not dropped by a hurricane) that comes down on another pygmy's head
/// merges with it.
pub fn check_to_absorb_pygmy_during_fall(world: &mut World, this: Entity) -> bool {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let cut_in_half_by_charlie = core.cut_in_half_by_charlie;
    let dropped_by_hurricane = core.dropped_by_hurricane;
    if !cut_in_half_by_charlie || dropped_by_hurricane {
        return false;
    }

    let display_object = pg_engine::logic::display_object(world, this);
    let pygmies = pygmy_logic_array(world, this);

    for pygmy in pygmies {
        if pygmy == this {
            continue;
        }

        let can_be_absorbed = can_be_absorbed_by_half_pygmy(world, pygmy, this);
        if !can_be_absorbed {
            continue;
        }

        let y_vel = pg_engine::logic::logic(world, this).pos_vel.y;
        let moving_down = y_vel < 0.0;
        if !moving_down {
            continue;
        }

        let pos = display::object(world, display_object).pos;
        let pygmy_display_object = pg_engine::logic::display_object(world, pygmy);
        let pygmy_object = display::object(world, pygmy_display_object);
        let pygmy_pos = pygmy_object.pos;
        let pygmy_rot = pygmy_object.rot;
        let pygmy_scale = pygmy_object.scale;

        let x_distance = (pos.x - pygmy_pos.x).abs();
        let head_y_pos = pygmy_pos.y + 32.0;
        let y_distance = (pos.y - head_y_pos).abs();
        if x_distance < 20.0 && y_distance < 20.0 {
            stop_moving(world, this);

            let mut object = display::object_mut(world, display_object);
            object.pos = pygmy_pos;
            object.rot = pygmy_rot;
            object.scale = pygmy_scale;

            let personality = persona_personality(world, pygmy);
            let behavior = if personality == PERSONALITY_CHARLIE {
                set_sacrifice_id(world, this, "AbsorbedByCharlie");
                "HalfPygmyAbsorbCharlie"
            } else {
                // The half pygmy becomes whole again.
                world.obj_mut::<PygmyPygmyCoreFrame>(this).cut_in_half_by_charlie = false;
                "HalfPygmyAbsorbPygmy"
            };

            pg_engine::logic::set_behavior(world, this, Some(behavior));
            absorbed_by_half_pygmy(world, pygmy, this);
            return true;
        }
    }

    false
}

/// `-[PygmyLogic (Charlie) stopCharlieMischief:]` @ 0x10005b0dc
///
/// Charlie forgets `victim` if he is still cutting it.
pub fn stop_charlie_mischief_with_arg(world: &mut World, this: Entity, victim: Entity) {
    let cutting_pygmy = world.obj::<PygmyCharlieStory>(this).charlie_cutting_pygmy_logic;
    if cutting_pygmy == Some(victim) {
        world.obj_mut::<PygmyCharlieStory>(this).charlie_cutting_pygmy_logic = None;
    }
}

/// `-[PygmyLogic (Charlie) stopCutInHalfByCharlie:]` @ 0x10005b0fc
pub fn stop_cut_in_half_by_charlie(world: &mut World, this: Entity, charlie: Entity) {
    let charlie_pygmy = world.obj::<PygmyCharlieStory>(this).charlie_pygmy_logic;
    let explode_from_loving_pocket_god = world.obj::<PygmyPygmyCoreFrame>(this).explode_from_loving_pocket_god;

    if charlie_pygmy == Some(charlie) && !explode_from_loving_pocket_god {
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
        world.obj_mut::<PygmyCharlieStory>(this).charlie_pygmy_logic = None;
    }
}

/// `-[PygmyLogic (Charlie) charlieSmackBoulder:]` @ 0x10005b164
pub fn charlie_smack_boulder(world: &mut World, this: Entity, boulder_logic: Entity) -> bool {
    charlie_smack(world, this, boulder_logic, "PygmyCharlieSmackBoulder")
}

/// `-[PygmyLogic (Charlie) charlieSmackTRexEgg:]` @ 0x10005ba0c
pub fn charlie_smack_t_rex_egg(world: &mut World, this: Entity, t_rex_egg_logic: Entity) -> bool {
    charlie_smack(world, this, t_rex_egg_logic, "PygmyCharlieSmackTRexEgg")
}

/// `-[PygmyLogic (Charlie) checkForWarlock:::]` @ 0x10005c2b4
///
/// The warlock idle goes on while a vampire can be burnt, or while the lightning may be used and
/// some pygmy can be hit by it; otherwise Charlie finishes his idle.
pub fn seq_check_for_warlock(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let pygmies = pygmy_logic_array(world, this);

    // Every other pygmy is asked both questions.
    let mut any_burnable = false;
    let mut any_hittable = false;
    for pygmy in pygmies {
        if pygmy == this {
            continue;
        }

        let burnable = can_be_burnt_by_warlock(world, pygmy, this);
        let hittable = can_be_hit_by_lightning(world, pygmy);
        any_burnable |= burnable;
        any_hittable |= hittable;
    }

    if any_burnable {
        return ItemResult::Done;
    }

    let listener = listener_of(world, this);
    let can_use_lightning = match listener {
        Some(listener) => crate::controller::can_warlock_use_lightning(world, listener, this),
        None => false,
    };
    if !can_use_lightning || !any_hittable {
        pg_engine::logic::set_behavior(world, this, Some("FinishPygmyCharlieIdle"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Charlie) halfPygmyWalk:::]` @ 0x10005c484
pub fn seq_half_pygmy_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        // Only the facing is set: the walk animation moves the half pygmy.
        start_random_walk(world, this, call);
    }

    continue_random_walk(world, this, call)
}

/// `-[PygmyLogic (Charlie) halfPygmyIdle:::]` @ 0x10005c68c
pub fn seq_half_pygmy_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    stall_for_random_time(world, this, call)
}

/// `-[PygmyLogic (Charlie) charlieMonkeyWalk:::]` @ 0x10005c78c
pub fn seq_charlie_monkey_walk(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let left = start_random_walk(world, this, call);
        let x_vel = if left { -50.0 } else { 50.0 };
        pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;
    }

    continue_random_walk(world, this, call)
}

/// `-[PygmyLogic (Charlie) charlieMonkeyIdle:::]` @ 0x10005c9bc
pub fn seq_charlie_monkey_idle(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    stall_for_random_time(world, this, call)
}

/// `-[PygmyLogic (Charlie) charlieCutPygmyInHalfComplete:::]` @ 0x10005cabc
pub fn seq_charlie_cut_pygmy_in_half_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Done
}

/// `-[PygmyLogic (Story) openWordBalloon:]` @ 0x100174250
///
/// The balloon is created once (it follows the pygmy every frame) and reopened with `text`.
pub fn open_word_balloon(world: &mut World, this: Entity, text: Option<&str>) {
    let word_balloon_logic = world.obj::<PygmyCharlieStory>(this).word_balloon_logic;
    if word_balloon_logic.is_none() {
        let word_balloon = display::multi_model::spawn(world, Some(WORD_BALLOON_ID));
        let word_balloon_logic = pg_engine::logic::multi_model::spawn(world, word_balloon);
        world.obj_mut::<PygmyCharlieStory>(this).word_balloon_logic = Some(word_balloon_logic);
        pg_engine::logic::multi_model::set_multi_model_logic_listener(world, word_balloon_logic, Some(this));

        let listener = listener_of(world, this);
        let marker = match listener {
            Some(listener) => crate::controller::pygmy_drag_display_marker(world, listener),
            None => None,
        };
        if let Some(marker) = marker {
            display::insert_after(world, marker, Some(word_balloon));
        }
    }

    let dialogue_line = world.resource::<Library>().dynamic_text(DIALOGUE_LINE_TEXT_ID);
    if let Some(dialogue_line) = dialogue_line {
        display::text::set_text(world, dialogue_line, Some(""));
        display::text::set_wrap_width(world, dialogue_line, DIALOGUE_LINE_WRAP_WIDTH);
        display::text::set_text(world, dialogue_line, text);
    }

    let word_balloon_logic = world.obj::<PygmyCharlieStory>(this).word_balloon_logic;
    if let Some(word_balloon_logic) = word_balloon_logic {
        pg_engine::logic::set_behavior(world, word_balloon_logic, Some("StoryWordBalloonOpenRight"));
    }
}

/// `-[PygmyLogic (Story) openApeKidnapWordBalloon]` @ 0x1001743e4
pub fn open_ape_kidnap_word_balloon(world: &mut World, this: Entity) {
    open_word_balloon(world, this, Some("I'll save you!"));
}

/// `-[PygmyLogic (Story) closeWordBalloon]` @ 0x1001743fc
///
/// The closing animation ends with the `WordBalloonCloseComplete` event, whose handler calls
/// `closeWordBalloonNow`.
pub fn close_word_balloon(world: &mut World, this: Entity) {
    let word_balloon_logic = world.obj::<PygmyCharlieStory>(this).word_balloon_logic;
    if let Some(word_balloon_logic) = word_balloon_logic {
        pg_engine::logic::set_behavior(world, word_balloon_logic, Some("StoryWordBalloonCloseRight"));
    }
}

/// `-[PygmyLogic (Story) closeWordBalloonNow]` @ 0x10017442c
pub fn close_word_balloon_now(world: &mut World, this: Entity) {
    let word_balloon_logic = world.obj::<PygmyCharlieStory>(this).word_balloon_logic;
    let Some(word_balloon_logic) = word_balloon_logic else {
        return;
    };

    pg_engine::logic::shutdown(world, word_balloon_logic);
    let word_balloon = pg_engine::logic::display_object(world, word_balloon_logic);
    display::remove_from_display_group(world, word_balloon);

    // `[mWordBalloonLogic release]`: `shutdown` made the animation drop its reference, so the
    // pygmy held the last one.
    pg_engine::logic::dealloc(world, word_balloon_logic);
    world.obj_mut::<PygmyCharlieStory>(this).word_balloon_logic = None;
}

/// `-[PygmyLogic (Story) openWordBalloon:::]` @ 0x1001744a8
pub fn seq_open_word_balloon(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let message = call.param("message");
    open_word_balloon(world, this, message);
    ItemResult::Done
}

/// `-[PygmyLogic (Story) closeWordBalloon:::]` @ 0x1001744d4
pub fn seq_close_word_balloon(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    close_word_balloon(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Story) isReadyForStoryMode]` @ 0x1001744f8
pub fn is_ready_for_story_mode(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyCharlieStory>(this).story_mode_ready
}

/// `-[PygmyLogic (Story) setStoryMode]` @ 0x10017450c
pub fn set_story_mode(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyCharlieStory>(this).story_mode = true;
}

/// `-[PygmyLogic (Story) startStoryEarthquake]` @ 0x100174524
pub fn start_story_earthquake(world: &mut World, this: Entity) {
    stop_moving(world, this);

    let coin = world.resource_mut::<Random>().arc4random();
    let behavior = if coin & 1 == 0 {
        "PygmyLoseBalance1"
    } else {
        "PygmyLoseBalance2"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Story) stopStoryEarthquake]` @ 0x100174580
pub fn stop_story_earthquake(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
}

/// `-[PygmyLogic (Story) setStoryModeCharacterState:]` @ 0x100174598
///
/// `state` is the story element's character state (`Idle`, `Happy`, `KnockOffIsland`, …).
pub fn set_story_mode_character_state(world: &mut World, this: Entity, state: Option<&str>) {
    stop_moving(world, this);

    // `%@` prints a nil state as "(null)".
    let state = state.unwrap_or("(null)");
    let behavior = format!("PygmyStoryMode{state}");
    pg_engine::logic::set_behavior(world, this, Some(&behavior));
}

/// `-[PygmyLogic (Story) resumeFromStoryModeCharacterState]` @ 0x100174608
///
/// A pygmy dying in the story dies for good once its story death has completed.
pub fn resume_from_story_mode_character_state(world: &mut World, this: Entity) {
    let die_in_story = world.obj::<PygmyCharlieStory>(this).die_in_story;
    if !die_in_story {
        set_story_mode_character_state(world, this, Some("Idle"));
        return;
    }

    world.obj_mut::<PygmyCharlieStory>(this).die_in_story_resume = true;

    let die_in_story_complete = world.obj::<PygmyCharlieStory>(this).die_in_story_complete;
    if die_in_story_complete {
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
    }
}

/// `-[PygmyLogic (Story) removeFromStoryMode]` @ 0x100174678
///
/// Releases the pygmy from everything holding it (the containers keep their flags) and lets it
/// die; `mbRemoveFromStoryMode` keeps that death from becoming a ghost.
pub fn remove_from_story_mode(world: &mut World, this: Entity) {
    stop_activities_for_story_removal(world, this);

    let listener = listener_of(world, this);

    let doodle_jumping = world.obj::<PygmyPygmyCoreFrame>(this).doodle_jumping;
    if doodle_jumping {
        world.obj_mut::<PygmyMinigames>(this).doodle_on_ice = false;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.doodle_jumping = false;
        core.doodle_knocked_down = false;

        if let Some(listener) = listener {
            crate::controller::on_pygmy_stop_doodle_jump(world, listener, this);
        }
    }

    let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
    if let Some(out_house) = crapping_out_house {
        crate::logic::island_objects::out_house_logic::stop_crapping(world, out_house, this);
        world.obj_mut::<PygmyIslandLife>(this).crapping_out_house_logic = None;
    }

    let eaten_by_t_rex = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_t_rex;
    if eaten_by_t_rex {
        let t_rex = match listener {
            Some(listener) => crate::controller::t_rex_logic(world, listener, this),
            None => None,
        };
        crate::send::clear_pygmy_logic(world, t_rex, this);
    }

    let eaten_by_spider = world.obj::<PygmyPygmyCoreFrame>(this).eaten_by_spider;
    if eaten_by_spider {
        let spider = match listener {
            Some(listener) => crate::controller::spider_logic(world, listener, this),
            None => None,
        };
        crate::send::clear_pygmy_logic(world, spider, this);
    }

    let in_kong_statue = world.obj::<PygmyPygmyCoreFrame>(this).in_kong_statue;
    if in_kong_statue {
        let kong_statue = world.obj::<PygmyPygmyCoreFrame>(this).kong_statue_logic;
        if let Some(kong_statue) = kong_statue {
            crate::logic::konkeydong::kong_statue_logic::remove_pygmy_from_kong_statue(world, kong_statue, this);
        }
        world.obj_mut::<PygmyPygmyCoreFrame>(this).kong_statue_logic = None;
    }

    let in_ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_bongo;
    if in_ape_bongo {
        let ape_bongo = world.obj::<PygmyPygmyCoreFrame>(this).ape_bongo_logic;
        if let Some(ape_bongo) = ape_bongo {
            crate::logic::ape_ritual::ape_bongo_logic::remove_pygmy_from_ape_bongo(world, ape_bongo, this);
        }
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_bongo_logic = None;
    }

    let in_ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_wench;
    if in_ape_wench {
        let ape_wench = world.obj::<PygmyPygmyCoreFrame>(this).ape_wench_logic;
        if let Some(ape_wench) = ape_wench {
            crate::logic::ape_ritual::ape_wench_logic::remove_pygmy_from_ape_wench(world, ape_wench, this);
        }
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_wench_logic = None;
    }

    let in_ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).in_ape_altar;
    if in_ape_altar {
        let ape_altar = world.obj::<PygmyPygmyCoreFrame>(this).ape_altar_logic;
        if let Some(ape_altar) = ape_altar {
            crate::logic::ape_ritual::ape_altar_logic::remove_pygmy_from_ape_altar(world, ape_altar, this);
        }
        world.obj_mut::<PygmyPygmyCoreFrame>(this).ape_altar_logic = None;
    }

    let in_go_go_cage = world.obj::<PygmyPygmyCoreFrame>(this).in_go_go_cage;
    if in_go_go_cage {
        let go_go_cage = world.obj::<PygmyContainersAndCatchers>(this).go_go_cage_logic;
        if let Some(go_go_cage) = go_go_cage {
            crate::logic::apocalypse_dance::go_go_cage_logic::remove_pygmy_from_go_go_cage(world, go_go_cage, this);
        }
        world.obj_mut::<PygmyContainersAndCatchers>(this).go_go_cage_logic = None;
    }

    let in_dance_dance_execution = world.obj::<PygmyPygmyCoreFrame>(this).in_dance_dance_execution;
    if in_dance_dance_execution {
        let dance_dance_execution = world.obj::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic;
        if let Some(dance_dance_execution) = dance_dance_execution {
            crate::logic::apocalypse_dance::dance_dance_execution_logic::remove_pygmy_from_dance_dance_execution(
                world,
                dance_dance_execution,
                this,
            );
        }
        world.obj_mut::<PygmyContainersAndCatchers>(this).dance_dance_execution_logic = None;
    }

    let crushed = world.obj::<PygmyPygmyCoreFrame>(this).crushed;
    if crushed {
        let crushed_by_boulder = world.obj::<PygmyIslandHazards>(this).crushed_by_boulder;
        let crusher = match listener {
            Some(listener) if crushed_by_boulder => crate::controller::boulder_logic(world, listener, this),
            Some(listener) => crate::controller::t_rex_egg_logic(world, listener, this),
            None => None,
        };
        crate::send::clear_pygmy_logic(world, crusher, this);
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).remove_from_story_mode = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
}

/// `-[PygmyLogic (Story) storyModeKnockOffIsland:::]` @ 0x100174c58
pub fn seq_story_mode_knock_off_island(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

    // x speed in [-200, 200], in hundredths.
    let random = world.resource_mut::<Random>().arc4random_mod(40001);
    let x_vel_hundredths = (random as i32).wrapping_sub(20000);
    let x_vel = x_vel_hundredths as f32 / 100.0;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = 600.0;

    reset_z_rot(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Story) storyModeFlick:::]` @ 0x100174d10
pub fn seq_story_mode_flick(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    // x speed in [-100, 100], in hundredths.
    let random = world.resource_mut::<Random>().arc4random_mod(20001);
    let x_vel_hundredths = (random as i32).wrapping_sub(10000);
    let x_vel = x_vel_hundredths as f32 / 100.0;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = x_vel;
    logic.pos_vel.y = 1200.0;
    logic.pos_vel.z = -400.0;

    let display_object = pg_engine::logic::display_object(world, this);
    let pos = display::object(world, display_object).pos;
    world.obj_mut::<PygmyPygmyBehaviourItems>(this).tumble_pos = pos;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.falling = true;
    core.drowning = false;

    ItemResult::Done
}

/// `-[PygmyLogic (Story) dieInStory:::]` @ 0x100174de4
pub fn seq_die_in_story(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut story = world.obj_mut::<PygmyCharlieStory>(this);
    story.die_in_story = true;
    story.story_mode_ready = false;

    ItemResult::Done
}

/// `-[PygmyLogic (Story) dieInStoryComplete:::]` @ 0x100174e10
///
/// The story death is over; once the story has resumed the pygmy, it dies for good.
pub fn seq_die_in_story_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyCharlieStory>(this).die_in_story_complete = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).disable_labels = true;

    let die_in_story_resume = world.obj::<PygmyCharlieStory>(this).die_in_story_resume;
    if die_in_story_resume {
        pg_engine::logic::set_behavior(world, this, Some("PygmyDie"));
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Story) checkForStoryMode:::]` @ 0x100174e74
pub fn seq_check_for_story_mode(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let story_mode = world.obj::<PygmyCharlieStory>(this).story_mode;
    if !story_mode {
        return ItemResult::Done;
    }

    world.obj_mut::<PygmyCharlieStory>(this).story_mode_ready = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyStoryModeIdle"));
    ItemResult::Replaced
}
