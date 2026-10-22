//! `PygmyLogic` (superclass `SpriteLogic`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod apocalypse_1;
pub use apocalypse_1::*;
pub mod apocalypse_2;
pub use apocalypse_2::*;
pub mod charlie_story;
pub use charlie_story::*;
pub mod containers_and_catchers;
pub use containers_and_catchers::*;
pub mod ghost_zombie_vampire;
pub use ghost_zombie_vampire::*;
pub mod island_hazards_1;
pub use island_hazards_1::*;
pub mod island_hazards_2;
pub use island_hazards_2::*;
pub mod island_hazards_3;
pub use island_hazards_3::*;
pub mod island_life_1;
pub use island_life_1::*;
pub mod island_life_2;
pub use island_life_2::*;
pub mod minigames;
pub use minigames::*;
pub mod pygmy_behaviour_items;
pub use pygmy_behaviour_items::*;
pub mod pygmy_core_frame;
pub use pygmy_core_frame::*;
pub mod pygmy_touch_drag;
pub use pygmy_touch_drag::*;
pub mod underwater_nopee;
pub use underwater_nopee::*;

/// `PygmyLogic` ivars not owned by a unit component.
#[derive(Component, Clone, Debug, Default)]
pub struct PygmyLogic {
}

/// Adds every ivar component of `PygmyLogic` (all zero, like `alloc`).
pub fn insert_components(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(PygmyLogic::default());
    world.entity_mut(this).insert(PygmyPygmyCoreFrame::default());
    world.entity_mut(this).insert(PygmyPygmyTouchDrag::default());
    world.entity_mut(this).insert(PygmyPygmyBehaviourItems::default());
    world.entity_mut(this).insert(PygmyUnderwaterNopee::default());
    world.entity_mut(this).insert(PygmyIslandLife::default());
    world.entity_mut(this).insert(PygmyIslandHazards::default());
    world.entity_mut(this).insert(PygmyGhostZombieVampire::default());
    world.entity_mut(this).insert(PygmyContainersAndCatchers::default());
    world.entity_mut(this).insert(PygmyApocalypse::default());
    world.entity_mut(this).insert(PygmyCharlieStory::default());
    world.entity_mut(this).insert(PygmyMinigames::default());
}

// Ivar → component index for `PygmyLogic`:
//   mPygmyLogicListener → PygmyPygmyCoreFrame::pygmy_logic_listener
//   mPygmyPersona → PygmyPygmyCoreFrame::pygmy_persona
//   mPersonaPersonality → PygmyPygmyCoreFrame::persona_personality
//   mSpringForce → PygmyPygmyCoreFrame::spring_force
//   mLastPos → PygmyPygmyCoreFrame::last_pos
//   mGravityTarget → PygmyPygmyCoreFrame::gravity_target
//   mIslandYPos → PygmyPygmyCoreFrame::island_y_pos
//   mSlideFriction → PygmyPygmyCoreFrame::slide_friction
//   mActiveTouch → PygmyPygmyCoreFrame::active_touch
//   mbShakeUp → PygmyPygmyCoreFrame::shake_up
//   mShakeCount → PygmyPygmyCoreFrame::shake_count
//   mLastShakeTime → PygmyPygmyCoreFrame::last_shake_time
//   mbEarthquake → PygmyPygmyCoreFrame::earthquake
//   mTiltState → PygmyPygmyCoreFrame::tilt_state
//   mbOnIsland → PygmyPygmyCoreFrame::on_island
//   mCookTime → PygmyPygmyCoreFrame::cook_time
//   mbDroppedByHurricane → PygmyPygmyCoreFrame::dropped_by_hurricane
//   mDragLabelXOffset → PygmyPygmyCoreFrame::drag_label_x_offset
//   mDragLabelYOffset → PygmyPygmyCoreFrame::drag_label_y_offset
//   mNameTextXOffset → PygmyPygmyCoreFrame::name_text_x_offset
//   mNameTextYOffset → PygmyPygmyCoreFrame::name_text_y_offset
//   mNameBackground → PygmyPygmyCoreFrame::name_background
//   mNameText → PygmyPygmyCoreFrame::name_text
//   mName → PygmyPygmyCoreFrame::name
//   mAshPileDisplayObject → PygmyPygmyCoreFrame::ash_pile_display_object
//   mbEnableIslandFloorCollision → PygmyPygmyCoreFrame::enable_island_floor_collision
//   mbEnableIslandSideCollision → PygmyPygmyCoreFrame::enable_island_side_collision
//   mbIslandSideCollision → PygmyPygmyCoreFrame::island_side_collision
//   mbScreenSideCollision → PygmyPygmyCoreFrame::screen_side_collision
//   mbInvalidSacrifice → PygmyPygmyCoreFrame::invalid_sacrifice
//   mSacrificeId → PygmyPygmyCoreFrame::sacrifice_id
//   mbEatenByPiranha → PygmyPygmyCoreFrame::eaten_by_piranha
//   mbIsGhost → PygmyPygmyCoreFrame::is_ghost
//   mbTouchedByGhost → PygmyPygmyCoreFrame::touched_by_ghost
//   mbCarriedAwayByGhost → PygmyPygmyCoreFrame::carried_away_by_ghost
//   mbHoldEarsFromBansheeScream → PygmyPygmyCoreFrame::hold_ears_from_banshee_scream
//   mbExplodeFromBansheeScream → PygmyPygmyCoreFrame::explode_from_banshee_scream
//   mbCaughtByMonkey → PygmyPygmyCoreFrame::caught_by_monkey
//   mbMonkeyOnBack → PygmyPygmyCoreFrame::monkey_on_back
//   mbEatenByMoronSnake → PygmyPygmyCoreFrame::eaten_by_moron_snake
//   mbTrappedInZombieGrave → PygmyPygmyCoreFrame::trapped_in_zombie_grave
//   mbIsZombie → PygmyPygmyCoreFrame::is_zombie
//   mbZombieExplode → PygmyPygmyCoreFrame::zombie_explode
//   mbTrappedInIdolPedestal → PygmyPygmyCoreFrame::trapped_in_idol_pedestal
//   mbInsideTempleDoor → PygmyPygmyCoreFrame::inside_temple_door
//   mbInsideApocalypseTempleDoor → PygmyPygmyCoreFrame::inside_apocalypse_temple_door
//   mbOnTopApocalypseTimeTemplePlatform → PygmyPygmyCoreFrame::on_top_apocalypse_time_temple_platform
//   mbEatenByLocustQueen → PygmyPygmyCoreFrame::eaten_by_locust_queen
//   mbThrownByLocusts → PygmyPygmyCoreFrame::thrown_by_locusts
//   mbKilledByLocusts → PygmyPygmyCoreFrame::killed_by_locusts
//   mbMutatedByLocusts → PygmyPygmyCoreFrame::mutated_by_locusts
//   mbKilledByPlague → PygmyPygmyCoreFrame::killed_by_plague
//   mbKilledByPlagueDisappear → PygmyPygmyCoreFrame::killed_by_plague_disappear
//   mbKilledByPurification → PygmyPygmyCoreFrame::killed_by_purification
//   mbInApocalypseTempleFountain → PygmyPygmyCoreFrame::in_apocalypse_temple_fountain
//   mbPossessedByThetan → PygmyPygmyCoreFrame::possessed_by_thetan
//   mbThetanBrawl → PygmyPygmyCoreFrame::thetan_brawl
//   mbDiedFromThetanBrawl → PygmyPygmyCoreFrame::died_from_thetan_brawl
//   mbOnClosetTempleThetanMeter → PygmyPygmyCoreFrame::on_closet_temple_thetan_meter
//   mbOnClosetTempleCouch → PygmyPygmyCoreFrame::on_closet_temple_couch
//   mbInClosetTempleClosetDoor → PygmyPygmyCoreFrame::in_closet_temple_closet_door
//   mbSlammingClosetTempleClosetDoor → PygmyPygmyCoreFrame::slamming_closet_temple_closet_door
//   mbHeldByXenu → PygmyPygmyCoreFrame::held_by_xenu
//   mbHeldInTomsArms → PygmyPygmyCoreFrame::held_in_toms_arms
//   mbHoldingBabyPygmy → PygmyPygmyCoreFrame::holding_baby_pygmy
//   mTimeTemplePygmyType → PygmyPygmyCoreFrame::time_temple_pygmy_type
//   mbTrappedInSpiderWeb → PygmyPygmyCoreFrame::trapped_in_spider_web
//   mbOnFailBoat → PygmyPygmyCoreFrame::on_fail_boat
//   mbCutInHalfByCharlie → PygmyPygmyCoreFrame::cut_in_half_by_charlie
//   mbSunBurnt → PygmyPygmyCoreFrame::sun_burnt
//   mbSunFrozen → PygmyPygmyCoreFrame::sun_frozen
//   mbHitByIcicle → PygmyPygmyCoreFrame::hit_by_icicle
//   mbBeheadedBySquid → PygmyPygmyCoreFrame::beheaded_by_squid
//   mbAteSwordFish → PygmyPygmyCoreFrame::ate_sword_fish
//   mbPainDrain → PygmyPygmyCoreFrame::pain_drain
//   mbPainDrainSpiked → PygmyPygmyCoreFrame::pain_drain_spiked
//   mbSuckedIntoDrain → PygmyPygmyCoreFrame::sucked_into_drain
//   mbIglooRoll → PygmyPygmyCoreFrame::igloo_roll
//   mbHitBySnowBall → PygmyPygmyCoreFrame::hit_by_snow_ball
//   mbRestingInIgloo → PygmyPygmyCoreFrame::resting_in_igloo
//   mbInApeWench → PygmyPygmyCoreFrame::in_ape_wench
//   mApeWenchLogic → PygmyPygmyCoreFrame::ape_wench_logic
//   mbInApeBongo → PygmyPygmyCoreFrame::in_ape_bongo
//   mApeBongoLogic → PygmyPygmyCoreFrame::ape_bongo_logic
//   mbInApeAltar → PygmyPygmyCoreFrame::in_ape_altar
//   mApeAltarLogic → PygmyPygmyCoreFrame::ape_altar_logic
//   mbInDanceDanceExecution → PygmyPygmyCoreFrame::in_dance_dance_execution
//   mbDanceDanceExecuted → PygmyPygmyCoreFrame::dance_dance_executed
//   mbInGoGoCage → PygmyPygmyCoreFrame::in_go_go_cage
//   mbHighOnNitrous → PygmyPygmyCoreFrame::high_on_nitrous
//   mbRemoveFromStoryMode → PygmyPygmyCoreFrame::remove_from_story_mode
//   mbCaughtByBird → PygmyPygmyCoreFrame::caught_by_bird
//   mbInCrackHole → PygmyPygmyCoreFrame::in_crack_hole
//   mbTheRunsSideCollision → PygmyPygmyCoreFrame::the_runs_side_collision
//   mbTheRunsBurnComplete → PygmyPygmyCoreFrame::the_runs_burn_complete
//   mbTheRunsFallInLava → PygmyPygmyCoreFrame::the_runs_fall_in_lava
//   mbKonkeyDongHit → PygmyPygmyCoreFrame::konkey_dong_hit
//   mKonkeyDongPickTime → PygmyPygmyCoreFrame::konkey_dong_pick_time
//   mbMoronPests → PygmyPygmyCoreFrame::moron_pests
//   mbMoronPestsStarted → PygmyPygmyCoreFrame::moron_pests_started
//   mbMoronPestsEatenByTurtle → PygmyPygmyCoreFrame::moron_pests_eaten_by_turtle
//   mbWearingWaterWings → PygmyPygmyCoreFrame::wearing_water_wings
//   mbInKongStatue → PygmyPygmyCoreFrame::in_kong_statue
//   mbKidnappedByKonkeyDong → PygmyPygmyCoreFrame::kidnapped_by_konkey_dong
//   mKongStatueLogic → PygmyPygmyCoreFrame::kong_statue_logic
//   mbBattleOfTheGods → PygmyPygmyCoreFrame::battle_of_the_gods
//   mbDoodleJumping → PygmyPygmyCoreFrame::doodle_jumping
//   mbDoodleKnockedDown → PygmyPygmyCoreFrame::doodle_knocked_down
//   mbSkewered → PygmyPygmyCoreFrame::skewered
//   mbUnderwater → PygmyPygmyCoreFrame::underwater
//   mbCrapping → PygmyPygmyCoreFrame::crapping
//   mbEatenByTRex → PygmyPygmyCoreFrame::eaten_by_t_rex
//   mbEatenBySpider → PygmyPygmyCoreFrame::eaten_by_spider
//   mbExplodeFromNotCrapping → PygmyPygmyCoreFrame::explode_from_not_crapping
//   mbExplodeFromVampireSun → PygmyPygmyCoreFrame::explode_from_vampire_sun
//   mbExplodeFromVampireGinger → PygmyPygmyCoreFrame::explode_from_vampire_ginger
//   mbExplodeFromLovingPocketGod → PygmyPygmyCoreFrame::explode_from_loving_pocket_god
//   mbDisableLabels → PygmyPygmyCoreFrame::disable_labels
//   mbEnableUpsideDown → PygmyPygmyCoreFrame::enable_upside_down
//   mbTouchBeganInside → PygmyPygmyCoreFrame::touch_began_inside
//   mbCatchShark → PygmyPygmyCoreFrame::catch_shark
//   mbSharkOnHead → PygmyPygmyCoreFrame::shark_on_head
//   mbCrushed → PygmyPygmyCoreFrame::crushed
//   mbBurntByMagnifyingGlass → PygmyPygmyCoreFrame::burnt_by_magnifying_glass
//   mbSwarmed → PygmyPygmyCoreFrame::swarmed
//   mbSleeping → PygmyPygmyCoreFrame::sleeping
//   mbTumble → PygmyPygmyCoreFrame::tumble
//   mbDrowning → PygmyPygmyCoreFrame::drowning
//   mbDragging → PygmyPygmyCoreFrame::dragging
//   mbFalling → PygmyPygmyCoreFrame::falling
//   mbApplySpring → PygmyPygmyCoreFrame::apply_spring
//   mbOnFireFromLava → PygmyPygmyCoreFrame::on_fire_from_lava
//   mbOnFireFromBonFire → PygmyPygmyCoreFrame::on_fire_from_bon_fire
//   mbAteBurntFood → PygmyPygmyCoreFrame::ate_burnt_food
//   mbCooked → PygmyPygmyCoreFrame::cooked
//   mbHitByLightning → PygmyPygmyCoreFrame::hit_by_lightning
//   mTouchScreenPos → PygmyPygmyTouchDrag::touch_screen_pos
//   mOhStall → PygmyPygmyTouchDrag::oh_stall
//   mbOverFire → PygmyPygmyTouchDrag::over_fire
//   mbTheRuns → PygmyPygmyTouchDrag::the_runs
//   mbKonkeyDong → PygmyPygmyTouchDrag::konkey_dong
//   mbNoPee → PygmyPygmyTouchDrag::no_pee
//   mKonkeyDongLadderTouch → PygmyPygmyTouchDrag::konkey_dong_ladder_touch
//   mTumblePos → PygmyPygmyBehaviourItems::tumble_pos
//   mNextTurnStallTime → PygmyPygmyBehaviourItems::next_turn_stall_time
//   mBurningMagnifyingGlassLogic → PygmyPygmyBehaviourItems::burning_magnifying_glass_logic
//   mbHitIceMonster → PygmyPygmyBehaviourItems::hit_ice_monster
//   mAshPileLogic → PygmyPygmyBehaviourItems::ash_pile_logic
//   mbEnableSlide → PygmyPygmyBehaviourItems::enable_slide
//   mbIslandFloorCollision → PygmyPygmyBehaviourItems::island_floor_collision
//   mStartFallYPos → PygmyPygmyBehaviourItems::start_fall_y_pos
//   mbBrainsEaten → PygmyPygmyBehaviourItems::brains_eaten
//   mbMouthFilledByLocusts → PygmyPygmyBehaviourItems::mouth_filled_by_locusts
//   mbTellClosetToGrabNewTom → PygmyPygmyBehaviourItems::tell_closet_to_grab_new_tom
//   mbHadHeartAttack → PygmyPygmyBehaviourItems::had_heart_attack
//   mbIglooCannon → PygmyPygmyBehaviourItems::igloo_cannon
//   mbTornApartByApeAltar → PygmyPygmyBehaviourItems::torn_apart_by_ape_altar
//   mbFartherThanIceMonster → PygmyPygmyBehaviourItems::farther_than_ice_monster
//   mbDescending → PygmyPygmyBehaviourItems::descending
//   mbHitVolcano → PygmyPygmyBehaviourItems::hit_volcano
//   mbScreamDuringFall → PygmyPygmyBehaviourItems::scream_during_fall
//   mEatingPiranhaLogic → PygmyUnderwaterNopee::eating_piranha_logic
//   mbBaitMaster → PygmyUnderwaterNopee::bait_master
//   mHoldingChumLogic → PygmyUnderwaterNopee::holding_chum_logic
//   mbHoldingChum → PygmyUnderwaterNopee::holding_chum
//   mbPainDrainPlatformCollision → PygmyUnderwaterNopee::pain_drain_platform_collision
//   mbPainDrainCanTurn → PygmyUnderwaterNopee::pain_drain_can_turn
//   mbPainDrainTurnRight → PygmyUnderwaterNopee::pain_drain_turn_right
//   mbNoPeeIntroComplete → PygmyUnderwaterNopee::no_pee_intro_complete
//   mbNoPeeStarted → PygmyUnderwaterNopee::no_pee_started
//   mbNoPeeKilled → PygmyUnderwaterNopee::no_pee_killed
//   mbNoPeeThrustOn → PygmyUnderwaterNopee::no_pee_thrust_on
//   mbNoPeeChangeDirection → PygmyUnderwaterNopee::no_pee_change_direction
//   mbNoPeeTurnToStone → PygmyUnderwaterNopee::no_pee_turn_to_stone
//   mNoPeeThrustVel → PygmyUnderwaterNopee::no_pee_thrust_vel
//   mNoPeeTurnAroundTime → PygmyUnderwaterNopee::no_pee_turn_around_time
//   mNoPeeSwimXVel → PygmyUnderwaterNopee::no_pee_swim_x_vel
//   mNoPeeSwimTargetXVel → PygmyUnderwaterNopee::no_pee_swim_target_x_vel
//   mbNoPeeSwimmingLeft → PygmyUnderwaterNopee::no_pee_swimming_left
//   mbNoPeeChangingDirectionDownward → PygmyUnderwaterNopee::no_pee_changing_direction_downward
//   mNoPeePosVel → PygmyUnderwaterNopee::no_pee_pos_vel
//   mNoPeeEmbedLogic → PygmyUnderwaterNopee::no_pee_embed_logic
//   mJellyfishLogic → PygmyUnderwaterNopee::jellyfish_logic
//   mbAddedUnderwater → PygmyUnderwaterNopee::added_underwater
//   mbActiveUnderwater → PygmyUnderwaterNopee::active_underwater
//   mbSkeweredRight → PygmyUnderwaterNopee::skewered_right
//   mbHitBySharkLaser → PygmyUnderwaterNopee::hit_by_shark_laser
//   mbEatenByLaserShark → PygmyUnderwaterNopee::eaten_by_laser_shark
//   mbHitByJellyfish → PygmyUnderwaterNopee::hit_by_jellyfish
//   mbHitByUnderwaterPower → PygmyUnderwaterNopee::hit_by_underwater_power
//   mEatingSharkLogic → PygmyIslandLife::eating_shark_logic
//   mEatingCoconutLogic → PygmyIslandLife::eating_coconut_logic
//   mCarryingWoodLogic → PygmyIslandLife::carrying_wood_logic
//   mEatingFishLogic → PygmyIslandLife::eating_fish_logic
//   mEatingSwordFishLogic → PygmyIslandLife::eating_sword_fish_logic
//   mEatingBirdLogic → PygmyIslandLife::eating_bird_logic
//   mEatingBananaLogic → PygmyIslandLife::eating_banana_logic
//   mFishingPoleLogic → PygmyIslandLife::fishing_pole_logic
//   mbThrowingSpear → PygmyIslandLife::throwing_spear
//   mThrowingSpearLogic → PygmyIslandLife::throwing_spear_logic
//   mFollowOutHouseStall → PygmyIslandLife::follow_out_house_stall
//   mCrappingOutHouseLogic → PygmyIslandLife::crapping_out_house_logic
//   mWaitingOutHouseLogic → PygmyIslandLife::waiting_out_house_logic
//   mLeftWarmingBonFireLogic → PygmyIslandLife::left_warming_bon_fire_logic
//   mRightWarmingBonFireLogic → PygmyIslandLife::right_warming_bon_fire_logic
//   mThrowingMonkeyLogic → PygmyIslandLife::throwing_monkey_logic
//   mbPlayingCatchWithMonkey → PygmyIslandLife::playing_catch_with_monkey
//   mHumpingMonkeyLogic → PygmyIslandLife::humping_monkey_logic
//   mDreamBubbleLogic → PygmyIslandLife::dream_bubble_logic
//   mbCharlieNotSleeping → PygmyIslandLife::charlie_not_sleeping
//   mbIceFishing → PygmyIslandLife::ice_fishing
//   mbCrawledOutOfIgloo → PygmyIslandLife::crawled_out_of_igloo
//   mbStaringAtRainbow → PygmyIslandLife::staring_at_rainbow
//   mbGushingOverRainbow → PygmyIslandLife::gushing_over_rainbow
//   mbStaringAtKidnapping → PygmyIslandLife::staring_at_kidnapping
//   mbStoppingDance → PygmyIslandLife::stopping_dance
//   mbIsThemeSongBackup → PygmyIslandLife::is_theme_song_backup
//   mLastDanceSkinOptionValueId → PygmyIslandLife::last_dance_skin_option_value_id
//   mbFinishedCrapping → PygmyIslandLife::finished_crapping
//   mSafeIslandMinXPos → PygmyIslandHazards::safe_island_min_x_pos
//   mSafeIslandMaxXPos → PygmyIslandHazards::safe_island_max_x_pos
//   mSafeRunIslandMinXPos → PygmyIslandHazards::safe_run_island_min_x_pos
//   mSafeRunIslandMaxXPos → PygmyIslandHazards::safe_run_island_max_x_pos
//   mbGingerSunBurn → PygmyIslandHazards::ginger_sun_burn
//   mbWarlockSkipNextExplosion → PygmyIslandHazards::warlock_skip_next_explosion
//   mbIceShatter → PygmyIslandHazards::ice_shatter
//   mSunBurnFreezeCycleCount → PygmyIslandHazards::sun_burn_freeze_cycle_count
//   mbGrabbedByTentacle → PygmyIslandHazards::grabbed_by_tentacle
//   mTentacleStrengthRot → PygmyIslandHazards::tentacle_strength_rot
//   mTentacleStrengthRotVel → PygmyIslandHazards::tentacle_strength_rot_vel
//   mIceHoleLogic → PygmyIslandHazards::ice_hole_logic
//   mCarryingBirdLogic → PygmyIslandHazards::carrying_bird_logic
//   mbCrushedByBoulder → PygmyIslandHazards::crushed_by_boulder
//   mbGhostFollowingPath → PygmyGhostZombieVampire::ghost_following_path
//   mbGhostFlipping → PygmyGhostZombieVampire::ghost_flipping
//   mGhostTouchXOffset → PygmyGhostZombieVampire::ghost_touch_x_offset
//   mGhostTouchYOffset → PygmyGhostZombieVampire::ghost_touch_y_offset
//   mGhostTargetZRot → PygmyGhostZombieVampire::ghost_target_z_rot
//   mGhostTargetYPos → PygmyGhostZombieVampire::ghost_target_y_pos
//   mpGhostPathVector → PygmyGhostZombieVampire::ghost_path_vector
//   mGhostLastPathPos → PygmyGhostZombieVampire::ghost_last_path_pos
//   mGhostLastTouchPos → PygmyGhostZombieVampire::ghost_last_touch_pos
//   mGhostTouchPygmyLogic → PygmyGhostZombieVampire::ghost_touch_pygmy_logic
//   mGhostTouchType → PygmyGhostZombieVampire::ghost_touch_type
//   mGhostEmergeXPos → PygmyGhostZombieVampire::ghost_emerge_x_pos
//   mbGhostBansheeScream → PygmyGhostZombieVampire::ghost_banshee_scream
//   mbGhostBansheeScreamStarted → PygmyGhostZombieVampire::ghost_banshee_scream_started
//   mTheLightDisplayObject → PygmyGhostZombieVampire::the_light_display_object
//   mTheLightLogic → PygmyGhostZombieVampire::the_light_logic
//   mbGoIntoTheLight → PygmyGhostZombieVampire::go_into_the_light
//   mbGhostActionTriggered → PygmyGhostZombieVampire::ghost_action_triggered
//   mGhostPygmyLogic → PygmyGhostZombieVampire::ghost_pygmy_logic
//   mEatingBrainsPygmyLogic → PygmyGhostZombieVampire::eating_brains_pygmy_logic
//   mbZombieChooseDirection → PygmyGhostZombieVampire::zombie_choose_direction
//   mbBitByBat → PygmyGhostZombieVampire::bit_by_bat
//   mZombieGraveLogic → PygmyContainersAndCatchers::zombie_grave_logic
//   mIdolPedestalLogic → PygmyContainersAndCatchers::idol_pedestal_logic
//   mTempleDoorLogic → PygmyContainersAndCatchers::temple_door_logic
//   mApocalypseTempleDoorLogic → PygmyContainersAndCatchers::apocalypse_temple_door_logic
//   mApocalypseTimeTemplePlatformLogic → PygmyContainersAndCatchers::apocalypse_time_temple_platform_logic
//   mLocustQueenLogic → PygmyContainersAndCatchers::locust_queen_logic
//   mApocalypseTempleFountainLogic → PygmyContainersAndCatchers::apocalypse_temple_fountain_logic
//   mIglooLogic → PygmyContainersAndCatchers::igloo_logic
//   mDanceDanceExecutionLogic → PygmyContainersAndCatchers::dance_dance_execution_logic
//   mGoGoCageLogic → PygmyContainersAndCatchers::go_go_cage_logic
//   mbUseLocustCatch1 → PygmyApocalypse::use_locust_catch1
//   mThrowingLocustLogic → PygmyApocalypse::throwing_locust_logic
//   mMutantLocustIdleLoopCount → PygmyApocalypse::mutant_locust_idle_loop_count
//   mDiseaseIncubateIdleCycles → PygmyApocalypse::disease_incubate_idle_cycles
//   mbPlagueBodyInPile → PygmyApocalypse::plague_body_in_pile
//   mbPlagueBodyPickedUp → PygmyApocalypse::plague_body_picked_up
//   mbPlagueBodyBeingCarried → PygmyApocalypse::plague_body_being_carried
//   mPlagueBodyCarryingPygmyLogic → PygmyApocalypse::plague_body_carrying_pygmy_logic
//   mCarryingPlagueBodyLogic → PygmyApocalypse::carrying_plague_body_logic
//   mbWaitingForPossessionByThetan → PygmyApocalypse::waiting_for_possession_by_thetan
//   mThetanLogic → PygmyApocalypse::thetan_logic
//   mThetanBrawlCoolDown → PygmyApocalypse::thetan_brawl_cool_down
//   mThetanAttackerPygmyLogic → PygmyApocalypse::thetan_attacker_pygmy_logic
//   mThetanDefenderPygmyLogic → PygmyApocalypse::thetan_defender_pygmy_logic
//   mClosetTempleThetanMeterLogic → PygmyApocalypse::closet_temple_thetan_meter_logic
//   mClosetTempleCouchLogic → PygmyApocalypse::closet_temple_couch_logic
//   mClosetTempleClosetDoorLogic → PygmyApocalypse::closet_temple_closet_door_logic
//   mXenuLogic → PygmyApocalypse::xenu_logic
//   mbStartTomBabySacrifice → PygmyApocalypse::start_tom_baby_sacrifice
//   mbSacrificedToXenu → PygmyApocalypse::sacrificed_to_xenu
//   mTomPygmyLogic → PygmyApocalypse::tom_pygmy_logic
//   mBabyPygmyLogic → PygmyApocalypse::baby_pygmy_logic
//   mbTempleOldPygmyUpright → PygmyApocalypse::temple_old_pygmy_upright
//   mCharlieCuttingPygmyLogic → PygmyCharlieStory::charlie_cutting_pygmy_logic
//   mCharliePygmyLogic → PygmyCharlieStory::charlie_pygmy_logic
//   mCharlieOnBackMonkeyLogic → PygmyCharlieStory::charlie_on_back_monkey_logic
//   mbAbsorbedByHalfPygmy → PygmyCharlieStory::absorbed_by_half_pygmy
//   mbStoryMode → PygmyCharlieStory::story_mode
//   mbStoryModeReady → PygmyCharlieStory::story_mode_ready
//   mbDieInStory → PygmyCharlieStory::die_in_story
//   mbDieInStoryResume → PygmyCharlieStory::die_in_story_resume
//   mbDieInStoryComplete → PygmyCharlieStory::die_in_story_complete
//   mWordBalloonLogic → PygmyCharlieStory::word_balloon_logic
//   mbTheRunsStarted → PygmyMinigames::the_runs_started
//   mbTheRunsJumping → PygmyMinigames::the_runs_jumping
//   mbTheRunsSkidding → PygmyMinigames::the_runs_skidding
//   mbTheRunsRockExplode → PygmyMinigames::the_runs_rock_explode
//   mbTheRunsPlatformCollision → PygmyMinigames::the_runs_platform_collision
//   mbTheRunsFallOffSinkingPlatform → PygmyMinigames::the_runs_fall_off_sinking_platform
//   mbTheRunsMaxSpeed → PygmyMinigames::the_runs_max_speed
//   mbKonkeyDongStarted → PygmyMinigames::konkey_dong_started
//   mbKonkeyDongRunning → PygmyMinigames::konkey_dong_running
//   mbKonkeyDongFalling → PygmyMinigames::konkey_dong_falling
//   mbKonkeyDongOnLadder → PygmyMinigames::konkey_dong_on_ladder
//   mbKonkeyDongHoldingPick → PygmyMinigames::konkey_dong_holding_pick
//   mbKonkeyDongLevelTransition → PygmyMinigames::konkey_dong_level_transition
//   mbKonkeyDongPlatformCollision → PygmyMinigames::konkey_dong_platform_collision
//   mbKonkeyDongJumpBoulder → PygmyMinigames::konkey_dong_jump_boulder
//   mbKonkeyDongJumpBeetle → PygmyMinigames::konkey_dong_jump_beetle
//   mbMoronPestsHitTurtle → PygmyMinigames::moron_pests_hit_turtle
//   mbMoronPestsStartRunning → PygmyMinigames::moron_pests_start_running
//   mbMoronPestsFloorCollision → PygmyMinigames::moron_pests_floor_collision
//   mMoronPestsTargetXPosVel → PygmyMinigames::moron_pests_target_x_pos_vel
//   mbKonkeyDongClimbing → PygmyMinigames::konkey_dong_climbing
//   mbKonkeyDongLadderUp → PygmyMinigames::konkey_dong_ladder_up
//   mKonkeyDongJumpTouch → PygmyMinigames::konkey_dong_jump_touch
//   mKonkeyDongLadderLogic → PygmyMinigames::konkey_dong_ladder_logic
//   mbBattleOfTheGodsEnemy → PygmyMinigames::battle_of_the_gods_enemy
//   mbKilledDuringBattle → PygmyMinigames::killed_during_battle
//   mOpponentPygmyLogic → PygmyMinigames::opponent_pygmy_logic
//   mpStructXmlEpisodeNodeDef → PygmyMinigames::struct_xml_episode_node_def
//   mpOpponentStructXmlEpisodeNodeDef → PygmyMinigames::opponent_struct_xml_episode_node_def
//   mbDoodleOnIce → PygmyMinigames::doodle_on_ice
//   mbDoodleJetPack → PygmyMinigames::doodle_jet_pack

pub const PYGMY_LOGIC: Class = Class {
    name: "PygmyLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    game_frame_order: 30,
    sequence_item,
    perform,
    is_sound_disabled,
    apply_move_pos: apply_move_x_pos,
    on_game_frame,
    on_multi_model_event,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// Sequence-item handlers implemented by `PygmyLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "disableLabels" => seq_disable_labels(world, this, call),
        "flipXScale" => seq_flip_x_scale(world, this, call),
        "hangUpsideDown" => seq_hang_upside_down(world, this, call),
        "idle" => seq_idle(world, this, call),
        "die" => seq_die(world, this, call),
        "explode" => seq_explode(world, this, call),
        "checkPygmyDialog" => seq_check_pygmy_dialog(world, this, call),
        "resetVel" => seq_reset_vel(world, this, call),
        "walk" => seq_walk(world, this, call),
        "run" => seq_run(world, this, call),
        "panicRun" => seq_panic_run(world, this, call),
        "slide" => seq_slide(world, this, call),
        "fall" => seq_fall(world, this, call),
        "tumble" => seq_tumble(world, this, call),
        "drag" => seq_drag(world, this, call),
        "setOrigin" => seq_set_origin(world, this, call),
        "setAnimationBasedOnPygmyType" => seq_set_animation_based_on_pygmy_type(world, this, call),
        "pushSequenceBasedOnPersonality" => seq_push_sequence_based_on_personality(world, this, call),
        "runSequenceBasedOnPersonality" => seq_run_sequence_based_on_personality(world, this, call),
        "doodleJetPackComplete" => seq_doodle_jet_pack_complete(world, this, call),
        "konkeyDongIntroComplete" => seq_konkey_dong_intro_complete(world, this, call),
        "konkeyDongLevelJumpComplete" => seq_konkey_dong_level_jump_complete(world, this, call),
        "konkeyDongJumpUp" => seq_konkey_dong_jump_up(world, this, call),
        "konkeyDongFall" => seq_konkey_dong_fall(world, this, call),
        "onStartKonkeyDongFall" => seq_on_start_konkey_dong_fall(world, this, call),
        "checkTempleOldPygmyFallDownFromSlide" => seq_check_temple_old_pygmy_fall_down_from_slide(world, this, call),
        "templeOldPygmyFallDown" => seq_temple_old_pygmy_fall_down(world, this, call),
        "templePygmyWalk" => seq_temple_pygmy_walk(world, this, call),
        "templePygmyIdle" => seq_temple_pygmy_idle(world, this, call),
        "checkForTempleDoorJump" => seq_check_for_temple_door_jump(world, this, call),
        "jumpIntoTempleDoorComplete" => seq_jump_into_temple_door_complete(world, this, call),
        "runSequenceIfInXenuRoom" => seq_run_sequence_if_in_xenu_room(world, this, call),
        "checkTomJumpOnCouch" => seq_check_tom_jump_on_couch(world, this, call),
        "waitApexTomJumpOnCouch" => seq_wait_apex_tom_jump_on_couch(world, this, call),
        "waitLandTomJumpOnCouch" => seq_wait_land_tom_jump_on_couch(world, this, call),
        "tomJumpOffCouch" => seq_tom_jump_off_couch(world, this, call),
        "waitLandTomJumpOffCouch" => seq_wait_land_tom_jump_off_couch(world, this, call),
        "checkSlamDoorBouncingOnCouch" => seq_check_slam_door_bouncing_on_couch(world, this, call),
        "checkForPossessedByThetan" => seq_check_for_possessed_by_thetan(world, this, call),
        "thetanPummelPygmy" => seq_thetan_pummel_pygmy(world, this, call),
        "thetanEatPygmyFace" => seq_thetan_eat_pygmy_face(world, this, call),
        "chooseBrawlAttack" => seq_choose_brawl_attack(world, this, call),
        "brawlAttackComplete" => seq_brawl_attack_complete(world, this, call),
        "possessedByThetanWalk" => seq_possessed_by_thetan_walk(world, this, call),
        "possessedByThetanIdle" => seq_possessed_by_thetan_idle(world, this, call),
        "walkToXenu" => seq_walk_to_xenu(world, this, call),
        "checkForFinishClosetIdle" => seq_check_for_finish_closet_idle(world, this, call),
        "runToClosetDoor" => seq_run_to_closet_door(world, this, call),
        "checkDiseaseIncubation" => seq_check_disease_incubation(world, this, call),
        "locustThrowWaitForApex" => seq_locust_throw_wait_for_apex(world, this, call),
        "checkForLocusts" => seq_check_for_locusts(world, this, call),
        "checkForLocustsComplete" => seq_check_for_locusts_complete(world, this, call),
        "mutatedByLocustsComplete" => seq_mutated_by_locusts_complete(world, this, call),
        "checkMutantLocustFlyOff" => seq_check_mutant_locust_fly_off(world, this, call),
        "waitForMutantLocustFlyOffComplete" => seq_wait_for_mutant_locust_fly_off_complete(world, this, call),
        "mouthFilledByLocustsComplete" => seq_mouth_filled_by_locusts_complete(world, this, call),
        "closetIdle" => seq_closet_idle(world, this, call),
        "closetWalk" => seq_closet_walk(world, this, call),
        "respawnPlagueBody" => seq_respawn_plague_body(world, this, call),
        "plagueBodyFloat" => seq_plague_body_float(world, this, call),
        "throwPlagueBodyOnPile" => seq_throw_plague_body_on_pile(world, this, call),
        "checkForWarlock" => seq_check_for_warlock(world, this, call),
        "halfPygmyWalk" => seq_half_pygmy_walk(world, this, call),
        "halfPygmyIdle" => seq_half_pygmy_idle(world, this, call),
        "charlieMonkeyWalk" => seq_charlie_monkey_walk(world, this, call),
        "charlieMonkeyIdle" => seq_charlie_monkey_idle(world, this, call),
        "charlieCutPygmyInHalfComplete" => seq_charlie_cut_pygmy_in_half_complete(world, this, call),
        "checkDanceChange" => seq_check_dance_change(world, this, call),
        "stopStaringAtKidnapping" => seq_stop_staring_at_kidnapping(world, this, call),
        "chaseKonkeyDongComplete" => seq_chase_konkey_dong_complete(world, this, call),
        "stopDanceComplete" => seq_stop_dance_complete(world, this, call),
        "chooseRainbowAnimation" => seq_choose_rainbow_animation(world, this, call),
        "startStaringAtRainbow" => seq_start_staring_at_rainbow(world, this, call),
        "gushOverRainbowComplete" => seq_gush_over_rainbow_complete(world, this, call),
        "eatenByMoronSnakeComplete" => seq_eaten_by_moron_snake_complete(world, this, call),
        "tugOfWarWithSquid" => seq_tug_of_war_with_squid(world, this, call),
        "onFallIntoCrackHoleComplete" => seq_on_fall_into_crack_hole_complete(world, this, call),
        "possessedByGhostComplete" => seq_possessed_by_ghost_complete(world, this, call),
        "penetrateGroundComplete" => seq_penetrate_ground_complete(world, this, call),
        "pulledUnderComplete" => seq_pulled_under_complete(world, this, call),
        "initCarriedAwayByGhostComplete" => seq_init_carried_away_by_ghost_complete(world, this, call),
        "carryPygmyOffScreen" => seq_carry_pygmy_off_screen(world, this, call),
        "startBansheeScream" => seq_start_banshee_scream(world, this, call),
        "burstPygmyEarDrums" => seq_burst_pygmy_ear_drums(world, this, call),
        "explodeFromBansheeScream" => seq_explode_from_banshee_scream(world, this, call),
        "ghostFlyUp" => seq_ghost_fly_up(world, this, call),
        "ghostFlyIdle" => seq_ghost_fly_idle(world, this, call),
        "ghostFollowPath" => seq_ghost_follow_path(world, this, call),
        "eatBrainsComplete" => seq_eat_brains_complete(world, this, call),
        "tripIntoZombieGrave" => seq_trip_into_zombie_grave(world, this, call),
        "zombieWalk" => seq_zombie_walk(world, this, call),
        "zombieIdle" => seq_zombie_idle(world, this, call),
        "pickUpCoconut" => seq_pick_up_coconut(world, this, call),
        "finishEatingCoconut" => seq_finish_eating_coconut(world, this, call),
        "pickUpWood" => seq_pick_up_wood(world, this, call),
        "walkToBonFire" => seq_walk_to_bon_fire(world, this, call),
        "pickUpBanana" => seq_pick_up_banana(world, this, call),
        "finishEatingBanana" => seq_finish_eating_banana(world, this, call),
        "pickUpBird" => seq_pick_up_bird(world, this, call),
        "finishEatingBird" => seq_finish_eating_bird(world, this, call),
        "pickUpFish" => seq_pick_up_fish(world, this, call),
        "finishEatingFish" => seq_finish_eating_fish(world, this, call),
        "pickUpSwordFish" => seq_pick_up_sword_fish(world, this, call),
        "finishEatingSwordFish" => seq_finish_eating_sword_fish(world, this, call),
        "checkStayAwayFromTRex" => seq_check_stay_away_from_t_rex(world, this, call),
        "runUnderTRex" => seq_run_under_t_rex(world, this, call),
        "checkForTRex" => seq_check_for_t_rex(world, this, call),
        "checkStayAwayFromSpider" => seq_check_stay_away_from_spider(world, this, call),
        "runUnderSpider" => seq_run_under_spider(world, this, call),
        "checkForSpider" => seq_check_for_spider(world, this, call),
        "checkForNoBonFire" => seq_check_for_no_bon_fire(world, this, call),
        "checkForRainbow" => seq_check_for_rainbow(world, this, call),
        "checkForSunset" => seq_check_for_sunset(world, this, call),
        "checkForSunState" => seq_check_for_sun_state(world, this, call),
        "checkForGingerSunBurnt" => seq_check_for_ginger_sun_burnt(world, this, call),
        "checkForSunBurnt" => seq_check_for_sun_burnt(world, this, call),
        "checkForSunFrozen" => seq_check_for_sun_frozen(world, this, call),
        "checkForNoSunset" => seq_check_for_no_sunset(world, this, call),
        "checkForNight" => seq_check_for_night(world, this, call),
        "checkForDay" => seq_check_for_day(world, this, call),
        "checkForDreamBubble" => seq_check_for_dream_bubble(world, this, call),
        "checkForDance" => seq_check_for_dance(world, this, call),
        "startSleeping" => seq_start_sleeping(world, this, call),
        "explodeFromLovingPocketGod" => seq_explode_from_loving_pocket_god(world, this, call),
        "explodeFromNotCrapping" => seq_explode_from_not_crapping(world, this, call),
        "explodeVampireSun" => seq_explode_vampire_sun(world, this, call),
        "explodeVampireGinger" => seq_explode_vampire_ginger(world, this, call),
        "resetIgloo" => seq_reset_igloo(world, this, call),
        "resetCrapper" => seq_reset_crapper(world, this, call),
        "waitToCrap" => seq_wait_to_crap(world, this, call),
        "takeCrap" => seq_take_crap(world, this, call),
        "startSinking" => seq_start_sinking(world, this, call),
        "walkToIceFishingPole" => seq_walk_to_ice_fishing_pole(world, this, call),
        "sunBurntComplete" => seq_sun_burnt_complete(world, this, call),
        "sunWalk" => seq_sun_walk(world, this, call),
        "startFishing" => seq_start_fishing(world, this, call),
        "walkToFishingPole" => seq_walk_to_fishing_pole(world, this, call),
        "checkKillPygmies" => seq_check_kill_pygmies(world, this, call),
        "vampireRun" => seq_vampire_run(world, this, call),
        "checkForVampire" => seq_check_for_vampire(world, this, call),
        "waitForNight" => seq_wait_for_night(world, this, call),
        "killPygmies" => seq_kill_pygmies(world, this, call),
        "iglooRoll" => seq_igloo_roll(world, this, call),
        "jumpOnFire" => seq_jump_on_fire(world, this, call),
        "uncrushed" => seq_uncrushed(world, this, call),
        "sharkSwallow" => seq_shark_swallow(world, this, call),
        "watchShark" => seq_watch_shark(world, this, call),
        "onEatenByMoronTurtleEatComplete" => seq_on_eaten_by_moron_turtle_eat_complete(world, this, call),
        "onStartMoronPestsFall" => seq_on_start_moron_pests_fall(world, this, call),
        "moronPestsIntroComplete" => seq_moron_pests_intro_complete(world, this, call),
        "moronPestsRun" => seq_moron_pests_run(world, this, call),
        "onJellyfishSuffocateAttackComplete" => seq_on_jellyfish_suffocate_attack_complete(world, this, call),
        "checkForLaserSharkEnabled" => seq_check_for_laser_shark_enabled(world, this, call),
        "checkForLaserSharkDisabled" => seq_check_for_laser_shark_disabled(world, this, call),
        "addUnderwaterComplete" => seq_add_underwater_complete(world, this, call),
        "swim" => seq_swim(world, this, call),
        "panicSwim" => seq_panic_swim(world, this, call),
        "painDrainCanTurn" => seq_pain_drain_can_turn(world, this, call),
        "fallIntoDrain" => seq_fall_into_drain(world, this, call),
        "pushSequenceBasedOnOpponentWeaponType" => seq_push_sequence_based_on_opponent_weapon_type(world, this, call),
        "pushSequenceBasedOnOpponentBattleType" => seq_push_sequence_based_on_opponent_battle_type(world, this, call),
        "battleCheckNoRunBack" => seq_battle_check_no_run_back(world, this, call),
        "battleRunBackToStart" => seq_battle_run_back_to_start(world, this, call),
        "battleStrongRunTowardOpponent" => seq_battle_strong_run_toward_opponent(world, this, call),
        "battleFastRunTowardOpponent" => seq_battle_fast_run_toward_opponent(world, this, call),
        "battleEnergyFastMoveTowardOpponent" => seq_battle_energy_fast_move_toward_opponent(world, this, call),
        "battleEnergyFastMoveAboveOpponent" => seq_battle_energy_fast_move_above_opponent(world, this, call),
        "battleEnergyFastWaitForOpponent" => seq_battle_energy_fast_wait_for_opponent(world, this, call),
        "checkForStrongAttackWin" => seq_check_for_strong_attack_win(world, this, call),
        "checkForEnergyStrongAttackWin" => seq_check_for_energy_strong_attack_win(world, this, call),
        "checkForFastAttackWin" => seq_check_for_fast_attack_win(world, this, call),
        "checkForAttackTie" => seq_check_for_attack_tie(world, this, call),
        "battleShowLavaFlame" => seq_battle_show_lava_flame(world, this, call),
        "battleFall" => seq_battle_fall(world, this, call),
        "battleRespawn" => seq_battle_respawn(world, this, call),
        "onBattleReady" => seq_on_battle_ready(world, this, call),
        "onStartTheRunsFall" => seq_on_start_the_runs_fall(world, this, call),
        "theRunsJumpUp" => seq_the_runs_jump_up(world, this, call),
        "theRunsRun" => seq_the_runs_run(world, this, call),
        "theRunsFall" => seq_the_runs_fall(world, this, call),
        "onTheRunsSplashComplete" => seq_on_the_runs_splash_complete(world, this, call),
        "onTheRunsExplodeComplete" => seq_on_the_runs_explode_complete(world, this, call),
        "openWordBalloon" => seq_open_word_balloon(world, this, call),
        "closeWordBalloon" => seq_close_word_balloon(world, this, call),
        "storyModeKnockOffIsland" => seq_story_mode_knock_off_island(world, this, call),
        "storyModeFlick" => seq_story_mode_flick(world, this, call),
        "dieInStory" => seq_die_in_story(world, this, call),
        "dieInStoryComplete" => seq_die_in_story_complete(world, this, call),
        "checkForStoryMode" => seq_check_for_story_mode(world, this, call),
        "onPygmyNoPeeRollComplete" => seq_on_pygmy_no_pee_roll_complete(world, this, call),
        "waitUntilOffScreen" => seq_wait_until_off_screen(world, this, call),
        "onNoPeeHitComplete" => seq_on_no_pee_hit_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// Zero-argument selectors sent by name (animation frame actions) that `PygmyLogic` implements.
pub fn perform(world: &mut World, this: Entity, selector: &str) -> bool {
    match selector {
        "animChooseDirection" => anim_choose_direction(world, this),
        "animFlipComplete" => anim_flip_complete(world, this),
        "animReleaseThetan" => anim_release_thetan(world, this),
        "animPounceComplete" => anim_pounce_complete(world, this),
        "animLandThetanPunch" => anim_land_thetan_punch(world, this),
        "animStopThetanPossession" => anim_stop_thetan_possession(world, this),
        "animTomBabySacrificeComplete" => anim_tom_baby_sacrifice_complete(world, this),
        "animTomShowXenu" => anim_tom_show_xenu(world, this),
        "animSlamClosetDoorComplete" => anim_slam_closet_door_complete(world, this),
        "animSpawnSkullAndBone" => anim_spawn_skull_and_bone(world, this),
        "animStartPygmyRecoil" => anim_start_pygmy_recoil(world, this),
        "animInfectPygmies" => anim_infect_pygmies(world, this),
        "animStartFloat" => anim_start_float(world, this),
        "animThrowPlagueBody" => anim_throw_plague_body(world, this),
        "animPickupPlagueBody" => anim_pickup_plague_body(world, this),
        "animCutPygmyInHalf" => anim_cut_pygmy_in_half(world, this),
        "animCharlieLaunchShark" => anim_charlie_launch_shark(world, this),
        "animCharlieThrowFood" => anim_charlie_throw_food(world, this),
        "animWarlockBurnVampire" => anim_warlock_burn_vampire(world, this),
        "animWarlockStrike" => anim_warlock_strike(world, this),
        "hideFood" => hide_food(world, this),
        "animThrowSpear" => anim_throw_spear(world, this),
        "animInitBob" => anim_init_bob(world, this),
        "animHitHumpingMonkey" => anim_hit_humping_monkey(world, this),
        "animBabyThrowSnake" => anim_baby_throw_snake(world, this),
        "animPygmyScaredComplete" => anim_pygmy_scared_complete(world, this),
        "animGhostXFlip" => anim_ghost_x_flip(world, this),
        "animThrowWood" => anim_throw_wood(world, this),
        "animSwordFishSwallow" => anim_sword_fish_swallow(world, this),
        "animJellyfishAttackComplete" => anim_jellyfish_attack_complete(world, this),
        "animBattleChangeWeapon" => anim_battle_change_weapon(world, this),
        "animBattleThrowEnergyBall1" => anim_battle_throw_energy_ball1(world, this),
        "animBattleThrowEnergyBall2" => anim_battle_throw_energy_ball2(world, this),
        "animBattleThrowEnergyBall3" => anim_battle_throw_energy_ball3(world, this),
        "animBattleThrowEnergyBall4" => anim_battle_throw_energy_ball4(world, this),
        "animBattleTie" => anim_battle_tie(world, this),
        "animBattleEnergyCounterHit" => anim_battle_energy_counter_hit(world, this),
        "animBattleStrongHit" => anim_battle_strong_hit(world, this),
        "animBattleFastHit" => anim_battle_fast_hit(world, this),
        "openApeKidnapWordBalloon" => open_ape_kidnap_word_balloon(world, this),
        "closeWordBalloon" => close_word_balloon(world, this),
        "animNoPeeEmbedDie" => anim_no_pee_embed_die(world, this),
        "animNoPeeFloatUpward" => anim_no_pee_float_upward(world, this),
        "animNoPeeSinkDownward" => anim_no_pee_sink_downward(world, this),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.perform)(world, this, selector),
    }
    true
}

/// `[[PygmyLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&PYGMY_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}
