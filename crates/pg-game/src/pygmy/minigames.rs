//! `PygmyLogic`: methods in this part of the class: the mini-games played through the pygmy
//! (Ooga Jump, Konkey Dong, Moron Pests, Battle of the Gods and The Runs).

use crate::prelude::*;
use super::*;

use pg_assets::data::episodes::{Attack, AttackStyle, Episode, Realm};
use pg_engine::display::model::SPRITE_Z;
use pg_engine::sequence::SequenceItemDef;
use pg_engine::touch::BcTouch;

use crate::logic::battle::{battle_energy_ball_logic, temple_door_logic};
use crate::logic::island_core::sprite_logic::SpriteLogic;
use crate::logic::konkeydong::kong_statue_logic::{self, KongStatueLogic};
use crate::logic::konkeydong::{
    konkey_dong_beetle_logic, konkey_dong_boulder_logic, konkey_dong_ladder_logic, konkey_dong_pick_logic,
    konkey_dong_platform_logic,
};
use crate::logic::misfit_oogajump::doodle_cloud_logic;
use crate::logic::moronpests_chase::{moron_snake_logic, moron_turtle_logic};
use crate::logic::sky_weather::cloud_logic;
use crate::logic::theruns::crack_hole_logic::{self, CrackHoleLogic};
use crate::logic::theruns::{the_runs_platform_logic, the_runs_rock_logic, the_runs_stalactite_logic, the_runs_turd_logic};
use crate::logic_table;
use crate::managers::idol_manager;

/// `PygmyLogic` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyMinigames {
    /// `mbTheRunsStarted`: `B`
    pub the_runs_started: bool,

    /// `mbTheRunsJumping`: `B`
    pub the_runs_jumping: bool,

    /// `mbTheRunsSkidding`: `B`
    pub the_runs_skidding: bool,

    /// `mbTheRunsRockExplode`: `B`
    pub the_runs_rock_explode: bool,

    /// `mbTheRunsPlatformCollision`: `B`
    pub the_runs_platform_collision: bool,

    /// `mbTheRunsFallOffSinkingPlatform`: `B`
    pub the_runs_fall_off_sinking_platform: bool,

    /// `mbTheRunsMaxSpeed`: `B`
    pub the_runs_max_speed: bool,

    /// `mbKonkeyDongStarted`: `B`
    pub konkey_dong_started: bool,

    /// `mbKonkeyDongRunning`: `B`
    pub konkey_dong_running: bool,

    /// `mbKonkeyDongFalling`: `B`
    pub konkey_dong_falling: bool,

    /// `mbKonkeyDongOnLadder`: `B`
    pub konkey_dong_on_ladder: bool,

    /// `mbKonkeyDongHoldingPick`: `B`
    pub konkey_dong_holding_pick: bool,

    /// `mbKonkeyDongLevelTransition`: `B`
    pub konkey_dong_level_transition: bool,

    /// `mbKonkeyDongPlatformCollision`: `B`
    pub konkey_dong_platform_collision: bool,

    /// `mbKonkeyDongJumpBoulder`: `B`
    pub konkey_dong_jump_boulder: bool,

    /// `mbKonkeyDongJumpBeetle`: `B`
    pub konkey_dong_jump_beetle: bool,

    /// `mbMoronPestsHitTurtle`: `B`
    pub moron_pests_hit_turtle: bool,

    /// `mbMoronPestsStartRunning`: `B`
    pub moron_pests_start_running: bool,

    /// `mbMoronPestsFloorCollision`: `B`
    pub moron_pests_floor_collision: bool,

    /// `mMoronPestsTargetXPosVel`: `f`
    pub moron_pests_target_x_pos_vel: f32,

    /// `mbKonkeyDongClimbing`: `B`
    pub konkey_dong_climbing: bool,

    /// `mbKonkeyDongLadderUp`: `B`
    pub konkey_dong_ladder_up: bool,

    /// `mKonkeyDongJumpTouch`: `@"BCTouch"`
    pub konkey_dong_jump_touch: Option<Entity>,

    /// `mKonkeyDongLadderLogic`: `@"KonkeyDongLadderLogic"`
    pub konkey_dong_ladder_logic: Option<Entity>,

    /// `mbBattleOfTheGodsEnemy`: `B`
    pub battle_of_the_gods_enemy: bool,

    /// `mbKilledDuringBattle`: `B`
    pub killed_during_battle: bool,

    /// `mOpponentPygmyLogic`: `@"PygmyLogic"`
    pub opponent_pygmy_logic: Option<Entity>,

    /// `mpStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub struct_xml_episode_node_def: Option<usize>,

    /// `mpOpponentStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub opponent_struct_xml_episode_node_def: Option<usize>,

    /// `mbDoodleOnIce`: `B`
    pub doodle_on_ice: bool,

    /// `mbDoodleJetPack`: `B`
    pub doodle_jet_pack: bool,
}

/// Animation channels 7…13 carry the fighter's battle objects.
const BATTLE_CHANNELS: [usize; 7] = [7, 8, 9, 10, 11, 12, 13];

/// The player fighter's objects on channels 7…13: sword, shield, energy, energy shield, beam,
/// beam start and beam end.
const PLAYER_BATTLE_OBJECTS: [usize; 7] = [
    logic_table::PLAYER_SWORD,
    logic_table::PLAYER_SHIELD,
    logic_table::PLAYER_ENERGY,
    logic_table::PLAYER_ENERGY_SHIELD,
    logic_table::PLAYER_ENERGY_BEAM,
    logic_table::PLAYER_ENERGY_BEAM_START,
    logic_table::PLAYER_ENERGY_BEAM_END,
];

/// The enemy fighter's objects on channels 7…13.
const ENEMY_BATTLE_OBJECTS: [usize; 7] = [
    logic_table::ENEMY_SWORD,
    logic_table::ENEMY_SHIELD,
    logic_table::ENEMY_ENERGY,
    logic_table::ENEMY_ENERGY_SHIELD,
    logic_table::ENEMY_ENERGY_BEAM,
    logic_table::ENEMY_ENERGY_BEAM_START,
    logic_table::ENEMY_ENERGY_BEAM_END,
];

/// `DAT_10029d9b0`: how far from a physical opponent an energy fast attack reappears, indexed by
/// whether the opponent's attack is the counter attack.
const ENERGY_FAST_ATTACK_OFFSETS: [f32; 2] = [100.0, 70.0];

/// `0x3d4ccccc` (0.049999997): the jump-time factor of `theRunsJumpUp:::`.
const THE_RUNS_JUMP_TIME_FACTOR: f32 = f32::from_bits(0x3d4c_cccc);

/// `mPygmyLogicListener` (the view controller).
fn listener_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyPygmyCoreFrame>(this).pygmy_logic_listener
}

/// `[[logic displayObject] pos]`.
fn logic_pos(world: &World, logic: Entity) -> Vec4 {
    let display = pg_engine::logic::display_object(world, logic);
    pg_engine::display::object(world, display).pos
}

/// `[[logic displayObject] pos]` of an object the running mini-game always holds. The original
/// reads through the returned pointer, which a nil object would make NULL; the port reads zero.
fn optional_logic_pos(world: &World, logic: Option<Entity>) -> Vec4 {
    match logic {
        Some(logic) => logic_pos(world, logic),
        None => Vec4::ZERO,
    }
}

/// The pygmy's own display object, to write its `pos`, `scale`, `rot` or selectability.
fn own_display_mut(world: &mut World, this: Entity) -> Mut<'_, DisplayObject> {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object_mut(world, display)
}

/// `[[self displayObject] scale].x`: the pygmy's facing.
fn own_x_scale(world: &World, this: Entity) -> f32 {
    let display = pg_engine::logic::display_object(world, this);
    pg_engine::display::object(world, display).scale.x
}

/// `vel.xy = 0`: only x and y of `mPosVel` are cleared.
fn clear_xy_vel(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 0.0;
    logic.pos_vel.y = 0.0;
}

/// `mSacrificeId = @"…"`.
fn set_sacrifice_id(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).sacrifice_id = Some(sacrifice_id.to_owned());
}

/// `[[IdolManager instance] triggerEpisodeEvent:event]`.
fn trigger_episode_event(world: &mut World, event: &str) {
    let Some(manager) = idol_manager::instance(world) else {
        return;
    };
    idol_manager::trigger_episode_event(world, manager, event);
}

/// The start shared by Konkey Dong, Moron Pests and The Runs: `vel.xy = 0`,
/// `pos = (x_pos, 350, -277.12)`, scale 0.75, `zRot = 0` and alpha 1.
fn place_at_minigame_start(world: &mut World, this: Entity, x_pos: f32) {
    clear_xy_vel(world, this);

    let mut display = own_display_mut(world, this);
    display.pos.x = x_pos;
    display.pos.y = 350.0;
    display.pos.z = SPRITE_Z;
    display.scale.x = 0.75;
    display.scale.y = 0.75;
    display.rot.z = 0.0;
    display.set_alpha(1.0);
}

/// Ooga Jump: the pygmy bounces off `cloud` with `y_vel`, landing 10.5 above the cloud's y.
fn bounce_off_doodle_cloud(world: &mut World, this: Entity, cloud: Entity, y_vel: f32) {
    pg_engine::logic::logic_mut(world, this).pos_vel.y = y_vel;

    let cloud_y = logic_pos(world, cloud).y;
    own_display_mut(world, this).pos.y = cloud_y + 10.5;
}

/// Ooga Jump: the pygmy is knocked off the clouds and tells the controller.
fn knock_down_doodle_jump(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyDoodleKnockedDown"));

    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_pygmy_doodle_jump_fall(world, listener, this);
    }
}

/// `-[PygmyLogic (OogaJump) doodleBounce:]` @ 0x100007bf8
pub fn doodle_bounce(world: &mut World, this: Entity, doodle_cloud: Entity) {
    world.obj_mut::<PygmyMinigames>(this).doodle_jet_pack = false;
    bounce_off_doodle_cloud(world, this, doodle_cloud, 600.0);

    let on_ice = world.obj::<PygmyMinigames>(this).doodle_on_ice;
    let behavior = if on_ice {
        world.obj_mut::<PygmyMinigames>(this).doodle_on_ice = false;
        "PygmyDoodleBreakIceBounce"
    } else {
        "PygmyDoodleBounce"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (OogaJump) doodleElectricBounce:]` @ 0x100007ce4
pub fn doodle_electric_bounce(world: &mut World, this: Entity, doodle_cloud: Entity) {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.doodle_jet_pack = false;
    minigames.doodle_on_ice = false;

    bounce_off_doodle_cloud(world, this, doodle_cloud, 1400.0);
    pg_engine::logic::set_behavior(world, this, Some("PygmyDoodleElectricBounce"));
}

/// `-[PygmyLogic (OogaJump) doodleSnowBounce:]` @ 0x100007db0
pub fn doodle_snow_bounce(world: &mut World, this: Entity, doodle_cloud: Entity) {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.doodle_jet_pack = false;
    minigames.doodle_on_ice = true;

    bounce_off_doodle_cloud(world, this, doodle_cloud, 500.0);
    pg_engine::logic::set_behavior(world, this, Some("PygmyDoodleSnowBounce"));
}

/// `-[PygmyLogic (OogaJump) doodleKnockedDown:]` @ 0x100007e80
pub fn doodle_knocked_down(world: &mut World, this: Entity, _doodle_cloud: Entity) {
    let jet_pack = world.obj::<PygmyMinigames>(this).doodle_jet_pack;
    if jet_pack {
        return;
    }

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.doodle_jet_pack = false;
    minigames.doodle_on_ice = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).doodle_knocked_down = true;

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 1000.0;
    let x_pos = logic_pos(world, this).x;
    let x_vel = if 240.0 < x_pos { -30.0 } else { 30.0 };
    pg_engine::logic::logic_mut(world, this).pos_vel.x = x_vel;

    knock_down_doodle_jump(world, this);
}

/// `-[PygmyLogic (OogaJump) doodleJetPack:]` @ 0x100007f7c
pub fn doodle_jet_pack(world: &mut World, this: Entity, _doodle_cloud: Entity) {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.doodle_on_ice = false;
    if minigames.doodle_jet_pack {
        return;
    }
    minigames.doodle_jet_pack = true;

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 500.0;
    pg_engine::logic::set_behavior(world, this, Some("PygmyDoodleJetPack"));
}

/// `-[PygmyLogic (OogaJump) doodleCheckOffScreen:]` @ 0x100007fdc
pub fn doodle_check_off_screen(world: &mut World, this: Entity, screen_bottom: f32) {
    let knocked_down = world.obj::<PygmyPygmyCoreFrame>(this).doodle_knocked_down;
    if knocked_down {
        return;
    }

    let pos = logic_pos(world, this);
    let below_screen = pos.y + 8.0 < screen_bottom;
    if !below_screen {
        return;
    }

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.doodle_jet_pack = false;
    minigames.doodle_on_ice = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).doodle_knocked_down = true;

    pg_engine::logic::logic_mut(world, this).pos_vel.x = ((240.0 - pos.x) * 80.0) / 240.0;

    knock_down_doodle_jump(world, this);
}

/// `-[PygmyLogic (OogaJump) checkDoodleCloudCollision:]` @ 0x1000080f0
///
/// `last_pos` is `mLastPos` (the original passes a pointer to it).
pub fn check_doodle_cloud_collision(world: &mut World, this: Entity, last_pos: Vec4) {
    let core = world.obj::<PygmyPygmyCoreFrame>(this);
    let can_doodle_jump = core.falling
        && !core.cooked
        && !core.killed_by_plague
        && !core.killed_by_locusts
        && !core.killed_by_purification
        && !core.dance_dance_executed
        && !core.battle_of_the_gods
        && !core.cut_in_half_by_charlie
        && core.time_temple_pygmy_type == 0
        && !core.thrown_by_locusts
        && !core.invalid_sacrifice
        && !core.tumble
        && !core.is_zombie
        && !core.is_ghost
        && !core.beheaded_by_squid
        && !core.doodle_knocked_down;
    let listener = core.pygmy_logic_listener;
    if !can_doodle_jump {
        return;
    }

    // `[nil isDoodleCloudEnabled]` is NO.
    let Some(listener) = listener else {
        return;
    };
    let doodle_cloud_enabled = crate::controller::is_doodle_cloud_enabled(world, listener);
    if !doodle_cloud_enabled {
        return;
    }

    // `b.mi` / `b.ne`: a NaN height passes, a NaN depth does not.
    let pos = logic_pos(world, this);
    if pos.y < 350.0 || pos.z != SPRITE_Z {
        return;
    }

    let doodle_jumping = world.obj::<PygmyPygmyCoreFrame>(this).doodle_jumping;
    if doodle_jumping {
        // The cloud that registers a collision makes the pygmy bounce through the doodle*:
        // callbacks.
        let doodle_clouds = crate::controller::doodle_cloud_logic_array(world, listener, this);
        for doodle_cloud in doodle_clouds {
            let collided = doodle_cloud_logic::check_collision(world, doodle_cloud, this, last_pos);
            if collided {
                return;
            }
        }
        return;
    }

    let other_pygmy_jumping = crate::controller::is_doodle_jumping_pygmy(world, listener);
    if other_pygmy_jumping {
        return;
    }

    let clouds = crate::controller::cloud_logic_array(world, listener, this);
    for cloud in clouds {
        let is_doodle_cloud = cloud_logic::is_doodle_cloud(world, cloud);
        if !is_doodle_cloud {
            continue;
        }

        let cloud_pos = logic_pos(world, cloud);
        let pos = logic_pos(world, this);
        let over_cloud = cloud_pos.x + 10.0 < pos.x && pos.x < cloud_pos.x + 140.0;
        if !over_cloud {
            continue;
        }

        let cloud_top = cloud_pos.y + 21.0;
        let crossed_cloud_top = pos.y < cloud_top && last_pos.y > cloud_top;
        if !crossed_cloud_top {
            continue;
        }

        world.obj_mut::<PygmyPygmyCoreFrame>(this).doodle_jumping = true;
        pg_engine::logic::logic_mut(world, this).pos_vel.y = 1800.0;
        own_display_mut(world, this).rot.z = 0.0;
        pg_engine::logic::set_behavior(world, this, Some("PygmyDoodleBounceStart"));

        crate::controller::on_pygmy_start_doodle_jump(world, listener, this);
        return;
    }
}

/// `-[PygmyLogic (OogaJump) doodleJetPackComplete:::]` @ 0x1000085fc
pub fn seq_doodle_jet_pack_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    pg_engine::logic::logic_mut(world, this).pos_vel.y = 1000.0;
    world.obj_mut::<PygmyMinigames>(this).doodle_jet_pack = false;
    ItemResult::Done
}

/// The arrays of the Kong statue's current level (`mKonkeyDongLevelArray[mKonkeyDongLevelIndex]`)
/// that its `konkeyDong…LogicArray` getters return.
#[derive(Clone, Copy)]
enum KongLevelArray {
    Platforms,
    Ladders,
    Picks,
    Beetles,
}

/// `[listener kongStatueLogic:self]`.
fn kong_statue_of(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener_of(world, this)?;
    crate::controller::kong_statue_logic(world, listener, this)
}

/// `[kong konkeyDongBoulderLogicArray]`. The getter only returns the ivar, whose generated port
/// cannot return an array, so the ivar is read here; a nil statue gives no boulders.
fn kong_boulders(world: &World, kong: Option<Entity>) -> Vec<Entity> {
    let Some(kong) = kong else {
        return Vec::new();
    };
    world.obj::<KongStatueLogic>(kong).konkey_dong_boulder_logic_array.clone()
}

/// `[kong konkeyDongPlatformLogicArray]` and the other per-level getters, read from the ivars
/// like [`kong_boulders`].
fn kong_level_array(world: &World, kong: Option<Entity>, array: KongLevelArray) -> Vec<Entity> {
    let Some(kong) = kong else {
        return Vec::new();
    };

    let statue = world.obj::<KongStatueLogic>(kong);
    let level_index = statue.konkey_dong_level_index as usize;
    let level = &statue.konkey_dong_level_array[level_index];
    let logics = match array {
        KongLevelArray::Platforms => &level.platform_logic_array,
        KongLevelArray::Ladders => &level.ladder_logic_array,
        KongLevelArray::Picks => &level.pick_logic_array,
        KongLevelArray::Beetles => &level.beetle_logic_array,
    };
    logics.clone()
}

/// `[[kong konkeyDongLevelDisplayObject] pos]` (zero for a nil statue or level, see
/// [`optional_logic_pos`]).
fn kong_level_pos(world: &mut World, kong: Option<Entity>) -> Vec4 {
    let Some(kong) = kong else {
        return Vec4::ZERO;
    };

    let level_display = kong_statue_logic::konkey_dong_level_display_object(world, kong);
    match level_display {
        Some(level_display) => pg_engine::display::object(world, level_display).pos,
        None => Vec4::ZERO,
    }
}

/// A boulder or a beetle hits the pygmy.
fn hit_in_konkey_dong(world: &mut World, this: Entity, sacrifice_id: &str) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).konkey_dong_hit = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 100.0;
    logic.pos_vel.y = 400.0;

    set_sacrifice_id(world, this, sacrifice_id);
    pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongHit"));
}

/// `[kong showBonusSplash:&pos]` at the pygmy's position.
fn show_konkey_dong_bonus_splash(world: &mut World, this: Entity, kong: Option<Entity>) {
    let Some(kong) = kong else {
        return;
    };

    let pos = logic_pos(world, this);
    kong_statue_logic::show_bonus_splash(world, kong, pos);
}

/// `checkKonkeyDongCollision:`: the boulders. Returns whether one hit the pygmy, which ends the
/// check.
fn check_konkey_dong_boulders(world: &mut World, this: Entity, listener: Option<Entity>, kong: Option<Entity>) -> bool {
    let boulders = kong_boulders(world, kong);
    for boulder in boulders {
        let started = world.obj::<PygmyMinigames>(this).konkey_dong_started;
        if !started {
            continue;
        }
        let active = konkey_dong_boulder_logic::is_active(world, boulder);
        if !active {
            continue;
        }

        let boulder_pos = logic_pos(world, boulder);
        let pos = logic_pos(world, this);
        let x_distance = (boulder_pos.x - pos.x).abs();
        let height_above = pos.y - boulder_pos.y;

        let falling = world.obj::<PygmyMinigames>(this).konkey_dong_falling;
        if falling && x_distance < 16.0 && height_above < 45.0 && height_above > 0.0 {
            world.obj_mut::<PygmyMinigames>(this).konkey_dong_jump_boulder = true;
        }

        let touching = x_distance < 16.0 && pos.y > boulder_pos.y - 20.0 && pos.y < boulder_pos.y + 14.0;
        if !touching {
            continue;
        }

        let holding_pick = world.obj::<PygmyMinigames>(this).konkey_dong_holding_pick;
        if !holding_pick {
            hit_in_konkey_dong(world, this, "KonkeyDongBoulder");
            return true;
        }

        if let Some(listener) = listener {
            crate::controller::on_konkey_dong_score(world, listener, 100);
        }
        konkey_dong_boulder_logic::crack_boulder(world, boulder);
    }
    false
}

/// `checkKonkeyDongCollision:`: the beetles of the level (their positions are relative to the
/// level). Returns whether one hit the pygmy, which ends the check.
fn check_konkey_dong_beetles(
    world: &mut World,
    this: Entity,
    listener: Option<Entity>,
    kong: Option<Entity>,
    level_pos: Vec4,
) -> bool {
    let beetles = kong_level_array(world, kong, KongLevelArray::Beetles);
    for beetle in beetles {
        let started = world.obj::<PygmyMinigames>(this).konkey_dong_started;
        if !started {
            continue;
        }
        let active = konkey_dong_beetle_logic::is_active(world, beetle);
        if !active {
            continue;
        }

        let beetle_pos = logic_pos(world, beetle);
        let pos = logic_pos(world, this);
        let beetle_x = beetle_pos.x + level_pos.x;
        let beetle_y = beetle_pos.y + level_pos.y;
        let x_distance = (beetle_x - pos.x).abs();

        // Unlike the boulders', this marker's lower bound is 20 below the beetle.
        let falling = world.obj::<PygmyMinigames>(this).konkey_dong_falling;
        if falling && x_distance < 16.0 {
            let height_above = pos.y - beetle_y;
            if height_above < 45.0 && pos.y > beetle_y - 20.0 {
                world.obj_mut::<PygmyMinigames>(this).konkey_dong_jump_beetle = true;
            }
        }

        let touching = x_distance < 16.0 && pos.y > beetle_y - 20.0 && pos.y < beetle_y + 10.0;
        if !touching {
            continue;
        }

        let holding_pick = world.obj::<PygmyMinigames>(this).konkey_dong_holding_pick;
        if holding_pick {
            if let Some(listener) = listener {
                crate::controller::on_konkey_dong_score(world, listener, 100);
            }
            konkey_dong_beetle_logic::smash_beetle(world, beetle);
            continue;
        }

        let falling = world.obj::<PygmyMinigames>(this).konkey_dong_falling;
        if falling {
            let rolled_up = konkey_dong_beetle_logic::is_rolled_up(world, beetle);
            if !rolled_up {
                let y_vel = pg_engine::logic::logic(world, this).pos_vel.y;
                if y_vel < 0.0 {
                    pg_engine::logic::logic_mut(world, this).pos_vel.y = 525.0;
                    konkey_dong_beetle_logic::jump_on_beetle(world, beetle, this);
                }
                continue;
            }
        }

        hit_in_konkey_dong(world, this, "KonkeyDongBeetle");
        return true;
    }
    false
}

/// `checkKonkeyDongCollision:`: the picks of the level. Returns whether the pygmy grabbed one,
/// which ends the check.
fn check_konkey_dong_picks(world: &mut World, this: Entity, kong: Option<Entity>, level_pos: Vec4) -> bool {
    let picks = kong_level_array(world, kong, KongLevelArray::Picks);
    for pick in picks {
        let started = world.obj::<PygmyMinigames>(this).konkey_dong_started;
        if !started {
            continue;
        }
        let active = konkey_dong_pick_logic::is_active(world, pick);
        if !active {
            continue;
        }

        // Only the pick's y is offset by the level position.
        let pick_pos = logic_pos(world, pick);
        let pos = logic_pos(world, this);
        let x_distance = pos.x - pick_pos.x;
        let y_distance = pos.y - (pick_pos.y + level_pos.y);
        let distance = (x_distance * x_distance + y_distance * y_distance).sqrt();
        if distance < 16.0 {
            world.obj_mut::<PygmyMinigames>(this).konkey_dong_holding_pick = true;
            konkey_dong_pick_logic::deactivate(world, pick);
            world.obj_mut::<PygmyPygmyCoreFrame>(this).konkey_dong_pick_time = 5.0;
            pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongGrabPick"));
            return true;
        }
    }
    false
}

/// `checkKonkeyDongCollision:`: on a ladder, the pygmy keeps climbing between its bottom and top
/// (which ends the check); outside them it leaves the ladder. Returns whether it is still on it.
fn check_konkey_dong_ladder(world: &mut World, this: Entity, level_pos: Vec4) -> bool {
    let minigames = world.obj::<PygmyMinigames>(this);
    let on_ladder = minigames.konkey_dong_on_ladder;
    let ladder = minigames.konkey_dong_ladder_logic;
    if !on_ladder {
        return false;
    }

    let ladder_pos = optional_logic_pos(world, ladder);
    let ladder_height = match ladder {
        Some(ladder) => konkey_dong_ladder_logic::ladder_height(world, ladder),
        None => 0.0,
    };
    let ladder_bottom = level_pos.y + ladder_pos.y;
    let ladder_top = ladder_height + ladder_bottom;

    let y_pos = logic_pos(world, this).y;
    if y_pos >= ladder_bottom && y_pos <= ladder_top {
        return true;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.konkey_dong_ladder_logic = None;
    minigames.konkey_dong_on_ladder = false;
    minigames.konkey_dong_platform_collision = false;
    false
}

/// `checkKonkeyDongCollision:`: the first platform of the level that reports a collision (only
/// the start platform until the game has started).
fn konkey_dong_platform_under(world: &mut World, this: Entity, kong: Option<Entity>, last_pos: Vec4) -> Option<Entity> {
    let platforms = kong_level_array(world, kong, KongLevelArray::Platforms);
    for platform in platforms {
        let started = world.obj::<PygmyMinigames>(this).konkey_dong_started;
        let eligible = started || konkey_dong_platform_logic::is_start_platform(world, platform);
        if !eligible {
            continue;
        }

        let collided = konkey_dong_platform_logic::check_collision(world, platform, this, last_pos);
        if collided {
            return Some(platform);
        }
    }
    None
}

/// `checkKonkeyDongCollision:`: leaving a platform starts a fall; landing on one ends it,
/// awarding the bonuses for the boulders and beetles jumped over.
fn update_konkey_dong_platform_collision(world: &mut World, this: Entity, kong: Option<Entity>, on_platform: bool) {
    let platform_collision = world.obj::<PygmyMinigames>(this).konkey_dong_platform_collision;
    if platform_collision == on_platform {
        return;
    }
    world.obj_mut::<PygmyMinigames>(this).konkey_dong_platform_collision = on_platform;

    if !on_platform {
        let falling = world.obj::<PygmyMinigames>(this).konkey_dong_falling;
        if falling {
            return;
        }
        world.obj_mut::<PygmyMinigames>(this).konkey_dong_falling = true;

        let holding_pick = world.obj::<PygmyMinigames>(this).konkey_dong_holding_pick;
        if !holding_pick {
            pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongFall"));
        }
        return;
    }

    let started = world.obj::<PygmyMinigames>(this).konkey_dong_started;
    if !started {
        let holding_pick = world.obj::<PygmyMinigames>(this).konkey_dong_holding_pick;
        if !holding_pick {
            pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongFirstLand"));
        }
        return;
    }

    let jump_boulder = world.obj::<PygmyMinigames>(this).konkey_dong_jump_boulder;
    if jump_boulder {
        world.obj_mut::<PygmyMinigames>(this).konkey_dong_jump_boulder = false;
        trigger_episode_event(world, "JumpBoulder");
        show_konkey_dong_bonus_splash(world, this, kong);
    }

    let jump_beetle = world.obj::<PygmyMinigames>(this).konkey_dong_jump_beetle;
    if jump_beetle {
        world.obj_mut::<PygmyMinigames>(this).konkey_dong_jump_beetle = false;
        trigger_episode_event(world, "JumpBeetle");
        show_konkey_dong_bonus_splash(world, this, kong);
    }

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.konkey_dong_falling = false;
    minigames.konkey_dong_running = false;
    let holding_pick = minigames.konkey_dong_holding_pick;
    if !holding_pick {
        pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongLand"));
    }
}

/// Snaps the pygmy onto its new ladder at `(x_pos, y_pos)` and starts climbing.
fn climb_konkey_dong_ladder(world: &mut World, this: Entity, x_pos: f32, y_pos: f32) {
    clear_xy_vel(world, this);

    let mut display = own_display_mut(world, this);
    display.pos.x = x_pos;
    display.pos.y = y_pos;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.konkey_dong_on_ladder = true;
    minigames.konkey_dong_climbing = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongLadderClimb"));
}

/// `checkKonkeyDongCollision:`: with a ladder touch held, the pygmy gets onto a ladder at its top
/// (going down) or its bottom (going up). Returns whether it did, which ends the check.
fn enter_konkey_dong_ladder(world: &mut World, this: Entity, kong: Option<Entity>, level_pos: Vec4, last_pos: Vec4) -> bool {
    let ladders = kong_level_array(world, kong, KongLevelArray::Ladders);
    for ladder in ladders {
        let ladder_up = world.obj::<PygmyMinigames>(this).konkey_dong_ladder_up;
        if !ladder_up {
            let at_top = konkey_dong_ladder_logic::check_top_collision(world, ladder, this, last_pos);
            if at_top {
                world.obj_mut::<PygmyMinigames>(this).konkey_dong_ladder_logic = Some(ladder);

                let ladder_pos = logic_pos(world, ladder);
                let ladder_height = konkey_dong_ladder_logic::ladder_height(world, ladder);
                let ladder_bottom = level_pos.y + ladder_pos.y;
                climb_konkey_dong_ladder(world, this, ladder_pos.x, ladder_height + ladder_bottom);
                return true;
            }

            let ladder_up = world.obj::<PygmyMinigames>(this).konkey_dong_ladder_up;
            if !ladder_up {
                continue;
            }
        }

        let at_bottom = konkey_dong_ladder_logic::check_bottom_collision(world, ladder, this, last_pos);
        if at_bottom {
            world.obj_mut::<PygmyMinigames>(this).konkey_dong_ladder_logic = Some(ladder);

            let ladder_pos = logic_pos(world, ladder);
            climb_konkey_dong_ladder(world, this, ladder_pos.x, level_pos.y + ladder_pos.y);
            return true;
        }
    }
    false
}

/// `-[PygmyLogic (KonkeyDong) checkKonkeyDongCollision:]` @ 0x100039028
///
/// `last_pos` is `mLastPos` (the original passes a pointer to it).
pub fn check_konkey_dong_collision(world: &mut World, this: Entity, last_pos: Vec4) {
    let konkey_dong = world.obj::<PygmyPygmyTouchDrag>(this).konkey_dong;
    let hit = world.obj::<PygmyPygmyCoreFrame>(this).konkey_dong_hit;
    let level_transition = world.obj::<PygmyMinigames>(this).konkey_dong_level_transition;
    if !konkey_dong || hit || level_transition {
        return;
    }

    let listener = listener_of(world, this);
    let kong = kong_statue_of(world, this);
    let level_pos = kong_level_pos(world, kong);

    if check_konkey_dong_boulders(world, this, listener, kong) {
        return;
    }
    if check_konkey_dong_beetles(world, this, listener, kong, level_pos) {
        return;
    }
    if check_konkey_dong_picks(world, this, kong, level_pos) {
        return;
    }
    if check_konkey_dong_ladder(world, this, level_pos) {
        return;
    }

    let platform = konkey_dong_platform_under(world, this, kong, last_pos);
    update_konkey_dong_platform_collision(world, this, kong, platform.is_some());

    // `fcsel …, gt`: a NaN speed faces left.
    let x_vel = pg_engine::logic::logic(world, this).pos_vel.x;
    let x_scale = if x_vel > 0.0 { 0.75 } else { -0.75 };
    own_display_mut(world, this).scale.x = x_scale;

    let minigames = world.obj::<PygmyMinigames>(this);
    let platform_collision = minigames.konkey_dong_platform_collision;
    let started = minigames.konkey_dong_started;
    let holding_pick = minigames.konkey_dong_holding_pick;
    if !platform_collision || !started || holding_pick {
        return;
    }

    let ladder_touch = world.obj::<PygmyPygmyTouchDrag>(this).konkey_dong_ladder_touch;
    if ladder_touch.is_some() && enter_konkey_dong_ladder(world, this, kong, level_pos, last_pos) {
        return;
    }

    let final_platform = match platform {
        Some(platform) => konkey_dong_platform_logic::is_final_platform(world, platform),
        None => false,
    };
    if final_platform {
        let konkey_dong_logic = match kong {
            Some(kong) => kong_statue_logic::konkey_dong_logic(world, kong),
            None => None,
        };
        let konkey_dong_x = optional_logic_pos(world, konkey_dong_logic).x;
        let x_pos = logic_pos(world, this).x;

        // `b.pl`: a NaN position keeps running.
        if x_pos < konkey_dong_x + 110.0 {
            clear_xy_vel(world, this);
            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            world.obj_mut::<PygmyMinigames>(this).konkey_dong_level_transition = true;

            if let Some(kong) = kong {
                kong_statue_logic::end_level(world, kong);
            }
            pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongWatchKidnap"));
            return;
        }
    }

    let minigames = world.obj::<PygmyMinigames>(this);
    if minigames.konkey_dong_running || minigames.konkey_dong_holding_pick {
        return;
    }
    world.obj_mut::<PygmyMinigames>(this).konkey_dong_running = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongRun"));
}

/// `-[PygmyLogic (KonkeyDong) startKonkeyDong]` @ 0x100039f38
pub fn start_konkey_dong(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyTouchDrag>(this).konkey_dong = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = false;

    own_display_mut(world, this).is_selectable = true;
    place_at_minigame_start(world, this, 34.0);
    pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongStart"));
}

/// `-[PygmyLogic (KonkeyDong) jumpToNextKonkeyDongLevel]` @ 0x10003a064
pub fn jump_to_next_konkey_dong_level(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongJumpToNextLevel"));
}

/// `-[PygmyLogic (KonkeyDong) konkeyDongIntroComplete:::]` @ 0x10003a07c
pub fn seq_konkey_dong_intro_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.konkey_dong_started = true;
    minigames.konkey_dong_level_transition = false;

    own_display_mut(world, this).is_selectable = true;

    let kong = kong_statue_of(world, this);
    if let Some(kong) = kong {
        kong_statue_logic::pygmy_intro_complete(world, kong);
    }
    ItemResult::Done
}

/// `-[PygmyLogic (KonkeyDong) konkeyDongLevelJumpComplete:::]` @ 0x10003a118
pub fn seq_konkey_dong_level_jump_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.konkey_dong_running = false;
    minigames.konkey_dong_level_transition = false;
    minigames.konkey_dong_platform_collision = false;
    ItemResult::Done
}

/// `-[PygmyLogic (KonkeyDong) konkeyDongJumpUp:::]` @ 0x10003a160
pub fn seq_konkey_dong_jump_up(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    if call.first_time {
        logic.stall_time = 0.2;
    }
    logic.pos_vel.y = 300.0;
    let stall_time = logic.stall_time;

    if stall_time <= 0.0 {
        world.obj_mut::<PygmyMinigames>(this).konkey_dong_jump_touch = None;
        pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongJumpDown"));
        return ItemResult::Replaced;
    }

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[PygmyLogic (KonkeyDong) konkeyDongFall:::]` @ 0x10003a220
pub fn seq_konkey_dong_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    ItemResult::Pending
}

/// `-[PygmyLogic (KonkeyDong) onStartKonkeyDongFall:::]` @ 0x10003a238
pub fn seq_on_start_konkey_dong_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    ItemResult::Done
}

/// `-[PygmyLogic (KonkeyDong) touchBeganKonkeyDong:allTouches:]` @ 0x10003a250
pub fn touch_began_konkey_dong(world: &mut World, this: Entity, touch: Entity, _all_touches: &[Entity]) {
    let falling = world.obj::<PygmyMinigames>(this).konkey_dong_falling;
    let hit = world.obj::<PygmyPygmyCoreFrame>(this).konkey_dong_hit;
    if falling || hit {
        return;
    }

    // `[nil viewToDisplayObject:]` answers all zero.
    let group = pg_engine::library::display_group(world, "PocketGod");
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let point = match group {
        Some(group) => pg_engine::display::view_to_display_object(world, group, touch_pos),
        None => Vec4::ZERO,
    };

    let minigames = world.obj::<PygmyMinigames>(this);
    let can_jump = minigames.konkey_dong_started
        && !minigames.konkey_dong_on_ladder
        && !minigames.konkey_dong_holding_pick
        && !minigames.konkey_dong_level_transition;
    if point.x > 240.0 && can_jump {
        let mut minigames = world.obj_mut::<PygmyMinigames>(this);
        minigames.konkey_dong_falling = true;
        minigames.konkey_dong_jump_touch = Some(touch);

        pg_engine::logic::set_behavior(world, this, Some("PygmyKonkeyDongJumpUp"));
        return;
    }

    if point.x <= 240.0 {
        world.obj_mut::<PygmyMinigames>(this).konkey_dong_ladder_up = point.y > 160.0;
        world.obj_mut::<PygmyPygmyTouchDrag>(this).konkey_dong_ladder_touch = Some(touch);
    }
}

/// `[listener moronSnakeLogic:self]`.
fn moron_snake_of(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener_of(world, this)?;
    crate::controller::moron_snake_logic(world, listener, this)
}

/// `-[PygmyLogic (MoronPests) checkMoronPestsCollision:]` @ 0x100113474
///
/// `last_pos` is `mLastPos` (the original passes a pointer to it and does not read it).
pub fn check_moron_pests_collision(world: &mut World, this: Entity, _last_pos: Vec4) {
    let moron_pests = world.obj::<PygmyPygmyCoreFrame>(this).moron_pests;
    if !moron_pests {
        return;
    }

    let y_pos = logic_pos(world, this).y;
    let hit_turtle = world.obj::<PygmyMinigames>(this).moron_pests_hit_turtle;
    if !hit_turtle {
        if y_pos <= 170.0 {
            pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
            own_display_mut(world, this).pos.y = 170.0;
            world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
            world.obj_mut::<PygmyMinigames>(this).moron_pests_hit_turtle = true;

            let turtle = pg_engine::library::logic(world, logic_table::MORON_TURTLE);
            moron_turtle_logic::pygmy_jump_on_back(world, turtle, this);
        }
        return;
    }

    let start_running = world.obj::<PygmyMinigames>(this).moron_pests_start_running;
    let on_turtle_back = start_running && y_pos <= 130.0;
    if !on_turtle_back {
        return;
    }

    pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
    own_display_mut(world, this).pos.y = 130.0;

    let floor_collision = world.obj::<PygmyMinigames>(this).moron_pests_floor_collision;
    if !floor_collision {
        world.obj_mut::<PygmyMinigames>(this).moron_pests_floor_collision = true;
        pg_engine::logic::set_behavior(world, this, Some("PygmyMoronPestsLand"));
    }
}

/// `-[PygmyLogic (MoronPests) startMoronPests]` @ 0x1001135e4
pub fn start_moron_pests(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).moron_pests = true;

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.moron_pests_hit_turtle = false;
    minigames.moron_pests_start_running = false;

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = false;

    place_at_minigame_start(world, this, 60.0);
    pg_engine::logic::set_behavior(world, this, Some("PygmyMoronPestsStart"));
}

/// `-[PygmyLogic (MoronPests) jumpOnMoronTurtleBackComplete:]` @ 0x100113718
pub fn jump_on_moron_turtle_back_complete(world: &mut World, this: Entity, _turtle: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyMinigames>(this).moron_pests_start_running = true;
}

/// `-[PygmyLogic (MoronPests) onMoronPestsAnswerCorrect]` @ 0x100113740
pub fn on_moron_pests_answer_correct(world: &mut World, this: Entity) {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    let target_x_vel = minigames.moron_pests_target_x_pos_vel + 100.0;

    // `fcsel …, gt`: only a target above 540 is capped.
    minigames.moron_pests_target_x_pos_vel = if target_x_vel > 540.0 { 540.0 } else { target_x_vel };

    pg_engine::logic::set_behavior(world, this, Some("PygmyMoronPestsAnswerCorrect"));
}

/// `-[PygmyLogic (MoronPests) onMoronPestsAnswerIncorrect]` @ 0x100113788
pub fn on_moron_pests_answer_incorrect(world: &mut World, this: Entity) {
    world.obj_mut::<PygmyMinigames>(this).moron_pests_target_x_pos_vel -= 10.0;
    pg_engine::logic::set_behavior(world, this, Some("PygmyMoronPestsAnswerIncorrect"));
}

/// `-[PygmyLogic (MoronPests) eatenByMoronTurtle:]` @ 0x1001137bc
pub fn eaten_by_moron_turtle(world: &mut World, this: Entity, _turtle: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).moron_pests_eaten_by_turtle = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
}

/// `-[PygmyLogic (MoronPests) eatenByMoronTurtleStopMoving:]` @ 0x1001137e4
pub fn eaten_by_moron_turtle_stop_moving(world: &mut World, this: Entity, _turtle: Entity) {
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
}

/// `-[PygmyLogic (MoronPests) eatenByMoronTurtleComplete:]` @ 0x1001137f8
pub fn eaten_by_moron_turtle_complete(world: &mut World, this: Entity, _turtle: Entity) {
    pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    set_sacrifice_id(world, this, "EatenByMoronTurtle");
    pg_engine::logic::set_behavior(world, this, Some("PygmyEatenByMoronTurtle"));
}

/// `-[PygmyLogic (MoronPests) onEatenByMoronTurtleEatComplete:::]` @ 0x100113838
pub fn seq_on_eaten_by_moron_turtle_eat_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let snake = moron_snake_of(world, this);
    if let Some(snake) = snake {
        moron_snake_logic::stop_moron_pests(world, snake);
    }
    ItemResult::Done
}

/// `-[PygmyLogic (MoronPests) onStartMoronPestsFall:::]` @ 0x100113884
pub fn seq_on_start_moron_pests_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    ItemResult::Done
}

/// `-[PygmyLogic (MoronPests) moronPestsIntroComplete:::]` @ 0x10011389c
pub fn seq_moron_pests_intro_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).moron_pests_started = true;

    let snake = moron_snake_of(world, this);
    if let Some(snake) = snake {
        moron_snake_logic::pygmy_intro_complete(world, snake);
    }

    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = false;

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 500.0;
    world.obj_mut::<PygmyMinigames>(this).moron_pests_target_x_pos_vel = 500.0;
    ItemResult::Done
}

/// `-[PygmyLogic (MoronPests) moronPestsRun:::]` @ 0x100113958
pub fn seq_moron_pests_run(_world: &mut World, _this: Entity, _call: &mut SequenceCall) -> ItemResult {
    ItemResult::Pending
}

/// The episode behind a `BCStructXmlEpisodeNodeDef *` (an index into the episode table). The
/// original reads the node unchecked; the temple door sets both of a fighter's nodes before any
/// attack sequence or frame action reads them.
fn episode_def(world: &mut World, node: Option<usize>) -> &'static Episode {
    let node = node.expect("a battle pygmy's episode nodes are set before its attack");
    let table = idol_manager::episode_table(world);
    table.def(node)
}

/// `*mpStructXmlEpisodeNodeDef`: the fighter's own idol.
fn own_idol(world: &mut World, this: Entity) -> &'static Episode {
    let node = world.obj::<PygmyMinigames>(this).struct_xml_episode_node_def;
    episode_def(world, node)
}

/// `*mpOpponentStructXmlEpisodeNodeDef`: the opponent's idol.
fn opponent_idol(world: &mut World, this: Entity) -> &'static Episode {
    let node = world.obj::<PygmyMinigames>(this).opponent_struct_xml_episode_node_def;
    episode_def(world, node)
}

/// `mOpponentPygmyLogic`.
fn opponent_of(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<PygmyMinigames>(this).opponent_pygmy_logic
}

/// `[BCLibrary getLogic:]` of one of the fighter's battle objects: `player_slot` for the player,
/// `enemy_slot` for the enemy (`mbBattleOfTheGodsEnemy`).
fn fighter_logic(world: &World, this: Entity, player_slot: usize, enemy_slot: usize) -> Entity {
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
    let slot = if enemy { enemy_slot } else { player_slot };
    pg_engine::library::logic(world, slot)
}

/// The realm as the behaviour and episode-event ids spell it.
fn realm_name(realm: Realm) -> &'static str {
    match realm {
        Realm::Oceanus => "Oceanus",
        Realm::Aeris => "Aeris",
        Realm::Infernis => "Infernis",
        Realm::Animus => "Animus",
        Realm::Pax => "Pax",
    }
}

/// `startBattleOfTheGods:` and `startBattleOfTheGodsEnemy:` after their flags: the opponent, the
/// arena floor, the fighter's battle objects on channels 7…13, then the spawn.
fn enter_battle(world: &mut World, this: Entity, opponent: Entity, battle_objects: [usize; 7]) {
    world.obj_mut::<PygmyMinigames>(this).opponent_pygmy_logic = Some(opponent);
    world.obj_mut::<PygmyPygmyCoreFrame>(this).island_y_pos = 55.0;

    let animation = pg_engine::logic::animation(world, this);
    for (object, channel) in battle_objects.into_iter().zip(BATTLE_CHANNELS) {
        let logic = pg_engine::library::logic(world, object);
        let display = pg_engine::logic::display_object(world, logic);
        pg_engine::animation::set_tweenable(world, animation, Some(display), channel);
    }

    let sword = pg_engine::library::logic(world, battle_objects[0]);
    let shield = pg_engine::library::logic(world, battle_objects[1]);
    pg_engine::logic::set_behavior(world, sword, Some("BlankAnim"));
    pg_engine::logic::set_behavior(world, shield, Some("BlankAnim"));

    own_display_mut(world, this).is_selectable = false;
    spawn_from_battle_of_the_gods(world, this);
}

/// `[self die:nil :YES :NULL]`: the death sequence item called directly, outside any sequence.
fn die_now(world: &mut World, this: Entity) {
    let item = SequenceItemDef {
        method: "die".to_owned(),
        params: Vec::new(),
    };
    let mut call = SequenceCall {
        sequence: Entity::PLACEHOLDER,
        first_time: true,
        delta_time: 0.0,
        item: &item,
    };
    seq_die(world, this, &mut call);
}

/// `-[PygmyLogic (BattleOfTheGods) spawnFromBattleOfTheGods]` @ 0x100121c54
pub fn spawn_from_battle_of_the_gods(world: &mut World, this: Entity) {
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;

    let mut display = own_display_mut(world, this);
    display.pos.x = 100.0;
    display.pos.y = 400.0;
    display.scale.x = 1.0;
    display.scale.y = 1.0;
    if enemy {
        display.pos.x = 380.0;
        display.scale.x = -1.0;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    world.obj_mut::<PygmyMinigames>(this).killed_during_battle = false;
    world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyBattleSpawn"));
}

/// `-[PygmyLogic (BattleOfTheGods) startBattleOfTheGods:]` @ 0x100121d48
pub fn start_battle_of_the_gods(world: &mut World, this: Entity, opponent: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).battle_of_the_gods = true;
    enter_battle(world, this, opponent, PLAYER_BATTLE_OBJECTS);
}

/// `-[PygmyLogic (BattleOfTheGods) startBattleOfTheGodsEnemy:]` @ 0x10012201c
pub fn start_battle_of_the_gods_enemy(world: &mut World, this: Entity, opponent: Entity) {
    world.obj_mut::<PygmyPygmyCoreFrame>(this).battle_of_the_gods = true;
    world.obj_mut::<PygmyMinigames>(this).battle_of_the_gods_enemy = true;
    enter_battle(world, this, opponent, ENEMY_BATTLE_OBJECTS);
}

/// `-[PygmyLogic (BattleOfTheGods) startBattlePowerUp:opponentIdol:]` @ 0x100122300
pub fn start_battle_power_up(world: &mut World, this: Entity, idol_node: Option<usize>, opponent_idol_node: Option<usize>) {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.struct_xml_episode_node_def = idol_node;
    minigames.opponent_struct_xml_episode_node_def = opponent_idol_node;

    pg_engine::logic::set_behavior(world, this, Some("PygmyBattlePowerUp"));
}

/// `-[PygmyLogic (BattleOfTheGods) setBattleBeamPos]` @ 0x100122338
pub fn set_battle_beam_pos(world: &mut World, this: Entity) {
    let pos = logic_pos(world, this);
    let x_scale = own_x_scale(world, this);

    let beam = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BEAM, logic_table::ENEMY_ENERGY_BEAM);
    let beam_end = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BEAM_END, logic_table::ENEMY_ENERGY_BEAM_END);
    let beam_display = pg_engine::logic::display_object(world, beam);
    let beam_end_display = pg_engine::logic::display_object(world, beam_end);

    let mut beam_x_scale = x_scale * 5.0;
    pg_engine::display::object_mut(world, beam_display).scale.x = beam_x_scale;

    // Against a strong attack the beam reaches the opponent.
    let opponent = opponent_idol(world, this);
    if opponent.attack == Attack::Strong {
        beam_x_scale = if opponent.attack_style == AttackStyle::Physical {
            let opponent_pygmy = opponent_of(world, this);
            let opponent_x = optional_logic_pos(world, opponent_pygmy).x;
            (x_scale * ((opponent_x - pos.x).abs() - 55.0)) / 100.0
        } else {
            x_scale * 1.1
        };
        pg_engine::display::object_mut(world, beam_display).scale.x = beam_x_scale;
    }

    let beam_end_x = pos.x + beam_x_scale * 100.0 + x_scale * 23.0;
    pg_engine::display::object_mut(world, beam_end_display).pos.x = beam_end_x;
}

/// `-[PygmyLogic (BattleOfTheGods) startBattleAttack:opponentIdol:]` @ 0x100122554
pub fn start_battle_attack(world: &mut World, this: Entity, idol_node: Option<usize>, opponent_idol_node: Option<usize>) {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.struct_xml_episode_node_def = idol_node;
    minigames.opponent_struct_xml_episode_node_def = opponent_idol_node;
    let enemy = minigames.battle_of_the_gods_enemy;

    if !enemy {
        let own = own_idol(world, this);
        let attack_event = match own.attack {
            Attack::Fast => "FastAttack",
            Attack::Counter => "CounterAttack",
            Attack::Strong => "StrongAttack",
        };
        trigger_episode_event(world, attack_event);

        let realm = realm_name(own.realm);
        let realm_event = format!("{realm}Attack");
        trigger_episode_event(world, &realm_event);
    }

    let own = own_idol(world, this);
    let energy = own.attack_style == AttackStyle::Energy;
    let behavior = match own.attack {
        Attack::Fast => {
            if energy {
                "PygmyBattleEnergyAttackFast"
            } else {
                "PygmyBattleAttackFast"
            }
        }
        Attack::Counter => {
            let opponent = opponent_idol(world, this);
            if opponent.attack == Attack::Counter {
                if energy {
                    "PygmyBattleEnergyAttackCounterTie"
                } else {
                    "PygmyBattleAttackCounterTie"
                }
            } else if energy {
                set_battle_beam_pos(world, this);
                "PygmyBattleEnergyAttackCounter"
            } else {
                "PygmyBattleAttackCounter"
            }
        }
        Attack::Strong => {
            if energy {
                set_battle_beam_pos(world, this);
                "PygmyBattleEnergyAttackStrong"
            } else {
                "PygmyBattleAttackStrong"
            }
        }
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (BattleOfTheGods) wasKilledDuringBattle]` @ 0x1001228b8
pub fn was_killed_during_battle(world: &mut World, this: Entity) -> bool {
    world.obj::<PygmyMinigames>(this).killed_during_battle
}

/// `-[PygmyLogic (BattleOfTheGods) killedInBattle]` @ 0x1001228cc
pub fn killed_in_battle(world: &mut World, this: Entity) {
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;

    // Thrown away from the opponent.
    let mut logic = pg_engine::logic::logic_mut(world, this);
    logic.pos_vel.x = 500.0;
    logic.pos_vel.y = 600.0;
    if !enemy {
        logic.pos_vel.x = -logic.pos_vel.x;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;
    world.obj_mut::<PygmyMinigames>(this).killed_during_battle = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyBattleKilled"));

    let Some(manager) = idol_manager::instance(world) else {
        return;
    };
    idol_manager::on_battle_damage_complete(world, manager);
}

/// `-[PygmyLogic (BattleOfTheGods) fatalityInBattle]` @ 0x10012297c
pub fn fatality_in_battle(world: &mut World, this: Entity) {
    clear_xy_vel(world, this);

    let listener = listener_of(world, this);
    if let Some(listener) = listener {
        crate::controller::on_trigger_sacrifice(world, listener, this, Some("BattleFatality"));
    }

    // The velocity was just cleared, so the player's x velocity becomes -0.
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
    if !enemy {
        let mut logic = pg_engine::logic::logic_mut(world, this);
        logic.pos_vel.x = -logic.pos_vel.x;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
    world.obj_mut::<PygmyMinigames>(this).killed_during_battle = false;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    // The opponent's realm decides how the pygmy dies.
    let opponent = opponent_idol(world, this);
    let behavior = match opponent.realm {
        Realm::Oceanus => "PygmyBattleFatalityOceanus",
        Realm::Aeris => {
            let beam = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BEAM, logic_table::ENEMY_ENERGY_BEAM);
            pg_engine::logic::set_behavior(world, beam, Some("PygmyBattleFatalityAerisTornado"));
            "PygmyBattleFatalityAeris"
        }
        Realm::Infernis => "PygmyBattleFatalityInfernis",
        Realm::Animus => "PygmyBattleFatalityAnimus",
        Realm::Pax => "PygmyBattleFatalityPax",
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (BattleOfTheGods) removeFromBattleOfTheGods]` @ 0x100122b78
pub fn remove_from_battle_of_the_gods(world: &mut World, this: Entity) {
    let animation = pg_engine::logic::animation(world, this);
    for channel in BATTLE_CHANNELS {
        pg_engine::animation::clear_tweenable_channel_id(world, animation, channel);
    }

    die_now(world, this);
}

/// `-[PygmyLogic (BattleOfTheGods) battleCounterFastAttack]` @ 0x100122c40
pub fn battle_counter_fast_attack(world: &mut World, this: Entity) {
    let own = own_idol(world, this);
    let behavior = if own.attack_style == AttackStyle::Energy {
        "PygmyBattleEnergyAttackCounterFastAttack"
    } else {
        "PygmyBattleAttackCounterFastAttack"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `[sequence pushSequenceDef:[BCLibrary getSequenceDefById:p]]` for the item's parameter
/// `name`, when present.
fn push_sequence_param(world: &mut World, call: &SequenceCall, name: &str) {
    let Some(sequence_id) = call.param(name) else {
        return;
    };

    let sequence_def = pg_engine::library::sequence_def(world, sequence_id);
    pg_engine::sequence::push_sequence_def(world, call.sequence, sequence_def);
}

/// `-[PygmyLogic (BattleOfTheGods) pushSequenceBasedOnOpponentWeaponType:::]` @ 0x100122c80
pub fn seq_push_sequence_based_on_opponent_weapon_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let opponent = opponent_idol(world, this);
    let param = if opponent.attack_style == AttackStyle::Energy {
        "energyId"
    } else {
        "swordId"
    };

    push_sequence_param(world, call, param);
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) pushSequenceBasedOnOpponentBattleType:::]` @ 0x100122d10
pub fn seq_push_sequence_based_on_opponent_battle_type(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    let opponent = opponent_idol(world, this);
    let param = match (opponent.attack, opponent.attack_style) {
        (Attack::Fast, AttackStyle::Physical) => "fastSwordId",
        (Attack::Fast, AttackStyle::Energy) => "fastEnergyId",
        (Attack::Strong, AttackStyle::Physical) => "strongSwordId",
        _ => return ItemResult::Done,
    };

    push_sequence_param(world, call, param);
    ItemResult::Done
}

/// Battle of the Gods: whether the fighter is at (or behind) its start line, x 100 for the player
/// and 380 for the enemy. The original's branches leave a NaN position off the line.
fn at_battle_start_line(world: &World, this: Entity) -> bool {
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
    let x_pos = logic_pos(world, this).x;
    if enemy {
        x_pos >= 380.0
    } else {
        x_pos <= 100.0
    }
}

/// Battle of the Gods: puts the fighter on its start line, facing the opponent.
fn snap_to_battle_start_line(world: &mut World, this: Entity) {
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
    let (start_x, x_scale) = if enemy { (380.0, -1.0) } else { (100.0, 1.0) };

    let mut display = own_display_mut(world, this);
    display.pos.x = start_x;
    display.scale.x = x_scale;
}

/// `-[PygmyLogic (BattleOfTheGods) battleCheckNoRunBack:::]` @ 0x100122dd4
pub fn seq_battle_check_no_run_back(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let at_start_line = at_battle_start_line(world, this);
    if !at_start_line {
        return ItemResult::Done;
    }

    snap_to_battle_start_line(world, this);
    pg_engine::logic::set_behavior(world, this, Some("PygmyBattleNoRunBack"));
    ItemResult::Replaced
}

/// `-[PygmyLogic (BattleOfTheGods) battleRunBackToStart:::]` @ 0x100122ea4
pub fn seq_battle_run_back_to_start(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        // Turns away from the opponent.
        let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
        let x_scale = if enemy { 1.0 } else { -1.0 };
        own_display_mut(world, this).scale.x = x_scale;
    }

    let at_start_line = at_battle_start_line(world, this);
    if !at_start_line {
        return ItemResult::Pending;
    }

    snap_to_battle_start_line(world, this);
    ItemResult::Done
}

/// Battle of the Gods: the horizontal distance between the fighter and its opponent.
fn opponent_x_distance(world: &World, this: Entity) -> f32 {
    let x_pos = logic_pos(world, this).x;
    let opponent = opponent_of(world, this);
    let opponent_x = optional_logic_pos(world, opponent).x;
    (x_pos - opponent_x).abs()
}

/// `-[PygmyLogic (BattleOfTheGods) battleStrongRunTowardOpponent:::]` @ 0x100122f90
pub fn seq_battle_strong_run_toward_opponent(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    // `b.pl`: a NaN distance keeps running.
    let x_distance = opponent_x_distance(world, this);
    let reached = x_distance < 50.0;
    if !reached {
        return ItemResult::Pending;
    }

    let opponent = opponent_idol(world, this);
    if opponent.attack == Attack::Strong {
        pg_engine::logic::set_behavior(world, this, Some("PygmyBattleAttackStrongTie"));
        return ItemResult::Replaced;
    }
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) battleFastRunTowardOpponent:::]` @ 0x10012306c
pub fn seq_battle_fast_run_toward_opponent(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    // `b.pl`: a NaN distance keeps running.
    let x_distance = opponent_x_distance(world, this);
    let reached = x_distance < 100.0;
    if !reached {
        return ItemResult::Pending;
    }

    let opponent = opponent_idol(world, this);
    if opponent.attack == Attack::Fast {
        let behavior = if opponent.attack_style == AttackStyle::Energy {
            "PygmyBattleAttackFastTieAgainstEnergy"
        } else {
            "PygmyBattleAttackFastTie"
        };
        pg_engine::logic::set_behavior(world, this, Some(behavior));
        return ItemResult::Replaced;
    }
    ItemResult::Done
}

/// Battle of the Gods: the x at `offset` in front of the opponent (on the fighter's side).
fn in_front_of_opponent(world: &World, this: Entity, offset: f32) -> f32 {
    let opponent = opponent_of(world, this);
    let opponent_x = optional_logic_pos(world, opponent).x;
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
    if enemy {
        offset + opponent_x
    } else {
        opponent_x - offset
    }
}

/// `DAT_10029d9b0[opponent attack == counter]`: the reappearance offset in front of a physical
/// opponent.
fn energy_fast_attack_offset(opponent: &Episode) -> f32 {
    let index = usize::from(opponent.attack == Attack::Counter);
    ENERGY_FAST_ATTACK_OFFSETS[index]
}

/// `-[PygmyLogic (BattleOfTheGods) battleEnergyFastMoveTowardOpponent:::]` @ 0x100123164
pub fn seq_battle_energy_fast_move_toward_opponent(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let opponent = opponent_idol(world, this);
    let offset = if opponent.attack_style == AttackStyle::Physical {
        energy_fast_attack_offset(opponent)
    } else {
        100.0
    };

    let x_pos = in_front_of_opponent(world, this, offset);
    own_display_mut(world, this).pos.x = x_pos;
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) battleEnergyFastMoveAboveOpponent:::]` @ 0x100123240
pub fn seq_battle_energy_fast_move_above_opponent(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let opponent = opponent_idol(world, this);
    let x_pos = if opponent.attack_style == AttackStyle::Physical {
        let offset = energy_fast_attack_offset(opponent);
        in_front_of_opponent(world, this, offset)
    } else if opponent.attack == Attack::Fast {
        // An energy fast opponent: the pygmy only moves 60 forward.
        let own_x = logic_pos(world, this).x;
        let x_scale = own_x_scale(world, this);
        own_x + x_scale * 60.0
    } else {
        in_front_of_opponent(world, this, 100.0)
    };

    own_display_mut(world, this).pos.x = x_pos;
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) battleEnergyFastWaitForOpponent:::]` @ 0x100123360
pub fn seq_battle_energy_fast_wait_for_opponent(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let x_distance = opponent_x_distance(world, this);
    if x_distance < 100.0 {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[PygmyLogic (BattleOfTheGods) parryAgainstStrongEnergy]` @ 0x1001233f8
pub fn parry_against_strong_energy(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyBattleAttackStrongEnergyTie"));
}

/// `-[PygmyLogic (BattleOfTheGods) checkForStrongAttackWin:::]` @ 0x100123410
pub fn seq_check_for_strong_attack_win(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let opponent = opponent_idol(world, this);
    if opponent.attack == Attack::Counter {
        pg_engine::logic::set_behavior(world, this, Some("PygmyBattleAttackStrongWin"));
    }

    // 2 even after the replacement, as in the original.
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) checkForEnergyStrongAttackWin:::]` @ 0x100123458
pub fn seq_check_for_energy_strong_attack_win(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut opponent = opponent_idol(world, this);
    if opponent.attack == Attack::Counter {
        pg_engine::logic::set_behavior(world, this, Some("PygmyBattleEnergyAttackStrongWin"));

        let opponent_pygmy = opponent_of(world, this);
        if let Some(opponent_pygmy) = opponent_pygmy {
            killed_in_battle(world, opponent_pygmy);
        }
        opponent = opponent_idol(world, this);
    }

    if opponent.attack == Attack::Strong {
        if opponent.attack_style == AttackStyle::Physical {
            set_battle_beam_pos(world, this);

            let opponent_pygmy = opponent_of(world, this);
            if let Some(opponent_pygmy) = opponent_pygmy {
                parry_against_strong_energy(world, opponent_pygmy);
            }
        }
        pg_engine::logic::set_behavior(world, this, Some("PygmyBattleEnergyAttackStrongTie"));
    }

    // 2 even after a replacement, as in the original.
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) checkForFastAttackWin:::]` @ 0x100123538
pub fn seq_check_for_fast_attack_win(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let opponent = opponent_idol(world, this);
    match opponent.attack {
        Attack::Fast => {
            let own = own_idol(world, this);
            if own.attack_style == AttackStyle::Energy {
                pg_engine::logic::set_behavior(world, this, Some("PygmyBattleEnergyAttackFastTie"));
                return ItemResult::Replaced;
            }
        }
        Attack::Counter => {
            let own = own_idol(world, this);
            if own.attack_style == AttackStyle::Energy {
                let opponent_pygmy = opponent_of(world, this);
                if let Some(opponent_pygmy) = opponent_pygmy {
                    battle_counter_fast_attack(world, opponent_pygmy);
                }
            }
        }
        Attack::Strong => {
            pg_engine::logic::set_behavior(world, this, Some("PygmyBattleAttackWin"));
            return ItemResult::Replaced;
        }
    }
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleChangeWeapon]` @ 0x100123608
pub fn anim_battle_change_weapon(world: &mut World, this: Entity) {
    let sword = fighter_logic(world, this, logic_table::PLAYER_SWORD, logic_table::ENEMY_SWORD);
    let shield = fighter_logic(world, this, logic_table::PLAYER_SHIELD, logic_table::ENEMY_SHIELD);
    let energy = fighter_logic(world, this, logic_table::PLAYER_ENERGY, logic_table::ENEMY_ENERGY);
    let energy_balls = [
        fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL1, logic_table::ENEMY_ENERGY_BALL1),
        fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL2, logic_table::ENEMY_ENERGY_BALL2),
        fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL3, logic_table::ENEMY_ENERGY_BALL3),
        fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL4, logic_table::ENEMY_ENERGY_BALL4),
    ];
    let energy_shield = fighter_logic(world, this, logic_table::PLAYER_ENERGY_SHIELD, logic_table::ENEMY_ENERGY_SHIELD);
    let beam = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BEAM, logic_table::ENEMY_ENERGY_BEAM);
    let beam_start = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BEAM_START, logic_table::ENEMY_ENERGY_BEAM_START);
    let beam_end = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BEAM_END, logic_table::ENEMY_ENERGY_BEAM_END);

    let own = own_idol(world, this);
    let realm = realm_name(own.realm);

    if own.attack_style == AttackStyle::Physical {
        let sword_behavior = format!("PygmySword{realm}");
        let shield_behavior = format!("PygmyShield{realm}");
        pg_engine::logic::set_behavior(world, sword, Some(&sword_behavior));
        pg_engine::logic::set_behavior(world, shield, Some(&shield_behavior));
        return;
    }

    let sword_behavior = format!("PygmyEnergyBallSword{realm}");
    let shield_behavior = format!("PygmyEnergyBallShield{realm}");
    let energy_behavior = format!("PygmyEnergy{realm}");
    let ball_behavior = format!("PygmyEnergyBall{realm}");
    let energy_shield_behavior = format!("PygmyEnergyShield{realm}");
    let beam_behavior = format!("PygmyEnergyBeam{realm}");

    pg_engine::logic::set_behavior(world, sword, Some(&sword_behavior));
    pg_engine::logic::set_behavior(world, shield, Some(&shield_behavior));
    pg_engine::logic::set_behavior(world, energy, Some(&energy_behavior));
    for energy_ball in energy_balls {
        pg_engine::logic::set_behavior(world, energy_ball, Some(&ball_behavior));
    }
    pg_engine::logic::set_behavior(world, energy_shield, Some(&energy_shield_behavior));
    pg_engine::logic::set_behavior(world, beam, Some(&beam_behavior));
    pg_engine::logic::set_behavior(world, beam_start, Some(&ball_behavior));
    pg_engine::logic::set_behavior(world, beam_end, Some(&ball_behavior));
}

/// `-[PygmyLogic (BattleOfTheGods) hitByBattleEnergyBall]` @ 0x100123e2c
pub fn hit_by_battle_energy_ball(world: &mut World, this: Entity) {
    pg_engine::logic::set_behavior(world, this, Some("PygmyBattleHitByEnergyBall"));
}

/// `-[PygmyLogic (BattleOfTheGods) killedByBattleEnergyBall]` @ 0x100123e44
pub fn killed_by_battle_energy_ball(world: &mut World, this: Entity) {
    killed_in_battle(world, this);
}

/// `-[PygmyLogic (BattleOfTheGods) throwEnergyBall:]` @ 0x100123e54
pub fn throw_energy_ball(world: &mut World, this: Entity, energy_ball: Entity) {
    let pos = logic_pos(world, this);
    let x_scale = own_x_scale(world, this);
    let start_pos = Vec4::new(pos.x + x_scale * 18.0, pos.y + 111.0, pos.z, pos.w);

    // The balls only damage an opponent whose attack is the strong one.
    battle_energy_ball_logic::set_no_damage(world, energy_ball, false);
    let opponent = opponent_idol(world, this);
    if opponent.attack != Attack::Strong {
        battle_energy_ball_logic::set_no_damage(world, energy_ball, true);
    }

    let opponent_pygmy = opponent_of(world, this);
    battle_energy_ball_logic::activate_with_start_pos(world, energy_ball, start_pos, opponent_pygmy);
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleThrowEnergyBall1]` @ 0x100123f5c
pub fn anim_battle_throw_energy_ball1(world: &mut World, this: Entity) {
    let energy_ball = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL1, logic_table::ENEMY_ENERGY_BALL1);
    throw_energy_ball(world, this, energy_ball);
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleThrowEnergyBall2]` @ 0x100123ff8
pub fn anim_battle_throw_energy_ball2(world: &mut World, this: Entity) {
    let energy_ball = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL2, logic_table::ENEMY_ENERGY_BALL2);
    throw_energy_ball(world, this, energy_ball);
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleThrowEnergyBall3]` @ 0x100124094
pub fn anim_battle_throw_energy_ball3(world: &mut World, this: Entity) {
    let energy_ball = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL3, logic_table::ENEMY_ENERGY_BALL3);
    throw_energy_ball(world, this, energy_ball);
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleThrowEnergyBall4]` @ 0x100124130
pub fn anim_battle_throw_energy_ball4(world: &mut World, this: Entity) {
    let energy_ball = fighter_logic(world, this, logic_table::PLAYER_ENERGY_BALL4, logic_table::ENEMY_ENERGY_BALL4);
    battle_energy_ball_logic::set_last_ball(world, energy_ball, true);
    throw_energy_ball(world, this, energy_ball);
}

/// `[[IdolManager instance] onBattleAttackTie]`.
fn report_battle_attack_tie(world: &mut World) {
    let Some(manager) = idol_manager::instance(world) else {
        return;
    };
    idol_manager::on_battle_attack_tie(world, manager);
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleTie]` @ 0x1001241f0
pub fn anim_battle_tie(world: &mut World, this: Entity) {
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
    if enemy {
        return;
    }
    report_battle_attack_tie(world);
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleEnergyCounterHit]` @ 0x100124240
pub fn anim_battle_energy_counter_hit(world: &mut World, this: Entity) {
    let opponent = opponent_idol(world, this);
    if opponent.attack != Attack::Fast {
        return;
    }

    let opponent_pygmy = opponent_of(world, this);
    if let Some(opponent_pygmy) = opponent_pygmy {
        killed_in_battle(world, opponent_pygmy);
    }
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleStrongHit]` @ 0x100124280
pub fn anim_battle_strong_hit(world: &mut World, this: Entity) {
    // The counter attack beats the fast one, the strong attack beats the counter.
    let own = own_idol(world, this);
    let beats_opponent = match own.attack {
        Attack::Counter => opponent_idol(world, this).attack == Attack::Fast,
        Attack::Strong => opponent_idol(world, this).attack == Attack::Counter,
        Attack::Fast => false,
    };
    if !beats_opponent {
        return;
    }

    let opponent_pygmy = opponent_of(world, this);
    if let Some(opponent_pygmy) = opponent_pygmy {
        killed_in_battle(world, opponent_pygmy);
    }
}

/// `-[PygmyLogic (BattleOfTheGods) animBattleFastHit]` @ 0x100124300
pub fn anim_battle_fast_hit(world: &mut World, this: Entity) {
    let opponent = opponent_idol(world, this);
    let opponent_pygmy = opponent_of(world, this);
    match opponent.attack {
        Attack::Counter => {
            if let Some(opponent_pygmy) = opponent_pygmy {
                battle_counter_fast_attack(world, opponent_pygmy);
            }
            pg_engine::logic::set_behavior(world, this, Some("PygmyBattleAttackFastWaitForCounter"));
        }
        Attack::Strong => {
            if let Some(opponent_pygmy) = opponent_pygmy {
                killed_in_battle(world, opponent_pygmy);
            }
        }
        Attack::Fast => {}
    }
}

/// `-[PygmyLogic (BattleOfTheGods) checkForAttackTie:::]` @ 0x1001243a8
pub fn seq_check_for_attack_tie(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let own = own_idol(world, this);
    let opponent = opponent_idol(world, this);
    let enemy = world.obj::<PygmyMinigames>(this).battle_of_the_gods_enemy;
    if own.attack == opponent.attack && !enemy {
        report_battle_attack_tie(world);
    }
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) battleShowLavaFlame:::]` @ 0x100124428
pub fn seq_battle_show_lava_flame(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let flame = fighter_logic(world, this, logic_table::LAVA_PIT_LEFT, logic_table::LAVA_PIT_RIGHT);
    pg_engine::logic::set_behavior(world, flame, Some("BattleLavaPitFlame"));
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) battleFall:::]` @ 0x1001244c8
pub fn seq_battle_fall(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let island_floor_collision = world.obj::<PygmyPygmyBehaviourItems>(this).island_floor_collision;

        let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
        if !island_floor_collision {
            core.on_island = false;
        }
        core.enable_island_side_collision = false;
        core.enable_island_floor_collision = true;

        world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = true;

        let ignore_island = call.bool("ignoreIsland");
        if ignore_island {
            world.obj_mut::<PygmyPygmyCoreFrame>(this).enable_island_floor_collision = false;
        }
    }

    // Kept inside the arena, or inside a wider band once killed. Only an out-of-range position
    // is written; a NaN one stays.
    let killed = world.obj::<PygmyMinigames>(this).killed_during_battle;
    let x_pos = logic_pos(world, this).x;
    let (min_x, max_x) = if killed { (-50.0, 530.0) } else { (20.0, 460.0) };
    if x_pos > max_x {
        own_display_mut(world, this).pos.x = max_x;
    } else if x_pos < min_x {
        own_display_mut(world, this).pos.x = min_x;
    }

    let island_floor_collision = world.obj::<PygmyPygmyBehaviourItems>(this).island_floor_collision;
    if island_floor_collision {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
        pg_engine::logic::logic_mut(world, this).pos_vel.x = 0.0;
    } else {
        let killed = world.obj::<PygmyMinigames>(this).killed_during_battle;
        let y_pos = logic_pos(world, this).y;
        let below_arena = killed && y_pos < 0.0;
        if !below_arena {
            return ItemResult::Pending;
        }

        world.obj_mut::<PygmyPygmyCoreFrame>(this).falling = false;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        clear_xy_vel(world, this);
        own_display_mut(world, this).pos.y = 0.0;
        world.obj_mut::<PygmyPygmyCoreFrame>(this).tilt_state = 0;
    }

    own_display_mut(world, this).rot.z = 0.0;
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) battleRespawn:::]` @ 0x1001246c0
pub fn seq_battle_respawn(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    spawn_from_battle_of_the_gods(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (BattleOfTheGods) onBattleReady:::]` @ 0x1001246e4
pub fn seq_on_battle_ready(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener_of(world, this);
    let temple_door = match listener {
        Some(listener) => crate::controller::temple_door_logic(world, listener, this),
        None => None,
    };
    if let Some(temple_door) = temple_door {
        temple_door_logic::on_pygmy_battle_ready(world, temple_door, this);
    }
    ItemResult::Done
}

/// The arrays of the crack hole that its `theRuns…LogicArray` getters return.
#[derive(Clone, Copy)]
enum CrackHoleArray {
    Platforms,
    Stalactites,
    Turds,
}

/// `[listener crackHoleLogic:self]`.
fn crack_hole_of(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener_of(world, this)?;
    crate::controller::crack_hole_logic(world, listener, this)
}

/// `[crackHole theRunsPlatformLogicArray]` and the other array getters. They only return the
/// ivars, whose generated ports cannot return arrays, so the ivars are read here; a nil crack
/// hole gives empty arrays.
fn crack_hole_array(world: &World, crack_hole: Option<Entity>, array: CrackHoleArray) -> Vec<Entity> {
    let Some(crack_hole) = crack_hole else {
        return Vec::new();
    };

    let state = world.obj::<CrackHoleLogic>(crack_hole);
    let logics = match array {
        CrackHoleArray::Platforms => &state.the_runs_platform_logic_array,
        CrackHoleArray::Stalactites => &state.the_runs_stalactite_logic_array,
        CrackHoleArray::Turds => &state.the_runs_turd_logic_array,
    };
    logics.clone()
}

/// The Runs: a bad landing costs 100 of speed, down to 300 (`fcsel …, mi`, which keeps NaN).
fn slow_down_after_bad_landing(world: &mut World, this: Entity) {
    let mut logic = pg_engine::logic::logic_mut(world, this);
    let x_vel = logic.pos_vel.x - 100.0;
    logic.pos_vel.x = if x_vel < 300.0 { 300.0 } else { x_vel };
}

/// `-[PygmyLogic (TheRuns) checkTheRunsCollision:]` @ 0x100139998
///
/// `last_pos` is `mLastPos` (the original passes a pointer to it).
pub fn check_the_runs_collision(world: &mut World, this: Entity, last_pos: Vec4) {
    let the_runs = world.obj::<PygmyPygmyTouchDrag>(this).the_runs;
    let fall_in_lava = world.obj::<PygmyPygmyCoreFrame>(this).the_runs_fall_in_lava;
    let minigames = world.obj::<PygmyMinigames>(this);
    let rock_explode = minigames.the_runs_rock_explode;
    let fall_off_sinking_platform = minigames.the_runs_fall_off_sinking_platform;
    if !the_runs || fall_in_lava || rock_explode || fall_off_sinking_platform {
        return;
    }

    let crack_hole = crack_hole_of(world, this);
    let previous_y_vel = pg_engine::logic::logic(world, this).pos_vel.y;

    // The platforms answer 1 landed, 2 hit the side, 3 still on it, 4 sinking, 5 rough landing.
    let mut landed_platform = None;
    let mut landed = false;
    let mut rough_landing = false;
    let mut still_on_platform = false;
    let platforms = crack_hole_array(world, crack_hole, CrackHoleArray::Platforms);
    for platform in platforms {
        let collision = the_runs_platform_logic::check_collision(world, platform, this, last_pos);
        match collision {
            1 => {
                pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
                landed = true;
                landed_platform = Some(platform);
                break;
            }
            2 => {
                let mut logic = pg_engine::logic::logic_mut(world, this);
                logic.pos_vel.x = -100.0;
                logic.pos_vel.y = 200.0;

                world.obj_mut::<PygmyPygmyCoreFrame>(this).the_runs_side_collision = true;
                pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsHitSide"));
                break;
            }
            3 => {
                still_on_platform = true;
            }
            4 => {
                pg_engine::logic::logic_mut(world, this).pos_vel.y = 500.0;

                let mut minigames = world.obj_mut::<PygmyMinigames>(this);
                minigames.the_runs_platform_collision = true;
                minigames.the_runs_fall_off_sinking_platform = true;

                pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsFall"));
                return;
            }
            5 => {
                pg_engine::logic::logic_mut(world, this).pos_vel.y = 0.0;
                landed = true;
                rough_landing = true;
                landed_platform = Some(platform);
                break;
            }
            _ => {}
        }
    }

    let stalactites = crack_hole_array(world, crack_hole, CrackHoleArray::Stalactites);
    for stalactite in stalactites {
        let collision = the_runs_stalactite_logic::check_collision(world, stalactite, this, last_pos);
        if collision == 1 {
            let mut logic = pg_engine::logic::logic_mut(world, this);
            logic.pos_vel.x = 0.0;
            logic.pos_vel.y = 500.0;

            world.obj_mut::<PygmyMinigames>(this).the_runs_fall_off_sinking_platform = true;
            pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsHitStalactite"));
            return;
        }
    }

    // Still on the platform it stood on: landed unless moving up.
    let platform_collision = world.obj::<PygmyMinigames>(this).the_runs_platform_collision;
    if platform_collision && !landed && still_on_platform {
        let y_vel = pg_engine::logic::logic(world, this).pos_vel.y;
        landed = y_vel <= 0.0;
    }

    let turds = crack_hole_array(world, crack_hole, CrackHoleArray::Turds);
    for turd in turds {
        let collided = the_runs_turd_logic::check_collision(world, turd, this, last_pos);
        if collided {
            // `fcsel …, mi`: 300 of speed lost, down to 300 (NaN stays).
            let mut logic = pg_engine::logic::logic_mut(world, this);
            let x_vel = logic.pos_vel.x - 300.0;
            logic.pos_vel.x = if x_vel < 300.0 { 300.0 } else { x_vel };
            logic.pos_vel.y = 0.0;

            world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = None;
            world.obj_mut::<PygmyMinigames>(this).the_runs_skidding = true;
            pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsTurdSkid"));
            return;
        }
    }

    let rock = match crack_hole {
        Some(crack_hole) => crack_hole_logic::the_runs_rock_logic(world, crack_hole),
        None => None,
    };
    let rock_hit = match rock {
        Some(rock) => the_runs_rock_logic::check_collision(world, rock, this, last_pos),
        None => false,
    };
    if rock_hit {
        clear_xy_vel(world, this);
        world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = None;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

        let mut minigames = world.obj_mut::<PygmyMinigames>(this);
        minigames.the_runs_skidding = false;
        minigames.the_runs_rock_explode = true;

        set_sacrifice_id(world, this, "LavaRock");
        pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsRockExplode"));
        return;
    }

    let platform_collision = world.obj::<PygmyMinigames>(this).the_runs_platform_collision;
    if platform_collision == landed {
        return;
    }
    world.obj_mut::<PygmyMinigames>(this).the_runs_platform_collision = landed;

    let started = world.obj::<PygmyMinigames>(this).the_runs_started;
    if !landed {
        let jumping = world.obj::<PygmyMinigames>(this).the_runs_jumping;
        if started && !jumping {
            pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsFall"));
        }
        return;
    }

    // The first landing starts the run (`[platform burnPygmy:self]`).
    if !started {
        world.obj_mut::<PygmyMinigames>(this).the_runs_started = true;
        world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;
        if let Some(platform) = landed_platform {
            the_runs_platform_logic::burn_pygmy(world, platform, this);
        }
        return;
    }

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.the_runs_jumping = false;
    if minigames.the_runs_skidding {
        return;
    }

    // `b.pl`: only a fall faster than 900 (not NaN) is a hard landing.
    let x_vel = pg_engine::logic::logic(world, this).pos_vel.x;
    let behavior = if previous_y_vel < -900.0 {
        slow_down_after_bad_landing(world, this);
        world.obj_mut::<PygmyMinigames>(this).the_runs_max_speed = false;
        "PygmyTheRunsTumble"
    } else if rough_landing {
        slow_down_after_bad_landing(world, this);
        world.obj_mut::<PygmyMinigames>(this).the_runs_max_speed = false;
        "PygmyTheRunsStumble"
    } else if x_vel >= 600.0 {
        world.obj_mut::<PygmyMinigames>(this).the_runs_max_speed = true;
        "PygmyTheRunsRunStartFast"
    } else {
        world.obj_mut::<PygmyMinigames>(this).the_runs_max_speed = false;
        "PygmyTheRunsRun"
    };
    pg_engine::logic::set_behavior(world, this, Some(behavior));
}

/// `-[PygmyLogic (TheRuns) startTheRuns]` @ 0x10013a15c
pub fn start_the_runs(world: &mut World, this: Entity) {
    let in_crack_hole = world.obj::<PygmyPygmyCoreFrame>(this).in_crack_hole;
    if !in_crack_hole {
        return;
    }

    world.obj_mut::<PygmyPygmyTouchDrag>(this).the_runs = true;
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = true;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = false;

    own_display_mut(world, this).is_selectable = false;
    place_at_minigame_start(world, this, 34.0);
    pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsStart"));
}

/// `-[PygmyLogic (TheRuns) theRunsBurnPygmyComplete]` @ 0x10013a2a8
pub fn the_runs_burn_pygmy_complete(world: &mut World, this: Entity) {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;

    let mut core = world.obj_mut::<PygmyPygmyCoreFrame>(this);
    core.enable_island_side_collision = false;
    core.enable_island_floor_collision = false;
    core.the_runs_burn_complete = true;

    pg_engine::logic::logic_mut(world, this).pos_vel.x = 300.0;
    own_display_mut(world, this).is_selectable = true;
    pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsRun"));
}

/// `-[PygmyLogic (TheRuns) onStartTheRunsFall:::]` @ 0x10013a358
pub fn seq_on_start_the_runs_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    ItemResult::Done
}

/// `-[PygmyLogic (TheRuns) theRunsJumpUp:::]` @ 0x10013a370
pub fn seq_the_runs_jump_up(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    // The faster the run, the longer and higher the jump.
    let mut logic = pg_engine::logic::logic_mut(world, this);
    let speed_factor = (logic.pos_vel.x - 300.0) / 300.0;
    if call.first_time {
        logic.stall_time = speed_factor * THE_RUNS_JUMP_TIME_FACTOR + 0.15;
    }
    logic.pos_vel.y = speed_factor * 100.0 + 300.0;
    let stall_time = logic.stall_time;

    if stall_time <= 0.0 {
        world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = None;
        pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsJumpDown"));
        return ItemResult::Replaced;
    }

    pg_engine::logic::logic_mut(world, this).stall_time = stall_time - call.delta_time;
    ItemResult::Pending
}

/// `-[PygmyLogic (TheRuns) theRunsRun:::]` @ 0x10013a46c
pub fn seq_the_runs_run(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.the_runs_skidding = false;
    if minigames.the_runs_max_speed {
        return ItemResult::Pending;
    }

    let x_vel = pg_engine::logic::logic(world, this).pos_vel.x;
    if x_vel >= 600.0 {
        world.obj_mut::<PygmyMinigames>(this).the_runs_max_speed = true;
        pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsRunFast"));
        return ItemResult::Replaced;
    }
    ItemResult::Pending
}

/// `-[PygmyLogic (TheRuns) theRunsFall:::]` @ 0x10013a4f4
pub fn seq_the_runs_fall(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<SpriteLogic>(this).ignore_gravity = false;
    ItemResult::Pending
}

/// `[[listener crackHoleLogic:self] stopTheRuns]`.
fn stop_the_runs_in_crack_hole(world: &mut World, this: Entity) {
    let crack_hole = crack_hole_of(world, this);
    if let Some(crack_hole) = crack_hole {
        crack_hole_logic::stop_the_runs(world, crack_hole);
    }
}

/// `-[PygmyLogic (TheRuns) onTheRunsSplashComplete:::]` @ 0x10013a50c
pub fn seq_on_the_runs_splash_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_the_runs_in_crack_hole(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (TheRuns) onTheRunsExplodeComplete:::]` @ 0x10013a558
pub fn seq_on_the_runs_explode_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    stop_the_runs_in_crack_hole(world, this);
    ItemResult::Done
}

/// `-[PygmyLogic (TheRuns) touchBeganTheRuns:allTouches:]` @ 0x10013a5a4
pub fn touch_began_the_runs(world: &mut World, this: Entity, touch: Entity, _all_touches: &[Entity]) {
    let minigames = world.obj::<PygmyMinigames>(this);
    let can_jump = minigames.the_runs_started && !minigames.the_runs_rock_explode && minigames.the_runs_platform_collision;
    if !can_jump {
        return;
    }

    world.obj_mut::<PygmyPygmyCoreFrame>(this).active_touch = Some(touch);

    let mut minigames = world.obj_mut::<PygmyMinigames>(this);
    minigames.the_runs_skidding = false;
    minigames.the_runs_jumping = true;

    pg_engine::logic::set_behavior(world, this, Some("PygmyTheRunsJumpUp"));
}
