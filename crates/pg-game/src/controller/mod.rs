//! `PocketGodViewController` (superclass `UIViewController`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod apocalypse;
pub use apocalypse::*;
pub mod app_platform;
pub use app_platform::*;
pub mod boot_and_frame;
pub use boot_and_frame::*;
pub mod god_powers;
pub use god_powers::*;
pub mod island_features;
pub use island_features::*;
pub mod island_locations;
pub use island_locations::*;
pub mod managers_bridge;
pub use managers_bridge::*;
pub mod menus_and_screens;
pub use menus_and_screens::*;
pub mod minigames;
pub use minigames::*;
pub mod pygmies;
pub use pygmies::*;
pub mod sky_weather;
pub use sky_weather::*;
pub mod stats_and_achievements;
pub use stats_and_achievements::*;
pub mod touch_input;
pub use touch_input::*;
pub mod world_object_events;
pub use world_object_events::*;
pub mod world_registry;
pub use world_registry::*;

/// `PocketGodViewController` ivars not owned by a unit component.
#[derive(Component, Clone, Debug, Default)]
pub struct PocketGodViewController {
}

/// Adds every ivar component of `PocketGodViewController` (all zero, like `alloc`).
pub fn insert_components(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(PocketGodViewController::default());
    world.entity_mut(this).insert(ControllerAppPlatform::default());
    world.entity_mut(this).insert(ControllerBootAndFrame::default());
    world.entity_mut(this).insert(ControllerStatsAndAchievements::default());
    world.entity_mut(this).insert(ControllerManagersBridge::default());
    world.entity_mut(this).insert(ControllerWorldRegistry::default());
    world.entity_mut(this).insert(ControllerTouchInput::default());
    world.entity_mut(this).insert(ControllerIslandLocations::default());
    world.entity_mut(this).insert(ControllerMinigames::default());
    world.entity_mut(this).insert(ControllerMenusAndScreens::default());
    world.entity_mut(this).insert(ControllerSkyWeather::default());
    world.entity_mut(this).insert(ControllerGodPowers::default());
    world.entity_mut(this).insert(ControllerPygmies::default());
    world.entity_mut(this).insert(ControllerWorldObjectEvents::default());
    world.entity_mut(this).insert(ControllerApocalypse::default());
    world.entity_mut(this).insert(ControllerIslandFeatures::default());
}

// Ivar → component index for `PocketGodViewController`:
//   mView → ControllerAppPlatform::view
//   mbComicUrlOpened → ControllerAppPlatform::comic_url_opened
//   mLaunchUrl → ControllerAppPlatform::launch_url
//   mbHideLoadingScreen → ControllerAppPlatform::hide_loading_screen
//   mbModalViewControllerActive → ControllerAppPlatform::modal_view_controller_active
//   mTextField → ControllerAppPlatform::text_field
//   mSaveEditText → ControllerAppPlatform::save_edit_text
//   mKeyboardListener → ControllerAppPlatform::keyboard_listener
//   mMuteSequence → ControllerAppPlatform::mute_sequence
//   mDelegate → ControllerAppPlatform::delegate
//   mbKeyboardOpen → ControllerAppPlatform::keyboard_open
//   mMaxEditCharCount → ControllerAppPlatform::max_edit_char_count
//   mEditingText → ControllerAppPlatform::editing_text
//   mActiveCancelEditDisplayObject → ControllerAppPlatform::active_cancel_edit_display_object
//   mbChartboostAdVisible → ControllerAppPlatform::chartboost_ad_visible
//   mLibrary → ControllerBootAndFrame::library
//   mAppStartTime → ControllerBootAndFrame::app_start_time
//   mbGameStarted → ControllerBootAndFrame::game_started
//   mFpsTime → ControllerBootAndFrame::fps_time
//   mTargetFps → ControllerBootAndFrame::target_fps
//   mLongestFrameTime → ControllerBootAndFrame::longest_frame_time
//   mSpikeCount → ControllerBootAndFrame::spike_count
//   mbInitComplete → ControllerBootAndFrame::init_complete
//   mPersonaPreferences → ControllerBootAndFrame::persona_preferences
//   mOptionPreferences → ControllerBootAndFrame::option_preferences
//   mSystemPreferences → ControllerBootAndFrame::system_preferences
//   mChallengePreferences → ControllerBootAndFrame::challenge_preferences
//   mSequence → ControllerBootAndFrame::sequence
//   mMenuDisplayGroup → ControllerBootAndFrame::menu_display_group
//   mMenuDisplayMarker → ControllerBootAndFrame::menu_display_marker
//   mStartTime → ControllerBootAndFrame::start_time
//   mTargetIslandYPosVel → ControllerBootAndFrame::target_island_y_pos_vel
//   mTargetIslandYPos → ControllerBootAndFrame::target_island_y_pos
//   mbAddPygmyAfterIdolStartScreen → ControllerBootAndFrame::add_pygmy_after_idol_start_screen
//   mGameServer → ControllerStatsAndAchievements::game_server
//   mbCheckAllAchievementsAndLeaderboards → ControllerStatsAndAchievements::check_all_achievements_and_leaderboards
//   mGodPersona → ControllerStatsAndAchievements::god_persona
//   mTribePersona → ControllerStatsAndAchievements::tribe_persona
//   mCharliePygmyPersona → ControllerStatsAndAchievements::charlie_pygmy_persona
//   mTomPygmyPersona → ControllerStatsAndAchievements::tom_pygmy_persona
//   mTotalGifts → ControllerStatsAndAchievements::total_gifts
//   mTotalSacrifices → ControllerStatsAndAchievements::total_sacrifices
//   mbSubmittingScore → ControllerStatsAndAchievements::submitting_score
//   mbSubmitSacrificesWhenComplete → ControllerStatsAndAchievements::submit_sacrifices_when_complete
//   mbSubmitGiftsWhenComplete → ControllerStatsAndAchievements::submit_gifts_when_complete
//   mStatsTribeUiLogicArray → ControllerStatsAndAchievements::stats_tribe_ui_logic_array
//   mStatsPygmyUiLogicArray → ControllerStatsAndAchievements::stats_pygmy_ui_logic_array
//   mStatsGodUiLogicArray → ControllerStatsAndAchievements::stats_god_ui_logic_array
//   mActiveStatsScreen → ControllerStatsAndAchievements::active_stats_screen
//   mStatsTribeCloseButtonLogic → ControllerStatsAndAchievements::stats_tribe_close_button_logic
//   mStatsTribeEditNameButtonLogic → ControllerStatsAndAchievements::stats_tribe_edit_name_button_logic
//   mStatsTribeCancelEditNameButtonLogic → ControllerStatsAndAchievements::stats_tribe_cancel_edit_name_button_logic
//   mGodStatsButtonLogic → ControllerStatsAndAchievements::god_stats_button_logic
//   mPygmyStatsButtonLogicArray → ControllerStatsAndAchievements::pygmy_stats_button_logic_array
//   mStatsPygmyCloseButtonLogic → ControllerStatsAndAchievements::stats_pygmy_close_button_logic
//   mStatsPygmyNextButtonLogic → ControllerStatsAndAchievements::stats_pygmy_next_button_logic
//   mStatsPygmyEditNameButtonLogic → ControllerStatsAndAchievements::stats_pygmy_edit_name_button_logic
//   mStatsPygmyEditDescriptionButtonLogic → ControllerStatsAndAchievements::stats_pygmy_edit_description_button_logic
//   mStatsPygmyCancelEditNameButtonLogic → ControllerStatsAndAchievements::stats_pygmy_cancel_edit_name_button_logic
//   mStatsPygmyCancelEditDescriptionButtonLogic → ControllerStatsAndAchievements::stats_pygmy_cancel_edit_description_button_logic
//   mActivePygmyPersona → ControllerStatsAndAchievements::active_pygmy_persona
//   mPygmyStatsType → ControllerStatsAndAchievements::pygmy_stats_type
//   mStatsGodCloseButtonLogic → ControllerStatsAndAchievements::stats_god_close_button_logic
//   mStatsGodNextGodTypeButtonLogic → ControllerStatsAndAchievements::stats_god_next_god_type_button_logic
//   mStatsGodEditNameButtonLogic → ControllerStatsAndAchievements::stats_god_edit_name_button_logic
//   mStatsGodEditDescriptionButtonLogic → ControllerStatsAndAchievements::stats_god_edit_description_button_logic
//   mStatsGodCancelEditNameButtonLogic → ControllerStatsAndAchievements::stats_god_cancel_edit_name_button_logic
//   mStatsGodCancelEditDescriptionButtonLogic → ControllerStatsAndAchievements::stats_god_cancel_edit_description_button_logic
//   mChallengeManager → ControllerManagersBridge::challenge_manager
//   mSkinPackManager → ControllerManagersBridge::skin_pack_manager
//   mBonusPackManager → ControllerManagersBridge::bonus_pack_manager
//   mStoryManager → ControllerManagersBridge::story_manager
//   mDebugScreen → ControllerManagersBridge::debug_screen
//   mNewsManager → ControllerManagersBridge::news_manager
//   mIdolManager → ControllerManagersBridge::idol_manager
//   mGameUiManager → ControllerManagersBridge::game_ui_manager
//   mMapScreen → ControllerManagersBridge::map_screen
//   mbSkinPackScreenOpen → ControllerManagersBridge::skin_pack_screen_open
//   mbBonusPackScreenOpen → ControllerManagersBridge::bonus_pack_screen_open
//   mbMapScreenOpen → ControllerManagersBridge::map_screen_open
//   mbStoryScreenOpen → ControllerManagersBridge::story_screen_open
//   mbStoryActive → ControllerManagersBridge::story_active
//   mStorySaveLabelMode → ControllerManagersBridge::story_save_label_mode
//   mbNewsTeaserOpen → ControllerManagersBridge::news_teaser_open
//   mbNewsScreenOpen → ControllerManagersBridge::news_screen_open
//   mbIdolScreenOpen → ControllerManagersBridge::idol_screen_open
//   mbIdolAwardScreenOpen → ControllerManagersBridge::idol_award_screen_open
//   mbIdolBattleResultsScreenOpen → ControllerManagersBridge::idol_battle_results_screen_open
//   mbChallengeScreenOpen → ControllerManagersBridge::challenge_screen_open
//   mbChallengeDialogOpen → ControllerManagersBridge::challenge_dialog_open
//   mbChallengeActive → ControllerManagersBridge::challenge_active
//   mMapPaperLogic → ControllerWorldRegistry::map_paper_logic
//   mSmallObjectDisplayMarker → ControllerWorldRegistry::small_object_display_marker
//   mLogicArray → ControllerWorldRegistry::logic_array
//   mMainUiLogicArray → ControllerWorldRegistry::main_ui_logic_array
//   mHelpUiLogicArray → ControllerWorldRegistry::help_ui_logic_array
//   mWhatsNewScreenUiLogicArray → ControllerWorldRegistry::whats_new_screen_ui_logic_array
//   mCoconutLogicArray → ControllerWorldRegistry::coconut_logic_array
//   mWoodLogicArray → ControllerWorldRegistry::wood_logic_array
//   mUnderwaterSceneryLogicArray → ControllerWorldRegistry::underwater_scenery_logic_array
//   mSpearLogicArray → ControllerWorldRegistry::spear_logic_array
//   mApeTorchLogicArray → ControllerWorldRegistry::ape_torch_logic_array
//   mMonkeyLogicArray → ControllerWorldRegistry::monkey_logic_array
//   mJellyfishLogicArray → ControllerWorldRegistry::jellyfish_logic_array
//   mGodIdolLogicArray → ControllerWorldRegistry::god_idol_logic_array
//   mApocalypseGemPedestalLogicArray → ControllerWorldRegistry::apocalypse_gem_pedestal_logic_array
//   mTimeTempleDoorLogicArray → ControllerWorldRegistry::time_temple_door_logic_array
//   mLocustTempleDoorLogicArray → ControllerWorldRegistry::locust_temple_door_logic_array
//   mDanceTempleDoorLogicArray → ControllerWorldRegistry::dance_temple_door_logic_array
//   mPlagueTempleDoorLogicArray → ControllerWorldRegistry::plague_temple_door_logic_array
//   mPurificationTempleDoorLogicArray → ControllerWorldRegistry::purification_temple_door_logic_array
//   mClosetTempleDoorLogicArray → ControllerWorldRegistry::closet_temple_door_logic_array
//   mXenuTempleDoorLogicArray → ControllerWorldRegistry::xenu_temple_door_logic_array
//   mIdolPedestalLogicArray → ControllerWorldRegistry::idol_pedestal_logic_array
//   mWaterWingsLogic → ControllerWorldRegistry::water_wings_logic
//   mFishingPoleLogic → ControllerWorldRegistry::fishing_pole_logic
//   mTRexEggLogic → ControllerWorldRegistry::t_rex_egg_logic
//   mTRexLogic → ControllerWorldRegistry::t_rex_logic
//   mSpiderLogic → ControllerWorldRegistry::spider_logic
//   mSpiderTreeLogic → ControllerWorldRegistry::spider_tree_logic
//   mFishLogic → ControllerWorldRegistry::fish_logic
//   mSwordFishLogic → ControllerWorldRegistry::sword_fish_logic
//   mBoulderLogic → ControllerWorldRegistry::boulder_logic
//   mOutHouseLogic → ControllerWorldRegistry::out_house_logic
//   mIceMonsterLogic → ControllerWorldRegistry::ice_monster_logic
//   mGlacierLogic → ControllerWorldRegistry::glacier_logic
//   mIglooLogic → ControllerWorldRegistry::igloo_logic
//   mIceTreeLogic → ControllerWorldRegistry::ice_tree_logic
//   mIceHoleLogic → ControllerWorldRegistry::ice_hole_logic
//   mApeLogic → ControllerWorldRegistry::ape_logic
//   mApeAltarLogic → ControllerWorldRegistry::ape_altar_logic
//   mApeBongoLogic → ControllerWorldRegistry::ape_bongo_logic
//   mApeWenchLogic → ControllerWorldRegistry::ape_wench_logic
//   mGoGoCageLogic → ControllerWorldRegistry::go_go_cage_logic
//   mDiscoBallLogic → ControllerWorldRegistry::disco_ball_logic
//   mDiscoLightsLogic → ControllerWorldRegistry::disco_lights_logic
//   mDanceBalloonBundleLogic → ControllerWorldRegistry::dance_balloon_bundle_logic
//   mDanceBalloonLogic → ControllerWorldRegistry::dance_balloon_logic
//   mDanceDanceExecutionLogic → ControllerWorldRegistry::dance_dance_execution_logic
//   mBananaTreeLogic → ControllerWorldRegistry::banana_tree_logic
//   mBananaLogic → ControllerWorldRegistry::banana_logic
//   mMoronSnakeLogic → ControllerWorldRegistry::moron_snake_logic
//   mNoPeeLogic → ControllerWorldRegistry::no_pee_logic
//   mUnderwaterTempleLogic → ControllerWorldRegistry::underwater_temple_logic
//   mTempleDoorLogic → ControllerWorldRegistry::temple_door_logic
//   mApocalypseTempleDoorLogic → ControllerWorldRegistry::apocalypse_temple_door_logic
//   mApocalypsePortalTempleDoorLogic → ControllerWorldRegistry::apocalypse_portal_temple_door_logic
//   mApocalypseCalendarLogic → ControllerWorldRegistry::apocalypse_calendar_logic
//   mApocalypseRoomFaderLogic → ControllerWorldRegistry::apocalypse_room_fader_logic
//   mApocalypseTimeTempleLeverLogic → ControllerWorldRegistry::apocalypse_time_temple_lever_logic
//   mApocalypseTimeTemplePlatformLogic → ControllerWorldRegistry::apocalypse_time_temple_platform_logic
//   mLocustQueenLogic → ControllerWorldRegistry::locust_queen_logic
//   mApocalypsePlagueTempleFountainLogic → ControllerWorldRegistry::apocalypse_plague_temple_fountain_logic
//   mApocalypsePurificationTempleFountainLogic → ControllerWorldRegistry::apocalypse_purification_temple_fountain_logic
//   mClosetTempleThetanMeterLogic → ControllerWorldRegistry::closet_temple_thetan_meter_logic
//   mClosetTempleCouchLogic → ControllerWorldRegistry::closet_temple_couch_logic
//   mClosetTempleClosetDoorLogic → ControllerWorldRegistry::closet_temple_closet_door_logic
//   mXenuTempleLogic → ControllerWorldRegistry::xenu_temple_logic
//   mXenuLogic → ControllerWorldRegistry::xenu_logic
//   mMisfitOceanRockLogic → ControllerWorldRegistry::misfit_ocean_rock_logic
//   mZombieGraveLogic → ControllerWorldRegistry::zombie_grave_logic
//   mOctoStatueLogic → ControllerWorldRegistry::octo_statue_logic
//   mKongStatueLogic → ControllerWorldRegistry::kong_statue_logic
//   mSpearStatueLogic → ControllerWorldRegistry::spear_statue_logic
//   mAnchorLogic → ControllerWorldRegistry::anchor_logic
//   mTileDrainLogic → ControllerWorldRegistry::tile_drain_logic
//   mTilePlugLogic → ControllerWorldRegistry::tile_plug_logic
//   mChumBucketLogic → ControllerWorldRegistry::chum_bucket_logic
//   mHealthBarLogic → ControllerWorldRegistry::health_bar_logic
//   mAntHillLogic → ControllerWorldRegistry::ant_hill_logic
//   mBonFireLogic → ControllerWorldRegistry::bon_fire_logic
//   mMagnifyingGlassLogic → ControllerWorldRegistry::magnifying_glass_logic
//   mSharkLogic → ControllerWorldRegistry::shark_logic
//   mBirdLogic → ControllerWorldRegistry::bird_logic
//   mCrackHoleLogic → ControllerWorldRegistry::crack_hole_logic
//   mAllTouches → ControllerTouchInput::all_touches
//   mTouchScreenPos → ControllerTouchInput::touch_screen_pos
//   mTouchLogicArray → ControllerTouchInput::touch_logic_array
//   mCancelTouchLogicArray → ControllerTouchInput::cancel_touch_logic_array
//   mApeWenchTouch → ControllerTouchInput::ape_wench_touch
//   mLastApeWenchPos → ControllerTouchInput::last_ape_wench_pos
//   mLastBubbleTime → ControllerTouchInput::last_bubble_time
//   mPinchSunState → ControllerTouchInput::pinch_sun_state
//   mPinchSunDistance → ControllerTouchInput::pinch_sun_distance
//   mPinchSunTouch0 → ControllerTouchInput::pinch_sun_touch0
//   mPinchSunTouch1 → ControllerTouchInput::pinch_sun_touch1
//   mbDragCrack → ControllerTouchInput::drag_crack
//   mDragCrackTouch → ControllerTouchInput::drag_crack_touch
//   mbDragOcean → ControllerTouchInput::drag_ocean
//   mOceanDragVel → ControllerTouchInput::ocean_drag_vel
//   mDragOceanAveVel → ControllerTouchInput::drag_ocean_ave_vel
//   mDragOceanTimeStamp → ControllerTouchInput::drag_ocean_time_stamp
//   mDragOceanTouch → ControllerTouchInput::drag_ocean_touch
//   mDragOceanTouchPos → ControllerTouchInput::drag_ocean_touch_pos
//   mFrontOceanLogic → ControllerTouchInput::front_ocean_logic
//   mBehindOceanLogic → ControllerTouchInput::behind_ocean_logic
//   mUnderwaterTouchStartYPos → ControllerTouchInput::underwater_touch_start_y_pos
//   mUnderwaterTouch → ControllerTouchInput::underwater_touch
//   mUnderwaterTouchPos → ControllerTouchInput::underwater_touch_pos
//   mMiscSequence → ControllerIslandLocations::misc_sequence
//   mUnderwaterDisplayGroup → ControllerIslandLocations::underwater_display_group
//   mUnderwaterRuinsDisplayGroup → ControllerIslandLocations::underwater_ruins_display_group
//   mUnderwaterCoralReefDisplayGroup → ControllerIslandLocations::underwater_coral_reef_display_group
//   mIslandDisplayGroup → ControllerIslandLocations::island_display_group
//   mIslandRootDisplayMarker → ControllerIslandLocations::island_root_display_marker
//   mUnderwaterDisplayMarker → ControllerIslandLocations::underwater_display_marker
//   mUnderwaterRealmDisplayMarker → ControllerIslandLocations::underwater_realm_display_marker
//   mIslandDisplayMarker → ControllerIslandLocations::island_display_marker
//   mSandIslandDisplayGroup → ControllerIslandLocations::sand_island_display_group
//   mRockIslandDisplayGroup → ControllerIslandLocations::rock_island_display_group
//   mGraveyardIslandDisplayGroup → ControllerIslandLocations::graveyard_island_display_group
//   mIceIslandDisplayGroup → ControllerIslandLocations::ice_island_display_group
//   mApeIslandDisplayGroup → ControllerIslandLocations::ape_island_display_group
//   mMisfitIslandDisplayGroup → ControllerIslandLocations::misfit_island_display_group
//   mGodIslandDisplayGroup → ControllerIslandLocations::god_island_display_group
//   mFailBoatOceanDisplayGroup → ControllerIslandLocations::fail_boat_ocean_display_group
//   mApocalypseIslandDisplayGroup → ControllerIslandLocations::apocalypse_island_display_group
//   mTimeTempleDisplayGroup → ControllerIslandLocations::time_temple_display_group
//   mLocustTempleDisplayGroup → ControllerIslandLocations::locust_temple_display_group
//   mDanceTempleDisplayGroup → ControllerIslandLocations::dance_temple_display_group
//   mPlagueTempleDisplayGroup → ControllerIslandLocations::plague_temple_display_group
//   mPurificationTempleDisplayGroup → ControllerIslandLocations::purification_temple_display_group
//   mClosetTempleDisplayGroup → ControllerIslandLocations::closet_temple_display_group
//   mXenuTempleDisplayGroup → ControllerIslandLocations::xenu_temple_display_group
//   mFrontTsunamiLogic → ControllerIslandLocations::front_tsunami_logic
//   mBehindTsunamiLogic → ControllerIslandLocations::behind_tsunami_logic
//   mbIslandLocationTransition → ControllerIslandLocations::island_location_transition
//   mLastIslandLocationState → ControllerIslandLocations::last_island_location_state
//   mIslandLocationState → ControllerIslandLocations::island_location_state
//   mIslandMinXPos → ControllerIslandLocations::island_min_x_pos
//   mIslandMaxXPos → ControllerIslandLocations::island_max_x_pos
//   mIslandCenterXPos → ControllerIslandLocations::island_center_x_pos
//   mHorizonYPos → ControllerIslandLocations::horizon_y_pos
//   mSunTransitionYPos → ControllerIslandLocations::sun_transition_y_pos
//   mSunUnderHorizonTriggerYPos → ControllerIslandLocations::sun_under_horizon_trigger_y_pos
//   mSunUnderHorizonYPos → ControllerIslandLocations::sun_under_horizon_y_pos
//   mSunOverHorizonYPos → ControllerIslandLocations::sun_over_horizon_y_pos
//   mMoonUnderHorizonTriggerYPos → ControllerIslandLocations::moon_under_horizon_trigger_y_pos
//   mMoonUnderHorizonYPos → ControllerIslandLocations::moon_under_horizon_y_pos
//   mMoonOverHorizonYPos → ControllerIslandLocations::moon_over_horizon_y_pos
//   mDoodleDisplayGroup → ControllerMinigames::doodle_display_group
//   mPainDrainDisplayGroup → ControllerMinigames::pain_drain_display_group
//   mTheRunsDisplayGroup → ControllerMinigames::the_runs_display_group
//   mKonkeyDongDisplayGroup → ControllerMinigames::konkey_dong_display_group
//   mMoronPestsDisplayGroup → ControllerMinigames::moron_pests_display_group
//   mNoPeeDisplayGroup → ControllerMinigames::no_pee_display_group
//   mBattleOfTheGodsDisplayGroup → ControllerMinigames::battle_of_the_gods_display_group
//   mDoodleCloudDisplayMarker → ControllerMinigames::doodle_cloud_display_marker
//   mDoodlePygmyDisplayMarker → ControllerMinigames::doodle_pygmy_display_marker
//   mPainDrainPlatformDisplayMarker → ControllerMinigames::pain_drain_platform_display_marker
//   mPainDrainPygmyDisplayMarker → ControllerMinigames::pain_drain_pygmy_display_marker
//   mTheRunsPygmyDisplayMarker → ControllerMinigames::the_runs_pygmy_display_marker
//   mKonkeyDongPygmyDisplayMarker → ControllerMinigames::konkey_dong_pygmy_display_marker
//   mMoronPestsPygmyDisplayMarker → ControllerMinigames::moron_pests_pygmy_display_marker
//   mNoPeePygmyDisplayMarker → ControllerMinigames::no_pee_pygmy_display_marker
//   mBattleOfTheGodsPygmyDisplayMarker → ControllerMinigames::battle_of_the_gods_pygmy_display_marker
//   mTheRunsSequence → ControllerMinigames::the_runs_sequence
//   mKonkeyDongSequence → ControllerMinigames::konkey_dong_sequence
//   mMoronPestsSequence → ControllerMinigames::moron_pests_sequence
//   mNoPeeSequence → ControllerMinigames::no_pee_sequence
//   mBattleOfTheGodsSequence → ControllerMinigames::battle_of_the_gods_sequence
//   mDoodleJumpPygmyLogic → ControllerMinigames::doodle_jump_pygmy_logic
//   mDoodleCloudLogicArray → ControllerMinigames::doodle_cloud_logic_array
//   mbDoodleJumpFall → ControllerMinigames::doodle_jump_fall
//   mbDoodleDebugPress → ControllerMinigames::doodle_debug_press
//   mbShowDoodleAssets → ControllerMinigames::show_doodle_assets
//   mPainDrainPygmyLogic → ControllerMinigames::pain_drain_pygmy_logic
//   mbShowPainDrainAssets → ControllerMinigames::show_pain_drain_assets
//   mbTheRuns → ControllerMinigames::the_runs
//   mbTheRunsStarting → ControllerMinigames::the_runs_starting
//   mbTheRunsMoveCamera → ControllerMinigames::the_runs_move_camera
//   mTheRunsPygmyLogic → ControllerMinigames::the_runs_pygmy_logic
//   mbShowTheRunsAssets → ControllerMinigames::show_the_runs_assets
//   mbKonkeyDong → ControllerMinigames::konkey_dong
//   mbKonkeyDongStarting → ControllerMinigames::konkey_dong_starting
//   mbKonkeyDongMoveCamera → ControllerMinigames::konkey_dong_move_camera
//   mbKonkeyDongMoveCameraToNextLevel → ControllerMinigames::konkey_dong_move_camera_to_next_level
//   mKonkeyDongLevelYPos → ControllerMinigames::konkey_dong_level_y_pos
//   mKonkeyDongPygmyLogic → ControllerMinigames::konkey_dong_pygmy_logic
//   mbShowKonkeyDongAssets → ControllerMinigames::show_konkey_dong_assets
//   mbMoronPests → ControllerMinigames::moron_pests
//   mbMoronPestsMoveCamera → ControllerMinigames::moron_pests_move_camera
//   mMoronPestsPygmyLogic → ControllerMinigames::moron_pests_pygmy_logic
//   mbShowMoronPestsAssets → ControllerMinigames::show_moron_pests_assets
//   mbNoPee → ControllerMinigames::no_pee
//   mbNoPeeStarting → ControllerMinigames::no_pee_starting
//   mbNoPeeMoveCamera → ControllerMinigames::no_pee_move_camera
//   mNoPeePygmyLogic → ControllerMinigames::no_pee_pygmy_logic
//   mbShowNoPeeAssets → ControllerMinigames::show_no_pee_assets
//   mbBattleOfTheGods → ControllerMinigames::battle_of_the_gods
//   mbShowBattleOfTheGodsAssets → ControllerMinigames::show_battle_of_the_gods_assets
//   mBattleOfTheGodsPygmyLogic → ControllerMinigames::battle_of_the_gods_pygmy_logic
//   mbDanceDanceExecution → ControllerMinigames::dance_dance_execution
//   mbPainDrain → ControllerMinigames::pain_drain
//   mSandIslandMenuDisplayGroup → ControllerMenusAndScreens::sand_island_menu_display_group
//   mRockIslandMenuDisplayGroup → ControllerMenusAndScreens::rock_island_menu_display_group
//   mGraveyardIslandMenuDisplayGroup → ControllerMenusAndScreens::graveyard_island_menu_display_group
//   mIceIslandMenuDisplayGroup → ControllerMenusAndScreens::ice_island_menu_display_group
//   mApeIslandMenuDisplayGroup → ControllerMenusAndScreens::ape_island_menu_display_group
//   mMisfitIslandMenuDisplayGroup → ControllerMenusAndScreens::misfit_island_menu_display_group
//   mGodIslandMenuDisplayGroup → ControllerMenusAndScreens::god_island_menu_display_group
//   mFailBoatMenuDisplayGroup → ControllerMenusAndScreens::fail_boat_menu_display_group
//   mApocalypseIslandMenuDisplayGroup → ControllerMenusAndScreens::apocalypse_island_menu_display_group
//   mTimeTempleMenuDisplayGroup → ControllerMenusAndScreens::time_temple_menu_display_group
//   mLocustTempleMenuDisplayGroup → ControllerMenusAndScreens::locust_temple_menu_display_group
//   mDanceTempleMenuDisplayGroup → ControllerMenusAndScreens::dance_temple_menu_display_group
//   mPlagueTempleMenuDisplayGroup → ControllerMenusAndScreens::plague_temple_menu_display_group
//   mPurificationTempleMenuDisplayGroup → ControllerMenusAndScreens::purification_temple_menu_display_group
//   mClosetTempleMenuDisplayGroup → ControllerMenusAndScreens::closet_temple_menu_display_group
//   mXenuTempleMenuDisplayGroup → ControllerMenusAndScreens::xenu_temple_menu_display_group
//   mUnderwaterMenuDisplayGroup → ControllerMenusAndScreens::underwater_menu_display_group
//   mUnderwaterCoralReefMenuDisplayGroup → ControllerMenusAndScreens::underwater_coral_reef_menu_display_group
//   mGlobalIslandMenuDisplayGroup → ControllerMenusAndScreens::global_island_menu_display_group
//   mGodPowerMenuDisplayGroup → ControllerMenusAndScreens::god_power_menu_display_group
//   mCustomizeMenuDisplayGroup → ControllerMenusAndScreens::customize_menu_display_group
//   mCommunityMenuDisplayGroup → ControllerMenusAndScreens::community_menu_display_group
//   mSettingsMenuDisplayGroup → ControllerMenusAndScreens::settings_menu_display_group
//   mMenuLabelsDisplayGroup → ControllerMenusAndScreens::menu_labels_display_group
//   mCategoryMenuDisplayMarker → ControllerMenusAndScreens::category_menu_display_marker
//   mCategoryMenuState → ControllerMenusAndScreens::category_menu_state
//   mMessageDisplayObject → ControllerMenusAndScreens::message_display_object
//   mMessageSequence → ControllerMenusAndScreens::message_sequence
//   mbMenuOpen → ControllerMenusAndScreens::menu_open
//   mTargetMenuButtonZRot → ControllerMenusAndScreens::target_menu_button_z_rot
//   mTargetMenuYPos → ControllerMenusAndScreens::target_menu_y_pos
//   mNewPygmyButtonLogic → ControllerMenusAndScreens::new_pygmy_button_logic
//   mMenuButtonLogic → ControllerMenusAndScreens::menu_button_logic
//   mIslandLocationButtonLogic → ControllerMenusAndScreens::island_location_button_logic
//   mIslandMenuButtonLogic → ControllerMenusAndScreens::island_menu_button_logic
//   mGlobalIslandMenuButtonLogic → ControllerMenusAndScreens::global_island_menu_button_logic
//   mGodPowerMenuButtonLogic → ControllerMenusAndScreens::god_power_menu_button_logic
//   mCustomizeMenuButtonLogic → ControllerMenusAndScreens::customize_menu_button_logic
//   mCommunityMenuButtonLogic → ControllerMenusAndScreens::community_menu_button_logic
//   mSettingsMenuButtonLogic → ControllerMenusAndScreens::settings_menu_button_logic
//   mHelpButtonLogic → ControllerMenusAndScreens::help_button_logic
//   mDebugButtonLogic → ControllerMenusAndScreens::debug_button_logic
//   mIdolSelectionButtonLogic → ControllerMenusAndScreens::idol_selection_button_logic
//   mCrackHoleButtonLogic → ControllerMenusAndScreens::crack_hole_button_logic
//   mTheRunsButtonLogic → ControllerMenusAndScreens::the_runs_button_logic
//   mCharlieButtonLogic → ControllerMenusAndScreens::charlie_button_logic
//   mDrumButtonLogic → ControllerMenusAndScreens::drum_button_logic
//   mTemperatureButtonLogic → ControllerMenusAndScreens::temperature_button_logic
//   mHurricaneButtonLogic → ControllerMenusAndScreens::hurricane_button_logic
//   mRainbowButtonLogic → ControllerMenusAndScreens::rainbow_button_logic
//   mSkinPackButtonLogic → ControllerMenusAndScreens::skin_pack_button_logic
//   mBonusPackButtonLogic → ControllerMenusAndScreens::bonus_pack_button_logic
//   mStoryButtonLogic → ControllerMenusAndScreens::story_button_logic
//   mStatsButtonLogic → ControllerMenusAndScreens::stats_button_logic
//   mIdolsChallengeButtonLogic → ControllerMenusAndScreens::idols_challenge_button_logic
//   mNewsButtonLogic → ControllerMenusAndScreens::news_button_logic
//   mChallengeButtonLogic → ControllerMenusAndScreens::challenge_button_logic
//   mStartScreenButtonLogic → ControllerMenusAndScreens::start_screen_button_logic
//   mMuteButtonLogic → ControllerMenusAndScreens::mute_button_logic
//   mVibrateButtonLogic → ControllerMenusAndScreens::vibrate_button_logic
//   mGravityButtonLogic → ControllerMenusAndScreens::gravity_button_logic
//   mNamesButtonLogic → ControllerMenusAndScreens::names_button_logic
//   mLabelsButtonLogic → ControllerMenusAndScreens::labels_button_logic
//   mHelpPageIndex → ControllerMenusAndScreens::help_page_index
//   mbHelpScreenOpen → ControllerMenusAndScreens::help_screen_open
//   mHelpCloseButtonLogic → ControllerMenusAndScreens::help_close_button_logic
//   mHelpNextButtonLogic → ControllerMenusAndScreens::help_next_button_logic
//   mHelpPrevButtonLogic → ControllerMenusAndScreens::help_prev_button_logic
//   mbDebugScreenOpen → ControllerMenusAndScreens::debug_screen_open
//   mbWhatsNewScreenOpen → ControllerMenusAndScreens::whats_new_screen_open
//   mWhatsNewScreenGoButtonLogic → ControllerMenusAndScreens::whats_new_screen_go_button_logic
//   mbStatsScreenOpen → ControllerMenusAndScreens::stats_screen_open
//   mLavaButtonLogic → ControllerMenusAndScreens::lava_button_logic
//   mCloudFrontDisplayMarker → ControllerSkyWeather::cloud_front_display_marker
//   mCloudBehindDisplayMarker → ControllerSkyWeather::cloud_behind_display_marker
//   mBatTapTouchCount → ControllerSkyWeather::bat_tap_touch_count
//   mBatTapTouchTime → ControllerSkyWeather::bat_tap_touch_time
//   mBatLogic → ControllerSkyWeather::bat_logic
//   mWeatherSequence → ControllerSkyWeather::weather_sequence
//   mSunLogic → ControllerSkyWeather::sun_logic
//   mDragSunTouch → ControllerSkyWeather::drag_sun_touch
//   mDragSunTouchPos → ControllerSkyWeather::drag_sun_touch_pos
//   mSunReflectionDisplayObject → ControllerSkyWeather::sun_reflection_display_object
//   mbSunReflectionVisible → ControllerSkyWeather::sun_reflection_visible
//   mDragSunXOrigin → ControllerSkyWeather::drag_sun_x_origin
//   mSunState → ControllerSkyWeather::sun_state
//   mStarsLogic → ControllerSkyWeather::stars_logic
//   mMoonLogic → ControllerSkyWeather::moon_logic
//   mDragMoonTouch → ControllerSkyWeather::drag_moon_touch
//   mDragMoonTouchPos → ControllerSkyWeather::drag_moon_touch_pos
//   mbNightActive → ControllerSkyWeather::night_active
//   mbSunsetActive → ControllerSkyWeather::sunset_active
//   mCloudGradientTargetFade → ControllerSkyWeather::cloud_gradient_target_fade
//   mCloudGradientFade → ControllerSkyWeather::cloud_gradient_fade
//   mSunsetGradientTargetFade → ControllerSkyWeather::sunset_gradient_target_fade
//   mSunsetGradientFade → ControllerSkyWeather::sunset_gradient_fade
//   mNightGradientTargetFade → ControllerSkyWeather::night_gradient_target_fade
//   mNightGradientFade → ControllerSkyWeather::night_gradient_fade
//   mEclipseGradientTargetFade → ControllerSkyWeather::eclipse_gradient_target_fade
//   mEclipseGradientFade → ControllerSkyWeather::eclipse_gradient_fade
//   mBigSunGradientTargetFade → ControllerSkyWeather::big_sun_gradient_target_fade
//   mBigSunGradientFade → ControllerSkyWeather::big_sun_gradient_fade
//   mSmallSunGradientTargetFade → ControllerSkyWeather::small_sun_gradient_target_fade
//   mSmallSunGradientFade → ControllerSkyWeather::small_sun_gradient_fade
//   mApocalypseGradientTargetFade → ControllerSkyWeather::apocalypse_gradient_target_fade
//   mApocalypseGradientFade → ControllerSkyWeather::apocalypse_gradient_fade
//   mpSunsetGradient → ControllerSkyWeather::sunset_gradient
//   mpNightGradient → ControllerSkyWeather::night_gradient
//   mpDayGradient → ControllerSkyWeather::day_gradient
//   mpCloudyGradient → ControllerSkyWeather::cloudy_gradient
//   mpEclipseGradient → ControllerSkyWeather::eclipse_gradient
//   mpBigSunGradient → ControllerSkyWeather::big_sun_gradient
//   mpSmallSunGradient → ControllerSkyWeather::small_sun_gradient
//   mpApocalypseSunsetGradient → ControllerSkyWeather::apocalypse_sunset_gradient
//   mpApocalypseNightGradient → ControllerSkyWeather::apocalypse_night_gradient
//   mpApocalypseDayGradient → ControllerSkyWeather::apocalypse_day_gradient
//   mpApocalypseCloudyGradient → ControllerSkyWeather::apocalypse_cloudy_gradient
//   mWeatherState → ControllerSkyWeather::weather_state
//   mbDragClouds → ControllerSkyWeather::drag_clouds
//   mCloudDragVel → ControllerSkyWeather::cloud_drag_vel
//   mDragCloudsAveVel → ControllerSkyWeather::drag_clouds_ave_vel
//   mDragCloudsTimeStamp → ControllerSkyWeather::drag_clouds_time_stamp
//   mDragCloudsTouch → ControllerSkyWeather::drag_clouds_touch
//   mDragCloudsTouchPos → ControllerSkyWeather::drag_clouds_touch_pos
//   mCloudLogicArray → ControllerSkyWeather::cloud_logic_array
//   mFrontStormCloudLogic → ControllerSkyWeather::front_storm_cloud_logic
//   mBehindStormCloudLogic → ControllerSkyWeather::behind_storm_cloud_logic
//   mWeatherButtonLogic → ControllerSkyWeather::weather_button_logic
//   mFireworkDisplayMarker → ControllerGodPowers::firework_display_marker
//   mFireworkLogicArray → ControllerGodPowers::firework_logic_array
//   mUnderwaterPowerTrailLogicArray → ControllerGodPowers::underwater_power_trail_logic_array
//   mTapTouchCount → ControllerGodPowers::tap_touch_count
//   mTapTouchTime → ControllerGodPowers::tap_touch_time
//   mbCelebrationActive → ControllerGodPowers::celebration_active
//   mCelebrationSequence → ControllerGodPowers::celebration_sequence
//   mFireworksStallTime → ControllerGodPowers::fireworks_stall_time
//   mbThrillerActive → ControllerGodPowers::thriller_active
//   mbPantsOnTheGroundActive → ControllerGodPowers::pants_on_the_ground_active
//   mbUpdateSongActive → ControllerGodPowers::update_song_active
//   mbThemeSongActive → ControllerGodPowers::theme_song_active
//   mbFishmasActive → ControllerGodPowers::fishmas_active
//   mbFishmasTwelveDaysActive → ControllerGodPowers::fishmas_twelve_days_active
//   mbEruption → ControllerGodPowers::eruption
//   mVolcanoHitCount → ControllerGodPowers::volcano_hit_count
//   mVolcanoLogic → ControllerGodPowers::volcano_logic
//   mEarthquakeCount → ControllerGodPowers::earthquake_count
//   mEarthquakeSequence → ControllerGodPowers::earthquake_sequence
//   mShakeStallTime → ControllerGodPowers::shake_stall_time
//   mShakeRotVel → ControllerGodPowers::shake_rot_vel
//   mShakeRot → ControllerGodPowers::shake_rot
//   mHurricaneTouch → ControllerGodPowers::hurricane_touch
//   mLastHurricanePos → ControllerGodPowers::last_hurricane_pos
//   mLastHurricaneTouchTime → ControllerGodPowers::last_hurricane_touch_time
//   mpHurricaneList → ControllerGodPowers::hurricane_list
//   mHurricaneLogic → ControllerGodPowers::hurricane_logic
//   mWhirlPoolTouch → ControllerGodPowers::whirl_pool_touch
//   mpWhirlPoolList → ControllerGodPowers::whirl_pool_list
//   mWhirlPoolLogic → ControllerGodPowers::whirl_pool_logic
//   mUnderwaterPowerTouch → ControllerGodPowers::underwater_power_touch
//   mLastUnderwaterTempleMode → ControllerGodPowers::last_underwater_temple_mode
//   mLastUnderwaterPowerPos → ControllerGodPowers::last_underwater_power_pos
//   mbShockObject → ControllerGodPowers::shock_object
//   mbDrawLightning → ControllerGodPowers::draw_lightning
//   mLightningTouch → ControllerGodPowers::lightning_touch
//   mLastLightningPos → ControllerGodPowers::last_lightning_pos
//   mpLightningList → ControllerGodPowers::lightning_list
//   mLightningLogic → ControllerGodPowers::lightning_logic
//   mLightningTrailLogic → ControllerGodPowers::lightning_trail_logic
//   mHurricaneTrailLogic → ControllerGodPowers::hurricane_trail_logic
//   mWhirlPoolTrailLogic → ControllerGodPowers::whirl_pool_trail_logic
//   mUnderwaterPowerTrailLogic → ControllerGodPowers::underwater_power_trail_logic
//   mbDrawRainbow → ControllerGodPowers::draw_rainbow
//   mRainbowTouch → ControllerGodPowers::rainbow_touch
//   mLastRainbowPos → ControllerGodPowers::last_rainbow_pos
//   mpRainbowList → ControllerGodPowers::rainbow_list
//   mRainbowLogic → ControllerGodPowers::rainbow_logic
//   mRainbowTrailLogic → ControllerGodPowers::rainbow_trail_logic
//   mbDrawLocust → ControllerGodPowers::draw_locust
//   mLocustTouch → ControllerGodPowers::locust_touch
//   mLastLocustPos → ControllerGodPowers::last_locust_pos
//   mpLocustVector → ControllerGodPowers::locust_vector
//   mLocustLogic → ControllerGodPowers::locust_logic
//   mPygmyLogicArray → ControllerPygmies::pygmy_logic_array
//   mTomPygmyLogic → ControllerPygmies::tom_pygmy_logic
//   mPygmyCount → ControllerPygmies::pygmy_count
//   mLivePygmyCount → ControllerPygmies::live_pygmy_count
//   mSleepingCount → ControllerPygmies::sleeping_count
//   mDraggedGhostPygmyLogic → ControllerPygmies::dragged_ghost_pygmy_logic
//   mDraggedPygmyLogic → ControllerPygmies::dragged_pygmy_logic
//   mPiranhaLogicArray → ControllerWorldObjectEvents::piranha_logic_array
//   mFishingBobLogic → ControllerWorldObjectEvents::fishing_bob_logic
//   mbBaitMaster → ControllerWorldObjectEvents::bait_master
//   mDraggedSwordFishLogic → ControllerWorldObjectEvents::dragged_sword_fish_logic
//   mDraggedFishLogic → ControllerWorldObjectEvents::dragged_fish_logic
//   mDraggedDanceBalloonLogic → ControllerWorldObjectEvents::dragged_dance_balloon_logic
//   mDraggedBananaLogic → ControllerWorldObjectEvents::dragged_banana_logic
//   mDraggedMonkeyLogic → ControllerWorldObjectEvents::dragged_monkey_logic
//   mDraggedMoronSnakeLogic → ControllerWorldObjectEvents::dragged_moron_snake_logic
//   mDraggedJellyfishLogic → ControllerWorldObjectEvents::dragged_jellyfish_logic
//   mDraggedGodIdolLogic → ControllerWorldObjectEvents::dragged_god_idol_logic
//   mDraggedApocalypseGemLogic → ControllerWorldObjectEvents::dragged_apocalypse_gem_logic
//   mDraggedSpearLogic → ControllerWorldObjectEvents::dragged_spear_logic
//   mDraggedFishingBobLogic → ControllerWorldObjectEvents::dragged_fishing_bob_logic
//   mDraggedFishingPoleLogic → ControllerWorldObjectEvents::dragged_fishing_pole_logic
//   mDraggedWaterWingsLogic → ControllerWorldObjectEvents::dragged_water_wings_logic
//   mDraggedCoconutLogic → ControllerWorldObjectEvents::dragged_coconut_logic
//   mDraggedWoodLogic → ControllerWorldObjectEvents::dragged_wood_logic
//   mDraggedAnchorLogic → ControllerWorldObjectEvents::dragged_anchor_logic
//   mDraggedTilePlugLogic → ControllerWorldObjectEvents::dragged_tile_plug_logic
//   mDraggedBoulderLogic → ControllerWorldObjectEvents::dragged_boulder_logic
//   mDraggedTRexEggLogic → ControllerWorldObjectEvents::dragged_t_rex_egg_logic
//   mDirectedAntHillLogic → ControllerWorldObjectEvents::directed_ant_hill_logic
//   mDraggedMagnifyingGlassLogic → ControllerWorldObjectEvents::dragged_magnifying_glass_logic
//   mDraggedSharkLogic → ControllerWorldObjectEvents::dragged_shark_logic
//   mDraggedBirdLogic → ControllerWorldObjectEvents::dragged_bird_logic
//   mDraggedClosetTempleClosetDoorLogic → ControllerWorldObjectEvents::dragged_closet_temple_closet_door_logic
//   mDraggedThetanLogic → ControllerWorldObjectEvents::dragged_thetan_logic
//   mApocalypseGemLogicArray → ControllerApocalypse::apocalypse_gem_logic_array
//   mbApocalypseActive → ControllerApocalypse::apocalypse_active
//   mSharkState → ControllerIslandFeatures::shark_state
//   mBirdState → ControllerIslandFeatures::bird_state
//   mSandBoulderButtonLogic → ControllerIslandFeatures::sand_boulder_button_logic
//   mSandFishButtonLogic → ControllerIslandFeatures::sand_fish_button_logic
//   mSandAntHillButtonLogic → ControllerIslandFeatures::sand_ant_hill_button_logic
//   mSandWaterWingsButtonLogic → ControllerIslandFeatures::sand_water_wings_button_logic
//   mRockFishButtonLogic → ControllerIslandFeatures::rock_fish_button_logic
//   mRockWoodButtonLogic → ControllerIslandFeatures::rock_wood_button_logic
//   mRockEggButtonLogic → ControllerIslandFeatures::rock_egg_button_logic
//   mRockOutHouseButtonLogic → ControllerIslandFeatures::rock_out_house_button_logic
//   mRockTRexButtonLogic → ControllerIslandFeatures::rock_t_rex_button_logic
//   mRockSpearButtonLogic → ControllerIslandFeatures::rock_spear_button_logic
//   mGraveyardZombieGraveButtonLogic → ControllerIslandFeatures::graveyard_zombie_grave_button_logic
//   mGraveyardGhostButtonLogic → ControllerIslandFeatures::graveyard_ghost_button_logic
//   mGraveyardSpiderWebButtonLogic → ControllerIslandFeatures::graveyard_spider_web_button_logic
//   mGraveyardSpiderButtonLogic → ControllerIslandFeatures::graveyard_spider_button_logic
//   mGraveyardSpearButtonLogic → ControllerIslandFeatures::graveyard_spear_button_logic
//   mGraveyardWaterWingsButtonLogic → ControllerIslandFeatures::graveyard_water_wings_button_logic
//   mIceFishButtonLogic → ControllerIslandFeatures::ice_fish_button_logic
//   mIceWaterWingsButtonLogic → ControllerIslandFeatures::ice_water_wings_button_logic
//   mIceIceMonsterButtonLogic → ControllerIslandFeatures::ice_ice_monster_button_logic
//   mIceIglooButtonLogic → ControllerIslandFeatures::ice_igloo_button_logic
//   mIceIceHoleButtonLogic → ControllerIslandFeatures::ice_ice_hole_button_logic
//   mIceIceSquidButtonLogic → ControllerIslandFeatures::ice_ice_squid_button_logic
//   mApeApeAltarButtonLogic → ControllerIslandFeatures::ape_ape_altar_button_logic
//   mApeApeButtonLogic → ControllerIslandFeatures::ape_ape_button_logic
//   mApeBananaTreeButtonLogic → ControllerIslandFeatures::ape_banana_tree_button_logic
//   mApeMonkeyButtonLogic → ControllerIslandFeatures::ape_monkey_button_logic
//   mApeKonkeyDongButtonLogic → ControllerIslandFeatures::ape_konkey_dong_button_logic
//   mMisfitMoronSnakeButtonLogic → ControllerIslandFeatures::misfit_moron_snake_button_logic
//   mMisfitWaterWingsButtonLogic → ControllerIslandFeatures::misfit_water_wings_button_logic
//   mMisfitMoronPestsButtonLogic → ControllerIslandFeatures::misfit_moron_pests_button_logic
//   mIdolsButtonLogic → ControllerIslandFeatures::idols_button_logic
//   mIdolWaterWingsButtonLogic → ControllerIslandFeatures::idol_water_wings_button_logic
//   mApocalypseWaterWingsButtonLogic → ControllerIslandFeatures::apocalypse_water_wings_button_logic
//   mUnderwaterPiranhaButtonLogic → ControllerIslandFeatures::underwater_piranha_button_logic
//   mUnderwaterLaserSharkButtonLogic → ControllerIslandFeatures::underwater_laser_shark_button_logic
//   mUnderwaterTileDrainButtonLogic → ControllerIslandFeatures::underwater_tile_drain_button_logic
//   mUnderwaterCoralReefJellyfishButtonLogic → ControllerIslandFeatures::underwater_coral_reef_jellyfish_button_logic
//   mSharkButtonLogic → ControllerIslandFeatures::shark_button_logic
//   mBirdButtonLogic → ControllerIslandFeatures::bird_button_logic
//   mSandIslandState → ControllerIslandFeatures::sand_island_state
//   mRockIslandState → ControllerIslandFeatures::rock_island_state
//   mRockIslandFeatureState → ControllerIslandFeatures::rock_island_feature_state
//   mRockIslandOutHouseState → ControllerIslandFeatures::rock_island_out_house_state
//   mRockIslandTRexState → ControllerIslandFeatures::rock_island_t_rex_state
//   mRockIslandSpearState → ControllerIslandFeatures::rock_island_spear_state
//   mGraveyardIslandState → ControllerIslandFeatures::graveyard_island_state
//   mGraveyardIslandZombieGraveState → ControllerIslandFeatures::graveyard_island_zombie_grave_state
//   mGraveyardIslandGhostState → ControllerIslandFeatures::graveyard_island_ghost_state
//   mGraveyardIslandSpiderWebState → ControllerIslandFeatures::graveyard_island_spider_web_state
//   mGraveyardIslandSpiderState → ControllerIslandFeatures::graveyard_island_spider_state
//   mGraveyardIslandSpearState → ControllerIslandFeatures::graveyard_island_spear_state
//   mIceIslandState → ControllerIslandFeatures::ice_island_state
//   mIceIslandIceMonsterState → ControllerIslandFeatures::ice_island_ice_monster_state
//   mIceIslandIglooState → ControllerIslandFeatures::ice_island_igloo_state
//   mIceIslandIceHoleState → ControllerIslandFeatures::ice_island_ice_hole_state
//   mIceIslandIceSquidState → ControllerIslandFeatures::ice_island_ice_squid_state
//   mApeIslandState → ControllerIslandFeatures::ape_island_state
//   mApeIslandApeState → ControllerIslandFeatures::ape_island_ape_state
//   mApeIslandMonkeyState → ControllerIslandFeatures::ape_island_monkey_state
//   mMisfitIslandState → ControllerIslandFeatures::misfit_island_state
//   mGodIslandState → ControllerIslandFeatures::god_island_state
//   mGodIslandIdolsState → ControllerIslandFeatures::god_island_idols_state
//   mApocalypseIslandState → ControllerIslandFeatures::apocalypse_island_state
//   mFailBoatState → ControllerIslandFeatures::fail_boat_state
//   mUnderwaterState → ControllerIslandFeatures::underwater_state
//   mUnderwaterCoralReefState → ControllerIslandFeatures::underwater_coral_reef_state

pub const POCKET_GOD_VIEW_CONTROLLER: Class = Class {
    name: "PocketGodViewController",
    superclass: None,
    game_frame_order: 50,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    on_game_frame,
    on_multi_model_event,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `PocketGodViewController` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "playBackgroundLoop" => seq_play_background_loop(world, this, call),
        "stopBackgroundLoop" => seq_stop_background_loop(world, this, call),
        "pauseHurricaneSounds" => seq_pause_hurricane_sounds(world, this, call),
        "resumeHurricaneSounds" => seq_resume_hurricane_sounds(world, this, call),
        "updateHelpPage" => seq_update_help_page(world, this, call),
        "startTime" => seq_start_time(world, this, call),
        "stopTime" => seq_stop_time(world, this, call),
        "waitForHoldLoadingAd" => seq_wait_for_hold_loading_ad(world, this, call),
        "waitForPreferencesLoadComplete" => seq_wait_for_preferences_load_complete(world, this, call),
        "loadPreferences" => seq_load_preferences(world, this, call),
        "loadTextureAtlasChannel" => seq_load_texture_atlas_channel(world, this, call),
        "refreshSkinPackOption" => seq_refresh_skin_pack_option(world, this, call),
        "enableTouch" => seq_enable_touch(world, this, call),
        "checkShowAuthenticationViewController" => seq_check_show_authentication_view_controller(world, this, call),
        "checkStartScreen" => seq_check_start_screen(world, this, call),
        "hideLoadingScreen" => seq_hide_loading_screen(world, this, call),
        "checkForNewNews" => seq_check_for_new_news(world, this, call),
        "openMenu" => seq_open_menu(world, this, call),
        "closeMenu" => seq_close_menu(world, this, call),
        "showMessage" => seq_show_message(world, this, call),
        "removeMessage" => seq_remove_message(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        "startPygmiesDancing" => seq_start_pygmies_dancing(world, this, call),
        "stopPygmiesDancing" => seq_stop_pygmies_dancing(world, this, call),
        "triggerEpisodeEvent" => seq_trigger_episode_event(world, this, call),
        "fireworks" => seq_fireworks(world, this, call),
        "addPygmy" => seq_add_pygmy(world, this, call),
        "addBabyPygmy" => seq_add_baby_pygmy(world, this, call),
        "addTomPygmy" => seq_add_tom_pygmy(world, this, call),
        "setLogic" => seq_set_logic(world, this, call),
        "setCloudGradientFade" => seq_set_cloud_gradient_fade(world, this, call),
        "shakeBackground" => seq_shake_background(world, this, call),
        "stopEarthquake" => seq_stop_earthquake(world, this, call),
        "stopTheRunsShakeScreen" => seq_stop_the_runs_shake_screen(world, this, call),
        "transitionInStormClouds" => seq_transition_in_storm_clouds(world, this, call),
        "transitionOutStormClouds" => seq_transition_out_storm_clouds(world, this, call),
        "transitionInClouds" => seq_transition_in_clouds(world, this, call),
        "transitionOutClouds" => seq_transition_out_clouds(world, this, call),
        "pauseSleepingSounds" => seq_pause_sleeping_sounds(world, this, call),
        "resumeSleepingSounds" => seq_resume_sleeping_sounds(world, this, call),
        "pauseLocustSounds" => seq_pause_locust_sounds(world, this, call),
        "resumeLocustSounds" => seq_resume_locust_sounds(world, this, call),
        "showTheRunsDisplayGroup" => seq_show_the_runs_display_group(world, this, call),
        "showKonkeyDongDisplayGroup" => seq_show_konkey_dong_display_group(world, this, call),
        "showMoronPestsDisplayGroup" => seq_show_moron_pests_display_group(world, this, call),
        "showNoPeeDisplayGroup" => seq_show_no_pee_display_group(world, this, call),
        "showBattleOfTheGodsDisplayGroup" => seq_show_battle_of_the_gods_display_group(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
