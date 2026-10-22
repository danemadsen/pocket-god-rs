//! `PocketGodViewController`: methods in this part of the class.
//!
//! The minigames the controller hosts: Ooga Jump (the doodle clouds and the doodle sky), Pain
//! Drain, The Runs, Konkey Dong, Moron Pests, No Pee, Dance Dance Execution and Battle of the
//! Gods. A game takes the screen from the island (its display group replaces the island's, its
//! pygmy moves to the game's marker), runs its start and stop sequences and reports its score to
//! the challenge manager.

use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::multi_model;
use pg_engine::library::{self, Library};
use pg_engine::logic;
use pg_engine::prefs;
use pg_engine::sequence;
use pg_engine::texture;

use crate::logic::misfit_oogajump::doodle_cloud_logic;
use crate::logic::theruns::crack_hole_logic;
use crate::managers::{challenge_manager, idol_manager};
use crate::prelude::*;

use super::*;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerMinigames {
    /// `mDoodleDisplayGroup`: `@"BCDisplayGroup"`
    pub doodle_display_group: Option<Entity>,

    /// `mPainDrainDisplayGroup`: `@"BCDisplayGroup"`
    pub pain_drain_display_group: Option<Entity>,

    /// `mTheRunsDisplayGroup`: `@"BCDisplayGroup"`
    pub the_runs_display_group: Option<Entity>,

    /// `mKonkeyDongDisplayGroup`: `@"BCDisplayGroup"`
    pub konkey_dong_display_group: Option<Entity>,

    /// `mMoronPestsDisplayGroup`: `@"BCDisplayGroup"`
    pub moron_pests_display_group: Option<Entity>,

    /// `mNoPeeDisplayGroup`: `@"BCDisplayGroup"`
    pub no_pee_display_group: Option<Entity>,

    /// `mBattleOfTheGodsDisplayGroup`: `@"BCDisplayGroup"`
    pub battle_of_the_gods_display_group: Option<Entity>,

    /// `mDoodleCloudDisplayMarker`: `@"BCDisplayMarker"`
    pub doodle_cloud_display_marker: Option<Entity>,

    /// `mDoodlePygmyDisplayMarker`: `@"BCDisplayMarker"`
    pub doodle_pygmy_display_marker: Option<Entity>,

    /// `mPainDrainPlatformDisplayMarker`: `@"BCDisplayMarker"`
    pub pain_drain_platform_display_marker: Option<Entity>,

    /// `mPainDrainPygmyDisplayMarker`: `@"BCDisplayMarker"`
    pub pain_drain_pygmy_display_marker: Option<Entity>,

    /// `mTheRunsPygmyDisplayMarker`: `@"BCDisplayMarker"`
    pub the_runs_pygmy_display_marker: Option<Entity>,

    /// `mKonkeyDongPygmyDisplayMarker`: `@"BCDisplayMarker"`
    pub konkey_dong_pygmy_display_marker: Option<Entity>,

    /// `mMoronPestsPygmyDisplayMarker`: `@"BCDisplayMarker"`
    pub moron_pests_pygmy_display_marker: Option<Entity>,

    /// `mNoPeePygmyDisplayMarker`: `@"BCDisplayMarker"`
    pub no_pee_pygmy_display_marker: Option<Entity>,

    /// `mBattleOfTheGodsPygmyDisplayMarker`: `@"BCDisplayMarker"`
    pub battle_of_the_gods_pygmy_display_marker: Option<Entity>,

    /// `mTheRunsSequence`: `@"BCSequence"`
    pub the_runs_sequence: Option<Entity>,

    /// `mKonkeyDongSequence`: `@"BCSequence"`
    pub konkey_dong_sequence: Option<Entity>,

    /// `mMoronPestsSequence`: `@"BCSequence"`
    pub moron_pests_sequence: Option<Entity>,

    /// `mNoPeeSequence`: `@"BCSequence"`
    pub no_pee_sequence: Option<Entity>,

    /// `mBattleOfTheGodsSequence`: `@"BCSequence"`
    pub battle_of_the_gods_sequence: Option<Entity>,

    /// `mDoodleJumpPygmyLogic`: `@"PygmyLogic"`
    pub doodle_jump_pygmy_logic: Option<Entity>,

    /// `mDoodleCloudLogicArray`: `@"NSMutableArray"`
    pub doodle_cloud_logic_array: Vec<Entity>,

    /// `mbDoodleJumpFall`: `B`
    pub doodle_jump_fall: bool,

    /// `mbDoodleDebugPress`: `B`
    pub doodle_debug_press: bool,

    /// `mbShowDoodleAssets`: `B`
    pub show_doodle_assets: bool,

    /// `mPainDrainPygmyLogic`: `@"PygmyLogic"`
    pub pain_drain_pygmy_logic: Option<Entity>,

    /// `mbShowPainDrainAssets`: `B`
    pub show_pain_drain_assets: bool,

    /// `mbTheRuns`: `B`
    pub the_runs: bool,

    /// `mbTheRunsStarting`: `B`
    pub the_runs_starting: bool,

    /// `mbTheRunsMoveCamera`: `B`
    pub the_runs_move_camera: bool,

    /// `mTheRunsPygmyLogic`: `@"PygmyLogic"`
    pub the_runs_pygmy_logic: Option<Entity>,

    /// `mbShowTheRunsAssets`: `B`
    pub show_the_runs_assets: bool,

    /// `mbKonkeyDong`: `B`
    pub konkey_dong: bool,

    /// `mbKonkeyDongStarting`: `B`
    pub konkey_dong_starting: bool,

    /// `mbKonkeyDongMoveCamera`: `B`
    pub konkey_dong_move_camera: bool,

    /// `mbKonkeyDongMoveCameraToNextLevel`: `B`
    pub konkey_dong_move_camera_to_next_level: bool,

    /// `mKonkeyDongLevelYPos`: `f`
    pub konkey_dong_level_y_pos: f32,

    /// `mKonkeyDongPygmyLogic`: `@"PygmyLogic"`
    pub konkey_dong_pygmy_logic: Option<Entity>,

    /// `mbShowKonkeyDongAssets`: `B`
    pub show_konkey_dong_assets: bool,

    /// `mbMoronPests`: `B`
    pub moron_pests: bool,

    /// `mbMoronPestsMoveCamera`: `B`
    pub moron_pests_move_camera: bool,

    /// `mMoronPestsPygmyLogic`: `@"PygmyLogic"`
    pub moron_pests_pygmy_logic: Option<Entity>,

    /// `mbShowMoronPestsAssets`: `B`
    pub show_moron_pests_assets: bool,

    /// `mbNoPee`: `B`
    pub no_pee: bool,

    /// `mbNoPeeStarting`: `B`
    pub no_pee_starting: bool,

    /// `mbNoPeeMoveCamera`: `B`
    pub no_pee_move_camera: bool,

    /// `mNoPeePygmyLogic`: `@"PygmyLogic"`
    pub no_pee_pygmy_logic: Option<Entity>,

    /// `mbShowNoPeeAssets`: `B`
    pub show_no_pee_assets: bool,

    /// `mbBattleOfTheGods`: `B`
    pub battle_of_the_gods: bool,

    /// `mbShowBattleOfTheGodsAssets`: `B`
    pub show_battle_of_the_gods_assets: bool,

    /// `mBattleOfTheGodsPygmyLogic`: `@"PygmyLogic"`
    pub battle_of_the_gods_pygmy_logic: Option<Entity>,

    /// `mbDanceDanceExecution`: `B`
    pub dance_dance_execution: bool,

    /// `mbPainDrain`: `B`
    pub pain_drain: bool,
}

/// The clouds `createDoodleClouds` makes for one Ooga Jump game.
const DOODLE_CLOUD_COUNT: i32 = 50;

/// A cloud whose y is more than this below the bottom of the screen is placed again on top.
const DOODLE_CLOUD_RECYCLE_MARGIN: f32 = 20.0;

/// `mIslandLocationState` values where a fall into a crack does not start The Runs.
const LOCATION_UNDERWATER: i32 = 2;
const LOCATION_UNDERWATER_CORAL_REEF: i32 = 9;

/// The challenge manager's game ids of the minigames.
const OOGA_JUMP_CHALLENGE: i32 = 4;
const PAIN_DRAIN_CHALLENGE: i32 = 10;
const THE_RUNS_CHALLENGE: i32 = 11;
const KONKEY_DONG_CHALLENGE: i32 = 12;
const MORON_PESTS_CHALLENGE: i32 = 13;
const DANCE_DANCE_EXECUTION_CHALLENGE: i32 = 14;
const NO_PEE_CHALLENGE: i32 = 15;

/// `DAT_100751f38`: the main atlas slot, which the battle arena borrows for its own image.
const MAIN_ATLAS_ID: &str = "atlas0";
const MAIN_ATLAS_IMAGE: &str = "TextureAtlas0";
const MAIN_BATTLE_ATLAS_IMAGE: &str = "MainBattleAtlas0";

/// `-[PocketGodViewController (Game) stopTheRunsShakeScreen:::]` @ 0x10015e608
///
/// The end of `TheRunsShakeBackgroundStop`: `mEarthquakeSequence` is stopped and released, and
/// the course stands still again.
pub fn seq_stop_the_runs_shake_screen(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let earthquake_sequence = world.obj::<ControllerGodPowers>(this).earthquake_sequence;
    if let Some(earthquake_sequence) = earthquake_sequence {
        sequence::stop(world, earthquake_sequence);

        // The release frees the sequence. The one running this item is still used by its
        // runner when the item returns, so it is only left stopped.
        if earthquake_sequence != call.sequence {
            world.despawn(earthquake_sequence);
        }
    }
    world.obj_mut::<ControllerGodPowers>(this).earthquake_sequence = None;

    let the_runs_display_group = world.obj::<ControllerMinigames>(this).the_runs_display_group;
    if let Some(the_runs_display_group) = the_runs_display_group {
        display::object_mut(world, the_runs_display_group).pos.y = 0.0;
    }
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) removeDoodleClouds]` @ 0x10015ecc4
///
/// The Ooga Jump clouds leave the display tree and shut down; emptying the array releases them.
pub fn remove_doodle_clouds(world: &mut World, this: Entity) {
    let doodle_clouds = world.obj::<ControllerMinigames>(this).doodle_cloud_logic_array.clone();
    for &doodle_cloud in &doodle_clouds {
        let cloud_display_object = logic::display_object(world, doodle_cloud);
        display::remove_from_display_group(world, cloud_display_object);
        logic::shutdown(world, doodle_cloud);
    }

    world.obj_mut::<ControllerMinigames>(this).doodle_cloud_logic_array.clear();
    for doodle_cloud in doodle_clouds {
        doodle_cloud_logic::dealloc(world, doodle_cloud);
    }
}

/// `-[PocketGodViewController (Game) createDoodleClouds]` @ 0x10015ee34
///
/// 50 clouds, each a `DoodleCloud` multi model put right after `DoodleCloudDisplayMarker`.
pub fn create_doodle_clouds(world: &mut World, this: Entity) {
    for _ in 0..DOODLE_CLOUD_COUNT {
        let cloud_model = multi_model::spawn(world, Some("DoodleCloud"));
        let doodle_cloud_display_marker = world.obj::<ControllerMinigames>(this).doodle_cloud_display_marker;
        if let Some(doodle_cloud_display_marker) = doodle_cloud_display_marker {
            display::insert_after(world, doodle_cloud_display_marker, Some(cloud_model));
        }

        let doodle_cloud = doodle_cloud_logic::spawn(world, cloud_model);
        doodle_cloud_logic::set_doodle_cloud_logic_listener(world, doodle_cloud, this);
        world.obj_mut::<ControllerMinigames>(this).doodle_cloud_logic_array.push(doodle_cloud);
    }
}

/// `-[PocketGodViewController (Game) recycleDoodleClouds:]` @ 0x10015ef84
///
/// Clouds that dropped below the bottom of the screen (`screen_bottom`, in island coordinates)
/// are placed again above the highest one.
pub fn recycle_doodle_clouds(world: &mut World, this: Entity, screen_bottom: f32) {
    let doodle_clouds = world.obj::<ControllerMinigames>(this).doodle_cloud_logic_array.clone();
    for doodle_cloud in doodle_clouds {
        let cloud_display_object = logic::display_object(world, doodle_cloud);
        let cloud_y = display::object(world, cloud_display_object).pos.y;
        if cloud_y + DOODLE_CLOUD_RECYCLE_MARGIN < screen_bottom {
            doodle_cloud_logic::move_to_top(world, doodle_cloud);
        }
    }
}

/// `-[PocketGodViewController (Game) isDoodleJumpingPygmy]` @ 0x100163be0
pub fn is_doodle_jumping_pygmy(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic.is_some()
}

/// `-[PocketGodViewController (Game) onPygmyDoodleJumpFall:]` @ 0x1001640ec
pub fn on_pygmy_doodle_jump_fall(world: &mut World, this: Entity, _pygmy_logic: Entity) {
    world.obj_mut::<ControllerMinigames>(this).doodle_jump_fall = true;
}

/// `-[PocketGodViewController (Game) showDoodleDisplayGroup:]` @ 0x100164104
///
/// Once the camera climbs past the island the doodle sky takes its place, and the jumping
/// pygmy is drawn at `DoodlePygmyDisplayMarker`.
pub fn show_doodle_display_group(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let doodle_display_group = world.obj::<ControllerMinigames>(this).doodle_display_group;
    if parent_display_group(world, doodle_display_group).is_some() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_doodle_assets = true;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let doodle_display_group = world.obj::<ControllerMinigames>(this).doodle_display_group;
    replace_display_group(world, island_display_group, doodle_display_group, island_root_display_marker);

    if let Some(doodle_display_group) = doodle_display_group {
        let sky_tiles = group::get_display_object(world, doodle_display_group, "SkyTiles");
        if let Some(sky_tiles) = sky_tiles {
            display::object_mut(world, sky_tiles).hidden = false;
        }
    }

    move_pygmy_to_marker(world, "DoodlePygmyDisplayMarker", Some(pygmy_logic));
}

/// `-[PocketGodViewController (Game) hideDoodleDisplayGroup:]` @ 0x100164268
///
/// The island comes back in place of the doodle sky, with the pygmy at `PygmyDisplayMarker`.
pub fn hide_doodle_display_group(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let doodle_display_group = world.obj::<ControllerMinigames>(this).doodle_display_group;
    if parent_display_group(world, doodle_display_group).is_none() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_doodle_assets = false;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let doodle_display_group = world.obj::<ControllerMinigames>(this).doodle_display_group;
    replace_display_group(world, doodle_display_group, island_display_group, island_root_display_marker);

    move_pygmy_to_marker(world, "PygmyDisplayMarker", Some(pygmy_logic));
}

/// `-[PocketGodViewController (Game) onPygmyStartDoodleJump:]` @ 0x10016439c
pub fn on_pygmy_start_doodle_jump(world: &mut World, this: Entity, pygmy_logic: Entity) {
    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.doodle_jump_fall = false;
        minigames.doodle_jump_pygmy_logic = Some(pygmy_logic);
    }
    create_doodle_clouds(world, this);
}

/// `-[PocketGodViewController (Game) onPygmyStopDoodleJump:]` @ 0x1001643cc
pub fn on_pygmy_stop_doodle_jump(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let show_doodle_assets = world.obj::<ControllerMinigames>(this).show_doodle_assets;
    if show_doodle_assets {
        // Ghidra shows no argument, but x2 still holds this method's pygmy.
        hide_doodle_display_group(world, this, pygmy_logic);
    }

    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.doodle_jump_fall = false;
        minigames.doodle_debug_press = false;
        minigames.doodle_jump_pygmy_logic = None;
    }
    remove_doodle_clouds(world, this);
    stop_challenge_game(world, this, OOGA_JUMP_CHALLENGE);
}

/// `-[PocketGodViewController (Game) onPainDrainStart:pygmyLogic:]` @ 0x100166324
pub fn on_pain_drain_start(world: &mut World, this: Entity, _pain_drain_logic: Entity, pygmy_logic: Entity) {
    clear_screen_for_minigame(world, this);
    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.pain_drain = true;
        minigames.pain_drain_pygmy_logic = Some(pygmy_logic);
    }
    show_pain_drain_display_group(world, this);
}

/// `-[PocketGodViewController (Game) onPainDrainStop:]` @ 0x1001663b0
pub fn on_pain_drain_stop(world: &mut World, this: Entity, _pain_drain_logic: Entity) {
    world.obj_mut::<ControllerMinigames>(this).pain_drain = false;
    hide_pain_drain_display_group(world, this);
    world.obj_mut::<ControllerMinigames>(this).pain_drain_pygmy_logic = None;
    show_ui(world, this);
    stop_challenge_game(world, this, PAIN_DRAIN_CHALLENGE);
}

/// `-[PocketGodViewController (Game) onPainDrainScore:]` @ 0x100166430
pub fn on_pain_drain_score(world: &mut World, this: Entity, score: i32) {
    set_challenge_game_score(world, this, PAIN_DRAIN_CHALLENGE, score);
}

/// `-[PocketGodViewController (Game) showPainDrainDisplayGroup]` @ 0x10016645c
///
/// The drain replaces the underwater scene at `UnderwaterDisplayMarker`.
pub fn show_pain_drain_display_group(world: &mut World, this: Entity) {
    let pain_drain_display_group = world.obj::<ControllerMinigames>(this).pain_drain_display_group;
    if parent_display_group(world, pain_drain_display_group).is_some() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_pain_drain_assets = true;
    load_location_texture_atlas(world, this);

    let (underwater_display_group, underwater_display_marker) = underwater_display_group_and_marker(world, this);
    let pain_drain_display_group = world.obj::<ControllerMinigames>(this).pain_drain_display_group;
    replace_display_group(world, underwater_display_group, pain_drain_display_group, underwater_display_marker);

    let pain_drain_pygmy = world.obj::<ControllerMinigames>(this).pain_drain_pygmy_logic;
    move_pygmy_to_marker(world, "PainDrainPygmyDisplayMarker", pain_drain_pygmy);
}

/// `-[PocketGodViewController (Game) hidePainDrainDisplayGroup]` @ 0x10016659c
///
/// The underwater scene comes back; the pygmy is left where it is.
pub fn hide_pain_drain_display_group(world: &mut World, this: Entity) {
    let pain_drain_display_group = world.obj::<ControllerMinigames>(this).pain_drain_display_group;
    if parent_display_group(world, pain_drain_display_group).is_none() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_pain_drain_assets = false;
    load_location_texture_atlas(world, this);

    let (underwater_display_group, underwater_display_marker) = underwater_display_group_and_marker(world, this);
    let pain_drain_display_group = world.obj::<ControllerMinigames>(this).pain_drain_display_group;
    replace_display_group(world, pain_drain_display_group, underwater_display_group, underwater_display_marker);
}

/// `-[PocketGodViewController (Game) onPygmyFallIntoCrackHoleComplete:]` @ 0x10016664c
///
/// With The Runs on, a pygmy that fell into a crack starts the game, except underwater and
/// during an Ooga Jump.
pub fn on_pygmy_fall_into_crack_hole_complete(world: &mut World, this: Entity, pygmy_logic: Entity) {
    let the_runs_state = option_int_preference(world, this, "TheRunsState");
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let underwater =
        island_location_state == LOCATION_UNDERWATER || island_location_state == LOCATION_UNDERWATER_CORAL_REEF;
    let doodle_jump_pygmy = world.obj::<ControllerMinigames>(this).doodle_jump_pygmy_logic;
    if the_runs_state != 1 || underwater || doodle_jump_pygmy.is_some() {
        return;
    }

    cancel_all_touches(world, this);
    let crack_hole = world.obj::<ControllerWorldRegistry>(this).crack_hole_logic;
    if let Some(crack_hole) = crack_hole {
        crack_hole_logic::start_the_runs(world, crack_hole, pygmy_logic);
    }
}

/// `-[PocketGodViewController (Game) onTheRunsStart:pygmyLogic:]` @ 0x10016670c
pub fn on_the_runs_start(world: &mut World, this: Entity, _crack_hole: Entity, pygmy_logic: Entity) {
    clear_screen_for_minigame(world, this);
    let the_runs_sequence = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.the_runs = true;
        minigames.the_runs_move_camera = false;
        minigames.the_runs_pygmy_logic = Some(pygmy_logic);
        minigames.the_runs_sequence
    };
    set_minigame_sequence(world, the_runs_sequence, "StartTheRuns");
}

/// `-[PocketGodViewController (Game) onTheRunsStop:]` @ 0x1001667ec
pub fn on_the_runs_stop(world: &mut World, this: Entity, _crack_hole: Entity) {
    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.the_runs = false;
        minigames.the_runs_starting = false;
    }
    hide_the_runs_display_group(world, this);

    let the_runs_display_group = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.the_runs_move_camera = false;
        minigames.the_runs_pygmy_logic = None;
        minigames.the_runs_display_group
    };
    reset_display_group_x(world, the_runs_display_group);
    show_ui(world, this);
    stop_challenge_game(world, this, THE_RUNS_CHALLENGE);

    let the_runs_sequence = world.obj::<ControllerMinigames>(this).the_runs_sequence;
    set_minigame_sequence(world, the_runs_sequence, "StopTheRuns");
}

/// `-[PocketGodViewController (Game) onTheRunsScore:]` @ 0x100166908
pub fn on_the_runs_score(world: &mut World, this: Entity, score: i32) {
    set_challenge_game_score(world, this, THE_RUNS_CHALLENGE, score);
}

/// `-[PocketGodViewController (Game) onTheRunsStartMovingCamera]` @ 0x100166934
pub fn on_the_runs_start_moving_camera(world: &mut World, this: Entity) {
    let the_runs_sequence = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.the_runs_move_camera = true;
        minigames.the_runs_sequence
    };
    set_minigame_sequence(world, the_runs_sequence, "StartTheRunsLoop");
}

/// `-[PocketGodViewController (Game) onTheRunsShakeScreenStart]` @ 0x1001669b4
///
/// A new `TheRunsShakeBackgroundStart` sequence becomes `mEarthquakeSequence`. A sequence still
/// there is overwritten as it is: neither stopped nor released.
pub fn on_the_runs_shake_screen_start(world: &mut World, this: Entity) {
    let shake_start_def = library::sequence_def(world, "TheRunsShakeBackgroundStart");
    let shake_sequence = sequence::spawn(world, shake_start_def, Some(this));
    world.obj_mut::<ControllerGodPowers>(this).earthquake_sequence = Some(shake_sequence);
}

/// `-[PocketGodViewController (Game) onTheRunsShakeScreenStop]` @ 0x100166a48
pub fn on_the_runs_shake_screen_stop(world: &mut World, this: Entity) {
    let earthquake_sequence = world.obj::<ControllerGodPowers>(this).earthquake_sequence;
    set_minigame_sequence(world, earthquake_sequence, "TheRunsShakeBackgroundStop");
}

/// `-[PocketGodViewController (Game) showTheRunsDisplayGroup]` @ 0x100166ab4
pub fn show_the_runs_display_group(world: &mut World, this: Entity) {
    let the_runs_display_group = world.obj::<ControllerMinigames>(this).the_runs_display_group;
    if parent_display_group(world, the_runs_display_group).is_some() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_the_runs_assets = true;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let the_runs_display_group = world.obj::<ControllerMinigames>(this).the_runs_display_group;
    replace_display_group(world, island_display_group, the_runs_display_group, island_root_display_marker);

    let the_runs_pygmy = world.obj::<ControllerMinigames>(this).the_runs_pygmy_logic;
    move_pygmy_to_marker(world, "TheRunsPygmyDisplayMarker", the_runs_pygmy);
}

/// `-[PocketGodViewController (Game) hideTheRunsDisplayGroup]` @ 0x100166bf4
pub fn hide_the_runs_display_group(world: &mut World, this: Entity) {
    let the_runs_display_group = world.obj::<ControllerMinigames>(this).the_runs_display_group;
    if parent_display_group(world, the_runs_display_group).is_none() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_the_runs_assets = false;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let the_runs_display_group = world.obj::<ControllerMinigames>(this).the_runs_display_group;
    replace_display_group(world, the_runs_display_group, island_display_group, island_root_display_marker);
}

/// `-[PocketGodViewController (Game) onKonkeyDongStart:pygmyLogic:]` @ 0x100166ca4
pub fn on_konkey_dong_start(world: &mut World, this: Entity, _kong_statue: Entity, pygmy_logic: Entity) {
    clear_screen_for_minigame(world, this);
    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.konkey_dong = true;
        minigames.konkey_dong_move_camera = false;
        minigames.konkey_dong_pygmy_logic = Some(pygmy_logic);
    }
    set_challenge_game_score(world, this, KONKEY_DONG_CHALLENGE, 0);

    let konkey_dong_sequence = world.obj::<ControllerMinigames>(this).konkey_dong_sequence;
    set_minigame_sequence(world, konkey_dong_sequence, "StartKonkeyDong");
}

/// `-[PocketGodViewController (Game) onKonkeyDongStop:]` @ 0x100166dac
pub fn on_konkey_dong_stop(world: &mut World, this: Entity, _kong_statue: Entity) {
    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.konkey_dong = false;
        minigames.konkey_dong_starting = false;
    }
    hide_konkey_dong_display_group(world, this);

    let konkey_dong_display_group = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.konkey_dong_move_camera = false;
        minigames.konkey_dong_pygmy_logic = None;
        minigames.konkey_dong_display_group
    };
    reset_display_group_x(world, konkey_dong_display_group);
    show_ui(world, this);
    stop_challenge_game(world, this, KONKEY_DONG_CHALLENGE);

    let konkey_dong_sequence = world.obj::<ControllerMinigames>(this).konkey_dong_sequence;
    set_minigame_sequence(world, konkey_dong_sequence, "StopKonkeyDong");
}

/// `-[PocketGodViewController (Game) onKonkeyDongScore:]` @ 0x100166ec8
pub fn on_konkey_dong_score(world: &mut World, this: Entity, score: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::increment_game_score_score(world, challenge_manager, KONKEY_DONG_CHALLENGE, score);
    }
}

/// `-[PocketGodViewController (Game) onKonkeyDongStartMovingCamera]` @ 0x100166ef4
pub fn on_konkey_dong_start_moving_camera(world: &mut World, this: Entity) {
    let konkey_dong_sequence = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.konkey_dong_level_y_pos = 0.0;
        minigames.konkey_dong_move_camera = true;
        minigames.konkey_dong_move_camera_to_next_level = false;
        minigames.konkey_dong_sequence
    };
    set_minigame_sequence(world, konkey_dong_sequence, "StartKonkeyDongLoop");
}

/// `-[PocketGodViewController (Game) onKonkeyDongMoveCameraToNextLevel:]` @ 0x100166f94
pub fn on_konkey_dong_move_camera_to_next_level(world: &mut World, this: Entity, level_y_pos: f32) {
    let mut minigames = world.obj_mut::<ControllerMinigames>(this);
    minigames.konkey_dong_move_camera_to_next_level = true;
    minigames.konkey_dong_level_y_pos = level_y_pos;
}

/// `-[PocketGodViewController (Game) showKonkeyDongDisplayGroup]` @ 0x100166fbc
pub fn show_konkey_dong_display_group(world: &mut World, this: Entity) {
    let konkey_dong_display_group = world.obj::<ControllerMinigames>(this).konkey_dong_display_group;
    if parent_display_group(world, konkey_dong_display_group).is_some() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_konkey_dong_assets = true;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let konkey_dong_display_group = world.obj::<ControllerMinigames>(this).konkey_dong_display_group;
    replace_display_group(world, island_display_group, konkey_dong_display_group, island_root_display_marker);

    let konkey_dong_pygmy = world.obj::<ControllerMinigames>(this).konkey_dong_pygmy_logic;
    move_pygmy_to_marker(world, "KonkeyDongPygmyDisplayMarker", konkey_dong_pygmy);
}

/// `-[PocketGodViewController (Game) hideKonkeyDongDisplayGroup]` @ 0x1001670fc
pub fn hide_konkey_dong_display_group(world: &mut World, this: Entity) {
    let konkey_dong_display_group = world.obj::<ControllerMinigames>(this).konkey_dong_display_group;
    if parent_display_group(world, konkey_dong_display_group).is_none() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_konkey_dong_assets = false;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let konkey_dong_display_group = world.obj::<ControllerMinigames>(this).konkey_dong_display_group;
    replace_display_group(world, konkey_dong_display_group, island_display_group, island_root_display_marker);
}

/// `-[PocketGodViewController (Game) onMoronPestsStart:pygmyLogic:]` @ 0x1001671ac
pub fn on_moron_pests_start(world: &mut World, this: Entity, _moron_snake: Entity, pygmy_logic: Entity) {
    clear_screen_for_minigame(world, this);
    let moron_pests_sequence = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.moron_pests = true;
        minigames.moron_pests_move_camera = false;
        minigames.moron_pests_pygmy_logic = Some(pygmy_logic);
        minigames.moron_pests_sequence
    };
    set_minigame_sequence(world, moron_pests_sequence, "StartMoronPests");
}

/// `-[PocketGodViewController (Game) onMoronPestsStop:]` @ 0x10016728c
pub fn on_moron_pests_stop(world: &mut World, this: Entity, _moron_snake: Entity) {
    world.obj_mut::<ControllerMinigames>(this).moron_pests = false;
    hide_moron_pests_display_group(world, this);

    let moron_pests_display_group = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.moron_pests_move_camera = false;
        minigames.moron_pests_pygmy_logic = None;
        minigames.moron_pests_display_group
    };
    reset_display_group_x(world, moron_pests_display_group);
    show_ui(world, this);
    stop_challenge_game(world, this, MORON_PESTS_CHALLENGE);

    let moron_pests_sequence = world.obj::<ControllerMinigames>(this).moron_pests_sequence;
    set_minigame_sequence(world, moron_pests_sequence, "StopMoronPests");
}

/// `-[PocketGodViewController (Game) onMoronPestsScore:]` @ 0x100167398
pub fn on_moron_pests_score(world: &mut World, this: Entity, score: i32) {
    set_challenge_game_score(world, this, MORON_PESTS_CHALLENGE, score);
}

/// `-[PocketGodViewController (Game) onMoronPestsStartMovingCamera]` @ 0x1001673c4
pub fn on_moron_pests_start_moving_camera(world: &mut World, this: Entity) {
    let moron_pests_sequence = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.moron_pests_move_camera = true;
        minigames.moron_pests_sequence
    };
    set_minigame_sequence(world, moron_pests_sequence, "StartMoronPestsLoop");
}

/// `-[PocketGodViewController (Game) onMoronPestsStopMovingCamera]` @ 0x100167444
pub fn on_moron_pests_stop_moving_camera(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerMinigames>(this).moron_pests_move_camera = false;
}

/// `-[PocketGodViewController (Game) showMoronPestsDisplayGroup]` @ 0x100167458
pub fn show_moron_pests_display_group(world: &mut World, this: Entity) {
    let moron_pests_display_group = world.obj::<ControllerMinigames>(this).moron_pests_display_group;
    if parent_display_group(world, moron_pests_display_group).is_some() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_moron_pests_assets = true;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let moron_pests_display_group = world.obj::<ControllerMinigames>(this).moron_pests_display_group;
    replace_display_group(world, island_display_group, moron_pests_display_group, island_root_display_marker);

    let moron_pests_pygmy = world.obj::<ControllerMinigames>(this).moron_pests_pygmy_logic;
    move_pygmy_to_marker(world, "MoronPestsPygmyDisplayMarker", moron_pests_pygmy);
}

/// `-[PocketGodViewController (Game) hideMoronPestsDisplayGroup]` @ 0x100167598
pub fn hide_moron_pests_display_group(world: &mut World, this: Entity) {
    let moron_pests_display_group = world.obj::<ControllerMinigames>(this).moron_pests_display_group;
    if parent_display_group(world, moron_pests_display_group).is_none() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_moron_pests_assets = false;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let moron_pests_display_group = world.obj::<ControllerMinigames>(this).moron_pests_display_group;
    replace_display_group(world, moron_pests_display_group, island_display_group, island_root_display_marker);
}

/// `-[PocketGodViewController (Game) isNoPeeActive]` @ 0x100167648
pub fn is_no_pee_active(world: &mut World, this: Entity) -> bool {
    world.obj::<ControllerMinigames>(this).no_pee
}

/// `-[PocketGodViewController (Game) onNoPeeStart:pygmyLogic:]` @ 0x10016765c
pub fn on_no_pee_start(world: &mut World, this: Entity, _no_pee_logic: Entity, pygmy_logic: Entity) {
    clear_screen_for_minigame(world, this);
    let no_pee_sequence = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.no_pee = true;
        minigames.no_pee_move_camera = false;
        minigames.no_pee_pygmy_logic = Some(pygmy_logic);
        minigames.no_pee_sequence
    };
    set_minigame_sequence(world, no_pee_sequence, "StartNoPee");
}

/// `-[PocketGodViewController (Game) onNoPeeStop:]` @ 0x10016773c
pub fn on_no_pee_stop(world: &mut World, this: Entity, _no_pee_logic: Entity) {
    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.no_pee = false;
        minigames.no_pee_starting = false;
    }
    hide_no_pee_display_group(world, this);

    let no_pee_display_group = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.no_pee_move_camera = false;
        minigames.no_pee_pygmy_logic = None;
        minigames.no_pee_display_group
    };
    reset_display_group_x(world, no_pee_display_group);
    show_ui(world, this);
    stop_challenge_game(world, this, NO_PEE_CHALLENGE);

    let no_pee_sequence = world.obj::<ControllerMinigames>(this).no_pee_sequence;
    set_minigame_sequence(world, no_pee_sequence, "StopNoPee");
}

/// `-[PocketGodViewController (Game) onNoPeeRetry:]` @ 0x100167858
pub fn on_no_pee_retry(world: &mut World, this: Entity, _no_pee_logic: Entity) {
    {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.no_pee_starting = false;
        minigames.no_pee_move_camera = false;
    }
    stop_challenge_game(world, this, NO_PEE_CHALLENGE);
}

/// `-[PocketGodViewController (Game) onNoPeeStartMovingCamera]` @ 0x10016789c
pub fn on_no_pee_start_moving_camera(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerMinigames>(this).no_pee_move_camera = true;
}

/// `-[PocketGodViewController (Game) showNoPeeDisplayGroup]` @ 0x1001678b4
pub fn show_no_pee_display_group(world: &mut World, this: Entity) {
    let no_pee_display_group = world.obj::<ControllerMinigames>(this).no_pee_display_group;
    if parent_display_group(world, no_pee_display_group).is_some() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_no_pee_assets = true;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let no_pee_display_group = world.obj::<ControllerMinigames>(this).no_pee_display_group;
    replace_display_group(world, island_display_group, no_pee_display_group, island_root_display_marker);

    let no_pee_pygmy = world.obj::<ControllerMinigames>(this).no_pee_pygmy_logic;
    move_pygmy_to_marker(world, "NoPeePygmyDisplayMarker", no_pee_pygmy);
}

/// `-[PocketGodViewController (Game) hideNoPeeDisplayGroup]` @ 0x1001679f4
pub fn hide_no_pee_display_group(world: &mut World, this: Entity) {
    let no_pee_display_group = world.obj::<ControllerMinigames>(this).no_pee_display_group;
    if parent_display_group(world, no_pee_display_group).is_none() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_no_pee_assets = false;
    load_location_texture_atlas(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let no_pee_display_group = world.obj::<ControllerMinigames>(this).no_pee_display_group;
    replace_display_group(world, no_pee_display_group, island_display_group, island_root_display_marker);
}

/// `-[PocketGodViewController (Game) onDanceDanceExecutionStart:]` @ 0x100167aa4
pub fn on_dance_dance_execution_start(world: &mut World, this: Entity, _dance_dance_execution: Entity) {
    clear_screen_for_minigame(world, this);
    world.obj_mut::<ControllerMinigames>(this).dance_dance_execution = true;
}

/// `-[PocketGodViewController (Game) onDanceDanceExecutionStop:]` @ 0x100167b0c
pub fn on_dance_dance_execution_stop(world: &mut World, this: Entity, _dance_dance_execution: Entity) {
    world.obj_mut::<ControllerMinigames>(this).dance_dance_execution = false;
    show_ui(world, this);
    stop_challenge_game(world, this, DANCE_DANCE_EXECUTION_CHALLENGE);
}

/// `-[PocketGodViewController (Game) onDanceDanceExecutionScore:]` @ 0x100167b68
pub fn on_dance_dance_execution_score(world: &mut World, this: Entity, score: i32) {
    set_challenge_game_score(world, this, DANCE_DANCE_EXECUTION_CHALLENGE, score);
}

/// `-[PocketGodViewController (Game) onBattleOfTheGodsStart:pygmyLogic:]` @ 0x100167b94
pub fn on_battle_of_the_gods_start(world: &mut World, this: Entity, _temple_door: Entity, pygmy_logic: Entity) {
    clear_screen_for_minigame(world, this);
    let battle_of_the_gods_sequence = {
        let mut minigames = world.obj_mut::<ControllerMinigames>(this);
        minigames.battle_of_the_gods = true;
        minigames.battle_of_the_gods_pygmy_logic = Some(pygmy_logic);
        minigames.battle_of_the_gods_sequence
    };
    set_minigame_sequence(world, battle_of_the_gods_sequence, "StartBattleOfTheGods");
}

/// `-[PocketGodViewController (Game) onBattleOfTheGodsStop:]` @ 0x100167c64
pub fn on_battle_of_the_gods_stop(world: &mut World, this: Entity, _temple_door: Entity) {
    world.obj_mut::<ControllerMinigames>(this).battle_of_the_gods = false;
    hide_battle_of_the_gods_display_group(world, this);
    world.obj_mut::<ControllerMinigames>(this).battle_of_the_gods_pygmy_logic = None;
    show_ui(world, this);

    let battle_of_the_gods_sequence = world.obj::<ControllerMinigames>(this).battle_of_the_gods_sequence;
    set_minigame_sequence(world, battle_of_the_gods_sequence, "StopBattleOfTheGods");
}

/// `-[PocketGodViewController (Game) showBattleOfTheGodsDisplayGroup]` @ 0x100167d18
///
/// The arena replaces the island; its image `MainBattleAtlas0` takes the main atlas slot.
pub fn show_battle_of_the_gods_display_group(world: &mut World, this: Entity) {
    let battle_of_the_gods_display_group = world.obj::<ControllerMinigames>(this).battle_of_the_gods_display_group;
    if parent_display_group(world, battle_of_the_gods_display_group).is_some() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_battle_of_the_gods_assets = true;
    load_location_texture_atlas(world, this);
    load_main_atlas(world, MAIN_BATTLE_ATLAS_IMAGE);
    cancel_all_touches(world, this);

    let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
    let arena_display_group = world.obj::<ControllerMinigames>(this).battle_of_the_gods_display_group;
    replace_display_group(world, island_display_group, arena_display_group, island_root_display_marker);

    let battle_of_the_gods_pygmy = world.obj::<ControllerMinigames>(this).battle_of_the_gods_pygmy_logic;
    move_pygmy_to_marker(world, "BattleOfTheGodsPygmyDisplayMarker", battle_of_the_gods_pygmy);
}

/// `-[PocketGodViewController (Game) hideBattleOfTheGodsDisplayGroup]` @ 0x100167e88
///
/// While an idol screen is open the arena stays, with the battle atlas, until
/// `onIdolScreenClosed`; the pygmy goes back to `PygmyDisplayMarker` either way.
pub fn hide_battle_of_the_gods_display_group(world: &mut World, this: Entity) {
    let battle_of_the_gods_display_group = world.obj::<ControllerMinigames>(this).battle_of_the_gods_display_group;
    if parent_display_group(world, battle_of_the_gods_display_group).is_none() {
        return;
    }

    world.obj_mut::<ControllerMinigames>(this).show_battle_of_the_gods_assets = false;

    let idol_screen_open = world.obj::<ControllerManagersBridge>(this).idol_screen_open;
    if !idol_screen_open {
        load_location_texture_atlas(world, this);
        load_main_atlas(world, MAIN_ATLAS_IMAGE);

        let (island_display_group, island_root_display_marker) = island_display_group_and_marker(world, this);
        let arena_display_group = world.obj::<ControllerMinigames>(this).battle_of_the_gods_display_group;
        replace_display_group(world, arena_display_group, island_display_group, island_root_display_marker);
    }

    let battle_of_the_gods_pygmy = world.obj::<ControllerMinigames>(this).battle_of_the_gods_pygmy_logic;
    if battle_of_the_gods_pygmy.is_some() {
        move_pygmy_to_marker(world, "PygmyDisplayMarker", battle_of_the_gods_pygmy);
    }
}

/// `-[PocketGodViewController (Game) onOpenIdolBattleEnemyScreen]` @ 0x100167ffc
pub fn on_open_idol_battle_enemy_screen(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = true;
    cancel_all_touches(world, this);

    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idol_manager) = idol_manager {
        idol_manager::open_idol_battle_enemy_screen(world, idol_manager);
    }
}

/// `-[PocketGodViewController (Game) onOpenIdolBattleScreen]` @ 0x100168058
pub fn on_open_idol_battle_screen(world: &mut World, this: Entity) {
    world.obj_mut::<ControllerManagersBridge>(this).idol_screen_open = true;

    let idol_manager = world.obj::<ControllerManagersBridge>(this).idol_manager;
    if let Some(idol_manager) = idol_manager {
        idol_manager::open_idol_battle_screen(world, idol_manager);
    }
}

/// `-[PocketGodViewController (Game) onOpenIdolBattleResultsScreen]` @ 0x10016808c
pub fn on_open_idol_battle_results_screen(world: &mut World, this: Entity) {
    let idol_manager = {
        let mut bridge = world.obj_mut::<ControllerManagersBridge>(this);
        bridge.idol_screen_open = true;
        bridge.idol_battle_results_screen_open = true;
        bridge.idol_manager
    };
    if let Some(idol_manager) = idol_manager {
        idol_manager::open_idol_battle_results_screen(world, idol_manager);
    }
}

/// `-[PocketGodViewController (Game) showTheRunsDisplayGroup:::]` @ 0x1001691bc
pub fn seq_show_the_runs_display_group(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    show_the_runs_display_group(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) showKonkeyDongDisplayGroup:::]` @ 0x1001691e0
pub fn seq_show_konkey_dong_display_group(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    show_konkey_dong_display_group(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) showMoronPestsDisplayGroup:::]` @ 0x100169204
pub fn seq_show_moron_pests_display_group(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    show_moron_pests_display_group(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) showNoPeeDisplayGroup:::]` @ 0x100169228
pub fn seq_show_no_pee_display_group(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    show_no_pee_display_group(world, this);
    ItemResult::Done
}

/// `-[PocketGodViewController (Game) showBattleOfTheGodsDisplayGroup:::]` @ 0x10016924c
pub fn seq_show_battle_of_the_gods_display_group(
    world: &mut World,
    this: Entity,
    _call: &mut SequenceCall,
) -> ItemResult {
    show_battle_of_the_gods_display_group(world, this);
    ItemResult::Done
}

/// `[object displayGroup]`: the group an object is attached to (nil for a nil object).
fn parent_display_group(world: &World, object: Option<Entity>) -> Option<Entity> {
    let object = object?;
    display::object(world, object).display_group
}

/// `mIslandDisplayGroup` and `mIslandRootDisplayMarker`.
fn island_display_group_and_marker(world: &World, this: Entity) -> (Option<Entity>, Option<Entity>) {
    let locations = world.obj::<ControllerIslandLocations>(this);
    (locations.island_display_group, locations.island_root_display_marker)
}

/// `mUnderwaterDisplayGroup` and `mUnderwaterDisplayMarker`.
fn underwater_display_group_and_marker(world: &World, this: Entity) -> (Option<Entity>, Option<Entity>) {
    let locations = world.obj::<ControllerIslandLocations>(this);
    (locations.underwater_display_group, locations.underwater_display_marker)
}

/// `[old_group removeFromDisplayGroup]; [new_group moveToDisplayGroup:marker]`: one scene takes
/// the other's place under the marker.
fn replace_display_group(
    world: &mut World,
    old_group: Option<Entity>,
    new_group: Option<Entity>,
    marker: Option<Entity>,
) {
    if let Some(old_group) = old_group {
        display::remove_from_display_group(world, old_group);
    }
    if let Some(new_group) = new_group {
        display::move_to_display_group(world, new_group, marker);
    }
}

/// `[[[BCLibrary instance] getDisplayMarkerById:marker_id] insertAfter:[pygmy displayObject]];
/// [pygmy updateLabelLayer]`: the pygmy, and its name label with it, is drawn at that marker.
fn move_pygmy_to_marker(world: &mut World, marker_id: &str, pygmy: Option<Entity>) {
    let marker = library::display_marker(world, marker_id);
    let pygmy_display_object = match pygmy {
        Some(pygmy) => Some(logic::display_object(world, pygmy)),
        None => None,
    };
    if let Some(marker) = marker {
        display::insert_after(world, marker, pygmy_display_object);
    }

    if let Some(pygmy) = pygmy {
        crate::pygmy::update_label_layer(world, pygmy);
    }
}

/// `[group pos]->x = 0`: a scrolled course back at its start.
fn reset_display_group_x(world: &mut World, group: Option<Entity>) {
    if let Some(group) = group {
        display::object_mut(world, group).pos.x = 0.0;
    }
}

/// `[minigame_sequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]`.
fn set_minigame_sequence(world: &mut World, minigame_sequence: Option<Entity>, id: &str) {
    let def = library::sequence_def(world, id);
    if let Some(minigame_sequence) = minigame_sequence {
        sequence::set_sequence_def(world, minigame_sequence, def);
    }
}

/// `[self cancelAllTouches]; [self closeScreens]; [self hideUi]`: a minigame takes the screen.
fn clear_screen_for_minigame(world: &mut World, this: Entity) {
    cancel_all_touches(world, this);
    close_screens(world, this);
    hide_ui(world, this);
}

/// `[mChallengeManager stopGame:challenge_type]`.
fn stop_challenge_game(world: &mut World, this: Entity, challenge_type: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::stop_game(world, challenge_manager, challenge_type);
    }
}

/// `[mChallengeManager setGameScore:challenge_type score:score]`.
fn set_challenge_game_score(world: &mut World, this: Entity, challenge_type: i32, score: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::set_game_score(world, challenge_manager, challenge_type, score);
    }
}

/// `[mOptionPreferences getIntPreference:key]` (0 while there is no document).
fn option_int_preference(world: &World, this: Entity, key: &str) -> i32 {
    let option_preferences = world.obj::<ControllerBootAndFrame>(this).option_preferences;
    match option_preferences {
        Some(option_preferences) => prefs::get_int_preference(world, option_preferences, key),
        None => 0,
    }
}

/// `FUN_100038d68(image, @"raw", &DAT_100751f38)`: an image into the main atlas slot.
fn load_main_atlas(world: &mut World, image: &str) {
    let main_atlas = world.resource::<Library>().atlas_id(MAIN_ATLAS_ID);
    let Some(main_atlas) = main_atlas else {
        panic!("there is no atlas slot {MAIN_ATLAS_ID}");
    };
    texture::load_atlas(world, main_atlas, image);
}
