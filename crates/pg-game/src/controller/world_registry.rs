//! `PocketGodViewController`: methods in this part of the class.

use crate::prelude::*;
use super::*;

use crate::logic::idol_ui_widgets::map_paper_logic::{MAP_PAPER_LOGIC, MapPaperLogic};
use crate::logic::underwater_stage::underwater_scenery_logic::UNDERWATER_SCENERY_LOGIC;

/// `PocketGodViewController` ivars owned by this unit.
#[derive(Component, Clone, Debug, Default)]
pub struct ControllerWorldRegistry {
    /// `mMapPaperLogic`: `@"MapPaperLogic"`
    pub map_paper_logic: Option<Entity>,

    /// `mSmallObjectDisplayMarker`: `@"BCDisplayMarker"`
    pub small_object_display_marker: Option<Entity>,

    /// `mLogicArray`: `@"NSMutableArray"`
    pub logic_array: Vec<Entity>,

    /// `mMainUiLogicArray`: `@"NSMutableArray"`
    pub main_ui_logic_array: Vec<Entity>,

    /// `mHelpUiLogicArray`: `@"NSMutableArray"`
    pub help_ui_logic_array: Vec<Entity>,

    /// `mWhatsNewScreenUiLogicArray`: `@"NSMutableArray"`
    pub whats_new_screen_ui_logic_array: Vec<Entity>,

    /// `mCoconutLogicArray`: `@"NSMutableArray"`
    pub coconut_logic_array: Vec<Entity>,

    /// `mWoodLogicArray`: `@"NSMutableArray"`
    pub wood_logic_array: Vec<Entity>,

    /// `mUnderwaterSceneryLogicArray`: `@"NSMutableArray"`
    pub underwater_scenery_logic_array: Vec<Entity>,

    /// `mSpearLogicArray`: `@"NSMutableArray"`
    pub spear_logic_array: Vec<Entity>,

    /// `mApeTorchLogicArray`: `@"NSMutableArray"`
    pub ape_torch_logic_array: Vec<Entity>,

    /// `mMonkeyLogicArray`: `@"NSMutableArray"`
    pub monkey_logic_array: Vec<Entity>,

    /// `mJellyfishLogicArray`: `@"NSMutableArray"`
    pub jellyfish_logic_array: Vec<Entity>,

    /// `mGodIdolLogicArray`: `@"NSMutableArray"`
    pub god_idol_logic_array: Vec<Entity>,

    /// `mApocalypseGemPedestalLogicArray`: `@"NSMutableArray"`
    pub apocalypse_gem_pedestal_logic_array: Vec<Entity>,

    /// `mTimeTempleDoorLogicArray`: `@"NSMutableArray"`
    pub time_temple_door_logic_array: Vec<Entity>,

    /// `mLocustTempleDoorLogicArray`: `@"NSMutableArray"`
    pub locust_temple_door_logic_array: Vec<Entity>,

    /// `mDanceTempleDoorLogicArray`: `@"NSMutableArray"`
    pub dance_temple_door_logic_array: Vec<Entity>,

    /// `mPlagueTempleDoorLogicArray`: `@"NSMutableArray"`
    pub plague_temple_door_logic_array: Vec<Entity>,

    /// `mPurificationTempleDoorLogicArray`: `@"NSMutableArray"`
    pub purification_temple_door_logic_array: Vec<Entity>,

    /// `mClosetTempleDoorLogicArray`: `@"NSMutableArray"`
    pub closet_temple_door_logic_array: Vec<Entity>,

    /// `mXenuTempleDoorLogicArray`: `@"NSMutableArray"`
    pub xenu_temple_door_logic_array: Vec<Entity>,

    /// `mIdolPedestalLogicArray`: `@"NSMutableArray"`
    pub idol_pedestal_logic_array: Vec<Entity>,

    /// `mWaterWingsLogic`: `@"WaterWingsLogic"`
    pub water_wings_logic: Option<Entity>,

    /// `mFishingPoleLogic`: `@"FishingPoleLogic"`
    pub fishing_pole_logic: Option<Entity>,

    /// `mTRexEggLogic`: `@"TRexEggLogic"`
    pub t_rex_egg_logic: Option<Entity>,

    /// `mTRexLogic`: `@"TRexLogic"`
    pub t_rex_logic: Option<Entity>,

    /// `mSpiderLogic`: `@"SpiderLogic"`
    pub spider_logic: Option<Entity>,

    /// `mSpiderTreeLogic`: `@"SpiderTreeLogic"`
    pub spider_tree_logic: Option<Entity>,

    /// `mFishLogic`: `@"FishLogic"`
    pub fish_logic: Option<Entity>,

    /// `mSwordFishLogic`: `@"SwordFishLogic"`
    pub sword_fish_logic: Option<Entity>,

    /// `mBoulderLogic`: `@"BoulderLogic"`
    pub boulder_logic: Option<Entity>,

    /// `mOutHouseLogic`: `@"OutHouseLogic"`
    pub out_house_logic: Option<Entity>,

    /// `mIceMonsterLogic`: `@"IceMonsterLogic"`
    pub ice_monster_logic: Option<Entity>,

    /// `mGlacierLogic`: `@"GlacierLogic"`
    pub glacier_logic: Option<Entity>,

    /// `mIglooLogic`: `@"IglooLogic"`
    pub igloo_logic: Option<Entity>,

    /// `mIceTreeLogic`: `@"IceTreeLogic"`
    pub ice_tree_logic: Option<Entity>,

    /// `mIceHoleLogic`: `@"IceHoleLogic"`
    pub ice_hole_logic: Option<Entity>,

    /// `mApeLogic`: `@"ApeLogic"`
    pub ape_logic: Option<Entity>,

    /// `mApeAltarLogic`: `@"ApeAltarLogic"`
    pub ape_altar_logic: Option<Entity>,

    /// `mApeBongoLogic`: `@"ApeBongoLogic"`
    pub ape_bongo_logic: Option<Entity>,

    /// `mApeWenchLogic`: `@"ApeWenchLogic"`
    pub ape_wench_logic: Option<Entity>,

    /// `mGoGoCageLogic`: `@"GoGoCageLogic"`
    pub go_go_cage_logic: Option<Entity>,

    /// `mDiscoBallLogic`: `@"DiscoBallLogic"`
    pub disco_ball_logic: Option<Entity>,

    /// `mDiscoLightsLogic`: `@"DiscoLightsLogic"`
    pub disco_lights_logic: Option<Entity>,

    /// `mDanceBalloonBundleLogic`: `@"DanceBalloonBundleLogic"`
    pub dance_balloon_bundle_logic: Option<Entity>,

    /// `mDanceBalloonLogic`: `@"DanceBalloonLogic"`
    pub dance_balloon_logic: Option<Entity>,

    /// `mDanceDanceExecutionLogic`: `@"DanceDanceExecutionLogic"`
    pub dance_dance_execution_logic: Option<Entity>,

    /// `mBananaTreeLogic`: `@"BananaTreeLogic"`
    pub banana_tree_logic: Option<Entity>,

    /// `mBananaLogic`: `@"BananaLogic"`
    pub banana_logic: Option<Entity>,

    /// `mMoronSnakeLogic`: `@"MoronSnakeLogic"`
    pub moron_snake_logic: Option<Entity>,

    /// `mNoPeeLogic`: `@"NoPeeLogic"`
    pub no_pee_logic: Option<Entity>,

    /// `mUnderwaterTempleLogic`: `@"UnderwaterTempleLogic"`
    pub underwater_temple_logic: Option<Entity>,

    /// `mTempleDoorLogic`: `@"TempleDoorLogic"`
    pub temple_door_logic: Option<Entity>,

    /// `mApocalypseTempleDoorLogic`: `@"ApocalypseTempleDoorLogic"`
    pub apocalypse_temple_door_logic: Option<Entity>,

    /// `mApocalypsePortalTempleDoorLogic`: `@"ApocalypseTempleDoorLogic"`
    pub apocalypse_portal_temple_door_logic: Option<Entity>,

    /// `mApocalypseCalendarLogic`: `@"ApocalypseCalendarLogic"`
    pub apocalypse_calendar_logic: Option<Entity>,

    /// `mApocalypseRoomFaderLogic`: `@"ApocalypseRoomFaderLogic"`
    pub apocalypse_room_fader_logic: Option<Entity>,

    /// `mApocalypseTimeTempleLeverLogic`: `@"ApocalypseTimeTempleLeverLogic"`
    pub apocalypse_time_temple_lever_logic: Option<Entity>,

    /// `mApocalypseTimeTemplePlatformLogic`: `@"ApocalypseTimeTemplePlatformLogic"`
    pub apocalypse_time_temple_platform_logic: Option<Entity>,

    /// `mLocustQueenLogic`: `@"LocustQueenLogic"`
    pub locust_queen_logic: Option<Entity>,

    /// `mApocalypsePlagueTempleFountainLogic`: `@"ApocalypseTempleFountainLogic"`
    pub apocalypse_plague_temple_fountain_logic: Option<Entity>,

    /// `mApocalypsePurificationTempleFountainLogic`: `@"ApocalypseTempleFountainLogic"`
    pub apocalypse_purification_temple_fountain_logic: Option<Entity>,

    /// `mClosetTempleThetanMeterLogic`: `@"ClosetTempleThetanMeterLogic"`
    pub closet_temple_thetan_meter_logic: Option<Entity>,

    /// `mClosetTempleCouchLogic`: `@"ClosetTempleCouchLogic"`
    pub closet_temple_couch_logic: Option<Entity>,

    /// `mClosetTempleClosetDoorLogic`: `@"ClosetTempleClosetDoorLogic"`
    pub closet_temple_closet_door_logic: Option<Entity>,

    /// `mXenuTempleLogic`: `@"XenuTempleLogic"`
    pub xenu_temple_logic: Option<Entity>,

    /// `mXenuLogic`: `@"XenuLogic"`
    pub xenu_logic: Option<Entity>,

    /// `mMisfitOceanRockLogic`: `@"IslandObjectLogic"`
    pub misfit_ocean_rock_logic: Option<Entity>,

    /// `mZombieGraveLogic`: `@"ZombieGraveLogic"`
    pub zombie_grave_logic: Option<Entity>,

    /// `mOctoStatueLogic`: `@"OctoStatueLogic"`
    pub octo_statue_logic: Option<Entity>,

    /// `mKongStatueLogic`: `@"KongStatueLogic"`
    pub kong_statue_logic: Option<Entity>,

    /// `mSpearStatueLogic`: `@"SpearStatueLogic"`
    pub spear_statue_logic: Option<Entity>,

    /// `mAnchorLogic`: `@"AnchorLogic"`
    pub anchor_logic: Option<Entity>,

    /// `mTileDrainLogic`: `@"TileDrainLogic"`
    pub tile_drain_logic: Option<Entity>,

    /// `mTilePlugLogic`: `@"TilePlugLogic"`
    pub tile_plug_logic: Option<Entity>,

    /// `mChumBucketLogic`: `@"ChumBucketLogic"`
    pub chum_bucket_logic: Option<Entity>,

    /// `mHealthBarLogic`: `@"HealthBarLogic"`
    pub health_bar_logic: Option<Entity>,

    /// `mAntHillLogic`: `@"AntHillLogic"`
    pub ant_hill_logic: Option<Entity>,

    /// `mBonFireLogic`: `@"BonFireLogic"`
    pub bon_fire_logic: Option<Entity>,

    /// `mMagnifyingGlassLogic`: `@"MagnifyingGlassLogic"`
    pub magnifying_glass_logic: Option<Entity>,

    /// `mSharkLogic`: `@"SharkLogic"`
    pub shark_logic: Option<Entity>,

    /// `mBirdLogic`: `@"BirdLogic"`
    pub bird_logic: Option<Entity>,

    /// `mCrackHoleLogic`: `@"CrackHoleLogic"`
    pub crack_hole_logic: Option<Entity>,
}

/// `mIslandLocationState` of the temples (`docs/re/controller.md` §4.1).
const TIME_TEMPLE: i32 = 11;
const LOCUST_TEMPLE: i32 = 12;
const DANCE_TEMPLE: i32 = 13;
const PLAGUE_TEMPLE: i32 = 14;
const PURIFICATION_TEMPLE: i32 = 15;
const CLOSET_TEMPLE: i32 = 16;
const XENU_TEMPLE: i32 = 17;

/// `-[PocketGodViewController rainbowLogic:]` @ 0x1000710e4
pub fn rainbow_logic(world: &mut World, this: Entity, _rainbow_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerGodPowers>(this).rainbow_logic
}

/// `-[PocketGodViewController locustLogic:]` @ 0x1000710f4
pub fn locust_logic(world: &mut World, this: Entity, _locust_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerGodPowers>(this).locust_logic
}

/// `-[PocketGodViewController starsLogic:]` @ 0x100071104
pub fn stars_logic(world: &mut World, this: Entity, _stars_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerSkyWeather>(this).stars_logic
}

/// `-[PocketGodViewController tRexEggLogic:]` @ 0x100071114
pub fn t_rex_egg_logic(world: &mut World, this: Entity, _t_rex_egg_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).t_rex_egg_logic
}

/// `-[PocketGodViewController tRexLogic:]` @ 0x100071124
pub fn t_rex_logic(world: &mut World, this: Entity, _t_rex_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).t_rex_logic
}

/// `-[PocketGodViewController boulderLogic:]` @ 0x100071134
pub fn boulder_logic(world: &mut World, this: Entity, _boulder_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).boulder_logic
}

/// `-[PocketGodViewController spiderLogic:]` @ 0x100071144
pub fn spider_logic(world: &mut World, this: Entity, _spider_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).spider_logic
}

/// `-[PocketGodViewController spiderTreeLogic:]` @ 0x100071154
pub fn spider_tree_logic(world: &mut World, this: Entity, _spider_tree_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).spider_tree_logic
}

/// `-[PocketGodViewController fishLogic:]` @ 0x100071164
pub fn fish_logic(world: &mut World, this: Entity, _fish_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).fish_logic
}

/// `-[PocketGodViewController swordFishLogic:]` @ 0x100071174
pub fn sword_fish_logic(world: &mut World, this: Entity, _sword_fish_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).sword_fish_logic
}

/// `-[PocketGodViewController fishingBobLogic:]` @ 0x100071184
pub fn fishing_bob_logic(world: &mut World, this: Entity, _fishing_bob_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldObjectEvents>(this).fishing_bob_logic
}

/// `-[PocketGodViewController fishingPoleLogic:]` @ 0x100071194
pub fn fishing_pole_logic(world: &mut World, this: Entity, _fishing_pole_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).fishing_pole_logic
}

/// `-[PocketGodViewController waterWingsLogic:]` @ 0x1000711a4
pub fn water_wings_logic(world: &mut World, this: Entity, _water_wings_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).water_wings_logic
}

/// `-[PocketGodViewController sharkLogic:]` @ 0x1000711b4
pub fn shark_logic(world: &mut World, this: Entity, _shark_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).shark_logic
}

/// `-[PocketGodViewController birdLogic:]` @ 0x1000711c4
pub fn bird_logic(world: &mut World, this: Entity, _bird_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).bird_logic
}

/// `-[PocketGodViewController outHouseLogic:]` @ 0x1000711d4
pub fn out_house_logic(world: &mut World, this: Entity, _out_house_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).out_house_logic
}

/// `-[PocketGodViewController iceMonsterLogic:]` @ 0x1000711e4
pub fn ice_monster_logic(world: &mut World, this: Entity, _ice_monster_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ice_monster_logic
}

/// `-[PocketGodViewController glacierLogic:]` @ 0x1000711f4
pub fn glacier_logic(world: &mut World, this: Entity, _glacier_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).glacier_logic
}

/// `-[PocketGodViewController iglooLogic:]` @ 0x100071204
pub fn igloo_logic(world: &mut World, this: Entity, _igloo_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).igloo_logic
}

/// `-[PocketGodViewController iceTreeLogic:]` @ 0x100071214
pub fn ice_tree_logic(world: &mut World, this: Entity, _ice_tree_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ice_tree_logic
}

/// `-[PocketGodViewController iceHoleLogic:]` @ 0x100071224
pub fn ice_hole_logic(world: &mut World, this: Entity, _ice_hole_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ice_hole_logic
}

/// `-[PocketGodViewController zombieGraveLogic:]` @ 0x100071234
pub fn zombie_grave_logic(world: &mut World, this: Entity, _zombie_grave_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).zombie_grave_logic
}

/// `-[PocketGodViewController apeLogic:]` @ 0x100071244
pub fn ape_logic(world: &mut World, this: Entity, _ape_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ape_logic
}

/// `-[PocketGodViewController apeAltarLogic:]` @ 0x100071254
pub fn ape_altar_logic(world: &mut World, this: Entity, _ape_altar_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ape_altar_logic
}

/// `-[PocketGodViewController apeBongoLogic:]` @ 0x100071264
pub fn ape_bongo_logic(world: &mut World, this: Entity, _ape_bongo_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ape_bongo_logic
}

/// `-[PocketGodViewController apeWenchLogic:]` @ 0x100071274
pub fn ape_wench_logic(world: &mut World, this: Entity, _ape_wench_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ape_wench_logic
}

/// `-[PocketGodViewController apeTorchLogicArray:]` @ 0x100071284
pub fn ape_torch_logic_array(world: &mut World, this: Entity, _ape_torch_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ape_torch_logic_array.clone()
}

/// `-[PocketGodViewController monkeyLogicArray:]` @ 0x100071294
pub fn monkey_logic_array(world: &mut World, this: Entity, _monkey_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).monkey_logic_array.clone()
}

/// `-[PocketGodViewController jellyfishLogicArray:]` @ 0x1000712a4
pub fn jellyfish_logic_array(world: &mut World, this: Entity, _jellyfish_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).jellyfish_logic_array.clone()
}

/// `-[PocketGodViewController idolPedestalLogicArray:]` @ 0x1000712b4
pub fn idol_pedestal_logic_array(world: &mut World, this: Entity, _idol_pedestal_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).idol_pedestal_logic_array.clone()
}

/// `-[PocketGodViewController apocalypseGemPedestalLogicArray:]` @ 0x1000712c4
pub fn apocalypse_gem_pedestal_logic_array(world: &mut World, this: Entity, _apocalypse_gem_pedestal_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).apocalypse_gem_pedestal_logic_array.clone()
}

/// `-[PocketGodViewController apocalypseGemLogicArray:]` @ 0x1000712d4
pub fn apocalypse_gem_logic_array(world: &mut World, this: Entity, _apocalypse_gem_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerApocalypse>(this).apocalypse_gem_logic_array.clone()
}

/// `-[PocketGodViewController timeTempleDoorLogicArray:]` @ 0x1000712e4
pub fn time_temple_door_logic_array(world: &mut World, this: Entity, _time_temple_door_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).time_temple_door_logic_array.clone()
}

/// `-[PocketGodViewController locustTempleDoorLogicArray:]` @ 0x1000712f4
pub fn locust_temple_door_logic_array(world: &mut World, this: Entity, _locust_temple_door_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).locust_temple_door_logic_array.clone()
}

/// `-[PocketGodViewController danceTempleDoorLogicArray:]` @ 0x100071304
pub fn dance_temple_door_logic_array(world: &mut World, this: Entity, _dance_temple_door_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).dance_temple_door_logic_array.clone()
}

/// `-[PocketGodViewController plagueTempleDoorLogicArray:]` @ 0x100071314
pub fn plague_temple_door_logic_array(world: &mut World, this: Entity, _plague_temple_door_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).plague_temple_door_logic_array.clone()
}

/// `-[PocketGodViewController purificationTempleDoorLogicArray:]` @ 0x100071324
pub fn purification_temple_door_logic_array(world: &mut World, this: Entity, _purification_temple_door_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).purification_temple_door_logic_array.clone()
}

/// `-[PocketGodViewController closetTempleDoorLogicArray:]` @ 0x100071334
pub fn closet_temple_door_logic_array(world: &mut World, this: Entity, _closet_temple_door_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).closet_temple_door_logic_array.clone()
}

/// `-[PocketGodViewController xenuTempleDoorLogicArray:]` @ 0x100071344
pub fn xenu_temple_door_logic_array(world: &mut World, this: Entity, _xenu_temple_door_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).xenu_temple_door_logic_array.clone()
}

/// `-[PocketGodViewController bananaTreeLogic:]` @ 0x100071354
pub fn banana_tree_logic(world: &mut World, this: Entity, _banana_tree_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).banana_tree_logic
}

/// `-[PocketGodViewController bananaLogic:]` @ 0x100071364
pub fn banana_logic(world: &mut World, this: Entity, _banana_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).banana_logic
}

/// `-[PocketGodViewController moronSnakeLogic:]` @ 0x100071374
pub fn moron_snake_logic(world: &mut World, this: Entity, _moron_snake_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).moron_snake_logic
}

/// `-[PocketGodViewController noPeeLogic:]` @ 0x100071384
pub fn no_pee_logic(world: &mut World, this: Entity, _no_pee_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).no_pee_logic
}

/// `-[PocketGodViewController templeDoorLogic:]` @ 0x100071394
pub fn temple_door_logic(world: &mut World, this: Entity, _temple_door_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).temple_door_logic
}

/// `-[PocketGodViewController godIdolLogicArray]` @ 0x1000713a4
pub fn god_idol_logic_array(world: &mut World, this: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).god_idol_logic_array.clone()
}

/// `-[PocketGodViewController apocalypseTempleDoorLogic:]` @ 0x1000713b4
pub fn apocalypse_temple_door_logic(world: &mut World, this: Entity, _apocalypse_temple_door_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).apocalypse_temple_door_logic
}

/// `-[PocketGodViewController apocalypsePortalTempleDoorLogic:]` @ 0x1000713c4
pub fn apocalypse_portal_temple_door_logic(world: &mut World, this: Entity, _apocalypse_portal_temple_door_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).apocalypse_portal_temple_door_logic
}

/// `-[PocketGodViewController apocalypseTimeTempleLeverLogic:]` @ 0x1000713d4
pub fn apocalypse_time_temple_lever_logic(world: &mut World, this: Entity, _apocalypse_time_temple_lever_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).apocalypse_time_temple_lever_logic
}

/// `-[PocketGodViewController apocalypseTimeTemplePlatformLogic:]` @ 0x1000713e4
pub fn apocalypse_time_temple_platform_logic(world: &mut World, this: Entity, _apocalypse_time_temple_platform_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).apocalypse_time_temple_platform_logic
}

/// `-[PocketGodViewController locustQueenLogic:]` @ 0x1000713f4
pub fn locust_queen_logic(world: &mut World, this: Entity, _locust_queen_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).locust_queen_logic
}

/// `-[PocketGodViewController goGoCageLogic:]` @ 0x100071404
pub fn go_go_cage_logic(world: &mut World, this: Entity, _go_go_cage_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).go_go_cage_logic
}

/// `-[PocketGodViewController discoBallLogic:]` @ 0x100071414
pub fn disco_ball_logic(world: &mut World, this: Entity, _disco_ball_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).disco_ball_logic
}

/// `-[PocketGodViewController discoLightsLogic:]` @ 0x100071424
pub fn disco_lights_logic(world: &mut World, this: Entity, _disco_lights_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).disco_lights_logic
}

/// `-[PocketGodViewController danceBalloonBundleLogic:]` @ 0x100071434
pub fn dance_balloon_bundle_logic(world: &mut World, this: Entity, _dance_balloon_bundle_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).dance_balloon_bundle_logic
}

/// `-[PocketGodViewController danceBalloonLogic:]` @ 0x100071444
pub fn dance_balloon_logic(world: &mut World, this: Entity, _dance_balloon_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).dance_balloon_logic
}

/// `-[PocketGodViewController danceDanceExecutionLogic:]` @ 0x100071454
pub fn dance_dance_execution_logic(world: &mut World, this: Entity, _dance_dance_execution_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).dance_dance_execution_logic
}

/// `-[PocketGodViewController apocalypsePlagueTempleFountainLogic:]` @ 0x100071464
pub fn apocalypse_plague_temple_fountain_logic(world: &mut World, this: Entity, _apocalypse_plague_temple_fountain_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).apocalypse_plague_temple_fountain_logic
}

/// `-[PocketGodViewController apocalypsePurificationTempleFountainLogic:]` @ 0x100071474
pub fn apocalypse_purification_temple_fountain_logic(world: &mut World, this: Entity, _apocalypse_purification_temple_fountain_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).apocalypse_purification_temple_fountain_logic
}

/// `-[PocketGodViewController closetTempleThetanMeterLogic:]` @ 0x100071484
pub fn closet_temple_thetan_meter_logic(world: &mut World, this: Entity, _closet_temple_thetan_meter_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).closet_temple_thetan_meter_logic
}

/// `-[PocketGodViewController closetTempleCouchLogic:]` @ 0x100071494
pub fn closet_temple_couch_logic(world: &mut World, this: Entity, _closet_temple_couch_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).closet_temple_couch_logic
}

/// `-[PocketGodViewController closetTempleClosetDoorLogic:]` @ 0x1000714a4
pub fn closet_temple_closet_door_logic(world: &mut World, this: Entity, _closet_temple_closet_door_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).closet_temple_closet_door_logic
}

/// `-[PocketGodViewController xenuTempleLogic:]` @ 0x1000714b4
pub fn xenu_temple_logic(world: &mut World, this: Entity, _xenu_temple_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).xenu_temple_logic
}

/// `-[PocketGodViewController xenuLogic:]` @ 0x1000714c4
pub fn xenu_logic(world: &mut World, this: Entity, _xenu_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).xenu_logic
}

/// `-[PocketGodViewController crackHoleLogic:]` @ 0x1000714d4
pub fn crack_hole_logic(world: &mut World, this: Entity, _crack_hole_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).crack_hole_logic
}

/// `-[PocketGodViewController octoStatueLogic:]` @ 0x1000714e4
pub fn octo_statue_logic(world: &mut World, this: Entity, _octo_statue_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).octo_statue_logic
}

/// `-[PocketGodViewController kongStatueLogic:]` @ 0x1000714f4
pub fn kong_statue_logic(world: &mut World, this: Entity, _kong_statue_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).kong_statue_logic
}

/// `-[PocketGodViewController spearStatueLogic:]` @ 0x100071504
pub fn spear_statue_logic(world: &mut World, this: Entity, _spear_statue_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).spear_statue_logic
}

/// `-[PocketGodViewController anchorLogic:]` @ 0x100071514
pub fn anchor_logic(world: &mut World, this: Entity, _anchor_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).anchor_logic
}

/// `-[PocketGodViewController tileDrainLogic:]` @ 0x100071524
pub fn tile_drain_logic(world: &mut World, this: Entity, _tile_drain_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).tile_drain_logic
}

/// `-[PocketGodViewController tilePlugLogic:]` @ 0x100071534
pub fn tile_plug_logic(world: &mut World, this: Entity, _tile_plug_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).tile_plug_logic
}

/// `-[PocketGodViewController chumBucketLogic:]` @ 0x100071544
pub fn chum_bucket_logic(world: &mut World, this: Entity, _chum_bucket_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).chum_bucket_logic
}

/// `-[PocketGodViewController healthBarLogic:]` @ 0x100071554
pub fn health_bar_logic(world: &mut World, this: Entity, _health_bar_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).health_bar_logic
}

/// `-[PocketGodViewController antHillLogic:]` @ 0x100071564
pub fn ant_hill_logic(world: &mut World, this: Entity, _ant_hill_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).ant_hill_logic
}

/// `-[PocketGodViewController bonFireLogic:]` @ 0x100071574
pub fn bon_fire_logic(world: &mut World, this: Entity, _bon_fire_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).bon_fire_logic
}

/// `-[PocketGodViewController coconutLogicArray:]` @ 0x100071584
pub fn coconut_logic_array(world: &mut World, this: Entity, _coconut_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).coconut_logic_array.clone()
}

/// `-[PocketGodViewController woodLogicArray:]` @ 0x100071594
pub fn wood_logic_array(world: &mut World, this: Entity, _wood_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).wood_logic_array.clone()
}

/// `-[PocketGodViewController spearLogicArray:]` @ 0x1000715a4
pub fn spear_logic_array(world: &mut World, this: Entity, _spear_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).spear_logic_array.clone()
}

/// `-[PocketGodViewController underwaterSceneryLogicArray:]` @ 0x1000715b4
pub fn underwater_scenery_logic_array(world: &mut World, this: Entity, _underwater_scenery_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldRegistry>(this).underwater_scenery_logic_array.clone()
}

/// `-[PocketGodViewController piranhaLogicArray:]` @ 0x1000715c4
pub fn piranha_logic_array(world: &mut World, this: Entity, _piranha_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerWorldObjectEvents>(this).piranha_logic_array.clone()
}

/// `-[PocketGodViewController pygmyLogicArray:]` @ 0x1000715d4
pub fn pygmy_logic_array(world: &mut World, this: Entity, _pygmy_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone()
}

/// `-[PocketGodViewController draggedPygmyLogic:]` @ 0x1000715f4
pub fn dragged_pygmy_logic(world: &mut World, this: Entity, _dragged_pygmy_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerPygmies>(this).dragged_pygmy_logic
}

/// `-[PocketGodViewController cloudLogicArray:]` @ 0x100071604
pub fn cloud_logic_array(world: &mut World, this: Entity, _cloud_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerSkyWeather>(this).cloud_logic_array.clone()
}

/// `-[PocketGodViewController doodleCloudLogicArray:]` @ 0x100071614
pub fn doodle_cloud_logic_array(world: &mut World, this: Entity, _doodle_cloud_logic_array: Entity) -> Vec<Entity> {
    world.obj::<ControllerMinigames>(this).doodle_cloud_logic_array.clone()
}

/// `-[PocketGodViewController spriteLogicArray:]` @ 0x100071624
pub fn sprite_logic_array(world: &mut World, this: Entity, _sprite_logic_array: Entity) -> Vec<Entity> {
    let bat_logic = world.obj::<ControllerSkyWeather>(this).bat_logic;
    let pygmy_logic_array = world.obj::<ControllerPygmies>(this).pygmy_logic_array.clone();
    let registry = world.obj::<ControllerWorldRegistry>(this);

    let mut logics = Vec::with_capacity(5);
    if let Some(bat_logic) = bat_logic {
        logics.push(bat_logic);
    }

    // `addObject:` would raise on a nil object; all of these exist once the game has loaded.
    let objects = [
        registry.anchor_logic,
        registry.shark_logic,
        registry.bird_logic,
        registry.boulder_logic,
        registry.t_rex_egg_logic,
        registry.t_rex_logic,
        registry.spider_logic,
        registry.out_house_logic,
        registry.ant_hill_logic,
        registry.bon_fire_logic,
        registry.magnifying_glass_logic,
        registry.fish_logic,
        registry.sword_fish_logic,
        registry.fishing_pole_logic,
        registry.banana_logic,
        registry.moron_snake_logic,
    ];
    logics.extend(objects.into_iter().flatten());

    logics.extend_from_slice(&pygmy_logic_array);
    logics.extend_from_slice(&registry.coconut_logic_array);
    logics.extend_from_slice(&registry.wood_logic_array);
    logics.extend_from_slice(&registry.spear_logic_array);
    logics
}

/// `-[PocketGodViewController mapPaperLogic]` @ 0x100078fc8
pub fn map_paper_logic(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<ControllerWorldRegistry>(this).map_paper_logic
}

/// `-[PocketGodViewController (Game) locustDisplayMarker]` @ 0x10015ad2c
pub fn locust_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let marker_id = match island_location_state {
        TIME_TEMPLE => "TimeTempleDragDisplayMarker",
        LOCUST_TEMPLE => "LocustTempleDragDisplayMarker",
        DANCE_TEMPLE => "DanceTempleDragDisplayMarker",
        PLAGUE_TEMPLE => "PlagueTempleDragDisplayMarker",
        PURIFICATION_TEMPLE => "PurificationTempleDragDisplayMarker",
        CLOSET_TEMPLE => "ClosetTempleDragDisplayMarker",
        XENU_TEMPLE => "XenuTempleDragDisplayMarker",
        _ => "DragDisplayMarker",
    };
    pg_engine::library::display_marker(world, marker_id)
}

/// `-[PocketGodViewController (Game) pygmyDragDisplayMarker]` @ 0x10015aeec
pub fn pygmy_drag_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let marker_id = match island_location_state {
        TIME_TEMPLE => "TimeTempleDragDisplayMarker",
        LOCUST_TEMPLE => "LocustTempleDragDisplayMarker",
        DANCE_TEMPLE => "DanceTempleDragDisplayMarker",
        PLAGUE_TEMPLE => "PlagueTempleDragDisplayMarker",
        PURIFICATION_TEMPLE => "PurificationTempleDragDisplayMarker",
        CLOSET_TEMPLE => "ClosetTempleDragDisplayMarker",
        XENU_TEMPLE => "XenuTempleDragDisplayMarker",
        _ => "DragDisplayMarker",
    };
    pg_engine::library::display_marker(world, marker_id)
}

/// `-[PocketGodViewController (Game) pygmyDisplayMarker]` @ 0x10015b0ac
pub fn pygmy_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let marker_id = match island_location_state {
        TIME_TEMPLE => "TimeTemplePygmyDisplayMarker",
        LOCUST_TEMPLE => "LocustTemplePygmyDisplayMarker",
        DANCE_TEMPLE => "DanceTemplePygmyDisplayMarker",
        PLAGUE_TEMPLE => "PlagueTemplePygmyDisplayMarker",
        PURIFICATION_TEMPLE => "PurificationTemplePygmyDisplayMarker",
        CLOSET_TEMPLE => "ClosetTemplePygmyDisplayMarker",
        XENU_TEMPLE => "XenuTemplePygmyDisplayMarker",
        _ => "PygmyDisplayMarker",
    };
    pg_engine::library::display_marker(world, marker_id)
}

/// `-[PocketGodViewController (Game) pygmyIslandDisplayMarker]` @ 0x10015b26c
pub fn pygmy_island_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let marker_id = match island_location_state {
        TIME_TEMPLE => "TimeTempleFlickedDisplayMarker",
        LOCUST_TEMPLE => "LocustTempleFlickedDisplayMarker",
        DANCE_TEMPLE => "DanceTempleFlickedDisplayMarker",
        PLAGUE_TEMPLE => "PlagueTempleFlickedDisplayMarker",
        PURIFICATION_TEMPLE => "PurificationTempleFlickedDisplayMarker",
        CLOSET_TEMPLE => "ClosetTempleFlickedDisplayMarker",
        XENU_TEMPLE => "XenuTempleFlickedDisplayMarker",
        _ => "IslandDisplayMarker",
    };
    pg_engine::library::display_marker(world, marker_id)
}

/// `-[PocketGodViewController (Game) pygmySmallObjectsDisplayMarker]` @ 0x10015b42c
pub fn pygmy_small_objects_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let marker_id = match island_location_state {
        TIME_TEMPLE => "TimeTemplePygmyDisplayMarker",
        LOCUST_TEMPLE => "LocustTemplePygmyDisplayMarker",
        DANCE_TEMPLE => "DanceTemplePygmyDisplayMarker",
        PLAGUE_TEMPLE => "PlagueTemplePygmyDisplayMarker",
        PURIFICATION_TEMPLE => "PurificationTemplePygmyDisplayMarker",
        CLOSET_TEMPLE => "ClosetTemplePygmyDisplayMarker",
        XENU_TEMPLE => "XenuTemplePygmyDisplayMarker",
        _ => "SmallObjectsMarker",
    };
    pg_engine::library::display_marker(world, marker_id)
}

/// `-[PocketGodViewController (Game) pygmyUnderwaterPygmyDisplayMarker]` @ 0x10015b5ec
pub fn pygmy_underwater_pygmy_display_marker(world: &mut World, _this: Entity) -> Option<Entity> {
    pg_engine::library::display_marker(world, "UnderwaterPygmyDisplayMarker")
}

/// `-[PocketGodViewController (Game) pygmyIslandBackDisplayMarker]` @ 0x10015b628
pub fn pygmy_island_back_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let marker_id = match island_location_state {
        TIME_TEMPLE => "TimeTempleFlickedDisplayMarker",
        LOCUST_TEMPLE => "LocustTempleFlickedDisplayMarker",
        DANCE_TEMPLE => "DanceTempleFlickedDisplayMarker",
        PLAGUE_TEMPLE => "PlagueTempleFlickedDisplayMarker",
        PURIFICATION_TEMPLE => "PurificationTempleFlickedDisplayMarker",
        CLOSET_TEMPLE => "ClosetTempleFlickedDisplayMarker",
        XENU_TEMPLE => "XenuTempleFlickedDisplayMarker",
        _ => "IslandBackDisplayMarker",
    };
    pg_engine::library::display_marker(world, marker_id)
}

/// `-[PocketGodViewController (Game) pygmyCloudBehindDisplayMarker]` @ 0x10015b7e8
pub fn pygmy_cloud_behind_display_marker(world: &mut World, this: Entity) -> Option<Entity> {
    let island_location_state = world.obj::<ControllerIslandLocations>(this).island_location_state;
    let marker_id = match island_location_state {
        TIME_TEMPLE => "TimeTempleFlickedDisplayMarker",
        LOCUST_TEMPLE => "LocustTempleFlickedDisplayMarker",
        DANCE_TEMPLE => "DanceTempleFlickedDisplayMarker",
        PLAGUE_TEMPLE => "PlagueTempleFlickedDisplayMarker",
        PURIFICATION_TEMPLE => "PurificationTempleFlickedDisplayMarker",
        CLOSET_TEMPLE => "ClosetTempleFlickedDisplayMarker",
        XENU_TEMPLE => "XenuTempleFlickedDisplayMarker",
        _ => "CloudBehindDisplayMarker",
    };
    pg_engine::library::display_marker(world, marker_id)
}

/// `-[PocketGodViewController (Game) tomPygmyLogic:]` @ 0x10015b9c4
pub fn tom_pygmy_logic(world: &mut World, this: Entity, _tom_pygmy_logic: Entity) -> Option<Entity> {
    world.obj::<ControllerPygmies>(this).tom_pygmy_logic
}

/// The UI arrays the buttons of `setLogic:::` join.
#[derive(Clone, Copy)]
enum UiArray {
    /// `mMainUiLogicArray`
    Main,

    /// `mWhatsNewScreenUiLogicArray`
    WhatsNewScreen,

    /// `mStatsTribeUiLogicArray`
    StatsTribe,

    /// `mStatsPygmyUiLogicArray`
    StatsPygmy,

    /// `mStatsGodUiLogicArray`
    StatsGod,

    /// `mHelpUiLogicArray`
    Help,
}

/// The tail every logic made by `setLogic:::` reaches except a coconut (`LAB_10015c3bc`):
/// `[mLogicArray addObject:logic]`.
fn add_to_logic_array(world: &mut World, this: Entity, logic: Entity) {
    let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
    registry.logic_array.push(logic);
}

/// The button tail of `setLogic:::` (`LAB_10015c378` and its siblings): the view controller
/// listens to the button, which joins `ui_array`, then `mLogicArray`.
fn add_button_logic(world: &mut World, this: Entity, button: Entity, ui_array: UiArray) {
    pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));

    match ui_array {
        UiArray::Main => {
            let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
            registry.main_ui_logic_array.push(button);
        }
        UiArray::WhatsNewScreen => {
            let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
            registry.whats_new_screen_ui_logic_array.push(button);
        }
        UiArray::StatsTribe => {
            let mut stats = world.obj_mut::<ControllerStatsAndAchievements>(this);
            stats.stats_tribe_ui_logic_array.push(button);
        }
        UiArray::StatsPygmy => {
            let mut stats = world.obj_mut::<ControllerStatsAndAchievements>(this);
            stats.stats_pygmy_ui_logic_array.push(button);
        }
        UiArray::StatsGod => {
            let mut stats = world.obj_mut::<ControllerStatsAndAchievements>(this);
            stats.stats_god_ui_logic_array.push(button);
        }
        UiArray::Help => {
            let mut registry = world.obj_mut::<ControllerWorldRegistry>(this);
            registry.help_ui_logic_array.push(button);
        }
    }

    add_to_logic_array(world, this, button);
}

/// `[NSString stringWithFormat:format, value]` for the item's `%d` name patterns
/// (`Pygmy%dStatsButton`).
fn string_with_int_format(format: &str, value: i32) -> String {
    let number = value.to_string();
    format.replacen("%d", &number, 1)
}

/// `[[UnderwaterSceneryLogic alloc] initWithDisplayObject:underwaterSceneryType:]`.
fn spawn_underwater_scenery_logic(world: &mut World, display_object: Entity, underwater_scenery_type: i32) -> Entity {
    let scenery = world.spawn(Isa(&UNDERWATER_SCENERY_LOGIC)).id();
    crate::logic::underwater_stage::underwater_scenery_logic::init_with_display_object(world, scenery, display_object, underwater_scenery_type);
    scenery
}

/// `[[MapPaperLogic alloc] initWithDisplayObject:]`: `MapPaperLogic` inherits `BCLogic`'s
/// initialiser, which does not run `-[MapPaperLogic init]`.
fn spawn_map_paper_logic(world: &mut World, display_object: Entity) -> Entity {
    let map_paper = world.spawn(Isa(&MAP_PAPER_LOGIC)).id();
    world.entity_mut(map_paper).insert(MapPaperLogic::default());
    pg_engine::logic::init_with_display_object(world, map_paper, display_object);
    map_paper
}

/// `-[PocketGodViewController (Game) setLogic:::]` @ 0x10015c11c
pub fn seq_set_logic(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    // `setLogic:` receives the whole item (`BCSequenceItemDef *`).
    match call.param("manager") {
        Some("ChallengeManager") => {
            let challenge_manager = world.obj::<ControllerManagersBridge>(this).challenge_manager;
            if let Some(challenge_manager) = challenge_manager {
                crate::managers::challenge_manager::set_logic(world, challenge_manager, call);
            }
            return ItemResult::Done;
        }
        Some("SkinPackManager") => {
            let skin_pack_manager = world.obj::<ControllerManagersBridge>(this).skin_pack_manager;
            if let Some(skin_pack_manager) = skin_pack_manager {
                crate::managers::skin_pack_manager::set_logic(world, skin_pack_manager, call);
            }
            return ItemResult::Done;
        }
        Some("BonusPackManager") => {
            let bonus_pack_manager = world.obj::<ControllerManagersBridge>(this).bonus_pack_manager;
            if let Some(bonus_pack_manager) = bonus_pack_manager {
                crate::managers::bonus_pack_manager::set_logic(world, bonus_pack_manager, call);
            }
            return ItemResult::Done;
        }
        Some("StoryManager") => {
            let story_manager = world.obj::<ControllerManagersBridge>(this).story_manager;
            if let Some(story_manager) = story_manager {
                crate::managers::story_manager::set_logic(world, story_manager, call);
            }
            return ItemResult::Done;
        }
        Some("NewsManager") => {
            let news_manager = world.obj::<ControllerManagersBridge>(this).news_manager;
            if let Some(news_manager) = news_manager {
                crate::managers::news_manager::set_logic(world, news_manager, call);
            }
            return ItemResult::Done;
        }
        Some("ChumBucket") => {
            let chum_bucket = world.obj::<ControllerWorldRegistry>(this).chum_bucket_logic;
            if let Some(chum_bucket) = chum_bucket {
                crate::logic::underwater_baitmaster_laser::chum_bucket_logic::set_logic(world, chum_bucket, call);
            }
            return ItemResult::Done;
        }
        _ => {}
    }

    let logic_id = call.param("logicId");
    let display_object_id = call.param("displayObjectId");
    let display_group_id = call.param("displayGroupId");

    let display_group = match display_group_id {
        Some(display_group_id) => pg_engine::library::display_group(world, display_group_id),
        None => None,
    };
    let display_object = match (display_group, display_object_id) {
        (Some(group), Some(object_id)) => pg_engine::display::group::get_display_object(world, group, object_id),
        _ => None,
    };

    // An unknown or missing id creates nothing.
    let Some(logic_id) = logic_id else {
        return ItemResult::Done;
    };

    // Checked before the other ids, which are all distinct: this id names no object itself but
    // `count` objects of its group.
    if logic_id == "PygmyStatsButtonLogic" {
        let count = call.int("count");
        if count < 1 {
            return ItemResult::Done;
        }

        let name_format = display_object_id.unwrap_or_default();
        for index in 0..count {
            let button_id = string_with_int_format(name_format, index);
            let button_object = match display_group {
                Some(group) => pg_engine::display::group::get_display_object(world, group, &button_id),
                None => None,
            };

            // PORT: the original also makes a button on a nil object; every button exists.
            let Some(button_object) = button_object else {
                continue;
            };

            let button = pg_engine::logic::button::spawn(world, button_object);
            pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));

            let mut stats = world.obj_mut::<ControllerStatsAndAchievements>(this);
            stats.stats_tribe_ui_logic_array.push(button);
            stats.pygmy_stats_button_logic_array.push(button);

            add_to_logic_array(world, this, button);
        }
        return ItemResult::Done;
    }

    // PORT: the original also creates the logic when the object is missing from its group (on a
    // nil display object); the engine's logics need one, so nothing is created. Every
    // `setLogic` item of the bundled scripts names an existing object.
    let Some(display_object) = display_object else {
        return ItemResult::Done;
    };

    match logic_id {
        "NewPygmyButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerMenusAndScreens>(this).new_pygmy_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::Main);
        }
        "MenuButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerMenusAndScreens>(this).menu_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::Main);
        }
        "WhatsNewScreenGoButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerMenusAndScreens>(this).whats_new_screen_go_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::WhatsNewScreen);
        }
        "StatsTribeCloseButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_tribe_close_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsTribe);
        }
        "StatsTribeEditNameButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_tribe_edit_name_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsTribe);
        }
        "StatsTribeCancelEditNameButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_tribe_cancel_edit_name_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsTribe);
        }
        "GodStatsButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).god_stats_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsTribe);
        }
        "StatsPygmyCloseButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_pygmy_close_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsPygmy);
        }
        "StatsPygmyNextButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_pygmy_next_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsPygmy);
        }
        "StatsPygmyEditNameButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_pygmy_edit_name_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsPygmy);
        }
        "StatsPygmyEditDescriptionButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_pygmy_edit_description_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsPygmy);
        }
        "StatsPygmyCancelEditNameButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_pygmy_cancel_edit_name_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsPygmy);
        }
        "StatsPygmyCancelEditDescriptionButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_pygmy_cancel_edit_description_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsPygmy);
        }
        "StatsGodCloseButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_god_close_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsGod);
        }
        "StatsGodNextGodTypeButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_god_next_god_type_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsGod);
        }
        "StatsGodEditNameButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_god_edit_name_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsGod);
        }
        "StatsGodEditDescriptionButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_god_edit_description_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsGod);
        }
        "StatsGodCancelEditNameButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_god_cancel_edit_name_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsGod);
        }
        "StatsGodCancelEditDescriptionButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerStatsAndAchievements>(this).stats_god_cancel_edit_description_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::StatsGod);
        }
        "HelpCloseButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerMenusAndScreens>(this).help_close_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::Help);
        }
        "HelpNextButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerMenusAndScreens>(this).help_next_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::Help);
        }
        "HelpPrevButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerMenusAndScreens>(this).help_prev_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::Help);
        }
        "LavaButtonLogic" => {
            let button = pg_engine::logic::button::spawn(world, display_object);
            world.obj_mut::<ControllerMenusAndScreens>(this).lava_button_logic = Some(button);
            add_button_logic(world, this, button, UiArray::Main);
        }
        "SpriteLogic" => {
            let sprite = crate::logic::island_core::sprite_logic::spawn(world, display_object);
            add_to_logic_array(world, this, sprite);
        }
        "CoconutLogic" => {
            // Kept only in `mCoconutLogicArray`, not in `mLogicArray`.
            let coconut = crate::logic::island_core::coconut_logic::spawn(world, display_object);
            crate::logic::island_core::coconut_logic::set_coconut_logic_listener(world, coconut, this);
            world.obj_mut::<ControllerWorldRegistry>(this).coconut_logic_array.push(coconut);
        }
        "WoodLogic" => {
            let wood = crate::logic::island_core::wood_logic::spawn(world, display_object);
            crate::logic::island_core::wood_logic::set_wood_logic_listener(world, wood, this);
            world.obj_mut::<ControllerWorldRegistry>(this).wood_logic_array.push(wood);
            add_to_logic_array(world, this, wood);
        }
        "BonFireLogic" => {
            let bon_fire = crate::logic::island_core::bon_fire_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).bon_fire_logic = Some(bon_fire);
            crate::logic::island_core::bon_fire_logic::set_bon_fire_logic_listener(world, bon_fire, this);
            add_to_logic_array(world, this, bon_fire);
        }
        "MultiModelLogic" => {
            let multi_model = pg_engine::logic::multi_model::spawn(world, display_object);
            add_to_logic_array(world, this, multi_model);
        }
        "SpearLogic" => {
            let spear = crate::logic::island_objects::spear_logic::spawn(world, display_object);
            crate::logic::island_objects::spear_logic::set_spear_logic_listener(world, spear, this);
            world.obj_mut::<ControllerWorldRegistry>(this).spear_logic_array.push(spear);
            add_to_logic_array(world, this, spear);
        }
        "FishLogic" => {
            let fish = crate::logic::ocean_fishing::fish_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).fish_logic = Some(fish);
            crate::logic::ocean_fishing::fish_logic::set_fish_logic_listener(world, fish, this);
            add_to_logic_array(world, this, fish);
        }
        "SwordFishLogic" => {
            let sword_fish = crate::logic::ocean_creatures::sword_fish_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).sword_fish_logic = Some(sword_fish);
            crate::logic::ocean_creatures::sword_fish_logic::set_sword_fish_logic_listener(world, sword_fish, this);
            add_to_logic_array(world, this, sword_fish);
        }
        "FishingPoleLogic" => {
            let fishing_pole = crate::logic::ocean_fishing::fishing_pole_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).fishing_pole_logic = Some(fishing_pole);
            crate::logic::ocean_fishing::fishing_pole_logic::set_fishing_pole_logic_listener(world, fishing_pole, this);
            add_to_logic_array(world, this, fishing_pole);
        }
        "BoulderLogic" => {
            let boulder = crate::logic::island_objects::boulder_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).boulder_logic = Some(boulder);
            crate::logic::island_objects::boulder_logic::set_boulder_logic_listener(world, boulder, this);
            add_to_logic_array(world, this, boulder);
        }
        "TRexEggLogic" => {
            let t_rex_egg = crate::logic::monsters::t_rex_egg_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).t_rex_egg_logic = Some(t_rex_egg);
            crate::logic::monsters::t_rex_egg_logic::set_t_rex_egg_logic_listener(world, t_rex_egg, this);
            add_to_logic_array(world, this, t_rex_egg);
        }
        "TRexLogic" => {
            let t_rex = crate::logic::monsters::t_rex_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).t_rex_logic = Some(t_rex);
            crate::logic::monsters::t_rex_logic::set_t_rex_logic_listener(world, t_rex, this);
            add_to_logic_array(world, this, t_rex);
        }
        "SpiderLogic" => {
            let spider = crate::logic::monsters::spider_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).spider_logic = Some(spider);
            crate::logic::monsters::spider_logic::set_spider_logic_listener(world, spider, this);
            add_to_logic_array(world, this, spider);
        }
        "SpiderTreeLogic" => {
            let spider_tree = crate::logic::monsters::spider_tree_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).spider_tree_logic = Some(spider_tree);
            crate::logic::monsters::spider_tree_logic::set_spider_tree_logic_listener(world, spider_tree, this);
            add_to_logic_array(world, this, spider_tree);
        }
        "OutHouseLogic" => {
            let out_house = crate::logic::island_objects::out_house_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).out_house_logic = Some(out_house);
            crate::logic::island_objects::out_house_logic::set_out_house_logic_listener(world, out_house, this);
            add_to_logic_array(world, this, out_house);
        }
        "IceMonsterLogic" => {
            let ice_monster = crate::logic::ice_island::ice_monster_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ice_monster_logic = Some(ice_monster);
            crate::logic::ice_island::ice_monster_logic::set_ice_monster_logic_listener(world, ice_monster, this);
            add_to_logic_array(world, this, ice_monster);
        }
        "GlacierLogic" => {
            let glacier = crate::logic::ice_island::glacier_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).glacier_logic = Some(glacier);
            crate::logic::ice_island::glacier_logic::set_glacier_logic_listener(world, glacier, this);
            add_to_logic_array(world, this, glacier);
        }
        "IglooLogic" => {
            let igloo = crate::logic::ice_island::igloo_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).igloo_logic = Some(igloo);
            crate::logic::ice_island::igloo_logic::set_igloo_logic_listener(world, igloo, this);
            add_to_logic_array(world, this, igloo);
        }
        "IceTreeLogic" => {
            let ice_tree = crate::logic::ice_island::ice_tree_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ice_tree_logic = Some(ice_tree);
            crate::logic::ice_island::ice_tree_logic::set_ice_tree_logic_listener(world, ice_tree, this);
            add_to_logic_array(world, this, ice_tree);
        }
        "IceHoleLogic" => {
            let ice_hole = crate::logic::ice_island::ice_hole_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ice_hole_logic = Some(ice_hole);
            crate::logic::ice_island::ice_hole_logic::set_ice_hole_logic_listener(world, ice_hole, this);
            add_to_logic_array(world, this, ice_hole);
        }
        "ZombieGraveLogic" => {
            let zombie_grave = crate::logic::monsters::zombie_grave_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).zombie_grave_logic = Some(zombie_grave);
            crate::logic::monsters::zombie_grave_logic::set_zombie_grave_logic_listener(world, zombie_grave, this);
            add_to_logic_array(world, this, zombie_grave);
        }
        "ApeLogic" => {
            let ape = crate::logic::ape_ritual::ape_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ape_logic = Some(ape);
            crate::logic::ape_ritual::ape_logic::set_ape_logic_listener(world, ape, this);
            add_to_logic_array(world, this, ape);
        }
        "ApeAltarLogic" => {
            let ape_altar = crate::logic::ape_ritual::ape_altar_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ape_altar_logic = Some(ape_altar);
            crate::logic::ape_ritual::ape_altar_logic::set_ape_altar_logic_listener(world, ape_altar, this);
            add_to_logic_array(world, this, ape_altar);
        }
        "ApeBongoLogic" => {
            let ape_bongo = crate::logic::ape_ritual::ape_bongo_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ape_bongo_logic = Some(ape_bongo);
            crate::logic::ape_ritual::ape_bongo_logic::set_ape_bongo_logic_listener(world, ape_bongo, this);
            add_to_logic_array(world, this, ape_bongo);
        }
        "ApeWenchLogic" => {
            let ape_wench = crate::logic::ape_ritual::ape_wench_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ape_wench_logic = Some(ape_wench);
            crate::logic::ape_ritual::ape_wench_logic::set_ape_wench_logic_listener(world, ape_wench, this);
            add_to_logic_array(world, this, ape_wench);
        }
        "ApeTorchLogic" => {
            let ape_torch = crate::logic::ape_ritual::ape_torch_logic::spawn(world, display_object);
            crate::logic::ape_ritual::ape_torch_logic::set_ape_torch_logic_listener(world, ape_torch, this);
            world.obj_mut::<ControllerWorldRegistry>(this).ape_torch_logic_array.push(ape_torch);
            add_to_logic_array(world, this, ape_torch);
        }
        "MonkeyLogic" => {
            let monkey = crate::logic::ape_monkeys::monkey_logic::spawn(world, display_object);
            crate::logic::ape_monkeys::monkey_logic::set_monkey_logic_listener(world, monkey, this);
            world.obj_mut::<ControllerWorldRegistry>(this).monkey_logic_array.push(monkey);
            add_to_logic_array(world, this, monkey);
        }
        "BananaTreeLogic" => {
            let banana_tree = crate::logic::ape_monkeys::banana_tree_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).banana_tree_logic = Some(banana_tree);
            crate::logic::ape_monkeys::banana_tree_logic::set_banana_tree_logic_listener(world, banana_tree, this);
            add_to_logic_array(world, this, banana_tree);
        }
        "BananaLogic" => {
            let banana = crate::logic::ape_monkeys::banana_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).banana_logic = Some(banana);
            crate::logic::ape_monkeys::banana_logic::set_banana_logic_listener(world, banana, this);
            add_to_logic_array(world, this, banana);
        }
        "CrackHoleLogic" => {
            let crack_hole = crate::logic::theruns::crack_hole_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).crack_hole_logic = Some(crack_hole);
            crate::logic::theruns::crack_hole_logic::set_crack_hole_logic_listener(world, crack_hole, this);
            add_to_logic_array(world, this, crack_hole);
        }
        "TheRunsPlatformLogic" => {
            let platform = crate::logic::theruns::the_runs_platform_logic::spawn(world, display_object);
            add_to_logic_array(world, this, platform);
        }
        "TheRunsStalactiteLogic" => {
            let stalactite = crate::logic::theruns::the_runs_stalactite_logic::spawn(world, display_object);
            add_to_logic_array(world, this, stalactite);
        }
        "OctoStatueLogic" => {
            let octo_statue = crate::logic::island_objects::octo_statue_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).octo_statue_logic = Some(octo_statue);
            crate::logic::island_objects::octo_statue_logic::set_octo_statue_logic_listener(world, octo_statue, this);
            add_to_logic_array(world, this, octo_statue);
        }
        "KongStatueLogic" => {
            let kong_statue = crate::logic::konkeydong::kong_statue_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).kong_statue_logic = Some(kong_statue);
            crate::logic::konkeydong::kong_statue_logic::set_kong_statue_logic_listener(world, kong_statue, this);
            add_to_logic_array(world, this, kong_statue);
        }
        "KonkeyDongLogic" => {
            let konkey_dong = crate::logic::konkeydong::konkey_dong_logic::spawn(world, display_object);
            add_to_logic_array(world, this, konkey_dong);
        }
        "SpearStatueLogic" => {
            let spear_statue = crate::logic::underwater_stage::spear_statue_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).spear_statue_logic = Some(spear_statue);
            crate::logic::underwater_stage::spear_statue_logic::set_spear_statue_logic_listener(world, spear_statue, this);
            add_to_logic_array(world, this, spear_statue);
        }
        "AnchorLogic" => {
            let anchor = crate::logic::underwater_baitmaster_laser::anchor_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).anchor_logic = Some(anchor);
            crate::logic::underwater_baitmaster_laser::anchor_logic::set_anchor_logic_listener(world, anchor, this);
            add_to_logic_array(world, this, anchor);
        }
        "TileDrainLogic" => {
            let tile_drain = crate::logic::underwater_paindrain::tile_drain_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).tile_drain_logic = Some(tile_drain);
            crate::logic::underwater_paindrain::tile_drain_logic::set_tile_drain_logic_listener(world, tile_drain, this);
            add_to_logic_array(world, this, tile_drain);
        }
        "TilePlugLogic" => {
            let tile_plug = crate::logic::underwater_paindrain::tile_plug_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).tile_plug_logic = Some(tile_plug);
            crate::logic::underwater_paindrain::tile_plug_logic::set_tile_plug_logic_listener(world, tile_plug, this);
            add_to_logic_array(world, this, tile_plug);
        }
        "ChumBucketLogic" => {
            let chum_bucket = crate::logic::underwater_baitmaster_laser::chum_bucket_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).chum_bucket_logic = Some(chum_bucket);
            crate::logic::underwater_baitmaster_laser::chum_bucket_logic::set_chum_bucket_logic_listener(world, chum_bucket, this);
            add_to_logic_array(world, this, chum_bucket);
        }
        "SchoolOfFishLogic" => {
            let scenery = spawn_underwater_scenery_logic(world, display_object, 0);
            world.obj_mut::<ControllerWorldRegistry>(this).underwater_scenery_logic_array.push(scenery);
            add_to_logic_array(world, this, scenery);
        }
        "UnderwaterBlob1Logic" => {
            let scenery = spawn_underwater_scenery_logic(world, display_object, 1);
            world.obj_mut::<ControllerWorldRegistry>(this).underwater_scenery_logic_array.push(scenery);
            add_to_logic_array(world, this, scenery);
        }
        "UnderwaterBlob2Logic" => {
            let scenery = spawn_underwater_scenery_logic(world, display_object, 2);
            world.obj_mut::<ControllerWorldRegistry>(this).underwater_scenery_logic_array.push(scenery);
            add_to_logic_array(world, this, scenery);
        }
        "HealthBarLogic" => {
            let health_bar = crate::logic::ice_island::health_bar_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).health_bar_logic = Some(health_bar);
            add_to_logic_array(world, this, health_bar);
        }
        "AntHillLogic" => {
            let ant_hill = crate::logic::island_anthill::ant_hill_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).ant_hill_logic = Some(ant_hill);
            crate::logic::island_anthill::ant_hill_logic::set_ant_hill_logic_listener(world, ant_hill, this);
            add_to_logic_array(world, this, ant_hill);
        }
        "MagnifyingGlassLogic" => {
            let magnifying_glass = crate::logic::island_anthill::magnifying_glass_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).magnifying_glass_logic = Some(magnifying_glass);
            crate::logic::island_anthill::magnifying_glass_logic::set_magnifying_glass_logic_listener(world, magnifying_glass, this);
            add_to_logic_array(world, this, magnifying_glass);
        }
        "SharkLogic" => {
            let shark = crate::logic::ocean_creatures::shark_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).shark_logic = Some(shark);
            crate::logic::ocean_creatures::shark_logic::set_shark_logic_listener(world, shark, this);
            add_to_logic_array(world, this, shark);
        }
        "BirdLogic" => {
            let bird = crate::logic::ocean_birds::bird_logic::spawn(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).bird_logic = Some(bird);
            crate::logic::ocean_birds::bird_logic::set_bird_logic_listener(world, bird, this);
            add_to_logic_array(world, this, bird);
        }
        "VolcanoLogic" => {
            let volcano = crate::logic::island_objects::volcano_logic::spawn(world, display_object);
            world.obj_mut::<ControllerGodPowers>(this).volcano_logic = Some(volcano);
            crate::logic::island_objects::volcano_logic::set_volcano_logic_listener(world, volcano, this);
            add_to_logic_array(world, this, volcano);
        }
        "SunLogic" => {
            let sun = crate::logic::sky_weather::sun_logic::spawn(world, display_object);
            world.obj_mut::<ControllerSkyWeather>(this).sun_logic = Some(sun);
            crate::logic::sky_weather::sun_logic::set_sun_logic_listener(world, sun, this);
            add_to_logic_array(world, this, sun);
        }
        "MoonLogic" => {
            let moon = crate::logic::sky_weather::moon_logic::spawn(world, display_object);
            world.obj_mut::<ControllerSkyWeather>(this).moon_logic = Some(moon);
            crate::logic::sky_weather::moon_logic::set_moon_logic_listener(world, moon, this);
            add_to_logic_array(world, this, moon);
        }
        "StarsLogic" => {
            let stars = pg_engine::logic::multi_model::spawn(world, display_object);
            world.obj_mut::<ControllerSkyWeather>(this).stars_logic = Some(stars);
            pg_engine::display::remove_from_display_group(world, display_object);
            add_to_logic_array(world, this, stars);
        }
        "FrontStormCloudLogic" => {
            let storm_cloud = crate::logic::sky_weather::storm_cloud_logic::spawn(world, display_object);
            world.obj_mut::<ControllerSkyWeather>(this).front_storm_cloud_logic = Some(storm_cloud);
            crate::logic::sky_weather::storm_cloud_logic::set_front_storm_cloud(world, storm_cloud, true, 1.0);
            add_to_logic_array(world, this, storm_cloud);
        }
        "BehindStormCloudLogic" => {
            let storm_cloud = crate::logic::sky_weather::storm_cloud_logic::spawn(world, display_object);
            world.obj_mut::<ControllerSkyWeather>(this).behind_storm_cloud_logic = Some(storm_cloud);
            crate::logic::sky_weather::storm_cloud_logic::set_front_storm_cloud(world, storm_cloud, false, 0.8);
            add_to_logic_array(world, this, storm_cloud);
        }
        "FrontOceanLogic" => {
            let ocean = crate::logic::sky_weather::ocean_logic::spawn(world, display_object);
            world.obj_mut::<ControllerTouchInput>(this).front_ocean_logic = Some(ocean);
            crate::logic::sky_weather::ocean_logic::set_front_ocean(world, ocean, true, 1.0);
            add_to_logic_array(world, this, ocean);
        }
        "BehindOceanLogic" => {
            let ocean = crate::logic::sky_weather::ocean_logic::spawn(world, display_object);
            world.obj_mut::<ControllerTouchInput>(this).behind_ocean_logic = Some(ocean);
            crate::logic::sky_weather::ocean_logic::set_front_ocean(world, ocean, false, 0.5);
            add_to_logic_array(world, this, ocean);
        }
        "FrontTsunamiLogic" => {
            let tsunami = crate::logic::underwater_stage::tsunami_logic::spawn(world, display_object);
            world.obj_mut::<ControllerIslandLocations>(this).front_tsunami_logic = Some(tsunami);
            crate::logic::underwater_stage::tsunami_logic::set_front_layer(world, tsunami, true);

            let front_tsunami_logic = world.obj::<ControllerIslandLocations>(this).front_tsunami_logic;
            if let Some(front_tsunami_logic) = front_tsunami_logic {
                crate::logic::underwater_stage::tsunami_logic::set_tsunami_logic_listener(world, front_tsunami_logic, this);
            }
            add_to_logic_array(world, this, tsunami);
        }
        "BehindTsunamiLogic" => {
            let tsunami = crate::logic::underwater_stage::tsunami_logic::spawn(world, display_object);
            world.obj_mut::<ControllerIslandLocations>(this).behind_tsunami_logic = Some(tsunami);
            crate::logic::underwater_stage::tsunami_logic::set_front_layer(world, tsunami, false);

            let behind_tsunami_logic = world.obj::<ControllerIslandLocations>(this).behind_tsunami_logic;
            if let Some(behind_tsunami_logic) = behind_tsunami_logic {
                crate::logic::underwater_stage::tsunami_logic::set_tsunami_logic_listener(world, behind_tsunami_logic, this);
            }
            add_to_logic_array(world, this, tsunami);
        }
        "MapPaperLogic" => {
            let map_paper = spawn_map_paper_logic(world, display_object);
            world.obj_mut::<ControllerWorldRegistry>(this).map_paper_logic = Some(map_paper);
            add_to_logic_array(world, this, map_paper);
        }
        _ => {}
    }

    ItemResult::Done
}
