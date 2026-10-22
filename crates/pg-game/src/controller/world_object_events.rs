//! `PocketGodViewController`: the callbacks of the world objects. The drag callbacks set and
//! clear the `mDragged<X>Logic` references; the coconut splits and regrows; the fishing bob, the
//! piranhas and Bait Master are tracked; boss visibility refreshes skins; and the hunts, shark
//! snaps, bird bombs, laser shark and coconut bounces score the challenge games
//! (`docs/re/controller.md` §4.11, §4.15).

use crate::prelude::*;
use super::*;

use pg_engine::{display, logic};

use crate::logic::island_core::coconut_logic;
use crate::logic::monsters::{spider_logic, t_rex_logic};
use crate::logic::ocean_fishing::fishing_bob_logic;
use crate::logic::underwater_baitmaster_laser::piranha_logic;
use crate::managers::{challenge_manager, skin_pack_manager};
use crate::pygmy;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerWorldObjectEvents {
    /// `mPiranhaLogicArray`: `@"NSMutableArray"`
    pub piranha_logic_array: Vec<Entity>,

    /// `mFishingBobLogic`: `@"FishingBobLogic"`
    pub fishing_bob_logic: Option<Entity>,

    /// `mbBaitMaster`: `B`
    pub bait_master: bool,

    /// `mDraggedSwordFishLogic`: `@"SwordFishLogic"`
    pub dragged_sword_fish_logic: Option<Entity>,

    /// `mDraggedFishLogic`: `@"FishLogic"`
    pub dragged_fish_logic: Option<Entity>,

    /// `mDraggedDanceBalloonLogic`: `@"DanceBalloonLogic"`
    pub dragged_dance_balloon_logic: Option<Entity>,

    /// `mDraggedBananaLogic`: `@"BananaLogic"`
    pub dragged_banana_logic: Option<Entity>,

    /// `mDraggedMonkeyLogic`: `@"MonkeyLogic"`
    pub dragged_monkey_logic: Option<Entity>,

    /// `mDraggedMoronSnakeLogic`: `@"MoronSnakeLogic"`
    pub dragged_moron_snake_logic: Option<Entity>,

    /// `mDraggedJellyfishLogic`: `@"JellyfishLogic"`
    pub dragged_jellyfish_logic: Option<Entity>,

    /// `mDraggedGodIdolLogic`: `@"GodIdolLogic"`
    pub dragged_god_idol_logic: Option<Entity>,

    /// `mDraggedApocalypseGemLogic`: `@"ApocalypseGemLogic"`
    pub dragged_apocalypse_gem_logic: Option<Entity>,

    /// `mDraggedSpearLogic`: `@"SpearLogic"`
    pub dragged_spear_logic: Option<Entity>,

    /// `mDraggedFishingBobLogic`: `@"FishingBobLogic"`
    pub dragged_fishing_bob_logic: Option<Entity>,

    /// `mDraggedFishingPoleLogic`: `@"FishingPoleLogic"`
    pub dragged_fishing_pole_logic: Option<Entity>,

    /// `mDraggedWaterWingsLogic`: `@"WaterWingsLogic"`
    pub dragged_water_wings_logic: Option<Entity>,

    /// `mDraggedCoconutLogic`: `@"CoconutLogic"`
    pub dragged_coconut_logic: Option<Entity>,

    /// `mDraggedWoodLogic`: `@"WoodLogic"`
    pub dragged_wood_logic: Option<Entity>,

    /// `mDraggedAnchorLogic`: `@"AnchorLogic"`
    pub dragged_anchor_logic: Option<Entity>,

    /// `mDraggedTilePlugLogic`: `@"TilePlugLogic"`
    pub dragged_tile_plug_logic: Option<Entity>,

    /// `mDraggedBoulderLogic`: `@"BoulderLogic"`
    pub dragged_boulder_logic: Option<Entity>,

    /// `mDraggedTRexEggLogic`: `@"TRexEggLogic"`
    pub dragged_t_rex_egg_logic: Option<Entity>,

    /// `mDirectedAntHillLogic`: `@"AntHillLogic"`
    pub directed_ant_hill_logic: Option<Entity>,

    /// `mDraggedMagnifyingGlassLogic`: `@"MagnifyingGlassLogic"`
    pub dragged_magnifying_glass_logic: Option<Entity>,

    /// `mDraggedSharkLogic`: `@"SharkLogic"`
    pub dragged_shark_logic: Option<Entity>,

    /// `mDraggedBirdLogic`: `@"BirdLogic"`
    pub dragged_bird_logic: Option<Entity>,

    /// `mDraggedClosetTempleClosetDoorLogic`: `@"ClosetTempleClosetDoorLogic"`
    pub dragged_closet_temple_closet_door_logic: Option<Entity>,

    /// `mDraggedThetanLogic`: `@"ThetanLogic"`
    pub dragged_thetan_logic: Option<Entity>,
}

// `mIslandLocationState` values.
const LOCATION_ROCK_ISLAND: i32 = 1;
const LOCATION_GRAVEYARD_ISLAND: i32 = 3;

// Challenge game ids (`docs/re/controller.md` §4.11).
const COCONUT_BOUNCES_GAME: i32 = 0;
const BIRD_BOMBS_GAME: i32 = 2;
const SHARK_SNAPS_GAME: i32 = 3;
const BAIT_MASTER_GAME: i32 = 5;
const LASER_SHARK_GAME: i32 = 6;
const KILL_T_REX_GAME: i32 = 7;
const KILL_SPIDER_GAME: i32 = 8;
const KILL_ICE_MONSTER_GAME: i32 = 9;

/// `@"Coconut"`: the multi model of the second coconut half.
const COCONUT_MODEL_ID: &str = "Coconut";

/// Where the last coconut regrows on the tree (`0x43230000`, `0x434d0000`).
const COCONUT_TREE_X_POS: f32 = 163.0;
const COCONUT_TREE_Y_POS: f32 = 205.0;

/// `checkAchievement:score:message1:message2:highScoreWins:`'s first message line.
const PERSONAL_BEST_MESSAGE: &str = "You've achieved a personal best of";

/// `-[PocketGodViewController (Game) isTRexActive]` @ 0x10016404c
pub fn is_t_rex_active(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_ROCK_ISLAND {
        return false;
    }

    let t_rex = world.obj::<ControllerWorldRegistry>(this).t_rex_logic;
    match t_rex {
        Some(t_rex) => t_rex_logic::is_active(world, t_rex),
        None => false,
    }
}

/// `-[PocketGodViewController (Game) isSpiderActive]` @ 0x10016409c
pub fn is_spider_active(world: &mut World, this: Entity) -> bool {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    if island_location_state != LOCATION_GRAVEYARD_ISLAND {
        return false;
    }

    let spider = world.obj::<ControllerWorldRegistry>(this).spider_logic;
    match spider {
        Some(spider) => spider_logic::is_active(world, spider),
        None => false,
    }
}

/// `-[PocketGodViewController (Game) onPygmiesSkewered:]` @ 0x10016468c
pub fn on_pygmies_skewered(world: &mut World, this: Entity, pygmies_skewered: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::check_achievement_score_message1_message2_high_score_wins(
            world,
            challenge_manager,
            "Skewered",
            pygmies_skewered,
            PERSONAL_BEST_MESSAGE,
            "%d Skewered Pygmies!",
            true,
        );
    }
}

/// `-[PocketGodViewController (Game) onPygmiesBowled:]` @ 0x1001646d0
pub fn on_pygmies_bowled(world: &mut World, this: Entity, pygmies_bowled: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::check_achievement_score_message1_message2_high_score_wins(
            world,
            challenge_manager,
            "Bowled",
            pygmies_bowled,
            PERSONAL_BEST_MESSAGE,
            "%d Pygmies Bowled!",
            true,
        );
    }
}

/// `-[PocketGodViewController (Game) onCoconutDrag:]` @ 0x1001650d8
pub fn on_coconut_drag(world: &mut World, this: Entity, coconut: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_coconut_logic = Some(coconut);
}

/// `-[PocketGodViewController (Game) onCoconutDragComplete:]` @ 0x1001650ec
pub fn on_coconut_drag_complete(world: &mut World, this: Entity, _coconut_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_coconut_logic = None;
}

/// `-[PocketGodViewController (Game) onWoodDrag:]` @ 0x100165100
pub fn on_wood_drag(world: &mut World, this: Entity, wood_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_wood_logic = Some(wood_logic);
}

/// `-[PocketGodViewController (Game) onWoodDragComplete:]` @ 0x100165114
pub fn on_wood_drag_complete(world: &mut World, this: Entity, _wood_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_wood_logic = None;
}

/// `-[PocketGodViewController (Game) onSpearDrag:]` @ 0x100165128
pub fn on_spear_drag(world: &mut World, this: Entity, spear_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_spear_logic = Some(spear_logic);
}

/// `-[PocketGodViewController (Game) onSpearDragComplete:]` @ 0x10016513c
pub fn on_spear_drag_complete(world: &mut World, this: Entity, _spear_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_spear_logic = None;
}

/// `-[PocketGodViewController (Game) onSpearThrow:]` @ 0x100165150
///
/// A thrown spear counts for whichever hunt runs, the T-Rex's first.
pub fn on_spear_throw(world: &mut World, this: Entity, _spear_logic: Entity) {
    let hunted_game = if is_challenge_game_active(world, this, KILL_T_REX_GAME) {
        KILL_T_REX_GAME
    } else if is_challenge_game_active(world, this, KILL_SPIDER_GAME) {
        KILL_SPIDER_GAME
    } else {
        return;
    };
    increment_challenge_game_score(world, this, hunted_game);
}

/// `-[PocketGodViewController (Game) onWaterWingsDrag:]` @ 0x1001651f0
pub fn on_water_wings_drag(world: &mut World, this: Entity, water_wings_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_water_wings_logic = Some(water_wings_logic);
}

/// `-[PocketGodViewController (Game) onWaterWingsDragComplete:]` @ 0x100165204
pub fn on_water_wings_drag_complete(world: &mut World, this: Entity, _water_wings_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_water_wings_logic = None;
}

/// `-[PocketGodViewController (Game) onFishingPoleDrag:]` @ 0x100165218
pub fn on_fishing_pole_drag(world: &mut World, this: Entity, fishing_pole_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_fishing_pole_logic = Some(fishing_pole_logic);
}

/// `-[PocketGodViewController (Game) onFishingPoleDragComplete:]` @ 0x10016522c
pub fn on_fishing_pole_drag_complete(world: &mut World, this: Entity, _fishing_pole_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_fishing_pole_logic = None;
}

/// `-[PocketGodViewController (Game) onFishingPoleCreateBob:fishingBobLogic:]` @ 0x100165240
pub fn on_fishing_pole_create_bob(world: &mut World, this: Entity, _fishing_pole_logic: Entity, fishing_bob: Entity) {
    fishing_bob_logic::set_fishing_bob_logic_listener(world, fishing_bob, this);
    world.obj_mut::<ControllerWorldObjectEvents>(this).fishing_bob_logic = Some(fishing_bob);
}

/// `-[PocketGodViewController (Game) onFishingPoleDestroyBob:fishingBobLogic:]` @ 0x100165288
pub fn on_fishing_pole_destroy_bob(world: &mut World, this: Entity, _fishing_pole_logic: Entity, _fishing_bob_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).fishing_bob_logic = None;
}

/// `-[PocketGodViewController (Game) onFishingBobDrag:]` @ 0x10016529c
pub fn on_fishing_bob_drag(world: &mut World, this: Entity, fishing_bob: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_fishing_bob_logic = Some(fishing_bob);
}

/// `-[PocketGodViewController (Game) onFishingBobDragComplete:]` @ 0x1001652b0
pub fn on_fishing_bob_drag_complete(world: &mut World, this: Entity, _fishing_bob_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_fishing_bob_logic = None;
}

/// `-[PocketGodViewController (Game) onFishDrag:]` @ 0x1001652c4
pub fn on_fish_drag(world: &mut World, this: Entity, fish_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_fish_logic = Some(fish_logic);
}

/// `-[PocketGodViewController (Game) onFishDragComplete:]` @ 0x1001652d8
pub fn on_fish_drag_complete(world: &mut World, this: Entity, _fish_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_fish_logic = None;
}

/// `-[PocketGodViewController (Game) onSwordFishDrag:]` @ 0x1001652ec
pub fn on_sword_fish_drag(world: &mut World, this: Entity, sword_fish_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_sword_fish_logic = Some(sword_fish_logic);
}

/// `-[PocketGodViewController (Game) onSwordFishDragComplete:]` @ 0x100165300
pub fn on_sword_fish_drag_complete(world: &mut World, this: Entity, _sword_fish_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_sword_fish_logic = None;
}

/// `-[PocketGodViewController (Game) onBananaDrag:]` @ 0x100165314
pub fn on_banana_drag(world: &mut World, this: Entity, banana_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_banana_logic = Some(banana_logic);
}

/// `-[PocketGodViewController (Game) onBananaDragComplete:]` @ 0x100165328
pub fn on_banana_drag_complete(world: &mut World, this: Entity, _banana_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_banana_logic = None;
}

/// `-[PocketGodViewController (Game) onMonkeyDrag:]` @ 0x10016533c
pub fn on_monkey_drag(world: &mut World, this: Entity, monkey_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_monkey_logic = Some(monkey_logic);
}

/// `-[PocketGodViewController (Game) onMonkeyDragComplete:]` @ 0x100165350
pub fn on_monkey_drag_complete(world: &mut World, this: Entity, _monkey_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_monkey_logic = None;
}

/// `-[PocketGodViewController (Game) onDanceBalloonDrag:]` @ 0x100165364
pub fn on_dance_balloon_drag(world: &mut World, this: Entity, dance_balloon_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_dance_balloon_logic = Some(dance_balloon_logic);
}

/// `-[PocketGodViewController (Game) onDanceBalloonDragComplete:]` @ 0x100165378
pub fn on_dance_balloon_drag_complete(world: &mut World, this: Entity, _dance_balloon_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_dance_balloon_logic = None;
}

/// `-[PocketGodViewController (Game) onMoronSnakeDrag:]` @ 0x10016538c
pub fn on_moron_snake_drag(world: &mut World, this: Entity, moron_snake_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_moron_snake_logic = Some(moron_snake_logic);
}

/// `-[PocketGodViewController (Game) onMoronSnakeDragComplete:]` @ 0x1001653a0
pub fn on_moron_snake_drag_complete(world: &mut World, this: Entity, _moron_snake_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_moron_snake_logic = None;
}

/// `-[PocketGodViewController (Game) onJellyfishDrag:]` @ 0x1001653b4
pub fn on_jellyfish_drag(world: &mut World, this: Entity, jellyfish_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_jellyfish_logic = Some(jellyfish_logic);
}

/// `-[PocketGodViewController (Game) onJellyfishDragComplete:]` @ 0x1001653c8
pub fn on_jellyfish_drag_complete(world: &mut World, this: Entity, _jellyfish_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_jellyfish_logic = None;
}

/// `-[PocketGodViewController (Game) onGodIdolDrag:]` @ 0x1001653dc
pub fn on_god_idol_drag(world: &mut World, this: Entity, god_idol_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_god_idol_logic = Some(god_idol_logic);
}

/// `-[PocketGodViewController (Game) onGodIdolDragComplete:]` @ 0x1001653f0
pub fn on_god_idol_drag_complete(world: &mut World, this: Entity, _god_idol_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_god_idol_logic = None;
}

/// `-[PocketGodViewController (Game) onApocalypseGemDrag:]` @ 0x100165404
pub fn on_apocalypse_gem_drag(world: &mut World, this: Entity, apocalypse_gem_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_apocalypse_gem_logic = Some(apocalypse_gem_logic);
}

/// `-[PocketGodViewController (Game) onApocalypseGemDragComplete:]` @ 0x100165418
pub fn on_apocalypse_gem_drag_complete(world: &mut World, this: Entity, _apocalypse_gem_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_apocalypse_gem_logic = None;
}

/// `-[PocketGodViewController (Game) checkCountCoconutBounces]` @ 0x1001656c0
///
/// The coconut-bounces game ends once no coconut (of the first two) is in the air.
pub fn check_count_coconut_bounces(world: &mut World, this: Entity) {
    let coconuts = world.obj::<ControllerWorldRegistry>(this).coconut_logic_array.clone();

    // `[[mCoconutLogicArray objectAtIndex:0] isInAir]`: the island always has its tree
    // coconut; an empty array answers NO here, as a nil one does in the original.
    let first_coconut = coconuts.first().copied();
    let mut in_air = match first_coconut {
        Some(first_coconut) => coconut_logic::is_in_air(world, first_coconut),
        None => false,
    };
    if coconuts.len() > 1 && !in_air {
        let second_coconut = coconuts[1];
        in_air = coconut_logic::is_in_air(world, second_coconut);
    }
    if in_air {
        return;
    }

    stop_challenge_game(world, this, COCONUT_BOUNCES_GAME);
}

/// `-[PocketGodViewController (Game) onCoconutHitPygmy:]` @ 0x1001657a4
///
/// Scores a coconut bounce; a whole coconut splits into two halves, the second a new
/// `CoconutLogic` on a new `BCMultiModel` that copies the first's position and collision box.
pub fn on_coconut_hit_pygmy(world: &mut World, this: Entity, coconut: Entity) {
    increment_challenge_game_score(world, this, COCONUT_BOUNCES_GAME);

    let already_split = coconut_logic::is_split(world, coconut);
    if already_split {
        return;
    }
    coconut_logic::split(world, coconut, false);

    let display_object = logic::display_object(world, coconut);
    let half_model = display::multi_model::spawn(world, Some(COCONUT_MODEL_ID));

    let source = display::object(world, display_object);
    let pos = source.pos;
    let is_selectable = source.is_selectable;
    let origin_pos = source.origin_pos;
    let col_x_min = source.col_x_min;
    let col_x_max = source.col_x_max;
    let col_y_min = source.col_y_min;
    let col_y_max = source.col_y_max;
    {
        let mut half_model_object = display::object_mut(world, half_model);
        half_model_object.pos = pos;
        half_model_object.is_selectable = is_selectable;
        half_model_object.origin_pos = origin_pos;
        half_model_object.col_x_min = col_x_min;
        half_model_object.col_x_max = col_x_max;
        half_model_object.col_y_min = col_y_min;
        half_model_object.col_y_max = col_y_max;
    }

    let half_coconut = coconut_logic::spawn(world, half_model);
    coconut_logic::set_coconut_logic_listener(world, half_coconut, this);
    world.obj_mut::<ControllerWorldRegistry>(this).coconut_logic_array.push(half_coconut);

    // Original bug, kept: the new half gets its own velocity back (`[half setPosVel:[half
    // posVel]]`), a no-op; the split coconut's velocity was probably meant.
    let half_pos_vel = logic::logic(world, half_coconut).pos_vel;
    logic::logic_mut(world, half_coconut).pos_vel = half_pos_vel;

    coconut_logic::split(world, half_coconut, true);
}

/// `-[PocketGodViewController (Game) onCoconutHitGround:]` @ 0x100165a70
pub fn on_coconut_hit_ground(world: &mut World, this: Entity, _coconut_logic: Entity) {
    check_count_coconut_bounces(world, this);
}

/// `-[PocketGodViewController (Game) onCoconutDie:]` @ 0x100165a80
///
/// A dying coconut half leaves the island while another coconut is left; the last one goes back
/// on the tree and grows again.
pub fn on_coconut_die(world: &mut World, this: Entity, coconut: Entity) {
    check_count_coconut_bounces(world, this);

    let coconut_count = world.obj::<ControllerWorldRegistry>(this).coconut_logic_array.len();
    let display_object = logic::display_object(world, coconut);
    if coconut_count > 1 {
        display::remove_from_display_group(world, display_object);
        logic::shutdown(world, coconut);

        // `removeObject:` drops the last reference, but the shut-down entity stays: the drag
        // references (`mDraggedCoconutLogic`) and the touch arrays may still name it.
        let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
        registry.coconut_logic_array.retain(|other_coconut| *other_coconut != coconut);
        return;
    }

    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = COCONUT_TREE_X_POS;
        object.pos.y = COCONUT_TREE_Y_POS;
    }
    coconut_logic::grow(world, coconut);
}

/// `-[PocketGodViewController (Game) onBaitMasterStart:]` @ 0x100169270
pub fn on_bait_master_start(world: &mut World, this: Entity, _anchor_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).bait_master = true;

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        pygmy::start_bait_master(world, pygmy);
    }
}

/// `-[PocketGodViewController (Game) onBaitMasterStop:]` @ 0x100169394
pub fn on_bait_master_stop(world: &mut World, this: Entity, _anchor_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).bait_master = false;
    stop_challenge_game(world, this, BAIT_MASTER_GAME);

    let pygmies = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    for pygmy in pygmies {
        pygmy::stop_bait_master(world, pygmy);
    }
}

/// `-[PocketGodViewController (Game) onNewPiranhaLogic:]` @ 0x1001694dc
pub fn on_new_piranha_logic(world: &mut World, this: Entity, piranha: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).piranha_logic_array.push(piranha);
}

/// `-[PocketGodViewController (Game) onRemovePiranhaLogic:]` @ 0x1001694fc
///
/// `removeObject:` releases the array's reference, the piranha's last one (the anchor released
/// its own after `onNewPiranhaLogic:`).
pub fn on_remove_piranha_logic(world: &mut World, this: Entity, piranha: Entity) {
    let removed = {
        let mut events = world.obj_mut::<ControllerWorldObjectEvents>(this);
        let count_before = events.piranha_logic_array.len();
        events.piranha_logic_array.retain(|other_piranha| *other_piranha != piranha);
        events.piranha_logic_array.len() != count_before
    };
    if removed {
        piranha_logic::dealloc(world, piranha);
    }
}

/// `-[PocketGodViewController (Game) onRemoveAllPiranhaLogics:]` @ 0x10016951c
///
/// `removeAllObjects` releases the last reference of every piranha.
pub fn on_remove_all_piranha_logics(world: &mut World, this: Entity, _anchor_logic: Entity) {
    let piranhas = {
        let mut events = world.obj_mut::<ControllerWorldObjectEvents>(this);
        std::mem::take(&mut events.piranha_logic_array)
    };
    for piranha in piranhas {
        piranha_logic::dealloc(world, piranha);
    }
}

/// `-[PocketGodViewController (Game) onPiranhaEatPygmy:]` @ 0x10016953c
pub fn on_piranha_eat_pygmy(world: &mut World, this: Entity, _anchor_logic: Entity) {
    let bait_master = world.obj::<ControllerWorldObjectEvents>(this).bait_master;
    if bait_master {
        increment_challenge_game_score(world, this, BAIT_MASTER_GAME);
    }
}

/// `-[PocketGodViewController (Game) onLaserSharkZapPygmy:]` @ 0x100169578
pub fn on_laser_shark_zap_pygmy(world: &mut World, this: Entity, _laser_shark_logic: Entity) {
    increment_challenge_game_score(world, this, LASER_SHARK_GAME);
}

/// `-[PocketGodViewController (Game) onLaserSharkZapComplete:]` @ 0x10016959c
pub fn on_laser_shark_zap_complete(world: &mut World, this: Entity, _laser_shark_logic: Entity) {
    stop_challenge_game(world, this, LASER_SHARK_GAME);
}

/// `-[PocketGodViewController (Game) onAnchorDrag:]` @ 0x1001695c0
pub fn on_anchor_drag(world: &mut World, this: Entity, anchor_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_anchor_logic = Some(anchor_logic);
}

/// `-[PocketGodViewController (Game) onAnchorDragComplete:]` @ 0x1001695d4
pub fn on_anchor_drag_complete(world: &mut World, this: Entity, _anchor_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_anchor_logic = None;
}

/// `-[PocketGodViewController (Game) onTilePlugDrag:]` @ 0x1001695e8
pub fn on_tile_plug_drag(world: &mut World, this: Entity, tile_plug_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_tile_plug_logic = Some(tile_plug_logic);
}

/// `-[PocketGodViewController (Game) onTilePlugDragComplete:]` @ 0x1001695fc
pub fn on_tile_plug_drag_complete(world: &mut World, this: Entity, _tile_plug_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_tile_plug_logic = None;
}

/// `-[PocketGodViewController (Game) onBoulderDrag:]` @ 0x100169610
pub fn on_boulder_drag(world: &mut World, this: Entity, boulder_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_boulder_logic = Some(boulder_logic);
}

/// `-[PocketGodViewController (Game) onBoulderDragComplete:]` @ 0x100169624
pub fn on_boulder_drag_complete(world: &mut World, this: Entity, _boulder_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_boulder_logic = None;
}

/// `-[PocketGodViewController (Game) onTRexEggDrag:]` @ 0x100169638
pub fn on_t_rex_egg_drag(world: &mut World, this: Entity, t_rex_egg_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_t_rex_egg_logic = Some(t_rex_egg_logic);
}

/// `-[PocketGodViewController (Game) onTRexEggDragComplete:]` @ 0x10016964c
pub fn on_t_rex_egg_drag_complete(world: &mut World, this: Entity, _t_rex_egg_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_t_rex_egg_logic = None;
}

/// `-[PocketGodViewController (Game) onTRexVisible:]` @ 0x100169660
pub fn on_t_rex_visible(world: &mut World, this: Entity, _t_rex_logic: Entity) {
    refresh_skin_pack_option(world, this, "TRex", "Tyrannosaurus Rex", "TRex Body");
}

/// `-[PocketGodViewController (Game) onTRexHitBySpear:]` @ 0x10016969c
pub fn on_t_rex_hit_by_spear(world: &mut World, this: Entity, _t_rex_logic: Entity) {
    if is_challenge_game_active(world, this, KILL_T_REX_GAME) {
        return;
    }
    set_challenge_game_score(world, this, KILL_T_REX_GAME, 1);
}

/// `-[PocketGodViewController (Game) onTRexKilledBySpear:]` @ 0x100169704
pub fn on_t_rex_killed_by_spear(world: &mut World, this: Entity, _t_rex_logic: Entity) {
    stop_challenge_game(world, this, KILL_T_REX_GAME);
}

/// `-[PocketGodViewController (Game) onTRexCancelHunt:]` @ 0x100169728
pub fn on_t_rex_cancel_hunt(world: &mut World, this: Entity, _t_rex_logic: Entity) {
    forfeit_challenge_game(world, this, KILL_T_REX_GAME);
}

/// `-[PocketGodViewController (Game) onSpiderVisible:]` @ 0x10016974c
pub fn on_spider_visible(world: &mut World, this: Entity, _spider_logic: Entity) {
    refresh_skin_pack_option(world, this, "Spider", "Spider", "Spider");
}

/// `-[PocketGodViewController (Game) onSpiderHitBySpear:]` @ 0x100169780
pub fn on_spider_hit_by_spear(world: &mut World, this: Entity, _spider_logic: Entity) {
    if is_challenge_game_active(world, this, KILL_SPIDER_GAME) {
        return;
    }
    set_challenge_game_score(world, this, KILL_SPIDER_GAME, 1);
}

/// `-[PocketGodViewController (Game) onSpiderKilledBySpear:]` @ 0x1001697e8
pub fn on_spider_killed_by_spear(world: &mut World, this: Entity, _spider_logic: Entity) {
    stop_challenge_game(world, this, KILL_SPIDER_GAME);
}

/// `-[PocketGodViewController (Game) onSpiderCancelHunt:]` @ 0x10016980c
pub fn on_spider_cancel_hunt(world: &mut World, this: Entity, _spider_logic: Entity) {
    forfeit_challenge_game(world, this, KILL_SPIDER_GAME);
}

/// `-[PocketGodViewController (Game) onLocustQueenVisible:]` @ 0x100169830
pub fn on_locust_queen_visible(world: &mut World, this: Entity, _locust_queen_logic: Entity) {
    refresh_skin_pack_option(world, this, "ZombiePack", "Locust Queen", "Locust Queen");
}

/// `-[PocketGodViewController (Game) onApeVisible:]` @ 0x100169868
pub fn on_ape_visible(world: &mut World, this: Entity, _ape_logic: Entity) {
    refresh_skin_pack_option(world, this, "ZombiePack", "Ape", "Ape");
}

/// `-[PocketGodViewController (Game) onIceMonsterVisible:]` @ 0x1001698a0
pub fn on_ice_monster_visible(world: &mut World, this: Entity, _ice_monster_logic: Entity) {
    refresh_skin_pack_option(world, this, "IceMonster", "Ice Monster", "Ice Monster");
}

/// `-[PocketGodViewController (Game) onIceMonsterHitByPygmy:]` @ 0x1001698d8
pub fn on_ice_monster_hit_by_pygmy(_world: &mut World, _this: Entity, _ice_monster_logic: Entity) {}

/// `-[PocketGodViewController (Game) onIceMonsterKilled:]` @ 0x1001698dc
pub fn on_ice_monster_killed(world: &mut World, this: Entity, _ice_monster_logic: Entity) {
    stop_challenge_game(world, this, KILL_ICE_MONSTER_GAME);
}

/// `-[PocketGodViewController (Game) onIceMonsterStartBattle:]` @ 0x100169900
pub fn on_ice_monster_start_battle(world: &mut World, this: Entity, _ice_monster_logic: Entity) {
    if is_challenge_game_active(world, this, KILL_ICE_MONSTER_GAME) {
        return;
    }
    set_challenge_game_score(world, this, KILL_ICE_MONSTER_GAME, 0);
}

/// `-[PocketGodViewController (Game) onIceMonsterCancelBattle:]` @ 0x100169968
pub fn on_ice_monster_cancel_battle(world: &mut World, this: Entity, _ice_monster_logic: Entity) {
    forfeit_challenge_game(world, this, KILL_ICE_MONSTER_GAME);
}

/// `-[PocketGodViewController (Game) onAntHillDirect:]` @ 0x10016998c
pub fn on_ant_hill_direct(world: &mut World, this: Entity, ant_hill_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).directed_ant_hill_logic = Some(ant_hill_logic);
}

/// `-[PocketGodViewController (Game) onAntHillDirectComplete:]` @ 0x1001699a0
pub fn on_ant_hill_direct_complete(world: &mut World, this: Entity, _ant_hill_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).directed_ant_hill_logic = None;
}

/// `-[PocketGodViewController (Game) onMagnifyingGlassDrag:]` @ 0x1001699b4
pub fn on_magnifying_glass_drag(world: &mut World, this: Entity, magnifying_glass_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_magnifying_glass_logic = Some(magnifying_glass_logic);
}

/// `-[PocketGodViewController (Game) onMagnifyingGlassDragComplete:]` @ 0x1001699c8
pub fn on_magnifying_glass_drag_complete(world: &mut World, this: Entity, _magnifying_glass_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_magnifying_glass_logic = None;
}

/// `-[PocketGodViewController (Game) checkCountSharkSnaps]` @ 0x1001699dc
///
/// The shark-snaps game ends once no pygmy is held over the water.
pub fn check_count_shark_snaps(world: &mut World, this: Entity) {
    let dragged_pygmy_logic = world.obj::<ControllerPygmies>(this).dragged_pygmy_logic;
    if dragged_pygmy_logic.is_some() {
        return;
    }
    stop_challenge_game(world, this, SHARK_SNAPS_GAME);
}

/// `-[PocketGodViewController (Game) onSharkDrag:]` @ 0x100169a18
pub fn on_shark_drag(world: &mut World, this: Entity, shark_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_shark_logic = Some(shark_logic);
}

/// `-[PocketGodViewController (Game) onSharkDragComplete:]` @ 0x100169a2c
pub fn on_shark_drag_complete(world: &mut World, this: Entity, _shark_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_shark_logic = None;
}

/// `-[PocketGodViewController (Game) onSharkSnap:]` @ 0x100169a40
pub fn on_shark_snap(world: &mut World, this: Entity, _shark_logic: Entity) {
    check_count_shark_snaps(world, this);
    increment_challenge_game_score(world, this, SHARK_SNAPS_GAME);
}

/// `-[PocketGodViewController (Game) stopCountingBirdBombs]` @ 0x100169a8c
pub fn stop_counting_bird_bombs(world: &mut World, this: Entity) {
    stop_challenge_game(world, this, BIRD_BOMBS_GAME);
}

/// `-[PocketGodViewController (Game) onSnowBallHitPygmy:]` @ 0x100169ab0
pub fn on_snow_ball_hit_pygmy(_world: &mut World, _this: Entity, _snow_ball_logic: Entity) {}

/// `-[PocketGodViewController (Game) onSnowBallMissPygmy:]` @ 0x100169ab4
pub fn on_snow_ball_miss_pygmy(_world: &mut World, _this: Entity, _snow_ball_logic: Entity) {}

/// `-[PocketGodViewController (Game) onBirdPooHitPygmy:]` @ 0x100169ab8
pub fn on_bird_poo_hit_pygmy(world: &mut World, this: Entity, _bird_poo_logic: Entity) {
    increment_challenge_game_score(world, this, BIRD_BOMBS_GAME);
}

/// `-[PocketGodViewController (Game) onBirdPooMissPygmy:]` @ 0x100169adc
pub fn on_bird_poo_miss_pygmy(world: &mut World, this: Entity, _bird_poo_logic: Entity) {
    stop_counting_bird_bombs(world, this);
}

/// `-[PocketGodViewController (Game) onBirdFlyOffScreen:]` @ 0x100169aec
pub fn on_bird_fly_off_screen(world: &mut World, this: Entity, _bird_logic: Entity) {
    stop_counting_bird_bombs(world, this);
}

/// `-[PocketGodViewController (Game) onBirdDrag:]` @ 0x100169afc
pub fn on_bird_drag(world: &mut World, this: Entity, bird_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_bird_logic = Some(bird_logic);
}

/// `-[PocketGodViewController (Game) onBirdDragComplete:]` @ 0x100169b10
pub fn on_bird_drag_complete(world: &mut World, this: Entity, _bird_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_bird_logic = None;
}

/// `-[PocketGodViewController (Game) onClosetTempleClosetDoorDrag:]` @ 0x100169ba0
pub fn on_closet_temple_closet_door_drag(world: &mut World, this: Entity, closet_temple_closet_door_logic: Entity) {
    let mut events = world.obj_mut::<ControllerWorldObjectEvents>(this);
    events.dragged_closet_temple_closet_door_logic = Some(closet_temple_closet_door_logic);
}

/// `-[PocketGodViewController (Game) onClosetTempleClosetDoorDragComplete:]` @ 0x100169bb4
pub fn on_closet_temple_closet_door_drag_complete(world: &mut World, this: Entity, _closet_temple_closet_door_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_closet_temple_closet_door_logic = None;
}

/// `-[PocketGodViewController (Game) onThetanDrag:]` @ 0x100169bc8
pub fn on_thetan_drag(world: &mut World, this: Entity, thetan_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_thetan_logic = Some(thetan_logic);
}

/// `-[PocketGodViewController (Game) onThetanDragComplete:]` @ 0x100169bdc
pub fn on_thetan_drag_complete(world: &mut World, this: Entity, _thetan_logic: Entity) {
    world.obj_mut::<ControllerWorldObjectEvents>(this).dragged_thetan_logic = None;
}

/// `[mChallengeManager isGameActive:challenge_type]` (NO while there is no manager).
fn is_challenge_game_active(world: &mut World, this: Entity, challenge_type: i32) -> bool {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    match challenge_manager {
        Some(challenge_manager) => challenge_manager::is_game_active(world, challenge_manager, challenge_type),
        None => false,
    }
}

/// `[mChallengeManager incrementGameScore:challenge_type]`.
fn increment_challenge_game_score(world: &mut World, this: Entity, challenge_type: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::increment_game_score(world, challenge_manager, challenge_type);
    }
}

/// `[mChallengeManager setGameScore:challenge_type score:score]`.
fn set_challenge_game_score(world: &mut World, this: Entity, challenge_type: i32, score: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::set_game_score(world, challenge_manager, challenge_type, score);
    }
}

/// `[mChallengeManager stopGame:challenge_type]`.
fn stop_challenge_game(world: &mut World, this: Entity, challenge_type: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::stop_game(world, challenge_manager, challenge_type);
    }
}

/// `[mChallengeManager forfeitGame:challenge_type]`.
fn forfeit_challenge_game(world: &mut World, this: Entity, challenge_type: i32) {
    let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
    if let Some(challenge_manager) = challenge_manager {
        challenge_manager::forfeit_game(world, challenge_manager, challenge_type);
    }
}

/// `[mSkinPackManager refreshSkinPackOption:skin_pack_id skinObjectId:skin_object_id
/// skinOptionId:skin_option_id useFirstPurchased:YES]`.
fn refresh_skin_pack_option(world: &mut World, this: Entity, skin_pack_id: &str, skin_object_id: &str, skin_option_id: &str) {
    let skin_pack_manager = world.obj::<ControllerManagersBridge>(this).skin_pack_manager;
    let Some(skin_pack_manager) = skin_pack_manager else {
        return;
    };

    skin_pack_manager::refresh_skin_pack_option_skin_object_id_skin_option_id_use_first_purchased(
        world,
        skin_pack_manager,
        skin_pack_id,
        skin_object_id,
        skin_option_id,
        true,
    );
}
