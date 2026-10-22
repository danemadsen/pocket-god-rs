//! `KongStatueLogic` (superclass `SpriteLogic`).
//!
//! The Kong statue on the ape island and host of the Konkey Dong mini-game: it kidnaps the pygmy
//! that falls into it and runs the levels (`mKonkeyDongLevelArray`) the pygmy climbs.

use std::sync::OnceLock;

use crate::prelude::*;

use crate::logic::island_core::sprite_logic;
use crate::logic::konkeydong::{
    konkey_dong_beetle_logic, konkey_dong_boulder_logic, konkey_dong_ladder_logic, konkey_dong_logic,
    konkey_dong_pick_logic, konkey_dong_platform_logic,
};
use pg_assets::data::konkey_dong_levels::{self, KonkeyDongLevelsFile};
use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::model::SPRITE_Z;

/// `KongStatueLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct KongStatueLogic {
    /// `mKongStatueLogicListener`: `@"<KongStatueLogicListener>"`
    pub kong_statue_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbKonkeyDong`: `B`
    pub konkey_dong: bool,

    /// `mKidnapPygmyLogic`: `@"PygmyLogic"`
    pub kidnap_pygmy_logic: Option<Entity>,

    /// `mKonkeyDongKidnapLogic`: `@"KonkeyDongLogic"`
    pub konkey_dong_kidnap_logic: Option<Entity>,

    /// `mbKidnappingPygmy`: `B`
    pub kidnapping_pygmy: bool,

    /// `mKonkeyDongOverlayLogic`: `@"MultiModelLogic"`
    pub konkey_dong_overlay_logic: Option<Entity>,

    /// `kOverlayStallTime`: `f`
    pub k_overlay_stall_time: f32,

    /// `mDifficulty`: `f`
    pub difficulty: f32,

    /// `mKonkeyDongDisplayGroup`: `@"BCDisplayGroup"`
    pub konkey_dong_display_group: Option<Entity>,

    /// `mKonkeyDongPygmyLogic`: `@"PygmyLogic"`
    pub konkey_dong_pygmy_logic: Option<Entity>,

    /// `mKonkeyDongLogic`: `@"KonkeyDongLogic"`
    pub konkey_dong_logic: Option<Entity>,

    /// `mKonkeyDongCageLogic`: `@"MultiModelLogic"`
    pub konkey_dong_cage_logic: Option<Entity>,

    /// `mKonkeyDongBackgroundLogic`: `@"MultiModelLogic"`
    pub konkey_dong_background_logic: Option<Entity>,

    /// `mKonkeyDongBonusSplashLogic`: `@"MultiModelLogic"`
    pub konkey_dong_bonus_splash_logic: Option<Entity>,

    /// `mKonkeyDongBoulderLogicArray`: `@"NSMutableArray"`
    pub konkey_dong_boulder_logic_array: Vec<Entity>,

    /// `mKonkeyDongLevelCompleteCount`: `i`
    pub konkey_dong_level_complete_count: i32,

    /// `mKonkeyDongLevelIndex`: `i`
    pub konkey_dong_level_index: i32,

    /// `mKonkeyDongLevelCount`: `i`
    pub konkey_dong_level_count: i32,

    /// `mKonkeyDongLevelArray`: `[10{KonkeyDongLevel="mDisplayGroup"@"BCDisplayGroup""mPlatformLogicArray"@"NSMutableArray""mLadderLogicArray"@"NSMutableArray""mPickLogicArray"@"NSMutableArray""mBeetleLogicArray"@"NSMutableArray""mVegetationLogic"@"MultiModelLogic""mLevelWidth"f"mLevelHeight"f"mKongPos"{?="x"f"y"f"z"f"w"f}"mStartPygmyPos"{?="x"f"y"f"z"f"w"f}}]`
    pub konkey_dong_level_array: [KonkeyDongLevel; 10],
}

pub const KONG_STATUE_LOGIC: Class = Class {
    name: "KongStatueLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    on_game_frame,
    on_multi_model_event,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"KongStatueInit"`
const STATUE_INIT_BEHAVIOR: &str = "KongStatueInit";

/// `@"KongStatuePygmyFallInside"`
const STATUE_PYGMY_FALL_INSIDE_BEHAVIOR: &str = "KongStatuePygmyFallInside";

/// `@"KonkeyDongDisplayGroup"`
const KONKEY_DONG_DISPLAY_GROUP: &str = "KonkeyDongDisplayGroup";

/// `@"KonkeyDongLevelDisplayMarker"`: where the active level's group goes.
const LEVEL_DISPLAY_MARKER: &str = "KonkeyDongLevelDisplayMarker";

/// `@"KonkeyDongPygmyDisplayMarker"`: boulders and the bonus splash go in front of it.
const PYGMY_DISPLAY_MARKER: &str = "KonkeyDongPygmyDisplayMarker";

/// The kidnapped pygmy's animation channel in the statue.
const KIDNAP_PYGMY_CHANNEL: usize = 1;

/// Konkey Dong's animation channel for the cage.
const CAGE_CHANNEL: usize = 2;

/// Sequence-item handlers implemented by `KongStatueLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "startKonkeyDongKidnap" => seq_start_konkey_dong_kidnap(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[KongStatueLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&KONG_STATUE_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// The level list `0x100649c68` (`assets/data/konkey-dong-levels.json`, list order), parsed once.
fn konkey_dong_level_defs() -> &'static KonkeyDongLevelsFile {
    static FILE: OnceLock<KonkeyDongLevelsFile> = OnceLock::new();
    FILE.get_or_init(load_konkey_dong_level_defs)
}

fn load_konkey_dong_level_defs() -> KonkeyDongLevelsFile {
    konkey_dong_levels::load().expect("assets/data/konkey-dong-levels.json matches its schema")
}

/// `sdiv`: ARM64 signed division, which gives 0 for a zero divisor (the decompiled
/// `if (count != 0)` guards around the level-count divisions).
fn arm_sdiv(numerator: i32, denominator: i32) -> i32 {
    if denominator == 0 {
        return 0;
    }
    numerator.wrapping_div(denominator)
}

/// `mKonkeyDongLevelCompleteCount / mKonkeyDongLevelCount`: how many times the player has gone
/// through all the levels.
fn completed_loop_count(world: &World, this: Entity) -> i32 {
    let statue = world.obj::<KongStatueLogic>(this);
    arm_sdiv(statue.konkey_dong_level_complete_count, statue.konkey_dong_level_count)
}

/// `mKonkeyDongLevelArray[mKonkeyDongLevelIndex]`.
fn active_level(statue: &KongStatueLogic) -> &KonkeyDongLevel {
    let level_index = statue.konkey_dong_level_index as usize;
    &statue.konkey_dong_level_array[level_index]
}

/// `-[KongStatueLogic initWithDisplayObject:]` @ 0x100186040
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(KongStatueLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    pg_engine::logic::stop_game_frame(world, this);
    let konkey_dong_display_group = pg_engine::library::display_group(world, KONKEY_DONG_DISPLAY_GROUP);
    {
        let mut statue = world.obj_mut::<KongStatueLogic>(this);
        statue.konkey_dong_level_index = 0;
        statue.konkey_dong_level_complete_count = 0;
        statue.konkey_dong_display_group = konkey_dong_display_group;
        statue.konkey_dong_boulder_logic_array = Vec::with_capacity(5);
    }

    let konkey_dong_object = konkey_dong_display_group.and_then(|group| group::get_display_object(world, group, "KonkeyDong"));
    let konkey_dong_logic = konkey_dong_object.and_then(|object| display::object(world, object).logic);
    world.obj_mut::<KongStatueLogic>(this).konkey_dong_logic = konkey_dong_logic;
    if let Some(konkey_dong_logic) = konkey_dong_logic {
        konkey_dong_logic::set_konkey_dong_logic_listener(world, konkey_dong_logic, this);
    }

    let overlay_model = display::multi_model::spawn(world, Some("KonkeyDongOverlay"));
    let overlay_logic = pg_engine::logic::multi_model::spawn(world, overlay_model);
    world.obj_mut::<KongStatueLogic>(this).konkey_dong_overlay_logic = Some(overlay_logic);
    pg_engine::logic::multi_model::set_multi_model_logic_listener(world, overlay_logic, Some(this));
    display::object_mut(world, overlay_model).pos.z = SPRITE_Z;

    let kidnap_model = display::multi_model::spawn(world, Some("KonkeyDongKidnap"));
    let kidnap_logic = konkey_dong_logic::spawn(world, kidnap_model);
    world.obj_mut::<KongStatueLogic>(this).konkey_dong_kidnap_logic = Some(kidnap_logic);
    konkey_dong_logic::set_konkey_dong_logic_listener(world, kidnap_logic, this);
    let statue_pos = display::object(world, display_object).pos;
    display::object_mut(world, kidnap_model).pos = statue_pos;

    let cage_model = display::multi_model::spawn(world, Some("KonkeyDongCage"));
    let cage_logic = pg_engine::logic::multi_model::spawn(world, cage_model);
    world.obj_mut::<KongStatueLogic>(this).konkey_dong_cage_logic = Some(cage_logic);
    if let Some(konkey_dong_logic) = konkey_dong_logic {
        let konkey_dong_animation = pg_engine::logic::animation(world, konkey_dong_logic);
        pg_engine::animation::set_tweenable(world, konkey_dong_animation, Some(cage_model), CAGE_CHANNEL);
    }
    pg_engine::logic::set_behavior(world, cage_logic, Some("KonkeyDongCageInit"));

    let background_object = konkey_dong_display_group.and_then(|group| group::get_display_object(world, group, "Background"));
    let background_logic = background_object.and_then(|object| display::object(world, object).logic);
    world.obj_mut::<KongStatueLogic>(this).konkey_dong_background_logic = background_logic;
    if let Some(background_logic) = background_logic {
        pg_engine::logic::set_behavior(world, background_logic, Some("KonkeyDongBackgroundInit"));
    }

    let bonus_splash_model = display::multi_model::spawn(world, Some("BonusSplash"));
    let bonus_splash_logic = pg_engine::logic::multi_model::spawn(world, bonus_splash_model);
    world.obj_mut::<KongStatueLogic>(this).konkey_dong_bonus_splash_logic = Some(bonus_splash_logic);
    pg_engine::logic::multi_model::set_multi_model_logic_listener(world, bonus_splash_logic, Some(this));

    parse_struct_xml(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[KongStatueLogic setKongStatueLogicListener:]` @ 0x100186458
pub fn set_kong_statue_logic_listener(world: &mut World, this: Entity, kong_statue_logic_listener: Entity) {
    world.obj_mut::<KongStatueLogic>(this).kong_statue_logic_listener = Some(kong_statue_logic_listener);
}

/// `-[KongStatueLogic konkeyDongLogic]` @ 0x1001864b4
pub fn konkey_dong_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<KongStatueLogic>(this).konkey_dong_logic
}

/// `-[KongStatueLogic konkeyDongLevel]` @ 0x1001864c4
///
/// The original returns `&mKonkeyDongLevelArray[mKonkeyDongLevelIndex]`; the port returns that
/// index into `konkey_dong_level_array` (never `None`).
pub fn konkey_dong_level(world: &mut World, this: Entity) -> Option<usize> {
    let level_index = world.obj::<KongStatueLogic>(this).konkey_dong_level_index;
    Some(level_index as usize)
}

/// `-[KongStatueLogic konkeyDongLevelDisplayObject]` @ 0x1001864e8
pub fn konkey_dong_level_display_object(world: &mut World, this: Entity) -> Option<Entity> {
    let statue = world.obj::<KongStatueLogic>(this);
    active_level(statue).display_group
}

/// `-[KongStatueLogic konkeyDongBoulderLogicArray]` @ 0x100186510
pub fn konkey_dong_boulder_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<KongStatueLogic>(this).konkey_dong_boulder_logic_array.clone()
}

/// `-[KongStatueLogic konkeyDongPlatformLogicArray]` @ 0x100186520
pub fn konkey_dong_platform_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let statue = world.obj::<KongStatueLogic>(this);
    active_level(statue).platform_logic_array.clone()
}

/// `-[KongStatueLogic konkeyDongLadderLogicArray]` @ 0x100186548
pub fn konkey_dong_ladder_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let statue = world.obj::<KongStatueLogic>(this);
    active_level(statue).ladder_logic_array.clone()
}

/// `-[KongStatueLogic konkeyDongPickLogicArray]` @ 0x100186570
pub fn konkey_dong_pick_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let statue = world.obj::<KongStatueLogic>(this);
    active_level(statue).pick_logic_array.clone()
}

/// `-[KongStatueLogic konkeyDongBeetleLogicArray]` @ 0x100186598
pub fn konkey_dong_beetle_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    let statue = world.obj::<KongStatueLogic>(this);
    active_level(statue).beetle_logic_array.clone()
}

/// `-[KongStatueLogic isAvailable]` @ 0x1001865c0
pub fn is_available(world: &mut World, this: Entity) -> bool {
    let statue = world.obj::<KongStatueLogic>(this);
    statue.kidnap_pygmy_logic.is_none() && !statue.kidnapping_pygmy
}

/// `-[KongStatueLogic pygmyFallIntoKongStatue:]` @ 0x1001865f0
pub fn pygmy_fall_into_kong_statue(world: &mut World, this: Entity, pygmy_logic: Entity) {
    if world.obj::<KongStatueLogic>(this).kidnap_pygmy_logic.is_some() {
        return;
    }
    world.obj_mut::<KongStatueLogic>(this).kidnap_pygmy_logic = Some(pygmy_logic);

    let animation = pg_engine::logic::animation(world, this);
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(pygmy_display_object), KIDNAP_PYGMY_CHANNEL);
    pg_engine::logic::set_behavior(world, this, Some(STATUE_PYGMY_FALL_INSIDE_BEHAVIOR));
}

/// `-[KongStatueLogic removePygmyFromKongStatue:]` @ 0x100186684
pub fn remove_pygmy_from_kong_statue(world: &mut World, this: Entity, pygmy_logic: Entity) {
    if world.obj::<KongStatueLogic>(this).kidnap_pygmy_logic != Some(pygmy_logic) {
        return;
    }
    world.obj_mut::<KongStatueLogic>(this).kidnap_pygmy_logic = None;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id_now(world, animation, KIDNAP_PYGMY_CHANNEL);
    pg_engine::logic::set_behavior(world, this, Some(STATUE_INIT_BEHAVIOR));
}

/// `-[KongStatueLogic kidnapPygmy]` @ 0x100186700
///
/// Konkey Dong has taken the pygmy over: the statue lets go of it and resets.
pub fn kidnap_pygmy(world: &mut World, this: Entity) {
    world.obj_mut::<KongStatueLogic>(this).kidnap_pygmy_logic = None;

    let animation = pg_engine::logic::animation(world, this);
    pg_engine::animation::clear_tweenable_channel_id(world, animation, KIDNAP_PYGMY_CHANNEL);
    pg_engine::logic::set_behavior(world, this, Some(STATUE_INIT_BEHAVIOR));
}

/// `-[KongStatueLogic kidnapPygmyComplete]` @ 0x100186764
pub fn kidnap_pygmy_complete(world: &mut World, this: Entity) {
    world.obj_mut::<KongStatueLogic>(this).kidnapping_pygmy = false;

    let kidnap_logic = world.obj::<KongStatueLogic>(this).konkey_dong_kidnap_logic;
    if let Some(kidnap_logic) = kidnap_logic {
        let kidnap_display_object = pg_engine::logic::display_object(world, kidnap_logic);
        display::remove_from_display_group(world, kidnap_display_object);
    }
}

/// `-[KongStatueLogic startPygmyKidnapReaction]` @ 0x1001867a8
///
/// Every pygmy stares at the kidnapping; the first one that starts staring chases Konkey Dong
/// when the Konkey Dong option is on.
pub fn start_pygmy_kidnap_reaction(world: &mut World, this: Entity) {
    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    let Some(listener) = listener else {
        return;
    };

    let pygmies = crate::controller::pygmy_logic_array(world, listener, this);
    let mut chasing = false;
    for pygmy in pygmies {
        let staring = crate::pygmy::start_staring_at_kidnapping(world, pygmy);
        if chasing || !staring {
            continue;
        }
        if crate::controller::is_konkey_dong_enabled(world, listener) {
            crate::pygmy::chase_konkey_dong(world, pygmy);
            chasing = true;
        } else {
            chasing = false;
        }
    }
}

/// `-[KongStatueLogic stopPygmyKidnapReaction]` @ 0x100186938
pub fn stop_pygmy_kidnap_reaction(world: &mut World, this: Entity) {
    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    let Some(listener) = listener else {
        return;
    };

    let pygmies = crate::controller::pygmy_logic_array(world, listener, this);
    for pygmy in pygmies {
        crate::pygmy::stop_staring_at_kidnapping(world, pygmy);
    }
}

/// `-[KongStatueLogic enterIsland:]` @ 0x100186a5c
pub fn enter_island(world: &mut World, this: Entity, _startup: bool) {
    let marker = pg_engine::library::display_marker(world, "ApeIslandObjectsDisplayMarker");
    let display_object = pg_engine::logic::display_object(world, this);
    if let Some(marker) = marker {
        display::insert_after(world, marker, Some(display_object));
    }
    pg_engine::logic::set_behavior(world, this, Some(STATUE_INIT_BEHAVIOR));
}

/// `-[KongStatueLogic exitIsland]` @ 0x100186ae0
///
/// Leaving the ape island throws out a pygmy still in the statue and calls off a kidnapping.
pub fn exit_island(world: &mut World, this: Entity) {
    let kidnap_pygmy_logic = world.obj::<KongStatueLogic>(this).kidnap_pygmy_logic;
    if let Some(pygmy) = kidnap_pygmy_logic {
        let animation = pg_engine::logic::animation(world, this);
        pg_engine::animation::clear_tweenable_channel_id_now(world, animation, KIDNAP_PYGMY_CHANNEL);
        crate::pygmy::toss_from_kong_statue(world, pygmy);
        world.obj_mut::<KongStatueLogic>(this).kidnap_pygmy_logic = None;
    }

    if world.obj::<KongStatueLogic>(this).kidnapping_pygmy {
        world.obj_mut::<KongStatueLogic>(this).kidnapping_pygmy = false;
        let kidnap_logic = world.obj::<KongStatueLogic>(this).konkey_dong_kidnap_logic;
        if let Some(kidnap_logic) = kidnap_logic {
            konkey_dong_logic::cancel_kidnap_pygmy(world, kidnap_logic);
        }
        stop_pygmy_kidnap_reaction(world, this);
    }

    let display_object = pg_engine::logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[KongStatueLogic showBonusSplash:]` @ 0x100186eb0
///
/// 100 points and the bonus splash at `pos` (the pygmy's position) for jumping a boulder or a
/// beetle.
pub fn show_bonus_splash(world: &mut World, this: Entity, pos: Vec4) {
    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_konkey_dong_score(world, listener, 100);
    }

    let bonus_splash_logic = world.obj::<KongStatueLogic>(this).konkey_dong_bonus_splash_logic;
    let bonus_splash_logic = bonus_splash_logic.expect("initWithDisplayObject: creates the bonus splash (the original writes through [nil pos])");
    let splash_display_object = pg_engine::logic::display_object(world, bonus_splash_logic);
    display::object_mut(world, splash_display_object).pos = pos;

    let marker = pg_engine::library::display_marker(world, PYGMY_DISPLAY_MARKER);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(splash_display_object));
    }
    pg_engine::logic::set_behavior(world, bonus_splash_logic, Some("KonkeyDongBonusSplashInit"));
}

/// `-[KongStatueLogic throwBoulderAtXPos:yPos:]` @ 0x100186f8c
///
/// Reuses the first boulder that is out of play, or makes a new one, and starts it at the given
/// position, faster on every loop of the levels.
pub fn throw_boulder_at_x_pos(world: &mut World, this: Entity, throw_boulder_at_x_pos: f32, y_pos: f32) {
    let boulders = world.obj::<KongStatueLogic>(this).konkey_dong_boulder_logic_array.clone();
    let mut free_boulder = None;
    for boulder in boulders {
        let boulder_display_object = pg_engine::logic::display_object(world, boulder);
        if display::object(world, boulder_display_object).display_group.is_none() {
            free_boulder = Some((boulder, boulder_display_object));
            break;
        }
    }

    let (boulder, boulder_display_object) = match free_boulder {
        Some(free_boulder) => free_boulder,
        None => {
            let boulder_model = display::multi_model::spawn(world, Some("KonkeyDongBoulder"));
            let boulder = konkey_dong_boulder_logic::spawn(world, boulder_model);
            konkey_dong_boulder_logic::set_konkey_dong_boulder_logic_listener(world, boulder, this);
            world.obj_mut::<KongStatueLogic>(this).konkey_dong_boulder_logic_array.push(boulder);
            (boulder, boulder_model)
        }
    };

    let marker = pg_engine::library::display_marker(world, PYGMY_DISPLAY_MARKER);
    if let Some(marker) = marker {
        display::insert_before(world, marker, Some(boulder_display_object));
    }
    let loop_count = completed_loop_count(world, this);
    konkey_dong_boulder_logic::activate_at_x_pos(world, boulder, throw_boulder_at_x_pos, y_pos, loop_count);
}

/// `-[KongStatueLogic initActiveLevel:]` @ 0x10018721c
///
/// Puts the active level's group at `y_pos`, Konkey Dong at the level's kong position, and
/// re-arms the level's picks and beetles for the current loop.
pub fn init_active_level(world: &mut World, this: Entity, y_pos: f32) {
    let statue = world.obj::<KongStatueLogic>(this);
    let konkey_dong_logic = statue.konkey_dong_logic;
    let level = active_level(statue);
    let level_display_group = level.display_group;
    let kong_pos = level.kong_pos;
    let picks = level.pick_logic_array.clone();
    let beetles = level.beetle_logic_array.clone();

    let konkey_dong_display_object = konkey_dong_logic.map(|logic| pg_engine::logic::display_object(world, logic));
    let konkey_dong_display_object = konkey_dong_display_object.expect("KongStatueLogic: no Konkey Dong (the original writes through [nil pos])");
    let level_display_group = level_display_group.expect("KongStatueLogic: the level has no group (the original writes through [nil pos])");
    display::object_mut(world, level_display_group).pos.y = y_pos;
    {
        let mut konkey_dong_object = display::object_mut(world, konkey_dong_display_object);
        konkey_dong_object.pos.x = kong_pos.x;
        konkey_dong_object.pos.y = kong_pos.y + y_pos;
    }

    let loop_count = completed_loop_count(world, this);
    for pick in picks {
        konkey_dong_pick_logic::activate(world, pick, loop_count);
    }
    for beetle in beetles {
        konkey_dong_beetle_logic::activate(world, beetle, loop_count);
    }
}

/// `-[KongStatueLogic startKonkeyDong:]` @ 0x10018747c
///
/// Starts the mini-game with `pygmy_logic` on the first level.
pub fn start_konkey_dong(world: &mut World, this: Entity, pygmy_logic: Entity) {
    pg_engine::view::add_game_frame_listener(world, this);
    {
        let mut statue = world.obj_mut::<KongStatueLogic>(this);
        statue.konkey_dong = true;
        statue.difficulty = 0.0;
        statue.konkey_dong_pygmy_logic = Some(pygmy_logic);
        statue.konkey_dong_level_index = 0;
        statue.konkey_dong_level_complete_count = 0;
    }

    let marker = pg_engine::library::display_marker(world, LEVEL_DISPLAY_MARKER);
    let statue = world.obj::<KongStatueLogic>(this);
    let level_display_group = active_level(statue).display_group;
    let konkey_dong_display_group = statue.konkey_dong_display_group;
    if let Some(marker) = marker {
        display::insert_after(world, marker, level_display_group);
    }
    let konkey_dong_display_group = konkey_dong_display_group.expect("KongStatueLogic: no KonkeyDongDisplayGroup (the original writes through [nil pos])");
    {
        let mut group_object = display::object_mut(world, konkey_dong_display_group);
        group_object.pos.x = 0.0;
        group_object.pos.y = 0.0;
    }

    init_active_level(world, this, 0.0);
    let konkey_dong_logic = world.obj::<KongStatueLogic>(this).konkey_dong_logic;
    if let Some(konkey_dong_logic) = konkey_dong_logic {
        konkey_dong_logic::start_konkey_dong(world, konkey_dong_logic);
    }
    crate::pygmy::start_konkey_dong(world, pygmy_logic);
    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_konkey_dong_start(world, listener, this, pygmy_logic);
    }

    let menu_marker = pg_engine::library::display_marker(world, "MenuDisplayMarker");
    let overlay_logic = world.obj::<KongStatueLogic>(this).konkey_dong_overlay_logic;
    let overlay_display_object = overlay_logic.map(|logic| pg_engine::logic::display_object(world, logic));
    if let Some(menu_marker) = menu_marker {
        display::insert_before(world, menu_marker, overlay_display_object);
    }
    if let Some(overlay_logic) = overlay_logic {
        pg_engine::logic::set_behavior(world, overlay_logic, Some("KonkeyDongOverlayInit"));
    }
}

/// `-[KongStatueLogic pygmyIntroComplete]` @ 0x100187668
pub fn pygmy_intro_complete(world: &mut World, this: Entity) {
    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_konkey_dong_start_moving_camera(world, listener);
    }
}

/// `-[KongStatueLogic stopKonkeyDong]` @ 0x100187684
pub fn stop_konkey_dong(world: &mut World, this: Entity) {
    if !world.obj::<KongStatueLogic>(this).konkey_dong {
        return;
    }
    world.obj_mut::<KongStatueLogic>(this).konkey_dong = false;
    pg_engine::view::remove_game_frame_listener(world, this);
    world.obj_mut::<KongStatueLogic>(this).konkey_dong_pygmy_logic = None;

    let bonus_splash_logic = world.obj::<KongStatueLogic>(this).konkey_dong_bonus_splash_logic;
    if let Some(bonus_splash_logic) = bonus_splash_logic {
        let splash_display_object = pg_engine::logic::display_object(world, bonus_splash_logic);
        display::remove_from_display_group(world, splash_display_object);
    }

    let boulders = world.obj::<KongStatueLogic>(this).konkey_dong_boulder_logic_array.clone();
    for boulder in boulders {
        let boulder_display_object = pg_engine::logic::display_object(world, boulder);
        display::remove_from_display_group(world, boulder_display_object);
    }

    let statue = world.obj::<KongStatueLogic>(this);
    let level_display_group = active_level(statue).display_group;
    if let Some(level_display_group) = level_display_group {
        display::remove_from_display_group(world, level_display_group);
    }

    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_konkey_dong_stop(world, listener, this);
    }
}

/// `-[KongStatueLogic endLevel]` @ 0x100187864
///
/// The pygmy reached the top: the boulders stop and Konkey Dong plays his end of level.
pub fn end_level(world: &mut World, this: Entity) {
    let boulders = world.obj::<KongStatueLogic>(this).konkey_dong_boulder_logic_array.clone();
    for boulder in boulders {
        konkey_dong_boulder_logic::end_level(world, boulder);
    }

    let konkey_dong_logic = world.obj::<KongStatueLogic>(this).konkey_dong_logic;
    if let Some(konkey_dong_logic) = konkey_dong_logic {
        konkey_dong_logic::end_level(world, konkey_dong_logic);
    }
}

/// `-[KongStatueLogic endLevelComplete]` @ 0x100187998
///
/// Moves on to the next level (back to the first after the last), stacked 320 above the
/// current camera position, scores 5000 and starts moving the camera up to it.
pub fn end_level_complete(world: &mut World, this: Entity) {
    {
        let mut statue = world.obj_mut::<KongStatueLogic>(this);
        let next_level_index = statue.konkey_dong_level_index + 1;
        let level_count = statue.konkey_dong_level_count;
        let loops = arm_sdiv(next_level_index, level_count);
        statue.konkey_dong_level_index = next_level_index - loops * level_count;
        statue.konkey_dong_level_complete_count = statue.konkey_dong_level_complete_count + 1;
    }

    let marker = pg_engine::library::display_marker(world, LEVEL_DISPLAY_MARKER);
    let statue = world.obj::<KongStatueLogic>(this);
    let level_display_group = active_level(statue).display_group;
    if let Some(marker) = marker {
        display::insert_after(world, marker, level_display_group);
    }

    let konkey_dong_display_group = pg_engine::library::display_group(world, KONKEY_DONG_DISPLAY_GROUP);
    let konkey_dong_display_group = konkey_dong_display_group.expect("KongStatueLogic: no KonkeyDongDisplayGroup (the original dereferences [nil pos])");
    let group_y_pos = display::object(world, konkey_dong_display_group).pos.y;
    let level_y_pos = 320.0 - group_y_pos;
    init_active_level(world, this, level_y_pos);

    let loop_count = completed_loop_count(world, this);
    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    if let Some(listener) = listener {
        crate::controller::on_konkey_dong_score(world, listener, 5000);
    }
    let konkey_dong_logic = world.obj::<KongStatueLogic>(this).konkey_dong_logic;
    if let Some(konkey_dong_logic) = konkey_dong_logic {
        konkey_dong_logic::start_next_level(world, konkey_dong_logic, loop_count);
    }
    if let Some(listener) = listener {
        crate::controller::on_konkey_dong_move_camera_to_next_level(world, listener, level_y_pos);
    }
}

/// `-[KongStatueLogic levelTransitionComplete]` @ 0x100187b28
///
/// The camera has reached the new level: the previous level leaves the display and the pygmy
/// jumps to the new level's start.
pub fn level_transition_complete(world: &mut World, this: Entity) {
    let statue = world.obj::<KongStatueLogic>(this);
    let level_count = statue.konkey_dong_level_count;
    let previous_level_index = statue.konkey_dong_level_index + level_count - 1;
    let loops = arm_sdiv(previous_level_index, level_count);
    let previous_level_index = previous_level_index - loops * level_count;
    let previous_display_group = statue.konkey_dong_level_array[previous_level_index as usize].display_group;
    if let Some(previous_display_group) = previous_display_group {
        display::remove_from_display_group(world, previous_display_group);
    }

    let statue = world.obj::<KongStatueLogic>(this);
    let level = active_level(statue);
    let level_display_group = level.display_group;
    let start_pygmy_pos = level.start_pygmy_pos;
    let pygmy_logic = statue.konkey_dong_pygmy_logic;

    let level_display_group = level_display_group.expect("KongStatueLogic: the level has no group (the original dereferences [nil pos])");
    let level_y_pos = display::object(world, level_display_group).pos.y;
    let pygmy_logic = pygmy_logic.expect("KongStatueLogic: no Konkey Dong pygmy (the original writes through [nil pos])");
    let pygmy_display_object = pg_engine::logic::display_object(world, pygmy_logic);
    {
        let mut pygmy_object = display::object_mut(world, pygmy_display_object);
        pygmy_object.pos.x = start_pygmy_pos.x;
        pygmy_object.pos.y = level_y_pos + start_pygmy_pos.y;
    }
    crate::pygmy::jump_to_next_konkey_dong_level(world, pygmy_logic);
}

/// `-[KongStatueLogic startKonkeyDongKidnap:::]` @ 0x100187c18
///
/// Konkey Dong comes out of the statue with the pygmy. When the Konkey Dong option is on and no
/// pygmy can chase him, a new pygmy is added to do it.
pub fn seq_start_konkey_dong_kidnap(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let marker = pg_engine::library::display_marker(world, "DragDisplayMarker");
    let kidnap_logic = world.obj::<KongStatueLogic>(this).konkey_dong_kidnap_logic;
    let kidnap_display_object = kidnap_logic.map(|logic| pg_engine::logic::display_object(world, logic));
    if let Some(marker) = marker {
        display::insert_before(world, marker, kidnap_display_object);
    }

    let kidnap_pygmy_logic = world.obj::<KongStatueLogic>(this).kidnap_pygmy_logic;
    if let Some(kidnap_logic) = kidnap_logic {
        konkey_dong_logic::kidnap_pygmy(world, kidnap_logic, kidnap_pygmy_logic);
    }
    world.obj_mut::<KongStatueLogic>(this).kidnapping_pygmy = true;

    let listener = world.obj::<KongStatueLogic>(this).kong_statue_logic_listener;
    let pygmies = match listener {
        Some(listener) => crate::controller::pygmy_logic_array(world, listener, this),
        None => Vec::new(),
    };
    let mut no_pygmy_can_chase = true;
    for pygmy in pygmies {
        if crate::pygmy::can_chase_konkey_dong(world, pygmy) {
            no_pygmy_can_chase = false;
            break;
        }
    }

    if let Some(listener) = listener {
        let konkey_dong_enabled = crate::controller::is_konkey_dong_enabled(world, listener);
        if konkey_dong_enabled && no_pygmy_can_chase {
            crate::controller::add_pygmy(world, listener);
        }
    }
    ItemResult::Done
}

/// `-[KongStatueLogic touchBeganInside:allTouches:]` @ 0x100187e20
pub fn touch_began_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[KongStatueLogic touchBeganOutside:allTouches:]` @ 0x100187e24
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[KongStatueLogic touchMovedEntered:allTouches:]` @ 0x100187e28
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[KongStatueLogic touchMovedExited:allTouches:]` @ 0x100187e2c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[KongStatueLogic touchMovedInside:allTouches:]` @ 0x100187e30
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// `-[KongStatueLogic touchMovedOutside:allTouches:]` @ 0x100187e40
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[KongStatueLogic touchEndedInside:allTouches:]` @ 0x100187e44
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[KongStatueLogic touchEndedOutside:allTouches:]` @ 0x100187e48
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}

/// `-[KongStatueLogic onGameFrame:]` @ 0x100187e58
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[KongStatueLogic onMultiModelEvent:eventId:]` @ 0x100187e5c
///
/// The bonus splash and the start overlay leave the display when their animations end.
pub fn on_multi_model_event(world: &mut World, this: Entity, _logic: Entity, event_id: &str) {
    let statue = world.obj::<KongStatueLogic>(this);
    let finished_logic = if event_id == "BonusSplashComplete" {
        statue.konkey_dong_bonus_splash_logic
    } else if event_id == "KonkeyDongOverlayComplete" {
        statue.konkey_dong_overlay_logic
    } else {
        return;
    };

    if let Some(finished_logic) = finished_logic {
        let finished_display_object = pg_engine::logic::display_object(world, finished_logic);
        display::remove_from_display_group(world, finished_display_object);
    }
}

/// `-[KongStatueLogic parseStructXml]` @ 0x100187f04
///
/// Builds `mKonkeyDongLevelArray` from the level list (`docs/re/struct-xml.md` §10), node by
/// node in list order: the level (its group, four arrays, the vegetation logic, its size), its
/// kong and pygmy start positions, then its platforms, ladders, picks and beetles.
pub fn parse_struct_xml(world: &mut World, this: Entity) {
    let level_defs = konkey_dong_level_defs();
    for level_def in &level_defs.levels {
        let level_index = world.obj::<KongStatueLogic>(this).konkey_dong_level_count as usize;
        let display_group = pg_engine::library::display_group(world, &level_def.display_group);
        {
            let mut statue = world.obj_mut::<KongStatueLogic>(this);
            let level = &mut statue.konkey_dong_level_array[level_index];
            level.display_group = display_group;
            level.platform_logic_array = Vec::with_capacity(5);
            level.ladder_logic_array = Vec::with_capacity(5);
            level.pick_logic_array = Vec::with_capacity(5);
            level.beetle_logic_array = Vec::with_capacity(5);
        }

        let vegetation_object = level_object(world, display_group, &level_def.vegetation_object);
        let vegetation_logic = pg_engine::logic::multi_model::spawn(world, vegetation_object);
        world.obj_mut::<KongStatueLogic>(this).konkey_dong_level_array[level_index].vegetation_logic = Some(vegetation_logic);
        pg_engine::logic::set_behavior(world, vegetation_logic, Some(level_def.vegetation_behavior.as_str()));
        {
            let mut statue = world.obj_mut::<KongStatueLogic>(this);
            let level = &mut statue.konkey_dong_level_array[level_index];
            level.level_width = level_def.level_width;
            level.level_height = level_def.level_height;
            statue.konkey_dong_level_count = statue.konkey_dong_level_count + 1;
        }

        {
            let mut statue = world.obj_mut::<KongStatueLogic>(this);
            let level = &mut statue.konkey_dong_level_array[level_index];
            if let Some([x_pos, y_pos]) = level_def.kong_pos {
                level.kong_pos.x = x_pos;
                level.kong_pos.y = y_pos;
            }
            if let Some([x_pos, y_pos]) = level_def.start_pygmy_pos {
                level.start_pygmy_pos.x = x_pos;
                level.start_pygmy_pos.y = y_pos;
            }
        }

        for platform_def in &level_def.platforms {
            let platform_object = level_object(world, display_group, &platform_def.object);
            let platform = konkey_dong_platform_logic::spawn(world, platform_object);
            konkey_dong_platform_logic::set_konkey_dong_platform_logic_listener(world, platform, this);
            konkey_dong_platform_logic::set_start_platform(world, platform, platform_def.start_platform);
            konkey_dong_platform_logic::set_final_platform(world, platform, platform_def.final_platform);
            konkey_dong_platform_logic::set_boulder_x_pos_vel(world, platform, platform_def.boulder_x_pos_vel);
            konkey_dong_platform_logic::set_flip_boulder_x_pos_vel(world, platform, platform_def.flip_boulder_x_pos_vel);
            pg_engine::logic::set_behavior(world, platform, Some(platform_def.behavior.as_str()));

            let mut statue = world.obj_mut::<KongStatueLogic>(this);
            statue.konkey_dong_level_array[level_index].platform_logic_array.push(platform);
        }

        for ladder_def in &level_def.ladders {
            let ladder_object = level_object(world, display_group, &ladder_def.object);
            let ladder = konkey_dong_ladder_logic::spawn(world, ladder_object);
            konkey_dong_ladder_logic::set_konkey_dong_ladder_logic_listener(world, ladder, this);
            konkey_dong_ladder_logic::set_force_boulder_drop(world, ladder, ladder_def.force_boulder_drop);
            konkey_dong_ladder_logic::set_do_not_pass(world, ladder, ladder_def.do_not_pass);
            pg_engine::logic::set_behavior(world, ladder, Some(ladder_def.behavior.as_str()));

            let mut statue = world.obj_mut::<KongStatueLogic>(this);
            statue.konkey_dong_level_array[level_index].ladder_logic_array.push(ladder);
        }

        for pick_def in &level_def.picks {
            let pick_object = level_object(world, display_group, &pick_def.object);
            let pick = konkey_dong_pick_logic::spawn(world, pick_object);
            konkey_dong_pick_logic::set_loop_lock(world, pick, pick_def.loop_lock);

            let mut statue = world.obj_mut::<KongStatueLogic>(this);
            statue.konkey_dong_level_array[level_index].pick_logic_array.push(pick);
        }

        for beetle_def in &level_def.beetles {
            let beetle_object = level_object(world, display_group, &beetle_def.object);
            let beetle = konkey_dong_beetle_logic::spawn(world, beetle_object);
            konkey_dong_beetle_logic::set_konkey_dong_beetle_logic_listener(world, beetle, this);
            konkey_dong_beetle_logic::set_loop_unlock(world, beetle, beetle_def.loop_unlock);

            let mut statue = world.obj_mut::<KongStatueLogic>(this);
            statue.konkey_dong_level_array[level_index].beetle_logic_array.push(beetle);
        }
    }
}

/// `[mDisplayGroup getDisplayObject:id]` for a level node's object.
fn level_object(world: &World, display_group: Option<Entity>, id: &str) -> Entity {
    let object = display_group.and_then(|group| group::get_display_object(world, group, id));
    object.expect("every Konkey Dong level object is in its level's display group")
}
