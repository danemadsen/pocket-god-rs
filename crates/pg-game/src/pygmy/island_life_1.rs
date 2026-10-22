//! `PygmyLogic`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::ape_monkeys::{banana_logic, monkey_logic};
use crate::logic::ice_island::ice_hole_logic;
use crate::logic::island_core::sprite_logic::{self, SpriteLogic};
use crate::logic::island_core::{bon_fire_logic, coconut_logic, wood_logic};
use crate::logic::island_objects::{out_house_logic, spear_logic};
use crate::logic::konkeydong::kong_statue_logic;
use crate::logic::misfit_oogajump::dream_bubble_logic;
use crate::logic::ocean_birds::bird_logic;
use crate::logic::ocean_creatures::{shark_logic, sword_fish_logic};
use crate::logic::ocean_fishing::{fish_logic, fishing_bob_logic, fishing_pole_logic};
use crate::logic::sky_powers::rainbow_logic;
use crate::managers::{idol_manager, skin_pack_manager};

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyIslandLife {
    /// `mEatingSharkLogic`: `@"SharkLogic"`
    pub eating_shark_logic: Option<Entity>,

    /// `mEatingCoconutLogic`: `@"CoconutLogic"`
    pub eating_coconut_logic: Option<Entity>,

    /// `mCarryingWoodLogic`: `@"WoodLogic"`
    pub carrying_wood_logic: Option<Entity>,

    /// `mEatingFishLogic`: `@"FishLogic"`
    pub eating_fish_logic: Option<Entity>,

    /// `mEatingSwordFishLogic`: `@"SwordFishLogic"`
    pub eating_sword_fish_logic: Option<Entity>,

    /// `mEatingBirdLogic`: `@"BirdLogic"`
    pub eating_bird_logic: Option<Entity>,

    /// `mEatingBananaLogic`: `@"BananaLogic"`
    pub eating_banana_logic: Option<Entity>,

    /// `mFishingPoleLogic`: `@"FishingPoleLogic"`
    pub fishing_pole_logic: Option<Entity>,

    /// `mbThrowingSpear`: `B`
    pub throwing_spear: bool,

    /// `mThrowingSpearLogic`: `@"SpearLogic"`
    pub throwing_spear_logic: Option<Entity>,

    /// `mFollowOutHouseStall`: `i`
    pub follow_out_house_stall: i32,

    /// `mCrappingOutHouseLogic`: `@"OutHouseLogic"`
    pub crapping_out_house_logic: Option<Entity>,

    /// `mWaitingOutHouseLogic`: `@"OutHouseLogic"`
    pub waiting_out_house_logic: Option<Entity>,

    /// `mLeftWarmingBonFireLogic`: `@"BonFireLogic"`
    pub left_warming_bon_fire_logic: Option<Entity>,

    /// `mRightWarmingBonFireLogic`: `@"BonFireLogic"`
    pub right_warming_bon_fire_logic: Option<Entity>,

    /// `mThrowingMonkeyLogic`: `@"MonkeyLogic"`
    pub throwing_monkey_logic: Option<Entity>,

    /// `mbPlayingCatchWithMonkey`: `B`
    pub playing_catch_with_monkey: bool,

    /// `mHumpingMonkeyLogic`: `@"MonkeyLogic"`
    pub humping_monkey_logic: Option<Entity>,

    /// `mDreamBubbleLogic`: `@"DreamBubbleLogic"`
    pub dream_bubble_logic: Option<Entity>,

    /// `mbCharlieNotSleeping`: `B`
    pub charlie_not_sleeping: bool,

    /// `mbIceFishing`: `B`
    pub ice_fishing: bool,

    /// `mbCrawledOutOfIgloo`: `B`
    pub crawled_out_of_igloo: bool,

    /// `mbStaringAtRainbow`: `B`
    pub staring_at_rainbow: bool,

    /// `mbGushingOverRainbow`: `B`
    pub gushing_over_rainbow: bool,

    /// `mbStaringAtKidnapping`: `B`
    pub staring_at_kidnapping: bool,

    /// `mbStoppingDance`: `B`
    pub stopping_dance: bool,

    /// `mbIsThemeSongBackup`: `B`
    pub is_theme_song_backup: bool,

    /// `mLastDanceSkinOptionValueId`: `@"NSString"`
    pub last_dance_skin_option_value_id: Option<String>,

    /// `mbFinishedCrapping`: `B`
    pub finished_crapping: bool,
}

/// File-scope statics of `PygmyLogic(Island)` used by this part.
#[derive(Resource, Default)]
pub struct PygmyIslandLifeStatics {
    /// `DAT_1007e7bd8`: the rainbow animations `chooseRainbowAnimation:::` still has to play,
    /// shared by every pygmy (an `NSMutableArray` refilled from the item's list when empty).
    pub rainbow_animation_pool: Vec<String>,
}

/// Tweenable channel of the food being eaten (coconut, fish, sword fish, bird, banana).
const FOOD_CHANNEL: usize = 14;

/// Tweenable channel of the carried wood.
const WOOD_CHANNEL: usize = 6;

/// Tweenable channel of the spear being thrown.
const SPEAR_CHANNEL: usize = 5;

/// Tweenable channels of the fishing pole, its bob and its string.
const FISHING_POLE_CHANNEL: usize = 2;
const FISHING_BOB_CHANNEL: usize = 3;
const FISHING_STRING_CHANNEL: usize = 4;

/// The busy set FREE of `docs/re/pygmy.md` §1.3 without its pygmy-type, `mbIsZombie` and
/// `mbFalling` terms, which the guards of this file add back as they need them: true when the
/// pygmy is in none of the other 80 states. Pure reads, so the order of the original's checks
/// does not matter.
fn is_free_base(world: &World, this: Entity) -> bool {
    let sprite = world.obj::<SpriteLogic>(this);
    if sprite.blown_by_hurricane || sprite.picked_up_by_hurricane {
        return false;
    }

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let busy = core.touch_began_inside
        || core.remove_from_story_mode
        || core.cut_in_half_by_charlie
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
        || core.in_kong_statue
        || core.in_ape_wench
        || core.in_ape_bongo
        || core.in_ape_altar
        || core.in_go_go_cage
        || core.in_dance_dance_execution
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
        || core.drowning;

    !busy
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

/// `mPosVel.x = mPosVel.y = 0` (z and w keep their values).
fn stop_moving(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `[mDisplayObject setZRot:0]`.
fn reset_z_rot(world: &mut World, this: Entity) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).rot.z = 0.0;
}

/// `[mDisplayObject setXScale:x_scale]`.
fn set_x_scale(world: &mut World, this: Entity, x_scale: f32) {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).scale.x = x_scale;
}

/// `[mPygmyLogicListener onPygmyGiftGiven:self giftId:gift_id]`.
fn give_gift(world: &mut World, this: Entity, gift_id: &str) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_pygmy_gift_given(world, listener, this, gift_id);
    }
}

/// `[mPygmyLogicListener monkeyLogicArray:self]` (nil listener: no monkeys).
fn monkey_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    match listener {
        Some(listener) => crate::controller::monkey_logic_array(world, listener, this),
        None => Vec::new(),
    }
}

/// `[[mPygmyLogicListener pygmyDisplayMarker] insertAfter:mDisplayObject]`: back into the
/// pygmies' own layer.
fn insert_after_pygmy_display_marker(world: &mut World, this: Entity) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let marker = listener.and_then(|listener| crate::controller::pygmy_display_marker(world, listener));

    if let Some(marker) = marker {
        let display = pg_engine::logic::display_object(world, this);
        pg_engine::display::insert_after(world, marker, Some(display));
    }
}

/// `[[mPygmyLogicListener pygmySmallObjectsDisplayMarker] insertBefore:mDisplayObject]`: the
/// pygmy is drawn just below the small objects (the shark or the monkey on it).
fn insert_before_pygmy_small_objects_display_marker(world: &mut World, this: Entity) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let marker = listener.and_then(|listener| crate::controller::pygmy_small_objects_display_marker(world, listener));

    if let Some(marker) = marker {
        let display = pg_engine::logic::display_object(world, this);
        pg_engine::display::insert_before(world, marker, Some(display));
    }
}

/// `[[mPygmyLogicListener iceHoleLogic:self] stopFishing:self]`.
fn stop_ice_hole_fishing(world: &mut World, this: Entity) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let ice_hole = listener.and_then(|listener| crate::controller::ice_hole_logic(world, listener, this));

    if let Some(ice_hole) = ice_hole {
        ice_hole_logic::stop_fishing(world, ice_hole, this);
    }
}

/// `[[mPygmyLogicListener rainbowLogic:self] gushOverRainbowComplete:self]`.
fn finish_gushing_over_rainbow(world: &mut World, this: Entity) {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let rainbow = listener.and_then(|listener| crate::controller::rainbow_logic(world, listener, this));

    if let Some(rainbow) = rainbow {
        rainbow_logic::gush_over_rainbow_complete(world, rainbow, this);
    }
}

/// The ground reset shared by the three fishing stops: gravity and floor collision back on,
/// `mLastPos.y = mIslandYPos + 2`, `[mDisplayObject setYPos:mIslandYPos - 1]`.
fn put_back_on_island_after_fishing(world: &mut World, this: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = true;
    let island_y_pos = core.island_y_pos;
    core.last_pos.y = island_y_pos + 2.0;

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).pos.y = island_y_pos - 1.0;
}

/// One food of `stopEating`: release it from the food channel at once, then drop it.
fn drop_eaten_food(world: &mut World, this: Entity, food: Entity, drop_while_eating: fn(&mut World, Entity)) {
    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FOOD_CHANNEL);

    drop_while_eating(world, food);
}

/// `[[SkinPackManager instance] getSkinPackOptionValueBySkinPackId:@"Dance3"
/// skinObjectId:@"Pygmies" skinOptionId:@"Pygmy Dance"]`: the id of the chosen pygmy dance.
fn pygmy_dance_option_value_id(world: &mut World) -> Option<String> {
    let manager = skin_pack_manager::instance(world)?;
    skin_pack_manager::get_skin_pack_option_value_by_skin_pack_id(world, manager, "Dance3", "Pygmies", "Pygmy Dance")
}

/// `-[PygmyLogic (Island) hideFood]` @ 0x100094b08
pub fn hide_food(world: &mut World, this: Entity) {
    let blank_texture = pg_engine::library::texture_def_id(world, "BlankTexture");

    let island_life = world.obj::<PygmyIslandLife>(this);
    let foods = [
        island_life.eating_coconut_logic,
        island_life.eating_fish_logic,
        island_life.eating_sword_fish_logic,
        island_life.eating_bird_logic,
        island_life.eating_banana_logic,
    ];

    for food in foods.into_iter().flatten() {
        let food_display = pg_engine::logic::display_object(world, food);
        pg_engine::display::set_texture_def(world, food_display, blank_texture);
    }
}

/// `-[PygmyLogic (Island) animThrowSpear]` @ 0x100094c78
pub fn anim_throw_spear(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyIslandLife>(this).throwing_spear = false;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, SPEAR_CHANNEL);

    let spear = world.obj::<PygmyIslandLife>(this).throwing_spear_logic;
    if let Some(spear) = spear {
        spear_logic::throw_spear(world, spear);
    }

    world.obj_mut::<PygmyIslandLife>(this).throwing_spear_logic = None;
}

/// `-[PygmyLogic (Island) animInitBob]` @ 0x100094cec
pub fn anim_init_bob(world: &mut World, this: Entity) {
    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if let Some(pole) = pole {
        fishing_pole_logic::pickup_pole(world, pole);
    }

    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    let bob = pole.and_then(|pole| fishing_pole_logic::fishing_bob_logic(world, pole));
    if let Some(bob) = bob {
        fishing_bob_logic::init_bob_texture(world, bob);
    }
}

/// `-[PygmyLogic (Island) stopMonkeyHumping]` @ 0x100094d48
pub fn stop_monkey_humping(world: &mut World, this: Entity) {
    let monkey = world.obj::<PygmyIslandLife>(this).humping_monkey_logic;
    if let Some(monkey) = monkey {
        monkey_logic::stop_humping_pygmy(world, monkey);
        world.obj_mut::<PygmyIslandLife>(this).humping_monkey_logic = None;
    }
}

/// `-[PygmyLogic (Island) playingCatchWithMonkey]` @ 0x100094d8c
pub fn playing_catch_with_monkey(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyIslandLife>(this).playing_catch_with_monkey
}

/// `-[PygmyLogic (Island) stopPlayingCatchWithMonkey]` @ 0x100094da0
pub fn stop_playing_catch_with_monkey(world: &mut World, this: Entity) {
    let playing_catch = world.obj::<PygmyIslandLife>(this).playing_catch_with_monkey;
    if !playing_catch {
        return;
    }

    world.obj_mut::<PygmyIslandLife>(this).playing_catch_with_monkey = false;

    let monkeys = monkey_logic_array(world, this);
    for monkey in monkeys {
        monkey_logic::stop_playing_catch(world, monkey);
    }
}

/// `-[PygmyLogic (Island) dreamBubbleSelected]` @ 0x100094ee0
pub fn dream_bubble_selected(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyIslandLife>(this).dream_bubble_logic = None;
}

/// `-[PygmyLogic (Island) stopDreamBubble]` @ 0x100094ef4
pub fn stop_dream_bubble(world: &mut World, this: Entity) {
    let dream_bubble = world.obj::<PygmyIslandLife>(this).dream_bubble_logic;
    if let Some(dream_bubble) = dream_bubble {
        dream_bubble_logic::deactivate(world, dream_bubble);
        world.obj_mut::<PygmyIslandLife>(this).dream_bubble_logic = None;
    }
}

/// `-[PygmyLogic (Island) stopSleeping]` @ 0x100094f38
pub fn stop_sleeping(world: &mut World, this: Entity) {
    let sleeping = world.obj::<PygmyPygmyCoreFrame>(this).sleeping;
    if !sleeping {
        return;
    }

    stop_monkey_humping(world, this);
    stop_dream_bubble(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sleeping = false;

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_pygmy_sleeping_stop(world, listener, this);
    }
}

/// `-[PygmyLogic (Island) stopBeingEaten]` @ 0x100094fc0
pub fn stop_being_eaten(world: &mut World, this: Entity) {
    let shark_on_head = world.obj::<PygmyPygmyCoreFrame>(this).shark_on_head;
    if !shark_on_head {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).shark_on_head = false;

    let shark = world.obj::<PygmyIslandLife>(this).eating_shark_logic;
    if let Some(shark) = shark {
        shark_logic::let_go_of_pygmy(world, shark);
    }

    world.obj_mut::<PygmyIslandLife>(this).eating_shark_logic = None;
}

/// `-[PygmyLogic (Island) stopEating]` @ 0x100095018
pub fn stop_eating(world: &mut World, this: Entity) {
    let coconut = world.obj::<PygmyIslandLife>(this).eating_coconut_logic;
    if let Some(coconut) = coconut {
        drop_eaten_food(world, this, coconut, coconut_logic::drop_while_eating);
        world.obj_mut::<PygmyIslandLife>(this).eating_coconut_logic = None;
    }

    let fish = world.obj::<PygmyIslandLife>(this).eating_fish_logic;
    if let Some(fish) = fish {
        drop_eaten_food(world, this, fish, fish_logic::drop_while_eating);
        world.obj_mut::<PygmyIslandLife>(this).eating_fish_logic = None;
    }

    let sword_fish = world.obj::<PygmyIslandLife>(this).eating_sword_fish_logic;
    if let Some(sword_fish) = sword_fish {
        drop_eaten_food(world, this, sword_fish, sword_fish_logic::drop_while_eating);
        world.obj_mut::<PygmyIslandLife>(this).eating_sword_fish_logic = None;
    }

    let bird = world.obj::<PygmyIslandLife>(this).eating_bird_logic;
    if let Some(bird) = bird {
        drop_eaten_food(world, this, bird, bird_logic::drop_while_eating);
        world.obj_mut::<PygmyIslandLife>(this).eating_bird_logic = None;
    }

    let banana = world.obj::<PygmyIslandLife>(this).eating_banana_logic;
    if let Some(banana) = banana {
        drop_eaten_food(world, this, banana, banana_logic::drop_while_eating);
        world.obj_mut::<PygmyIslandLife>(this).eating_banana_logic = None;
    }
}

/// `-[PygmyLogic (Island) stopCarrying]` @ 0x1000951c4
pub fn stop_carrying(world: &mut World, this: Entity) {
    let wood = world.obj::<PygmyIslandLife>(this).carrying_wood_logic;
    let Some(wood) = wood else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, WOOD_CHANNEL);

    wood_logic::drop_while_carrying(world, wood);
    world.obj_mut::<PygmyIslandLife>(this).carrying_wood_logic = None;
}

/// `-[PygmyLogic (Island) stopCarryingFromWood]` @ 0x100095230
pub fn stop_carrying_from_wood(world: &mut World, this: Entity) {
    let wood = world.obj::<PygmyIslandLife>(this).carrying_wood_logic;
    let Some(wood) = wood else {
        return;
    };

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, WOOD_CHANNEL);

    wood_logic::drop_while_carrying(world, wood);
    world.obj_mut::<PygmyIslandLife>(this).carrying_wood_logic = None;

    pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
}

/// `-[PygmyLogic (Island) stopThrowingSpear]` @ 0x10009532c
pub fn stop_throwing_spear(world: &mut World, this: Entity) {
    let throwing_spear = world.obj::<PygmyIslandLife>(this).throwing_spear;
    if !throwing_spear {
        return;
    }

    world.obj_mut::<PygmyIslandLife>(this).throwing_spear = false;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, SPEAR_CHANNEL);

    let spear = world.obj::<PygmyIslandLife>(this).throwing_spear_logic;
    if let Some(spear) = spear {
        spear_logic::drop_while_holding(world, spear);
    }

    world.obj_mut::<PygmyIslandLife>(this).throwing_spear_logic = None;
}

/// `-[PygmyLogic (Island) stopWarmingByBonFire]` @ 0x1000954bc
pub fn stop_warming_by_bon_fire(world: &mut World, this: Entity) {
    let right_bon_fire = world.obj::<PygmyIslandLife>(this).right_warming_bon_fire_logic;
    if let Some(bon_fire) = right_bon_fire {
        bon_fire_logic::stop_right_warming(world, bon_fire, this);
        world.obj_mut::<PygmyIslandLife>(this).right_warming_bon_fire_logic = None;
    }

    let left_bon_fire = world.obj::<PygmyIslandLife>(this).left_warming_bon_fire_logic;
    if let Some(bon_fire) = left_bon_fire {
        bon_fire_logic::stop_left_warming(world, bon_fire, this);
        world.obj_mut::<PygmyIslandLife>(this).left_warming_bon_fire_logic = None;
    }
}

/// `-[PygmyLogic (Island) tossFromBonFire]` @ 0x100095530
pub fn toss_from_bon_fire(world: &mut World, this: Entity) {
    let right_bon_fire = world.obj::<PygmyIslandLife>(this).right_warming_bon_fire_logic;
    if let Some(bon_fire) = right_bon_fire {
        bon_fire_logic::stop_right_warming(world, bon_fire, this);
        world.obj_mut::<PygmyIslandLife>(this).right_warming_bon_fire_logic = None;
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    let left_bon_fire = world.obj::<PygmyIslandLife>(this).left_warming_bon_fire_logic;
    if let Some(bon_fire) = left_bon_fire {
        bon_fire_logic::stop_left_warming(world, bon_fire, this);
        world.obj_mut::<PygmyIslandLife>(this).left_warming_bon_fire_logic = None;
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }
}

/// `-[PygmyLogic (Island) stopWaitingToCrap]` @ 0x1000955e4
pub fn stop_waiting_to_crap(world: &mut World, this: Entity) {
    let out_house = world.obj::<PygmyIslandLife>(this).waiting_out_house_logic;
    if let Some(out_house) = out_house {
        out_house_logic::stop_waiting_to_crap(world, out_house, this);
        world.obj_mut::<PygmyIslandLife>(this).waiting_out_house_logic = None;
    }
}

/// `-[PygmyLogic (Island) tossFromOutHouse]` @ 0x10009562c
pub fn toss_from_out_house(world: &mut World, this: Entity) {
    let waiting_out_house = world.obj::<PygmyIslandLife>(this).waiting_out_house_logic;
    if let Some(out_house) = waiting_out_house {
        out_house_logic::stop_waiting_to_crap(world, out_house, this);
        world.obj_mut::<PygmyIslandLife>(this).waiting_out_house_logic = None;
        pg_engine::logic::set_behavior(world, this, Some("PygmyIdle"));
    }

    let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
    if crapping_out_house.is_none() {
        return;
    }

    // The out-house is not told: it ejects the pygmy itself.
    world.obj_mut::<PygmyPygmyCoreFrame>(this).crapping = false;

    let mut island_life = world.obj_mut::<PygmyIslandLife>(this);
    island_life.finished_crapping = true;
    island_life.crapping_out_house_logic = None;

    let display = pg_engine::logic::display_object(world, this);
    let mut display_object = pg_engine::display::object_mut(world, display);
    display_object.scale.x = 1.0;
    display_object.scale.y = 1.0;
    display_object.is_selectable = true;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;
    pg_engine::logic::logic_mut(world, this).pos_vel.y = 200.0;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

    insert_after_pygmy_display_marker(world, this);
    update_label_layer(world, this);

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(String::from("DroppedDrown"));
    pg_engine::logic::set_behavior(world, this, Some("PygmyFall"));
}

/// `-[PygmyLogic (Island) finishCrapping]` @ 0x1000957e4
pub fn finish_crapping(world: &mut World, this: Entity) {
    let crapping_out_house = world.obj::<PygmyIslandLife>(this).crapping_out_house_logic;
    if crapping_out_house.is_none() {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).crapping = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).is_selectable = true;

    let mut island_life = world.obj_mut::<PygmyIslandLife>(this);
    island_life.finished_crapping = true;
    island_life.crapping_out_house_logic = None;

    let island_y_pos = world.obj::<PygmyPygmyCoreFrame>(this).island_y_pos;
    pg_engine::display::object_mut(world, display).pos.y = island_y_pos;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).last_pos.y = island_y_pos + 2.0;

    insert_after_pygmy_display_marker(world, this);
    update_label_layer(world, this);

    give_gift(world, this, "TakeCrap");
    pg_engine::logic::set_behavior(world, this, Some("PygmyFinishCrapping"));
}

/// `-[PygmyLogic (Island) stopFishing]` @ 0x100096ca0
pub fn stop_fishing(world: &mut World, this: Entity) {
    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if pole.is_none() {
        return;
    }

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::stop(world, animation);

    put_back_on_island_after_fishing(world, this);

    let ice_fishing = world.obj::<PygmyIslandLife>(this).ice_fishing;
    if ice_fishing {
        world.obj_mut::<PygmyIslandLife>(this).ice_fishing = false;
        stop_ice_hole_fishing(world, this);
    } else {
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_POLE_CHANNEL);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_BOB_CHANNEL);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_STRING_CHANNEL);
    }

    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if let Some(pole) = pole {
        fishing_pole_logic::drop_while_fishing(world, pole);
    }

    world.obj_mut::<PygmyIslandLife>(this).fishing_pole_logic = None;

    insert_after_pygmy_display_marker(world, this);
    update_label_layer(world, this);
}

/// `-[PygmyLogic (Island) stopFishingFromPole:]` @ 0x100096e7c
pub fn stop_fishing_from_pole(world: &mut World, this: Entity, _pole: Entity) {
    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if pole.is_none() {
        return;
    }

    let catch_shark = world.obj::<PygmyPygmyCoreFrame>(this).catch_shark;
    if catch_shark {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).catch_shark = false;
        reset_z_rot(world, this);
    }

    put_back_on_island_after_fishing(world, this);

    let ice_fishing = world.obj::<PygmyIslandLife>(this).ice_fishing;
    let behavior = if ice_fishing {
        world.obj_mut::<PygmyIslandLife>(this).ice_fishing = false;
        stop_ice_hole_fishing(world, this);
        "PygmyStopIceFishing"
    } else {
        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_POLE_CHANNEL);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_BOB_CHANNEL);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, FISHING_STRING_CHANNEL);
        "PygmyStopFishing"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));

    // The pole is not told to drop: it is the one stopping the fishing.
    world.obj_mut::<PygmyIslandLife>(this).fishing_pole_logic = None;

    insert_after_pygmy_display_marker(world, this);
    update_label_layer(world, this);
}

/// `-[PygmyLogic (Island) stopFishingFromIceHole:]` @ 0x10009709c
pub fn stop_fishing_from_ice_hole(world: &mut World, this: Entity, _ice_hole: Entity) {
    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if pole.is_none() {
        return;
    }

    put_back_on_island_after_fishing(world, this);

    world.obj_mut::<PygmyIslandLife>(this).ice_fishing = false;
    pg_engine::logic::set_behavior(world, this, Some("PygmyStopIceFishing"));

    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if let Some(pole) = pole {
        fishing_pole_logic::drop_while_fishing(world, pole);
    }

    world.obj_mut::<PygmyIslandLife>(this).fishing_pole_logic = None;

    insert_after_pygmy_display_marker(world, this);
    update_label_layer(world, this);
}

/// `-[PygmyLogic (Island) pullPole]` @ 0x1000971e8
pub fn pull_pole(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyPullPole"));
}

/// `-[PygmyLogic (Island) catchSwordFish]` @ 0x100097200
pub fn catch_sword_fish(world: &mut World, this: Entity) {
    give_gift(world, this, "CatchSwordFish");
}

/// `-[PygmyLogic (Island) catchFish]` @ 0x100097230
pub fn catch_fish(world: &mut World, this: Entity) {
    give_gift(world, this, "CatchFish");
    pg_engine::logic::set_behavior(world, this, Some("PygmyCatchFish"));
}

/// `-[PygmyLogic (Island) catchShark]` @ 0x100097290
pub fn catch_shark(world: &mut World, this: Entity) {
    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.catch_shark = true;
    core.sacrifice_id = Some(String::from("SharkFishingPole"));

    sprite_logic::hide_apocalypse_cloud(world, this);

    let pole = world.obj::<PygmyIslandLife>(this).fishing_pole_logic;
    if let Some(pole) = pole {
        sprite_logic::hide_apocalypse_cloud(world, pole);
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyCatchShark"));
}

/// `-[PygmyLogic (Island) kidnappedByKonkeyDong]` @ 0x1000a7ae4
pub fn kidnapped_by_konkey_dong(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).kidnapped_by_konkey_dong = true;
}

/// `-[PygmyLogic (Island) kidnappedByKonkeyDongComplete]` @ 0x1000a7afc
pub fn kidnapped_by_konkey_dong_complete(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(String::from("KidnappedByKonkeyDong"));
    pg_engine::logic::set_behavior(world, this, Some("PygmyKidnappedByKonkeyDong"));
}

/// `-[PygmyLogic (Island) startStaringAtKidnapping]` @ 0x1000a7b2c
pub fn start_staring_at_kidnapping(world: &mut World, this: Entity) -> bool {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_stare = free
        && core.time_temple_pygmy_type == 0
        && !core.is_zombie
        && !core.falling
        && !core.shark_on_head
        && core.tilt_state == 0;

    if !can_stare {
        return false;
    }

    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_rainbow(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);

    world.obj_mut::<PygmyIslandLife>(this).staring_at_kidnapping = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let kong_statue = listener.and_then(|listener| crate::controller::kong_statue_logic(world, listener, this));

    stop_moving(world, this);
    reset_z_rot(world, this);

    // Face the statue: left when Kong is further left than the pygmy.
    set_x_scale(world, this, 1.0);

    // PORT: with a nil kongStatueLogic: the original reads through a NULL pos pointer; the facing
    // comparison is skipped and xScale stays 1.
    if let Some(kong_statue) = kong_statue {
        let kong_display = pg_engine::logic::display_object(world, kong_statue);
        let kong_x_pos = pg_engine::display::object(world, kong_display).pos.x;

        let display = pg_engine::logic::display_object(world, this);
        let x_pos = pg_engine::display::object(world, display).pos.x;

        if kong_x_pos < x_pos {
            set_x_scale(world, this, -1.0);
        }
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyStartStaringAtKidnapping"));
    true
}

/// `-[PygmyLogic (Island) chaseKonkeyDong]` @ 0x1000a83f4
pub fn chase_konkey_dong(world: &mut World, this: Entity) {
    let staring_at_kidnapping = world.obj::<PygmyIslandLife>(this).staring_at_kidnapping;
    if staring_at_kidnapping {
        pg_engine::logic::set_behavior(world, this, Some("PygmyChaseKonkeyDong"));
    }
}

/// `-[PygmyLogic (Island) canChaseKonkeyDong]` @ 0x1000a8424
pub fn can_chase_konkey_dong(world: &mut World, this: Entity) -> bool {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    free
        && core.time_temple_pygmy_type == 0
        && !core.is_zombie
        && !core.falling
        && !core.shark_on_head
        && core.tilt_state == 0
}

/// `-[PygmyLogic (Island) stopStaringAtKidnapping]` @ 0x1000a8ad8
pub fn stop_staring_at_kidnapping(world: &mut World, this: Entity) {
    let staring_at_kidnapping = world.obj::<PygmyIslandLife>(this).staring_at_kidnapping;
    if staring_at_kidnapping {
        world.obj_mut::<PygmyIslandLife>(this).staring_at_kidnapping = false;
        pg_engine::logic::set_behavior(world, this, Some("PygmyStopStaringAtKidnapping"));
    }
}

/// `-[PygmyLogic (Island) checkDanceChange:::]` @ 0x1000a8b0c
pub fn seq_check_dance_change(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let option_value_id = pygmy_dance_option_value_id(world);

    if call.first_time {
        world.obj_mut::<PygmyIslandLife>(this).last_dance_skin_option_value_id = option_value_id.clone();
    }

    // `[mLastDanceSkinOptionValueId isEqualToString:value]`: NO when either is nil.
    let last_option_value_id = &world.obj::<PygmyIslandLife>(this).last_dance_skin_option_value_id;
    let unchanged = match (last_option_value_id, &option_value_id) {
        (Some(last), Some(current)) => last == current,
        _ => false,
    };

    if unchanged {
        return ItemResult::Pending;
    }

    pg_engine::logic::set_behavior(world, this, Some("PygmyRefreshGoGoCage"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (Island) stopStaringAtKidnapping:::]` @ 0x1000a8bd4
pub fn seq_stop_staring_at_kidnapping(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_staring_at_kidnapping(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Island) chaseKonkeyDongComplete:::]` @ 0x1000a8bf8
pub fn seq_chase_konkey_dong_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener;
    let kong_statue = listener.and_then(|listener| crate::controller::kong_statue_logic(world, listener, this));

    if let Some(kong_statue) = kong_statue {
        kong_statue_logic::start_konkey_dong(world, kong_statue, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) caughtByShark:]` @ 0x1000aa068
pub fn caught_by_shark(world: &mut World, this: Entity, _shark: Entity) {
    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let sacrifice_id = if pygmy_type == 2 { "BabyPygmyEatenByShark" } else { "FedToShark" };

    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(String::from(sacrifice_id));
    pg_engine::logic::set_behavior(world, this, Some("PygmyCaughtByShark"));
}

/// `-[PygmyLogic (Island) hasSharkOnHead]` @ 0x1000aa0b8
pub fn has_shark_on_head(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyPygmyCoreFrame>(this).shark_on_head
}

/// `-[PygmyLogic (Island) eatenByShark:]` @ 0x1000aa0cc
pub fn eaten_by_shark(world: &mut World, this: Entity, shark: Entity) -> bool {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_be_eaten = free
        && core.time_temple_pygmy_type == 0
        && !core.is_zombie
        && !core.falling
        && !core.shark_on_head
        && !core.sleeping
        && core.tilt_state == 0;

    if !can_be_eaten {
        return false;
    }

    stop_activities(world, this);

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.shark_on_head = true;
    core.tilt_state = 0;

    stop_moving(world, this);
    reset_z_rot(world, this);

    world.obj_mut::<PygmyIslandLife>(this).eating_shark_logic = Some(shark);

    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    let behavior = if personality == 1 { "PygmyCharlieEatenByShark" } else { "PygmyEatenByShark" };
    pg_engine::logic::set_behavior(world, this, Some(behavior));

    insert_before_pygmy_small_objects_display_marker(world, this);
    update_label_layer(world, this);
    true
}

/// `-[PygmyLogic (Island) uneatenByShark:]` @ 0x1000aa998
pub fn uneaten_by_shark(world: &mut World, this: Entity, _shark: Entity) {
    let shark_on_head = world.obj::<PygmyPygmyCoreFrame>(this).shark_on_head;
    if !shark_on_head {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).shark_on_head = false;
    world.obj_mut::<PygmyIslandLife>(this).eating_shark_logic = None;
    pg_engine::logic::set_behavior(world, this, Some("PygmyUneatenByShark"));
}

/// `-[PygmyLogic (Island) canDoNormalDance]` @ 0x1000aba40
pub fn can_do_normal_dance(world: &mut World, this: Entity) -> bool {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    free && !core.is_zombie && !core.falling && !core.shark_on_head && core.tilt_state == 0
}

/// `-[PygmyLogic (Island) startDancing:]` @ 0x1000ac0e0
pub fn start_dancing(world: &mut World, this: Entity, theme_song_backup: bool) {
    world.obj_mut::<PygmyIslandLife>(this).is_theme_song_backup = theme_song_backup;

    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_dance = free && !core.falling && !core.shark_on_head && core.tilt_state == 0;

    if !can_dance {
        let in_go_go_cage = core.in_go_go_cage;
        if in_go_go_cage {
            pg_engine::logic::set_behavior(world, this, Some("PygmyRefreshGoGoCage"));
        }
        return;
    }

    stop_activities(world, this);
    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    set_x_scale(world, this, 1.0);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let pygmy_type = core.time_temple_pygmy_type;
    let is_zombie = core.is_zombie;

    let behavior = if pygmy_type != 0 {
        "TemplePygmyStartDancing"
    } else if is_zombie {
        give_gift(world, this, "ZombieDance");
        "PygmyStartZombieDancing"
    } else {
        give_gift(world, this, "Dance");
        if theme_song_backup { "PygmyStartDancingThemeSongBackup" } else { "PygmyStartDancing" }
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) startVampireDancing]` @ 0x1000aca00
pub fn start_vampire_dancing(world: &mut World, this: Entity) {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_dance = free
        && core.time_temple_pygmy_type == 0
        && !core.falling
        && !core.shark_on_head
        && core.tilt_state == 0;

    if !can_dance {
        return;
    }

    stop_activities(world, this);
    stop_moving(world, this);
    reset_z_rot(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;

    let is_zombie = world.obj::<PygmyPygmyCoreFrame>(this).is_zombie;
    let behavior = if is_zombie {
        set_x_scale(world, this, 1.0);
        give_gift(world, this, "ZombieDance");
        "PygmyStartZombieDancing"
    } else {
        give_gift(world, this, "Dance");
        "PygmyStartVampireDancing"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) stopDancing:updateSong:themeSong:napoleonDynamite:fishmas:fishmasTwelveDays:]` @ 0x1000ad2b8
pub fn stop_dancing(world: &mut World, this: Entity, pants_dance: bool, update_song: bool, theme_song: bool, napoleon_dynamite: bool, fishmas: bool, fishmas_twelve_days: bool) {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_stop = free && !core.falling && !core.shark_on_head && core.tilt_state == 0;

    if !can_stop {
        return;
    }

    stop_activities(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);
    reset_z_rot(world, this);

    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let pygmy_type = core.time_temple_pygmy_type;
    let is_zombie = core.is_zombie;

    let behavior = if pygmy_type != 0 {
        "TemplePygmyStopDancing"
    } else if is_zombie {
        "PygmyStopZombieDancing"
    } else {
        let island_life = world.obj::<PygmyIslandLife>(this);
        let special_stop = if pants_dance {
            Some("PygmyStopPantsDancing")
        } else if update_song {
            Some("PygmyStopUpdateSongDancing")
        } else if theme_song {
            if island_life.is_theme_song_backup {
                Some("PygmyStopThemeSongDancingBackup")
            } else {
                Some("PygmyStopThemeSongDancing")
            }
        } else if napoleon_dynamite {
            Some("PygmyStopNapoleonDancing")
        } else if fishmas {
            Some("PygmyStopFishmasDancing")
        } else if fishmas_twelve_days {
            Some("PygmyStopTwelveDaysDancing")
        } else {
            None
        };

        match special_stop {
            Some(behavior) => {
                world.obj_mut::<PygmyIslandLife>(this).stopping_dance = true;
                behavior
            }
            None => "PygmyStopDancing",
        }
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) stopDancingNow]` @ 0x1000adc80
pub fn stop_dancing_now(world: &mut World, this: Entity) {
    let stopping_dance = world.obj::<PygmyIslandLife>(this).stopping_dance;
    if !stopping_dance {
        return;
    }

    world.obj_mut::<PygmyIslandLife>(this).stopping_dance = false;

    let pygmy_type = world.obj::<PygmyPygmyCoreFrame>(this).time_temple_pygmy_type;
    let behavior = if pygmy_type != 0 { "TemplePygmyIdleLoop" } else { "PygmyIdle" };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) startStaringAtRainbow]` @ 0x1000adcd4
pub fn start_staring_at_rainbow(world: &mut World, this: Entity) {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_stare = free
        && core.time_temple_pygmy_type == 0
        && !core.is_zombie
        && !core.falling
        && !core.shark_on_head
        && core.tilt_state == 0;

    if !can_stare {
        return;
    }

    stop_being_high_on_nitrous(world, this);
    stop_eating(world, this);
    stop_carrying(world, this);
    stop_carrying_plague_body(world, this);
    stop_fishing(world, this);
    stop_sleeping(world, this);
    stop_warming_by_bon_fire(world, this);
    stop_staring_at_kidnapping(world, this);
    stop_waiting_to_crap(world, this);
    stop_throwing_spear(world, this);
    close_word_balloon(world, this);
    stop_charlie_mischief(world, this);

    world.obj_mut::<PygmyIslandLife>(this).staring_at_rainbow = true;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    reset_z_rot(world, this);
    set_x_scale(world, this, 1.0);

    give_gift(world, this, "Dance");
    pg_engine::logic::set_behavior(world, this, Some("PygmyStartStaringAtRainbow"));
}

/// `-[PygmyLogic (Island) isStaringAtRainbow]` @ 0x1000ae528
pub fn is_staring_at_rainbow(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyIslandLife>(this).staring_at_rainbow
}

/// `-[PygmyLogic (Island) gushOverRainbow]` @ 0x1000ae53c
pub fn gush_over_rainbow(world: &mut World, this: Entity) {
    let staring_at_rainbow = world.obj::<PygmyIslandLife>(this).staring_at_rainbow;
    if staring_at_rainbow {
        world.obj_mut::<PygmyIslandLife>(this).gushing_over_rainbow = true;
        pg_engine::logic::set_behavior(world, this, Some("PygmyGushOverRainbow"));
    }
}

/// `-[PygmyLogic (Island) stopStaringAtRainbow]` @ 0x1000ae580
pub fn stop_staring_at_rainbow(world: &mut World, this: Entity) {
    let staring_at_rainbow = world.obj::<PygmyIslandLife>(this).staring_at_rainbow;
    if !staring_at_rainbow {
        return;
    }

    world.obj_mut::<PygmyIslandLife>(this).staring_at_rainbow = false;
    pg_engine::logic::set_behavior(world, this, Some("PygmyStopStaringAtRainbow"));

    let gushing_over_rainbow = world.obj::<PygmyIslandLife>(this).gushing_over_rainbow;
    if !gushing_over_rainbow {
        return;
    }

    world.obj_mut::<PygmyIslandLife>(this).gushing_over_rainbow = false;

    pg_engine::sound::stop_sound(world, "TripleRainbow");
    pg_engine::sound::stop_sound(world, "WhoaOhMyGod");
    pg_engine::sound::stop_sound(world, "WhatDoesThisMean");
    pg_engine::sound::stop_sound(world, "SoIntense");
    pg_engine::sound::stop_sound(world, "DoubleRainbowAllTheWay");
    pg_engine::sound::stop_sound(world, "AcrossTheSky");

    finish_gushing_over_rainbow(world, this);
}

/// `-[PygmyLogic (Island) stopDanceComplete:::]` @ 0x1000ae740
pub fn seq_stop_dance_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyIslandLife>(this).stopping_dance = true;
    ItemResult::Done
}

/// `-[PygmyLogic (Island) chooseRainbowAnimation:::]` @ 0x1000ae75c
pub fn seq_choose_rainbow_animation(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let statics = world.get_resource::<PygmyIslandLifeStatics>();
    let pool_is_empty = statics.is_none_or(|statics| statics.rainbow_animation_pool.is_empty());

    if pool_is_empty {
        let list = call.list("list");
        let animation_ids: Vec<String> = list.into_iter().map(String::from).collect();

        let mut statics = world.get_resource_or_insert_with(PygmyIslandLifeStatics::default);
        statics.rainbow_animation_pool.extend(animation_ids);
    }

    // `arc4random() % count`, kept as an `int` index.
    let pool_count = world.resource::<PygmyIslandLifeStatics>().rainbow_animation_pool.len() as u32;
    let random_index = world.resource_mut::<Random>().arc4random_mod(pool_count);
    let index = random_index as i32;

    // PORT: the original raises on objectAtIndex: (with the raw arc4random value) when the pool is
    // still empty, which needs an absent `list` parameter (an empty string adds one empty id, as
    // componentsSeparatedByString: does); the item finishes (2) without playing.
    let pool = &world.resource::<PygmyIslandLifeStatics>().rainbow_animation_pool;
    let pool_index = usize::try_from(index).ok();
    let animation_id = pool_index.and_then(|pool_index| pool.get(pool_index)).cloned();
    let (Some(pool_index), Some(animation_id)) = (pool_index, animation_id) else {
        return ItemResult::Done;
    };

    let episode_event = match animation_id.as_str() {
        "PygmyDoubleRainbow1Anim" => Some("DoubleRainbowAllTheWay"),
        "PygmyDoubleRainbow2Anim" => Some("DoubleRainbowSoIntense"),
        "PygmyDoubleRainbow3Anim" => Some("DoubleRainbowAllTheWayAcrossTheSky"),
        "PygmyDoubleRainbow4Anim" => Some("DoubleRainbowOhMyGod"),
        "PygmyDoubleRainbow5Anim" => Some("DoubleRainbowWhatDoesThisMean"),
        "PygmyDoubleRainbow6Anim" => Some("DoubleRainbowTripleRainbow"),
        _ => None,
    };

    if let Some(episode_event) = episode_event {
        let manager = idol_manager::instance(world);
        if let Some(manager) = manager {
            idol_manager::trigger_episode_event(world, manager, episode_event);
        }
    }

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::set_animation_listener(world, animation, Some(this));
    pg_engine::animation::play_animation_id(world, animation, Some(&animation_id));

    let mut statics = world.resource_mut::<PygmyIslandLifeStatics>();
    statics.rainbow_animation_pool.remove(pool_index);

    ItemResult::Done
}

/// `-[PygmyLogic (Island) startStaringAtRainbow:::]` @ 0x1000aea68
pub fn seq_start_staring_at_rainbow(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_moving(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (Island) gushOverRainbowComplete:::]` @ 0x1000aea84
pub fn seq_gush_over_rainbow_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let gushing_over_rainbow = world.obj::<PygmyIslandLife>(this).gushing_over_rainbow;
    if gushing_over_rainbow {
        world.obj_mut::<PygmyIslandLife>(this).gushing_over_rainbow = false;
        finish_gushing_over_rainbow(world, this);
    }

    ItemResult::Done
}

/// `-[PygmyLogic (Island) canBeHumpedByMonkey:]` @ 0x1000aeaf4
pub fn can_be_humped_by_monkey(world: &mut World, this: Entity, _monkey: Entity) -> bool {
    let sleeping = world.obj::<PygmyPygmyCoreFrame>(this).sleeping;
    let humping_monkey = world.obj::<PygmyIslandLife>(this).humping_monkey_logic;

    sleeping && humping_monkey.is_none()
}

/// `-[PygmyLogic (Island) animHitHumpingMonkey]` @ 0x1000aeb2c
pub fn anim_hit_humping_monkey(world: &mut World, this: Entity) {
    let monkey = world.obj::<PygmyIslandLife>(this).humping_monkey_logic;
    if let Some(monkey) = monkey {
        monkey_logic::punched_while_humping(world, monkey);
    }

    world.obj_mut::<PygmyIslandLife>(this).humping_monkey_logic = None;
}

/// `-[PygmyLogic (Island) humpedByMonkey:]` @ 0x1000aeb6c
pub fn humped_by_monkey(world: &mut World, this: Entity, monkey: Entity) {
    world.obj_mut::<PygmyIslandLife>(this).humping_monkey_logic = Some(monkey);

    let display = pg_engine::logic::display_object(world, this);
    let display_object = pg_engine::display::object(world, display);
    let x_pos = display_object.pos.x;
    let x_scale = display_object.scale.x;

    let monkey_display = pg_engine::logic::display_object(world, monkey);
    let monkey_x_pos = pg_engine::display::object(world, monkey_display).pos.x;

    // The Front variant when the monkey is level with the pygmy or on the side it faces away from.
    let behavior = if x_scale * (monkey_x_pos - x_pos) <= 0.0 {
        "PygmyHumpedByMonkeyFront"
    } else {
        "PygmyHumpedByMonkeyBehind"
    };

    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (Island) unhumpedByMonkey:]` @ 0x1000aec44
pub fn unhumped_by_monkey(world: &mut World, this: Entity, _monkey: Entity) {
    world.obj_mut::<PygmyIslandLife>(this).humping_monkey_logic = None;
    pg_engine::logic::set_behavior(world, this, Some("PygmyAsleep"));
}

/// `-[PygmyLogic (Island) monkeyOnBack:]` @ 0x1000aec6c
pub fn monkey_on_back(world: &mut World, this: Entity, monkey: Entity) -> bool {
    let free = is_free_base(world, this);
    let charlie_on_back_monkey = world.obj::<PygmyCharlieStory>(this).charlie_on_back_monkey_logic;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_carry_monkey = free
        && charlie_on_back_monkey.is_none()
        && core.time_temple_pygmy_type == 0
        && !core.is_zombie
        && !core.falling
        && !core.shark_on_head
        && !core.sleeping
        && core.tilt_state == 0;

    if !can_carry_monkey {
        return false;
    }

    stop_activities(world, this);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    stop_moving(world, this);
    reset_z_rot(world, this);

    let personality = world.obj::<PygmyPygmyCoreFrame>(this).persona_personality;
    if personality == 1 {
        world.obj_mut::<PygmyCharlieStory>(this).charlie_on_back_monkey_logic = Some(monkey);
        pg_engine::logic::set_behavior(world, this, Some("PygmyCharlieMonkeyOnBack"));
    } else {
        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        core.monkey_on_back = true;
        core.sacrifice_id = Some(String::from("MonkeyOnBack"));

        pg_engine::logic::set_behavior(world, this, Some("PygmyMonkeyOnBack"));
        insert_before_pygmy_small_objects_display_marker(world, this);
        update_label_layer(world, this);
    }

    true
}

/// `-[PygmyLogic (Island) canBeCaughtByMonkey:]` @ 0x1000af574
pub fn can_be_caught_by_monkey(world: &mut World, this: Entity, _monkey: Entity) -> bool {
    let free = is_free_base(world, this);
    let y_pos_vel = pg_engine::logic::logic(world, this).pos_vel.y;
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    free
        && core.time_temple_pygmy_type == 0
        && !core.is_zombie
        && !core.shark_on_head
        && core.tilt_state == 0
        && core.falling
        && y_pos_vel < 0.0
}

/// `-[PygmyLogic (Island) canBeChasedByMonkey:]` @ 0x1000afc44
pub fn can_be_chased_by_monkey(world: &mut World, this: Entity, _monkey: Entity) -> bool {
    let free = is_free_base(world, this);
    let core = world.obj::<PygmyPygmyCoreFrame>(this);

    free
        && core.time_temple_pygmy_type == 0
        && !core.is_zombie
        && !core.falling
        && !core.shark_on_head
        && core.tilt_state == 0
}

/// `-[PygmyLogic (Island) caughtByMonkey:]` @ 0x1000b02f8
pub fn caught_by_monkey(world: &mut World, this: Entity, monkey: Entity) {
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

    stop_moving(world, this);
    reset_z_rot(world, this);

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_floor_collision = false;
    core.tilt_state = 0;

    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display).is_selectable = false;

    world.obj_mut::<PygmyPygmyCoreFrame>(this).caught_by_monkey = true;

    let mut island_life = world.obj_mut::<PygmyIslandLife>(this);
    island_life.playing_catch_with_monkey = true;
    island_life.throwing_monkey_logic = Some(monkey);

    pg_engine::logic::set_behavior(world, this, Some("PygmyCaughtByMonkey"));

    let monkeys = monkey_logic_array(world, this);
    for other_monkey in monkeys {
        monkey_logic::start_playing_catch(world, other_monkey);
    }
}
