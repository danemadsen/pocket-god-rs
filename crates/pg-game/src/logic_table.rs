//! The slots of the logic array at `0x1007e3030` that `-[BCLibrary(Logic) createLogicClasses]`
//! fills, by name. Game code reads slot `i` as the global `DAT_<0x1007e3030 + 8 i>`; the port
//! reads it with `pg_engine::library::logic(world, NAME)`. The full record of every slot (class,
//! display object, start-up steps, listener) is in `assets/data/logic-objects.json`.

/// `DAT_1007e3030`: `PygmyBottomLogic` on a new `BCMultiModel` "PygmyBottom".
pub const PYGMY_BOTTOM: usize = 0;

/// `DAT_1007e3038`: `MultiModelLogic` on a new `BCMultiModel` "ApeVomit".
pub const APE_VOMIT: usize = 1;

/// `DAT_1007e3040`: `DreamBubbleLogic` on a new `BCMultiModel` "DreamBubble".
pub const DREAM_BUBBLE: usize = 2;

/// `DAT_1007e3048`: `BCButtonLogic` on "Blackbox" of the display group "StoryConfirmPanel".
pub const STORY_CONFIRM_PANEL_BLACKBOX: usize = 3;

/// `DAT_1007e3050`: `BCButtonLogic` on "Blackbox" of the display group "StoryLoading".
pub const STORY_LOADING_BLACKBOX: usize = 4;

/// `DAT_1007e3058`: `BCButtonLogic` on "BlackBox" of the display group "StoryOptionMenu".
pub const STORY_OPTION_MENU_BLACK_BOX: usize = 5;

/// `DAT_1007e3060`: `BCButtonLogic` on "BlackBox" of the display group "StoryWideOptionMenu".
pub const STORY_WIDE_OPTION_MENU_BLACK_BOX: usize = 6;

/// `DAT_1007e3068`: `MultiModelLogic` on "Background" of the display group "MoronPestsDisplayGroup".
pub const MORON_PESTS_BACKGROUND: usize = 7;

/// `DAT_1007e3070`: `MultiModelLogic` on "RibBackground" of the display group "MoronPestsDisplayGroup".
pub const RIB_BACKGROUND: usize = 8;

/// `DAT_1007e3078`: `MultiModelLogic` on "Foreground" of the display group "MoronPestsDisplayGroup".
pub const MORON_PESTS_FOREGROUND: usize = 9;

/// `DAT_1007e3080`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColor1".
pub const MORON_PESTS_COLOR1: usize = 10;

/// `DAT_1007e3088`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColor2".
pub const MORON_PESTS_COLOR2: usize = 11;

/// `DAT_1007e3090`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColor3".
pub const MORON_PESTS_COLOR3: usize = 12;

/// `DAT_1007e3098`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColor4".
pub const MORON_PESTS_COLOR4: usize = 13;

/// `DAT_1007e30a0`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColorOverlay1".
pub const MORON_PESTS_COLOR_OVERLAY1: usize = 14;

/// `DAT_1007e30a8`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColorOverlay2".
pub const MORON_PESTS_COLOR_OVERLAY2: usize = 15;

/// `DAT_1007e30b0`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColorOverlay3".
pub const MORON_PESTS_COLOR_OVERLAY3: usize = 16;

/// `DAT_1007e30b8`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsColorOverlay4".
pub const MORON_PESTS_COLOR_OVERLAY4: usize = 17;

/// `DAT_1007e30c0`: `BCButtonLogic` on "Color1Button" of the display group "MoronPestsColorButtons".
pub const COLOR1_BUTTON: usize = 18;

/// `DAT_1007e30c8`: `BCButtonLogic` on "Color2Button" of the display group "MoronPestsColorButtons".
pub const COLOR2_BUTTON: usize = 19;

/// `DAT_1007e30d0`: `BCButtonLogic` on "Color3Button" of the display group "MoronPestsColorButtons".
pub const COLOR3_BUTTON: usize = 20;

/// `DAT_1007e30d8`: `BCButtonLogic` on "Color4Button" of the display group "MoronPestsColorButtons".
pub const COLOR4_BUTTON: usize = 21;

/// `DAT_1007e30e0`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsSynonym1".
pub const MORON_PESTS_SYNONYM1: usize = 22;

/// `DAT_1007e30e8`: `MultiModelLogic` on a new `BCMultiModel` "MoronPestsSynonym2".
pub const MORON_PESTS_SYNONYM2: usize = 23;

/// `DAT_1007e30f0`: `BCButtonLogic` on "Synonym1Button" of the display group "MoronPestsSynonymButtons".
pub const SYNONYM1_BUTTON: usize = 24;

/// `DAT_1007e30f8`: `BCButtonLogic` on "Synonym2Button" of the display group "MoronPestsSynonymButtons".
pub const SYNONYM2_BUTTON: usize = 25;

/// `DAT_1007e3100`: `MultiModelLogic` on a new `BCMultiModel` "MoronDuckHead1".
pub const MORON_DUCK_HEAD1: usize = 26;

/// `DAT_1007e3108`: `MultiModelLogic` on a new `BCMultiModel` "MoronDuckHead2".
pub const MORON_DUCK_HEAD2: usize = 27;

/// `DAT_1007e3110`: `MultiModelLogic` on a new `BCMultiModel` "MoronDuckHead3".
pub const MORON_DUCK_HEAD3: usize = 28;

/// `DAT_1007e3118`: `MultiModelLogic` on a new `BCMultiModel` "MoronDuckHead4".
pub const MORON_DUCK_HEAD4: usize = 29;

/// `DAT_1007e3120`: `MoronDuckLogic` on a new `BCMultiModel` "MoronDuck1".
pub const MORON_DUCK1: usize = 30;

/// `DAT_1007e3128`: `MoronDuckLogic` on a new `BCMultiModel` "MoronDuck2".
pub const MORON_DUCK2: usize = 31;

/// `DAT_1007e3130`: `MoronDuckLogic` on a new `BCMultiModel` "MoronDuck3".
pub const MORON_DUCK3: usize = 32;

/// `DAT_1007e3138`: `MoronDuckLogic` on a new `BCMultiModel` "MoronDuck4".
pub const MORON_DUCK4: usize = 33;

/// `DAT_1007e3140`: `MultiModelLogic` on a new `BCMultiModel` "MoronBeeWings".
pub const MORON_BEE_WINGS: usize = 34;

/// `DAT_1007e3148`: `MultiModelLogic` on a new `BCMultiModel` "MoronBeeHead1".
pub const MORON_BEE_HEAD1: usize = 35;

/// `DAT_1007e3150`: `MultiModelLogic` on a new `BCMultiModel` "MoronBeeHead2".
pub const MORON_BEE_HEAD2: usize = 36;

/// `DAT_1007e3158`: `MultiModelLogic` on a new `BCMultiModel` "MoronBeeHead3".
pub const MORON_BEE_HEAD3: usize = 37;

/// `DAT_1007e3160`: `MultiModelLogic` on a new `BCMultiModel` "MoronBeeHead4".
pub const MORON_BEE_HEAD4: usize = 38;

/// `DAT_1007e3168`: `MultiModelLogic` on a new `BCMultiModel` "MoronBeeHead5".
pub const MORON_BEE_HEAD5: usize = 39;

/// `DAT_1007e3170`: `MoronBeeLogic` on a new `BCMultiModel` "MoronBee1".
pub const MORON_BEE1: usize = 40;

/// `DAT_1007e3178`: `MoronBeeLogic` on a new `BCMultiModel` "MoronBee2".
pub const MORON_BEE2: usize = 41;

/// `DAT_1007e3180`: `MoronBeeLogic` on a new `BCMultiModel` "MoronBee3".
pub const MORON_BEE3: usize = 42;

/// `DAT_1007e3188`: `MoronBeeLogic` on a new `BCMultiModel` "MoronBee4".
pub const MORON_BEE4: usize = 43;

/// `DAT_1007e3190`: `MoronBeeLogic` on a new `BCMultiModel` "MoronBee5".
pub const MORON_BEE5: usize = 44;

/// `DAT_1007e3198`: `MoronPestsQuestionScreenLogic` on a new `BCMultiModel` "MoronPestsQuestionScreen".
pub const MORON_PESTS_QUESTION_SCREEN: usize = 45;

/// `DAT_1007e31a0`: `MultiModelLogic` on "Island" of the display group "MisfitIslandDisplayGroup".
pub const MISFIT_ISLAND_ISLAND: usize = 46;

/// `DAT_1007e31a8`: `MultiModelLogic` on "Foreground" of the display group "MisfitIslandDisplayGroup".
pub const MISFIT_ISLAND_FOREGROUND: usize = 47;

/// `DAT_1007e31b0`: `MultiModelLogic` on "Tree" of the display group "MisfitIslandDisplayGroup".
pub const TREE: usize = 48;

/// `DAT_1007e31b8`: `IslandObjectLogic` on a new `BCMultiModel` "MisfitOceanRock", marker "MidOceanMarker".
pub const MISFIT_OCEAN_ROCK: usize = 49;

/// `DAT_1007e31c0`: `MoronTurtleLogic` on a new `BCMultiModel` "MoronTurtle".
pub const MORON_TURTLE: usize = 50;

/// `DAT_1007e31c8`: `MoronSnakeLogic` on a new `BCMultiModel` "MoronSnake".
pub const MORON_SNAKE: usize = 51;

/// `DAT_1007e31d0`: `MultiModelLogic` on "UnderwaterForeground" of the display group "NoPeeDisplayGroup".
pub const UNDERWATER_FOREGROUND: usize = 52;

/// `DAT_1007e31d8`: `MultiModelLogic` on "UnderwaterBackground1" of the display group "NoPeeDisplayGroup".
pub const UNDERWATER_BACKGROUND1: usize = 53;

/// `DAT_1007e31e0`: `MultiModelLogic` on "UnderwaterBackground2" of the display group "NoPeeDisplayGroup".
pub const UNDERWATER_BACKGROUND2: usize = 54;

/// `DAT_1007e31e8`: `MultiModelLogic` on "UnderwaterBackground3" of the display group "NoPeeDisplayGroup".
pub const UNDERWATER_BACKGROUND3: usize = 55;

/// `DAT_1007e31f0`: `NoPeeTreasureLogic` on a new `BCMultiModel` "NoPeeTreasure1".
pub const NO_PEE_TREASURE1: usize = 56;

/// `DAT_1007e31f8`: `NoPeeSeaBlobLogic` on a new `BCMultiModel` "NoPeeSeaBlob1".
pub const NO_PEE_SEA_BLOB1: usize = 57;

/// `DAT_1007e3200`: `NoPeeSeaBlobLogic` on a new `BCMultiModel` "NoPeeSeaBlob2".
pub const NO_PEE_SEA_BLOB2: usize = 58;

/// `DAT_1007e3208`: `NoPeeSeaBlobLogic` on a new `BCMultiModel` "NoPeeSeaBlob3".
pub const NO_PEE_SEA_BLOB3: usize = 59;

/// `DAT_1007e3210`: `NoPeeSeaBlobLogic` on a new `BCMultiModel` "NoPeeSeaBlob4".
pub const NO_PEE_SEA_BLOB4: usize = 60;

/// `DAT_1007e3218`: `NoPeeFishLogic` on a new `BCMultiModel` "NoPeeFish1".
pub const NO_PEE_FISH1: usize = 61;

/// `DAT_1007e3220`: `NoPeeFishLogic` on a new `BCMultiModel` "NoPeeFish2".
pub const NO_PEE_FISH2: usize = 62;

/// `DAT_1007e3228`: `NoPeeFishLogic` on a new `BCMultiModel` "NoPeeFish3".
pub const NO_PEE_FISH3: usize = 63;

/// `DAT_1007e3230`: `NoPeeFishLogic` on a new `BCMultiModel` "NoPeeFish4".
pub const NO_PEE_FISH4: usize = 64;

/// `DAT_1007e3238`: `NoPeeMineLogic` on a new `BCMultiModel` "NoPeeMine1".
pub const NO_PEE_MINE1: usize = 65;

/// `DAT_1007e3240`: `NoPeeMineLogic` on a new `BCMultiModel` "NoPeeMine2".
pub const NO_PEE_MINE2: usize = 66;

/// `DAT_1007e3248`: `NoPeeMineLogic` on a new `BCMultiModel` "NoPeeMine3".
pub const NO_PEE_MINE3: usize = 67;

/// `DAT_1007e3250`: `NoPeeMineLogic` on a new `BCMultiModel` "NoPeeMine4".
pub const NO_PEE_MINE4: usize = 68;

/// `DAT_1007e3258`: `NoPeeClamLogic` on a new `BCMultiModel` "NoPeeClam1".
pub const NO_PEE_CLAM1: usize = 69;

/// `DAT_1007e3260`: `NoPeeClamLogic` on a new `BCMultiModel` "NoPeeClam2".
pub const NO_PEE_CLAM2: usize = 70;

/// `DAT_1007e3268`: `NoPeeClamLogic` on a new `BCMultiModel` "NoPeeClam3".
pub const NO_PEE_CLAM3: usize = 71;

/// `DAT_1007e3270`: `NoPeeClamLogic` on a new `BCMultiModel` "NoPeeClam4".
pub const NO_PEE_CLAM4: usize = 72;

/// `DAT_1007e3278`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle1".
pub const NO_PEE_PARTICLE1: usize = 73;

/// `DAT_1007e3280`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle2".
pub const NO_PEE_PARTICLE2: usize = 74;

/// `DAT_1007e3288`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle3".
pub const NO_PEE_PARTICLE3: usize = 75;

/// `DAT_1007e3290`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle4".
pub const NO_PEE_PARTICLE4: usize = 76;

/// `DAT_1007e3298`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle5".
pub const NO_PEE_PARTICLE5: usize = 77;

/// `DAT_1007e32a0`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle6".
pub const NO_PEE_PARTICLE6: usize = 78;

/// `DAT_1007e32a8`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle7".
pub const NO_PEE_PARTICLE7: usize = 79;

/// `DAT_1007e32b0`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle8".
pub const NO_PEE_PARTICLE8: usize = 80;

/// `DAT_1007e32b8`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle9".
pub const NO_PEE_PARTICLE9: usize = 81;

/// `DAT_1007e32c0`: `NoPeeParticleLogic` on a new `BCMultiModel` "NoPeeParticle10".
pub const NO_PEE_PARTICLE10: usize = 82;

/// `DAT_1007e32c8`: `MultiModelLogic` on a new `BCMultiModel` "NoPeeSwimmer".
pub const NO_PEE_SWIMMER: usize = 83;

/// `DAT_1007e32d0`: `MultiModelLogic` on a new `BCMultiModel` "NoPeeEmbed".
pub const NO_PEE_EMBED: usize = 84;

/// `DAT_1007e32d8`: `MultiModelLogic` on "NoPeeIntro" of the display group "NoPeeDisplayGroup".
pub const NO_PEE_INTRO: usize = 85;

/// `DAT_1007e32e0`: `MultiModelLogic` on "Overlay" of the display group "NoPeeDisplayGroup".
pub const OVERLAY: usize = 86;

/// `DAT_1007e32e8`: `BCButtonLogic` on "RetryOverlay" of the display group "NoPeeDisplayGroup".
pub const RETRY_OVERLAY: usize = 87;

/// `DAT_1007e32f0`: `MultiModelLogic` on "MapBackground" of the display group "NoPeeMapDisplayGroup".
pub const MAP_BACKGROUND: usize = 88;

/// `DAT_1007e32f8`: `NoPeeMapScreenLogic` on "MapScreen" of the display group "NoPeeMapDisplayGroup".
pub const MAP_SCREEN: usize = 89;

/// `DAT_1007e3300`: `NoPeeMapLogic` on the display group "NoPeeMapDisplayGroup".
pub const NO_PEE_MAP_DISPLAY_GROUP: usize = 90;

/// `DAT_1007e3308`: `NoPeeLogic` on the display group "NoPeeDisplayGroup".
pub const NO_PEE_DISPLAY_GROUP: usize = 91;

/// `DAT_1007e3310`: `WaterWingsLogic` on a new `BCMultiModel` "WaterWings".
pub const WATER_WINGS: usize = 92;

/// `DAT_1007e3318`: `JellyfishLogic` on a new `BCMultiModel` "Jellyfish1".
pub const JELLYFISH1: usize = 93;

/// `DAT_1007e3320`: `JellyfishLogic` on a new `BCMultiModel` "Jellyfish2".
pub const JELLYFISH2: usize = 94;

/// `DAT_1007e3328`: `JellyfishLogic` on a new `BCMultiModel` "Jellyfish3".
pub const JELLYFISH3: usize = 95;

/// `DAT_1007e3330`: `JellyfishLogic` on a new `BCMultiModel` "Jellyfish4".
pub const JELLYFISH4: usize = 96;

/// `DAT_1007e3338`: `JellyfishLogic` on a new `BCMultiModel` "Jellyfish5".
pub const JELLYFISH5: usize = 97;

/// `DAT_1007e3340`: `MultiModelLogic` on a new `BCMultiModel` "JellyfishCircleJerk".
pub const JELLYFISH_CIRCLE_JERK: usize = 98;

/// `DAT_1007e3348`: `GodIdolLogic` on a new `BCMultiModel` "GodIdol1".
pub const GOD_IDOL1: usize = 99;

/// `DAT_1007e3350`: `GodIdolLogic` on a new `BCMultiModel` "GodIdol2".
pub const GOD_IDOL2: usize = 100;

/// `DAT_1007e3358`: `GodIdolLogic` on a new `BCMultiModel` "GodIdol3".
pub const GOD_IDOL3: usize = 101;

/// `DAT_1007e3360`: `GodIdolLogic` on a new `BCMultiModel` "GodIdol4".
pub const GOD_IDOL4: usize = 102;

/// `DAT_1007e3368`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo1".
pub const IDOL_ICON_HALO1: usize = 103;

/// `DAT_1007e3370`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo2".
pub const IDOL_ICON_HALO2: usize = 104;

/// `DAT_1007e3378`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo3".
pub const IDOL_ICON_HALO3: usize = 105;

/// `DAT_1007e3380`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo4".
pub const IDOL_ICON_HALO4: usize = 106;

/// `DAT_1007e3388`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo5".
pub const IDOL_ICON_HALO5: usize = 107;

/// `DAT_1007e3390`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo6".
pub const IDOL_ICON_HALO6: usize = 108;

/// `DAT_1007e3398`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo7".
pub const IDOL_ICON_HALO7: usize = 109;

/// `DAT_1007e33a0`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo8".
pub const IDOL_ICON_HALO8: usize = 110;

/// `DAT_1007e33a8`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconHalo9".
pub const IDOL_ICON_HALO9: usize = 111;

/// `DAT_1007e33b0`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark1".
pub const IDOL_ICON_CHECK_MARK1: usize = 112;

/// `DAT_1007e33b8`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark2".
pub const IDOL_ICON_CHECK_MARK2: usize = 113;

/// `DAT_1007e33c0`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark3".
pub const IDOL_ICON_CHECK_MARK3: usize = 114;

/// `DAT_1007e33c8`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark4".
pub const IDOL_ICON_CHECK_MARK4: usize = 115;

/// `DAT_1007e33d0`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark5".
pub const IDOL_ICON_CHECK_MARK5: usize = 116;

/// `DAT_1007e33d8`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark6".
pub const IDOL_ICON_CHECK_MARK6: usize = 117;

/// `DAT_1007e33e0`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark7".
pub const IDOL_ICON_CHECK_MARK7: usize = 118;

/// `DAT_1007e33e8`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark8".
pub const IDOL_ICON_CHECK_MARK8: usize = 119;

/// `DAT_1007e33f0`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckMark9".
pub const IDOL_ICON_CHECK_MARK9: usize = 120;

/// `DAT_1007e33f8`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton1".
pub const IDOL_ICON_BUTTON1: usize = 121;

/// `DAT_1007e3400`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton2".
pub const IDOL_ICON_BUTTON2: usize = 122;

/// `DAT_1007e3408`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton3".
pub const IDOL_ICON_BUTTON3: usize = 123;

/// `DAT_1007e3410`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton4".
pub const IDOL_ICON_BUTTON4: usize = 124;

/// `DAT_1007e3418`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton5".
pub const IDOL_ICON_BUTTON5: usize = 125;

/// `DAT_1007e3420`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton6".
pub const IDOL_ICON_BUTTON6: usize = 126;

/// `DAT_1007e3428`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton7".
pub const IDOL_ICON_BUTTON7: usize = 127;

/// `DAT_1007e3430`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton8".
pub const IDOL_ICON_BUTTON8: usize = 128;

/// `DAT_1007e3438`: `IdolIconLogic` on a new `BCMultiModel` "IdolIconButton9".
pub const IDOL_ICON_BUTTON9: usize = 129;

/// `DAT_1007e3440`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeHeader".
pub const IDOL_DETAIL_CHALLENGE_HEADER: usize = 130;

/// `DAT_1007e3448`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeHintItem".
pub const IDOL_DETAIL_CHALLENGE_HINT_ITEM: usize = 131;

/// `DAT_1007e3450`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpHeader".
pub const IDOL_DETAIL_LEVEL_UP_HEADER: usize = 132;

/// `DAT_1007e3458`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem1".
pub const IDOL_DETAIL_CHALLENGE_ITEM1: usize = 133;

/// `DAT_1007e3460`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem2".
pub const IDOL_DETAIL_CHALLENGE_ITEM2: usize = 134;

/// `DAT_1007e3468`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem3".
pub const IDOL_DETAIL_CHALLENGE_ITEM3: usize = 135;

/// `DAT_1007e3470`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem4".
pub const IDOL_DETAIL_CHALLENGE_ITEM4: usize = 136;

/// `DAT_1007e3478`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem5".
pub const IDOL_DETAIL_CHALLENGE_ITEM5: usize = 137;

/// `DAT_1007e3480`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem6".
pub const IDOL_DETAIL_CHALLENGE_ITEM6: usize = 138;

/// `DAT_1007e3488`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem7".
pub const IDOL_DETAIL_CHALLENGE_ITEM7: usize = 139;

/// `DAT_1007e3490`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem8".
pub const IDOL_DETAIL_CHALLENGE_ITEM8: usize = 140;

/// `DAT_1007e3498`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem9".
pub const IDOL_DETAIL_CHALLENGE_ITEM9: usize = 141;

/// `DAT_1007e34a0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem10".
pub const IDOL_DETAIL_CHALLENGE_ITEM10: usize = 142;

/// `DAT_1007e34a8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem11".
pub const IDOL_DETAIL_CHALLENGE_ITEM11: usize = 143;

/// `DAT_1007e34b0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem12".
pub const IDOL_DETAIL_CHALLENGE_ITEM12: usize = 144;

/// `DAT_1007e34b8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem13".
pub const IDOL_DETAIL_CHALLENGE_ITEM13: usize = 145;

/// `DAT_1007e34c0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem14".
pub const IDOL_DETAIL_CHALLENGE_ITEM14: usize = 146;

/// `DAT_1007e34c8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem15".
pub const IDOL_DETAIL_CHALLENGE_ITEM15: usize = 147;

/// `DAT_1007e34d0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem16".
pub const IDOL_DETAIL_CHALLENGE_ITEM16: usize = 148;

/// `DAT_1007e34d8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem17".
pub const IDOL_DETAIL_CHALLENGE_ITEM17: usize = 149;

/// `DAT_1007e34e0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem18".
pub const IDOL_DETAIL_CHALLENGE_ITEM18: usize = 150;

/// `DAT_1007e34e8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem19".
pub const IDOL_DETAIL_CHALLENGE_ITEM19: usize = 151;

/// `DAT_1007e34f0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem20".
pub const IDOL_DETAIL_CHALLENGE_ITEM20: usize = 152;

/// `DAT_1007e34f8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem21".
pub const IDOL_DETAIL_CHALLENGE_ITEM21: usize = 153;

/// `DAT_1007e3500`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem22".
pub const IDOL_DETAIL_CHALLENGE_ITEM22: usize = 154;

/// `DAT_1007e3508`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem23".
pub const IDOL_DETAIL_CHALLENGE_ITEM23: usize = 155;

/// `DAT_1007e3510`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem24".
pub const IDOL_DETAIL_CHALLENGE_ITEM24: usize = 156;

/// `DAT_1007e3518`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem25".
pub const IDOL_DETAIL_CHALLENGE_ITEM25: usize = 157;

/// `DAT_1007e3520`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem26".
pub const IDOL_DETAIL_CHALLENGE_ITEM26: usize = 158;

/// `DAT_1007e3528`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem27".
pub const IDOL_DETAIL_CHALLENGE_ITEM27: usize = 159;

/// `DAT_1007e3530`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem28".
pub const IDOL_DETAIL_CHALLENGE_ITEM28: usize = 160;

/// `DAT_1007e3538`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem29".
pub const IDOL_DETAIL_CHALLENGE_ITEM29: usize = 161;

/// `DAT_1007e3540`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem30".
pub const IDOL_DETAIL_CHALLENGE_ITEM30: usize = 162;

/// `DAT_1007e3548`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem31".
pub const IDOL_DETAIL_CHALLENGE_ITEM31: usize = 163;

/// `DAT_1007e3550`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem32".
pub const IDOL_DETAIL_CHALLENGE_ITEM32: usize = 164;

/// `DAT_1007e3558`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem33".
pub const IDOL_DETAIL_CHALLENGE_ITEM33: usize = 165;

/// `DAT_1007e3560`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem34".
pub const IDOL_DETAIL_CHALLENGE_ITEM34: usize = 166;

/// `DAT_1007e3568`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem35".
pub const IDOL_DETAIL_CHALLENGE_ITEM35: usize = 167;

/// `DAT_1007e3570`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem36".
pub const IDOL_DETAIL_CHALLENGE_ITEM36: usize = 168;

/// `DAT_1007e3578`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem37".
pub const IDOL_DETAIL_CHALLENGE_ITEM37: usize = 169;

/// `DAT_1007e3580`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem38".
pub const IDOL_DETAIL_CHALLENGE_ITEM38: usize = 170;

/// `DAT_1007e3588`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem39".
pub const IDOL_DETAIL_CHALLENGE_ITEM39: usize = 171;

/// `DAT_1007e3590`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem40".
pub const IDOL_DETAIL_CHALLENGE_ITEM40: usize = 172;

/// `DAT_1007e3598`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem41".
pub const IDOL_DETAIL_CHALLENGE_ITEM41: usize = 173;

/// `DAT_1007e35a0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem42".
pub const IDOL_DETAIL_CHALLENGE_ITEM42: usize = 174;

/// `DAT_1007e35a8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem43".
pub const IDOL_DETAIL_CHALLENGE_ITEM43: usize = 175;

/// `DAT_1007e35b0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem44".
pub const IDOL_DETAIL_CHALLENGE_ITEM44: usize = 176;

/// `DAT_1007e35b8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem45".
pub const IDOL_DETAIL_CHALLENGE_ITEM45: usize = 177;

/// `DAT_1007e35c0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem46".
pub const IDOL_DETAIL_CHALLENGE_ITEM46: usize = 178;

/// `DAT_1007e35c8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem47".
pub const IDOL_DETAIL_CHALLENGE_ITEM47: usize = 179;

/// `DAT_1007e35d0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem48".
pub const IDOL_DETAIL_CHALLENGE_ITEM48: usize = 180;

/// `DAT_1007e35d8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem49".
pub const IDOL_DETAIL_CHALLENGE_ITEM49: usize = 181;

/// `DAT_1007e35e0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem50".
pub const IDOL_DETAIL_CHALLENGE_ITEM50: usize = 182;

/// `DAT_1007e35e8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem51".
pub const IDOL_DETAIL_CHALLENGE_ITEM51: usize = 183;

/// `DAT_1007e35f0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem52".
pub const IDOL_DETAIL_CHALLENGE_ITEM52: usize = 184;

/// `DAT_1007e35f8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem53".
pub const IDOL_DETAIL_CHALLENGE_ITEM53: usize = 185;

/// `DAT_1007e3600`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem54".
pub const IDOL_DETAIL_CHALLENGE_ITEM54: usize = 186;

/// `DAT_1007e3608`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem55".
pub const IDOL_DETAIL_CHALLENGE_ITEM55: usize = 187;

/// `DAT_1007e3610`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem56".
pub const IDOL_DETAIL_CHALLENGE_ITEM56: usize = 188;

/// `DAT_1007e3618`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem57".
pub const IDOL_DETAIL_CHALLENGE_ITEM57: usize = 189;

/// `DAT_1007e3620`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem58".
pub const IDOL_DETAIL_CHALLENGE_ITEM58: usize = 190;

/// `DAT_1007e3628`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem59".
pub const IDOL_DETAIL_CHALLENGE_ITEM59: usize = 191;

/// `DAT_1007e3630`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem60".
pub const IDOL_DETAIL_CHALLENGE_ITEM60: usize = 192;

/// `DAT_1007e3638`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem61".
pub const IDOL_DETAIL_CHALLENGE_ITEM61: usize = 193;

/// `DAT_1007e3640`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem62".
pub const IDOL_DETAIL_CHALLENGE_ITEM62: usize = 194;

/// `DAT_1007e3648`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem63".
pub const IDOL_DETAIL_CHALLENGE_ITEM63: usize = 195;

/// `DAT_1007e3650`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem64".
pub const IDOL_DETAIL_CHALLENGE_ITEM64: usize = 196;

/// `DAT_1007e3658`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem65".
pub const IDOL_DETAIL_CHALLENGE_ITEM65: usize = 197;

/// `DAT_1007e3660`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem66".
pub const IDOL_DETAIL_CHALLENGE_ITEM66: usize = 198;

/// `DAT_1007e3668`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem67".
pub const IDOL_DETAIL_CHALLENGE_ITEM67: usize = 199;

/// `DAT_1007e3670`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem68".
pub const IDOL_DETAIL_CHALLENGE_ITEM68: usize = 200;

/// `DAT_1007e3678`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem69".
pub const IDOL_DETAIL_CHALLENGE_ITEM69: usize = 201;

/// `DAT_1007e3680`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem70".
pub const IDOL_DETAIL_CHALLENGE_ITEM70: usize = 202;

/// `DAT_1007e3688`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem71".
pub const IDOL_DETAIL_CHALLENGE_ITEM71: usize = 203;

/// `DAT_1007e3690`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem72".
pub const IDOL_DETAIL_CHALLENGE_ITEM72: usize = 204;

/// `DAT_1007e3698`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem73".
pub const IDOL_DETAIL_CHALLENGE_ITEM73: usize = 205;

/// `DAT_1007e36a0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem74".
pub const IDOL_DETAIL_CHALLENGE_ITEM74: usize = 206;

/// `DAT_1007e36a8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem75".
pub const IDOL_DETAIL_CHALLENGE_ITEM75: usize = 207;

/// `DAT_1007e36b0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem76".
pub const IDOL_DETAIL_CHALLENGE_ITEM76: usize = 208;

/// `DAT_1007e36b8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem77".
pub const IDOL_DETAIL_CHALLENGE_ITEM77: usize = 209;

/// `DAT_1007e36c0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem78".
pub const IDOL_DETAIL_CHALLENGE_ITEM78: usize = 210;

/// `DAT_1007e36c8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem79".
pub const IDOL_DETAIL_CHALLENGE_ITEM79: usize = 211;

/// `DAT_1007e36d0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailChallengeItem80".
pub const IDOL_DETAIL_CHALLENGE_ITEM80: usize = 212;

/// `DAT_1007e36d8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar1".
pub const IDOL_DETAIL_LEVEL_UP_BAR1: usize = 213;

/// `DAT_1007e36e0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar2".
pub const IDOL_DETAIL_LEVEL_UP_BAR2: usize = 214;

/// `DAT_1007e36e8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar3".
pub const IDOL_DETAIL_LEVEL_UP_BAR3: usize = 215;

/// `DAT_1007e36f0`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar4".
pub const IDOL_DETAIL_LEVEL_UP_BAR4: usize = 216;

/// `DAT_1007e36f8`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar5".
pub const IDOL_DETAIL_LEVEL_UP_BAR5: usize = 217;

/// `DAT_1007e3700`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar6".
pub const IDOL_DETAIL_LEVEL_UP_BAR6: usize = 218;

/// `DAT_1007e3708`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar7".
pub const IDOL_DETAIL_LEVEL_UP_BAR7: usize = 219;

/// `DAT_1007e3710`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar8".
pub const IDOL_DETAIL_LEVEL_UP_BAR8: usize = 220;

/// `DAT_1007e3718`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpBar9".
pub const IDOL_DETAIL_LEVEL_UP_BAR9: usize = 221;

/// `DAT_1007e3720`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem1".
pub const IDOL_DETAIL_LEVEL_UP_ITEM1: usize = 222;

/// `DAT_1007e3728`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem2".
pub const IDOL_DETAIL_LEVEL_UP_ITEM2: usize = 223;

/// `DAT_1007e3730`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem3".
pub const IDOL_DETAIL_LEVEL_UP_ITEM3: usize = 224;

/// `DAT_1007e3738`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem4".
pub const IDOL_DETAIL_LEVEL_UP_ITEM4: usize = 225;

/// `DAT_1007e3740`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem5".
pub const IDOL_DETAIL_LEVEL_UP_ITEM5: usize = 226;

/// `DAT_1007e3748`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem6".
pub const IDOL_DETAIL_LEVEL_UP_ITEM6: usize = 227;

/// `DAT_1007e3750`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem7".
pub const IDOL_DETAIL_LEVEL_UP_ITEM7: usize = 228;

/// `DAT_1007e3758`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem8".
pub const IDOL_DETAIL_LEVEL_UP_ITEM8: usize = 229;

/// `DAT_1007e3760`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailLevelUpItem9".
pub const IDOL_DETAIL_LEVEL_UP_ITEM9: usize = 230;

/// `DAT_1007e3768`: `BCButtonLogic` on "CloseButton" of the display group "IdolScreenButtons".
pub const IDOL_SCREEN_BUTTONS_CLOSE_BUTTON: usize = 231;

/// `DAT_1007e3770`: `BCButtonLogic` on "WhatsNewButton" of the display group "IdolStartButtons".
pub const WHATS_NEW_BUTTON: usize = 232;

/// `DAT_1007e3778`: `BCButtonLogic` on "OptionsButton" of the display group "IdolStartButtons".
pub const OPTIONS_BUTTON: usize = 233;

/// `DAT_1007e3780`: `BCButtonLogic` on "ChallengesButton" of the display group "IdolStartButtons".
pub const IDOL_START_BUTTONS_CHALLENGES_BUTTON: usize = 234;

/// `DAT_1007e3788`: `BCButtonLogic` on "CloseButton" of the display group "IdolStartButtons".
pub const IDOL_START_BUTTONS_CLOSE_BUTTON: usize = 235;

/// `DAT_1007e3790`: `BCButtonLogic` on "SkinPackButton" of the display group "IdolStartButtons".
pub const IDOL_START_BUTTONS_SKIN_PACK_BUTTON: usize = 236;

/// `DAT_1007e3798`: `MultiModelLogic` on a new `BCMultiModel` "IdolStartCheckBox".
pub const IDOL_START_CHECK_BOX: usize = 237;

/// `DAT_1007e37a0`: `MultiModelLogic` on a new `BCMultiModel` "IdolIconCheckBox".
pub const IDOL_ICON_CHECK_BOX: usize = 238;

/// `DAT_1007e37a8`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattleHelpCheckBox".
pub const IDOL_BATTLE_HELP_CHECK_BOX: usize = 239;

/// `DAT_1007e37b0`: `BCButtonLogic` on "BackButton" of the display group "IdolWhatsNewButtons".
pub const IDOL_WHATS_NEW_BUTTONS_BACK_BUTTON: usize = 240;

/// `DAT_1007e37b8`: `BCButtonLogic` on "StartCheckBoxButton" of the display group "IdolOptionsButtons".
pub const START_CHECK_BOX_BUTTON: usize = 241;

/// `DAT_1007e37c0`: `BCButtonLogic` on "IconCheckBoxButton" of the display group "IdolOptionsButtons".
pub const ICON_CHECK_BOX_BUTTON: usize = 242;

/// `DAT_1007e37c8`: `BCButtonLogic` on "ResetButton" of the display group "IdolOptionsButtons".
pub const IDOL_OPTIONS_BUTTONS_RESET_BUTTON: usize = 243;

/// `DAT_1007e37d0`: `BCButtonLogic` on "UnlockButton" of the display group "IdolOptionsButtons".
pub const UNLOCK_BUTTON: usize = 244;

/// `DAT_1007e37d8`: `BCButtonLogic` on "LevelUpButton" of the display group "IdolOptionsButtons".
pub const LEVEL_UP_BUTTON: usize = 245;

/// `DAT_1007e37e0`: `BCButtonLogic` on "BackButton" of the display group "IdolOptionsButtons".
pub const IDOL_OPTIONS_BUTTONS_BACK_BUTTON: usize = 246;

/// `DAT_1007e37e8`: `BCButtonLogic` on "IdolDetailBackButton" of the display group "IdolDetailButtons".
pub const IDOL_DETAIL_BACK_BUTTON: usize = 247;

/// `DAT_1007e37f0`: `BCButtonLogic` on "IdolDetailChallengeButton" of the display group "IdolDetailButtons".
pub const IDOL_DETAIL_CHALLENGE_BUTTON: usize = 248;

/// `DAT_1007e37f8`: `BCButtonLogic` on "IdolSummaryButton" of the display group "IdolDetailHeaderButtons".
pub const IDOL_SUMMARY_BUTTON: usize = 249;

/// `DAT_1007e3800`: `BCButtonLogic` on "HintButton" of the display group "IdolDetailHintButtons".
pub const HINT_BUTTON: usize = 250;

/// `DAT_1007e3808`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailAttackType".
pub const IDOL_DETAIL_ATTACK_TYPE: usize = 251;

/// `DAT_1007e3810`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailRealmType".
pub const IDOL_DETAIL_REALM_TYPE: usize = 252;

/// `DAT_1007e3818`: `MultiModelLogic` on "IdolDetailHeader" of the display group "IdolIconDetailDisplayGroup".
pub const IDOL_DETAIL_HEADER: usize = 253;

/// `DAT_1007e3820`: `MultiModelLogic` on a new `BCMultiModel` "IdolDetailHintPanel".
pub const IDOL_DETAIL_HINT_PANEL: usize = 254;

/// `DAT_1007e3828`: `BCButtonLogic` on "ButtonBlocker" of the display group "IdolDetailHintPanelButtons".
pub const IDOL_DETAIL_HINT_PANEL_BUTTONS_BUTTON_BLOCKER: usize = 255;

/// `DAT_1007e3830`: `BCButtonLogic` on "CloseButton" of the display group "IdolDetailHintPanelButtons".
pub const IDOL_DETAIL_HINT_PANEL_BUTTONS_CLOSE_BUTTON: usize = 256;

/// `DAT_1007e3838`: `MultiModelLogic` on a new `BCMultiModel` "IdolAreYouSurePanel".
pub const IDOL_ARE_YOU_SURE_PANEL: usize = 257;

/// `DAT_1007e3840`: `BCButtonLogic` on "ButtonBlocker" of the display group "IdolAreYouSurePanelButtons".
pub const IDOL_ARE_YOU_SURE_PANEL_BUTTONS_BUTTON_BLOCKER: usize = 258;

/// `DAT_1007e3848`: `BCButtonLogic` on "ResetButton" of the display group "IdolAreYouSurePanelButtons".
pub const IDOL_ARE_YOU_SURE_PANEL_BUTTONS_RESET_BUTTON: usize = 259;

/// `DAT_1007e3850`: `BCButtonLogic` on "CloseButton" of the display group "IdolAreYouSurePanelButtons".
pub const IDOL_ARE_YOU_SURE_PANEL_BUTTONS_CLOSE_BUTTON: usize = 260;

/// `DAT_1007e3858`: `BCButtonLogic` on "BackButton" of the display group "IdolSummaryButtons".
pub const IDOL_SUMMARY_BUTTONS_BACK_BUTTON: usize = 261;

/// `DAT_1007e3860`: `MultiModelLogic` on a new `BCMultiModel` "IdolSummaryAttackTable".
pub const IDOL_SUMMARY_ATTACK_TABLE: usize = 262;

/// `DAT_1007e3868`: `MultiModelLogic` on a new `BCMultiModel` "IdolSummaryRealmTable".
pub const IDOL_SUMMARY_REALM_TABLE: usize = 263;

/// `DAT_1007e3870`: `BCButtonLogic` on "CloseButton" of the display group "IdolAwardButtons".
pub const IDOL_AWARD_BUTTONS_CLOSE_BUTTON: usize = 264;

/// `DAT_1007e3878`: `BCButtonLogic` on "GodIslandButton" of the display group "IdolAwardButtons".
pub const IDOL_AWARD_BUTTONS_GOD_ISLAND_BUTTON: usize = 265;

/// `DAT_1007e3880`: `BCButtonLogic` on "ChallengesButton" of the display group "IdolAwardButtons".
pub const IDOL_AWARD_BUTTONS_CHALLENGES_BUTTON: usize = 266;

/// `DAT_1007e3888`: `BCButtonLogic` on "ChallengesButton" of the display group "IdolSelectionButtons".
pub const IDOL_SELECTION_BUTTONS_CHALLENGES_BUTTON: usize = 267;

/// `DAT_1007e3890`: `MultiModelLogic` on a new `BCMultiModel` "BattleHealthBar1".
pub const BATTLE_HEALTH_BAR1: usize = 268;

/// `DAT_1007e3898`: `MultiModelLogic` on a new `BCMultiModel` "BattleHealthBar2".
pub const BATTLE_HEALTH_BAR2: usize = 269;

/// `DAT_1007e38a0`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot1".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT1: usize = 270;

/// `DAT_1007e38a8`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot2".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT2: usize = 271;

/// `DAT_1007e38b0`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot3".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT3: usize = 272;

/// `DAT_1007e38b8`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot4".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT4: usize = 273;

/// `DAT_1007e38c0`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot5".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT5: usize = 274;

/// `DAT_1007e38c8`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot6".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT6: usize = 275;

/// `DAT_1007e38d0`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot7".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT7: usize = 276;

/// `DAT_1007e38d8`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectSlot8".
pub const IDOL_BATTLE_REALM_EFFECT_SLOT8: usize = 277;

/// `DAT_1007e38e0`: `IdolBattleRealmEffectLogic` on a new `BCMultiModel` "IdolBattleRealmEffectHighlight".
pub const IDOL_BATTLE_REALM_EFFECT_HIGHLIGHT: usize = 278;

/// `DAT_1007e38e8`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattleResultWinnerType".
pub const IDOL_BATTLE_RESULT_WINNER_TYPE: usize = 279;

/// `DAT_1007e38f0`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattleResultLoserType".
pub const IDOL_BATTLE_RESULT_LOSER_TYPE: usize = 280;

/// `DAT_1007e38f8`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattleResultMessage".
pub const IDOL_BATTLE_RESULT_MESSAGE: usize = 281;

/// `DAT_1007e3900`: `BCButtonLogic` on "ExitButton" of the display group "IdolBattleSelectButtons".
pub const EXIT_BUTTON: usize = 282;

/// `DAT_1007e3908`: `BCButtonLogic` on "AttackButton" of the display group "IdolBattleSelectButtons".
pub const ATTACK_BUTTON: usize = 283;

/// `DAT_1007e3910`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattleSelectionAttackButtonSkin".
pub const IDOL_BATTLE_SELECTION_ATTACK_BUTTON_SKIN: usize = 284;

/// `DAT_1007e3918`: `BCButtonLogic` on "BattleSelectButton" of the display group "IdolBattleResultsWinButtons".
pub const IDOL_BATTLE_RESULTS_WIN_BUTTONS_BATTLE_SELECT_BUTTON: usize = 285;

/// `DAT_1007e3920`: `BCButtonLogic` on "RetryButton" of the display group "IdolBattleResultsWinButtons".
pub const IDOL_BATTLE_RESULTS_WIN_BUTTONS_RETRY_BUTTON: usize = 286;

/// `DAT_1007e3928`: `BCButtonLogic` on "NextButton" of the display group "IdolBattleResultsWinButtons".
pub const NEXT_BUTTON: usize = 287;

/// `DAT_1007e3930`: `BCButtonLogic` on "BattleSelectButton" of the display group "IdolBattleResultsLoseButtons".
pub const IDOL_BATTLE_RESULTS_LOSE_BUTTONS_BATTLE_SELECT_BUTTON: usize = 288;

/// `DAT_1007e3938`: `BCButtonLogic` on "RetryButton" of the display group "IdolBattleResultsLoseButtons".
pub const IDOL_BATTLE_RESULTS_LOSE_BUTTONS_RETRY_BUTTON: usize = 289;

/// `DAT_1007e3940`: `BCButtonLogic` on "BattleSelectButton" of the display group "IdolBattleResultsFinalButtons".
pub const IDOL_BATTLE_RESULTS_FINAL_BUTTONS_BATTLE_SELECT_BUTTON: usize = 290;

/// `DAT_1007e3948`: `BCButtonLogic` on "RetryButton" of the display group "IdolBattleResultsFinalButtons".
pub const IDOL_BATTLE_RESULTS_FINAL_BUTTONS_RETRY_BUTTON: usize = 291;

/// `DAT_1007e3950`: `BCButtonLogic` on "BattleSelectButton" of the display group "IdolBattleResultsWinMultiplayerButtons".
pub const IDOL_BATTLE_RESULTS_WIN_MULTIPLAYER_BUTTONS_BATTLE_SELECT_BUTTON: usize = 292;

/// `DAT_1007e3958`: `BCButtonLogic` on "NewMatchButton" of the display group "IdolBattleResultsWinMultiplayerButtons".
pub const IDOL_BATTLE_RESULTS_WIN_MULTIPLAYER_BUTTONS_NEW_MATCH_BUTTON: usize = 293;

/// `DAT_1007e3960`: `BCButtonLogic` on "BattleSelectButton" of the display group "IdolBattleResultsLoseMultiplayerButtons".
pub const IDOL_BATTLE_RESULTS_LOSE_MULTIPLAYER_BUTTONS_BATTLE_SELECT_BUTTON: usize = 294;

/// `DAT_1007e3968`: `BCButtonLogic` on "NewMatchButton" of the display group "IdolBattleResultsLoseMultiplayerButtons".
pub const IDOL_BATTLE_RESULTS_LOSE_MULTIPLAYER_BUTTONS_NEW_MATCH_BUTTON: usize = 295;

/// `DAT_1007e3970`: `BCButtonLogic` on "HelpButton" of the display group "IdolBattleEnemyButtons".
pub const IDOL_BATTLE_ENEMY_BUTTONS_HELP_BUTTON: usize = 296;

/// `DAT_1007e3978`: `BCButtonLogic` on "MultiplayerButton" of the display group "IdolBattleEnemyButtons".
pub const MULTIPLAYER_BUTTON: usize = 297;

/// `DAT_1007e3980`: `BCButtonLogic` on "BattleButton" of the display group "IdolBattleHelpButtons".
pub const BATTLE_BUTTON: usize = 298;

/// `DAT_1007e3988`: `BCButtonLogic` on "CheckBoxButton" of the display group "IdolBattleHelpButtons".
pub const CHECK_BOX_BUTTON: usize = 299;

/// `DAT_1007e3990`: `BCButtonLogic` on "BackButton" of the display group "IdolListButtons".
pub const IDOL_LIST_BUTTONS_BACK_BUTTON: usize = 300;

/// `DAT_1007e3998`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon1AttackType".
pub const IDOL_SELECTION_ICON1_ATTACK_TYPE: usize = 301;

/// `DAT_1007e39a0`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon1RealmType".
pub const IDOL_SELECTION_ICON1_REALM_TYPE: usize = 302;

/// `DAT_1007e39a8`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon1Selected".
pub const IDOL_SELECTION_ICON1_SELECTED: usize = 303;

/// `DAT_1007e39b0`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon2AttackType".
pub const IDOL_SELECTION_ICON2_ATTACK_TYPE: usize = 304;

/// `DAT_1007e39b8`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon2RealmType".
pub const IDOL_SELECTION_ICON2_REALM_TYPE: usize = 305;

/// `DAT_1007e39c0`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon2Selected".
pub const IDOL_SELECTION_ICON2_SELECTED: usize = 306;

/// `DAT_1007e39c8`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon3AttackType".
pub const IDOL_SELECTION_ICON3_ATTACK_TYPE: usize = 307;

/// `DAT_1007e39d0`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon3RealmType".
pub const IDOL_SELECTION_ICON3_REALM_TYPE: usize = 308;

/// `DAT_1007e39d8`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon3Selected".
pub const IDOL_SELECTION_ICON3_SELECTED: usize = 309;

/// `DAT_1007e39e0`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon4AttackType".
pub const IDOL_SELECTION_ICON4_ATTACK_TYPE: usize = 310;

/// `DAT_1007e39e8`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon4RealmType".
pub const IDOL_SELECTION_ICON4_REALM_TYPE: usize = 311;

/// `DAT_1007e39f0`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon4Selected".
pub const IDOL_SELECTION_ICON4_SELECTED: usize = 312;

/// `DAT_1007e39f8`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon5AttackType".
pub const IDOL_SELECTION_ICON5_ATTACK_TYPE: usize = 313;

/// `DAT_1007e3a00`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon5RealmType".
pub const IDOL_SELECTION_ICON5_REALM_TYPE: usize = 314;

/// `DAT_1007e3a08`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon6AttackType".
pub const IDOL_SELECTION_ICON6_ATTACK_TYPE: usize = 315;

/// `DAT_1007e3a10`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon6RealmType".
pub const IDOL_SELECTION_ICON6_REALM_TYPE: usize = 316;

/// `DAT_1007e3a18`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon7AttackType".
pub const IDOL_SELECTION_ICON7_ATTACK_TYPE: usize = 317;

/// `DAT_1007e3a20`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon7RealmType".
pub const IDOL_SELECTION_ICON7_REALM_TYPE: usize = 318;

/// `DAT_1007e3a28`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon8AttackType".
pub const IDOL_SELECTION_ICON8_ATTACK_TYPE: usize = 319;

/// `DAT_1007e3a30`: `MultiModelLogic` on a new `BCMultiModel` "IdolSelectionIcon8RealmType".
pub const IDOL_SELECTION_ICON8_REALM_TYPE: usize = 320;

/// `DAT_1007e3a38`: `IdolSelectionIconLogic` on "IdolSelectionIcon1" of the display group "IdolSelectionLibraryDisplayGroup".
pub const IDOL_SELECTION_ICON1: usize = 321;

/// `DAT_1007e3a40`: `IdolSelectionIconLogic` on "IdolSelectionIcon2" of the display group "IdolSelectionLibraryDisplayGroup".
pub const IDOL_SELECTION_ICON2: usize = 322;

/// `DAT_1007e3a48`: `IdolSelectionIconLogic` on "IdolSelectionIcon3" of the display group "IdolSelectionLibraryDisplayGroup".
pub const IDOL_SELECTION_ICON3: usize = 323;

/// `DAT_1007e3a50`: `IdolSelectionIconLogic` on "IdolSelectionIcon4" of the display group "IdolSelectionLibraryDisplayGroup".
pub const IDOL_SELECTION_ICON4: usize = 324;

/// `DAT_1007e3a58`: `IdolSelectionIconLogic` on "LibraryIdolSelectionIcon1" of the display group "IdolSelectionLibraryDisplayGroup".
pub const LIBRARY_IDOL_SELECTION_ICON1: usize = 325;

/// `DAT_1007e3a60`: `IdolSelectionIconLogic` on "LibraryIdolSelectionIcon2" of the display group "IdolSelectionLibraryDisplayGroup".
pub const LIBRARY_IDOL_SELECTION_ICON2: usize = 326;

/// `DAT_1007e3a68`: `IdolSelectionIconLogic` on "LibraryIdolSelectionIcon3" of the display group "IdolSelectionLibraryDisplayGroup".
pub const LIBRARY_IDOL_SELECTION_ICON3: usize = 327;

/// `DAT_1007e3a70`: `IdolSelectionIconLogic` on "LibraryIdolSelectionIcon4" of the display group "IdolSelectionLibraryDisplayGroup".
pub const LIBRARY_IDOL_SELECTION_ICON4: usize = 328;

/// `DAT_1007e3a78`: `MultiModelLogic` on a new `BCMultiModel` "IdolPanel".
pub const IDOL_PANEL: usize = 329;

/// `DAT_1007e3a80`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattlePlayerAttackType".
pub const IDOL_BATTLE_PLAYER_ATTACK_TYPE: usize = 330;

/// `DAT_1007e3a88`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattlePlayerRealmType".
pub const IDOL_BATTLE_PLAYER_REALM_TYPE: usize = 331;

/// `DAT_1007e3a90`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattleEnemyAttackType".
pub const IDOL_BATTLE_ENEMY_ATTACK_TYPE: usize = 332;

/// `DAT_1007e3a98`: `MultiModelLogic` on a new `BCMultiModel` "IdolBattleEnemyRealmType".
pub const IDOL_BATTLE_ENEMY_REALM_TYPE: usize = 333;

/// `DAT_1007e3aa0`: `MultiModelLogic` on a new `BCMultiModel` "IdolScreenFrame".
pub const IDOL_SCREEN_FRAME: usize = 334;

/// `DAT_1007e3aa8`: `MultiModelLogic` on a new `BCMultiModel` "IdolAwardFrame".
pub const IDOL_AWARD_FRAME: usize = 335;

/// `DAT_1007e3ab0`: `MultiModelLogic` on "IdolListHeader" of the display group "IdolIconListDisplayGroup".
pub const IDOL_LIST_HEADER: usize = 336;

/// `DAT_1007e3ab8`: `IdolScreenLogic` on a new `BCMultiModel` "IdolScreen".
pub const IDOL_SCREEN: usize = 337;

/// `DAT_1007e3ac0`: `BCButtonLogic` on "ButtonBlocker" of the display group "ChallengeStartSendScreen".
pub const CHALLENGE_START_SEND_SCREEN_BUTTON_BLOCKER: usize = 338;

/// `DAT_1007e3ac8`: `BCButtonLogic` on "ButtonBlocker" of the display group "ChallengeConfirmSendScreen".
pub const CHALLENGE_CONFIRM_SEND_SCREEN_BUTTON_BLOCKER: usize = 339;

/// `DAT_1007e3ad0`: `BCButtonLogic` on "ButtonBlocker" of the display group "ChallengeStartRespondScreen".
pub const CHALLENGE_START_RESPOND_SCREEN_BUTTON_BLOCKER: usize = 340;

/// `DAT_1007e3ad8`: `BCButtonLogic` on "ButtonBlocker" of the display group "ChallengeResultsScreen".
pub const CHALLENGE_RESULTS_SCREEN_BUTTON_BLOCKER: usize = 341;

/// `DAT_1007e3ae0`: `BCButtonLogic` on "ButtonBlocker" of the display group "ChallengeLoadingScreen".
pub const CHALLENGE_LOADING_SCREEN_BUTTON_BLOCKER: usize = 342;

/// `DAT_1007e3ae8`: `BCButtonLogic` on "ButtonBlocker" of the display group "SystemError".
pub const SYSTEM_ERROR_BUTTON_BLOCKER: usize = 343;

/// `DAT_1007e3af0`: `BCButtonLogic` on "CloseButton" of the display group "DebugScreen".
pub const DEBUG_SCREEN_CLOSE_BUTTON: usize = 344;

/// `DAT_1007e3af8`: `BCButtonLogic` on "NextPage" of the display group "DebugScreen".
pub const NEXT_PAGE: usize = 345;

/// `DAT_1007e3b00`: `BCButtonLogic` on "PrevPage" of the display group "DebugScreen".
pub const PREV_PAGE: usize = 346;

/// `DAT_1007e3b08`: `BCButtonLogic` on "BlackBox0" of the display group "DebugScreen".
pub const BLACK_BOX0: usize = 347;

/// `DAT_1007e3b10`: `BCButtonLogic` on "BlackBox1" of the display group "DebugScreen".
pub const BLACK_BOX1: usize = 348;

/// `DAT_1007e3b18`: `BCButtonLogic` on "BlackBox2" of the display group "DebugScreen".
pub const BLACK_BOX2: usize = 349;

/// `DAT_1007e3b20`: `BCButtonLogic` on "BlackBox3" of the display group "DebugScreen".
pub const BLACK_BOX3: usize = 350;

/// `DAT_1007e3b28`: `BCButtonLogic` on "BlackBox4" of the display group "DebugScreen".
pub const BLACK_BOX4: usize = 351;

/// `DAT_1007e3b30`: `BCButtonLogic` on "BlackBox5" of the display group "DebugScreen".
pub const BLACK_BOX5: usize = 352;

/// `DAT_1007e3b38`: `BCButtonLogic` on "BlackBox6" of the display group "DebugScreen".
pub const BLACK_BOX6: usize = 353;

/// `DAT_1007e3b40`: `BCButtonLogic` on "BlackBox7" of the display group "DebugScreen".
pub const BLACK_BOX7: usize = 354;

/// `DAT_1007e3b48`: `BCButtonLogic` on "BlackBox8" of the display group "DebugScreen".
pub const BLACK_BOX8: usize = 355;

/// `DAT_1007e3b50`: `BCButtonLogic` on "BlackBox9" of the display group "DebugScreen".
pub const BLACK_BOX9: usize = 356;

/// `DAT_1007e3b58`: `BCButtonLogic` on "BlackBox10" of the display group "DebugScreen".
pub const BLACK_BOX10: usize = 357;

/// `DAT_1007e3b60`: `BCButtonLogic` on "BlackBox11" of the display group "DebugScreen".
pub const BLACK_BOX11: usize = 358;

/// `DAT_1007e3b68`: `BCButtonLogic` on "BlackBox12" of the display group "DebugScreen".
pub const BLACK_BOX12: usize = 359;

/// `DAT_1007e3b70`: `BCButtonLogic` on "BlackBox13" of the display group "DebugScreen".
pub const BLACK_BOX13: usize = 360;

/// `DAT_1007e3b78`: `BCButtonLogic` on "BlackBox14" of the display group "DebugScreen".
pub const BLACK_BOX14: usize = 361;

/// `DAT_1007e3b80`: `BCButtonLogic` on "Background" of the display group "BattleArenaDisplayGroup".
pub const BATTLE_ARENA_BACKGROUND: usize = 362;

/// `DAT_1007e3b88`: `MultiModelLogic` on "Foreground" of the display group "BattleArenaDisplayGroup".
pub const BATTLE_ARENA_FOREGROUND: usize = 363;

/// `DAT_1007e3b90`: `MultiModelLogic` on a new `BCMultiModel` "BattlePedestalRightIdol".
pub const BATTLE_PEDESTAL_RIGHT_IDOL: usize = 364;

/// `DAT_1007e3b98`: `MultiModelLogic` on a new `BCMultiModel` "BattlePedestalLeftIdol".
pub const BATTLE_PEDESTAL_LEFT_IDOL: usize = 365;

/// `DAT_1007e3ba0`: `MultiModelLogic` on "BattlePedestalRight" of the display group "BattleArenaDisplayGroup".
pub const BATTLE_PEDESTAL_RIGHT: usize = 366;

/// `DAT_1007e3ba8`: `MultiModelLogic` on "BattlePedestalLeft" of the display group "BattleArenaDisplayGroup".
pub const BATTLE_PEDESTAL_LEFT: usize = 367;

/// `DAT_1007e3bb0`: `MultiModelLogic` on "Island" of the display group "GodIslandDisplayGroup".
pub const GOD_ISLAND_ISLAND: usize = 368;

/// `DAT_1007e3bb8`: `MultiModelLogic` on "Foreground" of the display group "GodIslandDisplayGroup".
pub const GOD_ISLAND_FOREGROUND: usize = 369;

/// `DAT_1007e3bc0`: `IdolPedestalLogic` on "Pedestal1" of the display group "GodIslandDisplayGroup".
pub const GOD_ISLAND_PEDESTAL1: usize = 370;

/// `DAT_1007e3bc8`: `IdolPedestalLogic` on "Pedestal2" of the display group "GodIslandDisplayGroup".
pub const GOD_ISLAND_PEDESTAL2: usize = 371;

/// `DAT_1007e3bd0`: `IdolPedestalLogic` on "Pedestal3" of the display group "GodIslandDisplayGroup".
pub const GOD_ISLAND_PEDESTAL3: usize = 372;

/// `DAT_1007e3bd8`: `IdolPedestalLogic` on "Pedestal4" of the display group "GodIslandDisplayGroup".
pub const GOD_ISLAND_PEDESTAL4: usize = 373;

/// `DAT_1007e3be0`: `TempleDoorLogic` on "Temple" of the display group "GodIslandDisplayGroup".
pub const GOD_ISLAND_TEMPLE: usize = 374;

/// `DAT_1007e3be8`: `MultiModelLogic` on "LavaPitRight" of the display group "BattleArenaDisplayGroup".
pub const LAVA_PIT_RIGHT: usize = 375;

/// `DAT_1007e3bf0`: `MultiModelLogic` on "LavaPitLeft" of the display group "BattleArenaDisplayGroup".
pub const LAVA_PIT_LEFT: usize = 376;

/// `DAT_1007e3bf8`: `MultiModelLogic` on a new `BCMultiModel` "PlayerSword".
pub const PLAYER_SWORD: usize = 377;

/// `DAT_1007e3c00`: `MultiModelLogic` on a new `BCMultiModel` "PlayerShield".
pub const PLAYER_SHIELD: usize = 378;

/// `DAT_1007e3c08`: `MultiModelLogic` on a new `BCMultiModel` "PlayerEnergy".
pub const PLAYER_ENERGY: usize = 379;

/// `DAT_1007e3c10`: `BattleEnergyBallLogic` on a new `BCMultiModel` "PlayerEnergyBall1".
pub const PLAYER_ENERGY_BALL1: usize = 380;

/// `DAT_1007e3c18`: `BattleEnergyBallLogic` on a new `BCMultiModel` "PlayerEnergyBall2".
pub const PLAYER_ENERGY_BALL2: usize = 381;

/// `DAT_1007e3c20`: `BattleEnergyBallLogic` on a new `BCMultiModel` "PlayerEnergyBall3".
pub const PLAYER_ENERGY_BALL3: usize = 382;

/// `DAT_1007e3c28`: `BattleEnergyBallLogic` on a new `BCMultiModel` "PlayerEnergyBall4".
pub const PLAYER_ENERGY_BALL4: usize = 383;

/// `DAT_1007e3c30`: `MultiModelLogic` on a new `BCMultiModel` "PlayerEnergyShield".
pub const PLAYER_ENERGY_SHIELD: usize = 384;

/// `DAT_1007e3c38`: `MultiModelLogic` on a new `BCMultiModel` "PlayerEnergyBeam".
pub const PLAYER_ENERGY_BEAM: usize = 385;

/// `DAT_1007e3c40`: `MultiModelLogic` on a new `BCMultiModel` "PlayerEnergyBeamStart".
pub const PLAYER_ENERGY_BEAM_START: usize = 386;

/// `DAT_1007e3c48`: `MultiModelLogic` on a new `BCMultiModel` "PlayerEnergyBeamEnd".
pub const PLAYER_ENERGY_BEAM_END: usize = 387;

/// `DAT_1007e3c50`: `MultiModelLogic` on a new `BCMultiModel` "EnemySword".
pub const ENEMY_SWORD: usize = 388;

/// `DAT_1007e3c58`: `MultiModelLogic` on a new `BCMultiModel` "EnemyShield".
pub const ENEMY_SHIELD: usize = 389;

/// `DAT_1007e3c60`: `MultiModelLogic` on a new `BCMultiModel` "EnemyEnergy".
pub const ENEMY_ENERGY: usize = 390;

/// `DAT_1007e3c68`: `BattleEnergyBallLogic` on a new `BCMultiModel` "EnemyEnergyBall1".
pub const ENEMY_ENERGY_BALL1: usize = 391;

/// `DAT_1007e3c70`: `BattleEnergyBallLogic` on a new `BCMultiModel` "EnemyEnergyBall2".
pub const ENEMY_ENERGY_BALL2: usize = 392;

/// `DAT_1007e3c78`: `BattleEnergyBallLogic` on a new `BCMultiModel` "EnemyEnergyBall3".
pub const ENEMY_ENERGY_BALL3: usize = 393;

/// `DAT_1007e3c80`: `BattleEnergyBallLogic` on a new `BCMultiModel` "EnemyEnergyBall4".
pub const ENEMY_ENERGY_BALL4: usize = 394;

/// `DAT_1007e3c88`: `MultiModelLogic` on a new `BCMultiModel` "EnemyEnergyShield".
pub const ENEMY_ENERGY_SHIELD: usize = 395;

/// `DAT_1007e3c90`: `MultiModelLogic` on a new `BCMultiModel` "EnemyEnergyBeam".
pub const ENEMY_ENERGY_BEAM: usize = 396;

/// `DAT_1007e3c98`: `MultiModelLogic` on a new `BCMultiModel` "EnemyEnergyBeamStart".
pub const ENEMY_ENERGY_BEAM_START: usize = 397;

/// `DAT_1007e3ca0`: `MultiModelLogic` on a new `BCMultiModel` "EnemyEnergyBeamEnd".
pub const ENEMY_ENERGY_BEAM_END: usize = 398;

/// `DAT_1007e3ca8`: `EpisodeAlertLogic` on a new `BCMultiModel` "EpisodeAlert".
pub const EPISODE_ALERT: usize = 399;

/// `DAT_1007e3cb0`: `BattleAlertLogic` on a new `BCMultiModel` "BattleAlert".
pub const BATTLE_ALERT: usize = 400;

/// `DAT_1007e3cb8`: `FailBoatLogic` on the display group "FailBoatDisplayGroup".
pub const FAIL_BOAT_DISPLAY_GROUP: usize = 401;

/// `DAT_1007e3cc0`: `MultiModelLogic` on "Boat" of the display group "FailBoatDisplayGroup".
pub const BOAT: usize = 402;

/// `DAT_1007e3cc8`: `ApocalypseGemLogic` on a new `BCMultiModel` "ApocalypseGemTime".
pub const APOCALYPSE_GEM_TIME: usize = 403;

/// `DAT_1007e3cd0`: `ApocalypseGemLogic` on a new `BCMultiModel` "ApocalypseGemLocust".
pub const APOCALYPSE_GEM_LOCUST: usize = 404;

/// `DAT_1007e3cd8`: `ApocalypseGemLogic` on a new `BCMultiModel` "ApocalypseGemDance".
pub const APOCALYPSE_GEM_DANCE: usize = 405;

/// `DAT_1007e3ce0`: `ApocalypseGemLogic` on a new `BCMultiModel` "ApocalypseGemPlague".
pub const APOCALYPSE_GEM_PLAGUE: usize = 406;

/// `DAT_1007e3ce8`: `ApocalypseGemLogic` on a new `BCMultiModel` "ApocalypseGemPurification".
pub const APOCALYPSE_GEM_PURIFICATION: usize = 407;

/// `DAT_1007e3cf0`: `ApocalypseGemLogic` on a new `BCMultiModel` "ApocalypseGemCloset".
pub const APOCALYPSE_GEM_CLOSET: usize = 408;

/// `DAT_1007e3cf8`: `ApocalypseGemLogic` on a new `BCMultiModel` "ApocalypseGemXenu".
pub const APOCALYPSE_GEM_XENU: usize = 409;

/// `DAT_1007e3d00`: `MultiModelLogic` on "Background" of the display group "ApocalypseIslandDisplayGroup".
pub const APOCALYPSE_ISLAND_BACKGROUND: usize = 410;

/// `DAT_1007e3d08`: `MultiModelLogic` on "Foreground" of the display group "ApocalypseIslandDisplayGroup".
pub const APOCALYPSE_ISLAND_FOREGROUND: usize = 411;

/// `DAT_1007e3d10`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalFire1".
pub const APOCALYPSE_GEM_PEDESTAL_FIRE1: usize = 412;

/// `DAT_1007e3d18`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalFire2".
pub const APOCALYPSE_GEM_PEDESTAL_FIRE2: usize = 413;

/// `DAT_1007e3d20`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalFire3".
pub const APOCALYPSE_GEM_PEDESTAL_FIRE3: usize = 414;

/// `DAT_1007e3d28`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalFire4".
pub const APOCALYPSE_GEM_PEDESTAL_FIRE4: usize = 415;

/// `DAT_1007e3d30`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalFire5".
pub const APOCALYPSE_GEM_PEDESTAL_FIRE5: usize = 416;

/// `DAT_1007e3d38`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalFire6".
pub const APOCALYPSE_GEM_PEDESTAL_FIRE6: usize = 417;

/// `DAT_1007e3d40`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalFire7".
pub const APOCALYPSE_GEM_PEDESTAL_FIRE7: usize = 418;

/// `DAT_1007e3d48`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalSymbol1".
pub const APOCALYPSE_GEM_PEDESTAL_SYMBOL1: usize = 419;

/// `DAT_1007e3d50`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalSymbol2".
pub const APOCALYPSE_GEM_PEDESTAL_SYMBOL2: usize = 420;

/// `DAT_1007e3d58`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalSymbol3".
pub const APOCALYPSE_GEM_PEDESTAL_SYMBOL3: usize = 421;

/// `DAT_1007e3d60`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalSymbol4".
pub const APOCALYPSE_GEM_PEDESTAL_SYMBOL4: usize = 422;

/// `DAT_1007e3d68`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalSymbol5".
pub const APOCALYPSE_GEM_PEDESTAL_SYMBOL5: usize = 423;

/// `DAT_1007e3d70`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalSymbol6".
pub const APOCALYPSE_GEM_PEDESTAL_SYMBOL6: usize = 424;

/// `DAT_1007e3d78`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalSymbol7".
pub const APOCALYPSE_GEM_PEDESTAL_SYMBOL7: usize = 425;

/// `DAT_1007e3d80`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalColorSymbol1".
pub const APOCALYPSE_GEM_PEDESTAL_COLOR_SYMBOL1: usize = 426;

/// `DAT_1007e3d88`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalColorSymbol2".
pub const APOCALYPSE_GEM_PEDESTAL_COLOR_SYMBOL2: usize = 427;

/// `DAT_1007e3d90`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalColorSymbol3".
pub const APOCALYPSE_GEM_PEDESTAL_COLOR_SYMBOL3: usize = 428;

/// `DAT_1007e3d98`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalColorSymbol4".
pub const APOCALYPSE_GEM_PEDESTAL_COLOR_SYMBOL4: usize = 429;

/// `DAT_1007e3da0`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalColorSymbol5".
pub const APOCALYPSE_GEM_PEDESTAL_COLOR_SYMBOL5: usize = 430;

/// `DAT_1007e3da8`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalColorSymbol6".
pub const APOCALYPSE_GEM_PEDESTAL_COLOR_SYMBOL6: usize = 431;

/// `DAT_1007e3db0`: `MultiModelLogic` on a new `BCMultiModel` "ApocalypseGemPedestalColorSymbol7".
pub const APOCALYPSE_GEM_PEDESTAL_COLOR_SYMBOL7: usize = 432;

/// `DAT_1007e3db8`: `ApocalypseGemPedestalLogic` on "Pedestal1" of the display group "ApocalypseIslandDisplayGroup".
pub const APOCALYPSE_ISLAND_PEDESTAL1: usize = 433;

/// `DAT_1007e3dc0`: `ApocalypseGemPedestalLogic` on "Pedestal2" of the display group "ApocalypseIslandDisplayGroup".
pub const APOCALYPSE_ISLAND_PEDESTAL2: usize = 434;

/// `DAT_1007e3dc8`: `ApocalypseGemPedestalLogic` on "Pedestal3" of the display group "ApocalypseIslandDisplayGroup".
pub const APOCALYPSE_ISLAND_PEDESTAL3: usize = 435;

/// `DAT_1007e3dd0`: `ApocalypseGemPedestalLogic` on "Pedestal4" of the display group "ApocalypseIslandDisplayGroup".
pub const APOCALYPSE_ISLAND_PEDESTAL4: usize = 436;

/// `DAT_1007e3dd8`: `ApocalypseGemPedestalLogic` on "Pedestal5" of the display group "ApocalypseIslandDisplayGroup".
pub const PEDESTAL5: usize = 437;

/// `DAT_1007e3de0`: `ApocalypseGemPedestalLogic` on "Pedestal6" of the display group "ApocalypseIslandDisplayGroup".
pub const PEDESTAL6: usize = 438;

/// `DAT_1007e3de8`: `ApocalypseGemPedestalLogic` on "Pedestal7" of the display group "ApocalypseIslandDisplayGroup".
pub const PEDESTAL7: usize = 439;

/// `DAT_1007e3df0`: `ApocalypseRoomFaderLogic` on a new `BCMultiModel` "ApocalypseRoomFader".
pub const APOCALYPSE_ROOM_FADER: usize = 440;

/// `DAT_1007e3df8`: `MultiModelLogic` on "Temple" of the display group "ApocalypseIslandDisplayGroup".
pub const APOCALYPSE_ISLAND_TEMPLE: usize = 441;

/// `DAT_1007e3e00`: `MultiModelLogic` on "TempleBase" of the display group "ApocalypseIslandDisplayGroup".
pub const TEMPLE_BASE: usize = 442;

/// `DAT_1007e3e08`: `MultiModelLogic` on "ClockDial3MultiModel" of the display group "ApocalypseIslandDisplayGroup".
pub const CLOCK_DIAL3_MULTI_MODEL: usize = 443;

/// `DAT_1007e3e10`: `MultiModelLogic` on "ClockDial2MultiModel" of the display group "ApocalypseIslandDisplayGroup".
pub const CLOCK_DIAL2_MULTI_MODEL: usize = 444;

/// `DAT_1007e3e18`: `MultiModelLogic` on "ClockDial1MultiModel" of the display group "ApocalypseIslandDisplayGroup".
pub const CLOCK_DIAL1_MULTI_MODEL: usize = 445;

/// `DAT_1007e3e20`: `MultiModelLogic` on a new `BCMultiModel` "SunPygmy".
pub const SUN_PYGMY: usize = 446;

/// `DAT_1007e3e28`: `ApocalypseCalendarLogic` on "ClockWheelMultiModel" of the display group "ApocalypseIslandDisplayGroup".
pub const CLOCK_WHEEL_MULTI_MODEL: usize = 447;

/// `DAT_1007e3e30`: `ApocalypseTempleDoorLogic` on "TempleDoor" of the display group "ApocalypseIslandDisplayGroup".
pub const TEMPLE_DOOR: usize = 448;

/// `DAT_1007e3e38`: `ApocalypseTempleDoorLogic` on "TemplePortal" of the display group "ApocalypseIslandDisplayGroup".
pub const TEMPLE_PORTAL: usize = 449;

/// `DAT_1007e3e40`: `MultiModelLogic` on "RoomTopHalf" of the display group "TimeTempleDisplayGroup".
pub const TIME_TEMPLE_ROOM_TOP_HALF: usize = 450;

/// `DAT_1007e3e48`: `MultiModelLogic` on "Energy" of the display group "TimeTempleDisplayGroup".
pub const ENERGY: usize = 451;

/// `DAT_1007e3e50`: `MultiModelLogic` on "RoomBottomHalf" of the display group "TimeTempleDisplayGroup".
pub const TIME_TEMPLE_ROOM_BOTTOM_HALF: usize = 452;

/// `DAT_1007e3e58`: `ApocalypseTimeTemplePlatformLogic` on "Platform" of the display group "TimeTempleDisplayGroup".
pub const TIME_TEMPLE_PLATFORM: usize = 453;

/// `DAT_1007e3e60`: `ApocalypseTimeTempleLeverLogic` on "Lever" of the display group "TimeTempleDisplayGroup".
pub const LEVER: usize = 454;

/// `DAT_1007e3e68`: `BCButtonLogic` on "EnergyButton" of the display group "TimeTempleDisplayGroup".
pub const ENERGY_BUTTON: usize = 455;

/// `DAT_1007e3e70`: `ApocalypseTempleDoorLogic` on "Door2" of the display group "TimeTempleDisplayGroup".
pub const TIME_TEMPLE_DOOR2: usize = 456;

/// `DAT_1007e3e78`: `ApocalypseTempleDoorLogic` on "Door1" of the display group "TimeTempleDisplayGroup".
pub const TIME_TEMPLE_DOOR1: usize = 457;

/// `DAT_1007e3e80`: `MultiModelLogic` on "RoomTopHalf" of the display group "LocustTempleDisplayGroup".
pub const LOCUST_TEMPLE_ROOM_TOP_HALF: usize = 458;

/// `DAT_1007e3e88`: `MultiModelLogic` on "RoomBottomHalf" of the display group "LocustTempleDisplayGroup".
pub const LOCUST_TEMPLE_ROOM_BOTTOM_HALF: usize = 459;

/// `DAT_1007e3e90`: `MultiModelLogic` on a new `BCMultiModel` "LocustQueenBelly".
pub const LOCUST_QUEEN_BELLY: usize = 460;

/// `DAT_1007e3e98`: `LocustQueenLogic` on "Queen" of the display group "LocustTempleDisplayGroup".
pub const QUEEN: usize = 461;

/// `DAT_1007e3ea0`: `MultiModelLogic` on a new `BCMultiModel` "LocustSlash".
pub const LOCUST_SLASH: usize = 462;

/// `DAT_1007e3ea8`: `MultiModelLogic` on a new `BCMultiModel` "LocustSplat".
pub const LOCUST_SPLAT: usize = 463;

/// `DAT_1007e3eb0`: `PygmyBottomLogic` on a new `BCMultiModel` "LocustPygmySkull1".
pub const LOCUST_PYGMY_SKULL1: usize = 464;

/// `DAT_1007e3eb8`: `PygmyBottomLogic` on a new `BCMultiModel` "LocustPygmySkull2".
pub const LOCUST_PYGMY_SKULL2: usize = 465;

/// `DAT_1007e3ec0`: `PygmyBottomLogic` on a new `BCMultiModel` "LocustPygmySkull3".
pub const LOCUST_PYGMY_SKULL3: usize = 466;

/// `DAT_1007e3ec8`: `PygmyBottomLogic` on a new `BCMultiModel` "LocustPygmyBone1".
pub const LOCUST_PYGMY_BONE1: usize = 467;

/// `DAT_1007e3ed0`: `PygmyBottomLogic` on a new `BCMultiModel` "LocustPygmyBone2".
pub const LOCUST_PYGMY_BONE2: usize = 468;

/// `DAT_1007e3ed8`: `PygmyBottomLogic` on a new `BCMultiModel` "LocustPygmyBone3".
pub const LOCUST_PYGMY_BONE3: usize = 469;

/// `DAT_1007e3ee0`: `ApocalypseTempleDoorLogic` on "Door1" of the display group "LocustTempleDisplayGroup".
pub const LOCUST_TEMPLE_DOOR1: usize = 470;

/// `DAT_1007e3ee8`: `ApocalypseTempleDoorLogic` on "Door2" of the display group "LocustTempleDisplayGroup".
pub const LOCUST_TEMPLE_DOOR2: usize = 471;

/// `DAT_1007e3ef0`: `ApocalypseTempleDoorLogic` on "Door3" of the display group "LocustTempleDisplayGroup".
pub const DOOR3: usize = 472;

/// `DAT_1007e3ef8`: `MultiModelLogic` on "RoomTopHalf" of the display group "DanceTempleDisplayGroup".
pub const DANCE_TEMPLE_ROOM_TOP_HALF: usize = 473;

/// `DAT_1007e3f00`: `MultiModelLogic` on "RoomBottomHalf" of the display group "DanceTempleDisplayGroup".
pub const DANCE_TEMPLE_ROOM_BOTTOM_HALF: usize = 474;

/// `DAT_1007e3f08`: `ApocalypseTempleDoorLogic` on "Door1" of the display group "DanceTempleDisplayGroup".
pub const DANCE_TEMPLE_DOOR1: usize = 475;

/// `DAT_1007e3f10`: `ApocalypseTempleDoorLogic` on "Door2" of the display group "DanceTempleDisplayGroup".
pub const DANCE_TEMPLE_DOOR2: usize = 476;

/// `DAT_1007e3f18`: `GoGoCageLogic` on "DiscoCage" of the display group "DanceTempleDisplayGroup".
pub const DISCO_CAGE: usize = 477;

/// `DAT_1007e3f20`: `DiscoBallLogic` on "DiscoBall" of the display group "DanceTempleDisplayGroup".
pub const DISCO_BALL: usize = 478;

/// `DAT_1007e3f28`: `DiscoLightsLogic` on "DiscoLights" of the display group "DanceTempleDisplayGroup".
pub const DISCO_LIGHTS: usize = 479;

/// `DAT_1007e3f30`: `DanceBalloonBundleLogic` on "BalloonBundle" of the display group "DanceTempleDisplayGroup".
pub const BALLOON_BUNDLE: usize = 480;

/// `DAT_1007e3f38`: `DanceBalloonLogic` on a new `BCMultiModel` "DanceBalloon".
pub const DANCE_BALLOON: usize = 481;

/// `DAT_1007e3f40`: `BCButtonLogic` on "ButtonBlocker1" of the display group "DanceDanceExecutionButtons".
pub const BUTTON_BLOCKER1: usize = 482;

/// `DAT_1007e3f48`: `BCButtonLogic` on "ButtonBlocker2" of the display group "DanceDanceExecutionButtons".
pub const BUTTON_BLOCKER2: usize = 483;

/// `DAT_1007e3f50`: `BCButtonLogic` on "ButtonBlocker3" of the display group "DanceDanceExecutionButtons".
pub const BUTTON_BLOCKER3: usize = 484;

/// `DAT_1007e3f58`: `BCButtonLogic` on "ButtonBlocker4" of the display group "DanceDanceExecutionButtons".
pub const BUTTON_BLOCKER4: usize = 485;

/// `DAT_1007e3f60`: `BCButtonLogic` on "LeftButton" of the display group "DanceDanceExecutionButtons".
pub const LEFT_BUTTON: usize = 486;

/// `DAT_1007e3f68`: `BCButtonLogic` on "DownButton" of the display group "DanceDanceExecutionButtons".
pub const DOWN_BUTTON: usize = 487;

/// `DAT_1007e3f70`: `BCButtonLogic` on "UpButton" of the display group "DanceDanceExecutionButtons".
pub const UP_BUTTON: usize = 488;

/// `DAT_1007e3f78`: `BCButtonLogic` on "RightButton" of the display group "DanceDanceExecutionButtons".
pub const RIGHT_BUTTON: usize = 489;

/// `DAT_1007e3f80`: `MultiModelLogic` on a new `BCMultiModel` "DanceBar".
pub const DANCE_BAR: usize = 490;

/// `DAT_1007e3f88`: `MultiModelLogic` on a new `BCMultiModel` "DanceLeftOverlay".
pub const DANCE_LEFT_OVERLAY: usize = 491;

/// `DAT_1007e3f90`: `MultiModelLogic` on a new `BCMultiModel` "DanceDownOverlay".
pub const DANCE_DOWN_OVERLAY: usize = 492;

/// `DAT_1007e3f98`: `MultiModelLogic` on a new `BCMultiModel` "DanceUpOverlay".
pub const DANCE_UP_OVERLAY: usize = 493;

/// `DAT_1007e3fa0`: `MultiModelLogic` on a new `BCMultiModel` "DanceRightOverlay".
pub const DANCE_RIGHT_OVERLAY: usize = 494;

/// `DAT_1007e3fa8`: `MultiModelLogic` on a new `BCMultiModel` "DanceLeftFeedback".
pub const DANCE_LEFT_FEEDBACK: usize = 495;

/// `DAT_1007e3fb0`: `MultiModelLogic` on a new `BCMultiModel` "DanceDownFeedback".
pub const DANCE_DOWN_FEEDBACK: usize = 496;

/// `DAT_1007e3fb8`: `MultiModelLogic` on a new `BCMultiModel` "DanceUpFeedback".
pub const DANCE_UP_FEEDBACK: usize = 497;

/// `DAT_1007e3fc0`: `MultiModelLogic` on a new `BCMultiModel` "DanceRightFeedback".
pub const DANCE_RIGHT_FEEDBACK: usize = 498;

/// `DAT_1007e3fc8`: `MultiModelLogic` on a new `BCMultiModel` "DanceDancePygmyFace".
pub const DANCE_DANCE_PYGMY_FACE: usize = 499;

/// `DAT_1007e3fd0`: `DanceDanceExecutionLogic` on "DanceDanceExecution" of the display group "DanceTempleDisplayGroup".
pub const DANCE_DANCE_EXECUTION: usize = 500;

/// `DAT_1007e3fd8`: `DanceDanceExecutionScreenLogic` on a new `BCMultiModel` "DanceDanceExecutionScreen".
pub const DANCE_DANCE_EXECUTION_SCREEN: usize = 501;

/// `DAT_1007e3fe0`: `MultiModelLogic` on "RoomTopHalf" of the display group "PlagueTempleDisplayGroup".
pub const PLAGUE_TEMPLE_ROOM_TOP_HALF: usize = 502;

/// `DAT_1007e3fe8`: `ApocalypseTempleFountainLogic` on "PlagueFountain" of the display group "PlagueTempleDisplayGroup".
pub const PLAGUE_FOUNTAIN: usize = 503;

/// `DAT_1007e3ff0`: `ApocalypseTempleDoorLogic` on "Door1" of the display group "PlagueTempleDisplayGroup".
pub const PLAGUE_TEMPLE_DOOR1: usize = 504;

/// `DAT_1007e3ff8`: `ApocalypseTempleDoorLogic` on "Door2" of the display group "PlagueTempleDisplayGroup".
pub const PLAGUE_TEMPLE_DOOR2: usize = 505;

/// `DAT_1007e4000`: `MultiModelLogic` on "RoomTopHalf" of the display group "PurificationTempleDisplayGroup".
pub const PURIFICATION_TEMPLE_ROOM_TOP_HALF: usize = 506;

/// `DAT_1007e4008`: `ApocalypseTempleFountainLogic` on "PurificationFountain" of the display group "PurificationTempleDisplayGroup".
pub const PURIFICATION_FOUNTAIN: usize = 507;

/// `DAT_1007e4010`: `ApocalypseTempleDoorLogic` on "Door1" of the display group "PurificationTempleDisplayGroup".
pub const PURIFICATION_TEMPLE_DOOR1: usize = 508;

/// `DAT_1007e4018`: `MultiModelLogic` on "PlagueSign" of the display group "RockIslandDisplayGroup".
pub const PLAGUE_SIGN: usize = 509;

/// `DAT_1007e4020`: `MultiModelLogic` on "Background" of the display group "ClosetTempleDisplayGroup".
pub const CLOSET_TEMPLE_BACKGROUND: usize = 510;

/// `DAT_1007e4028`: `MultiModelLogic` on "Platform" of the display group "ClosetTempleDisplayGroup".
pub const CLOSET_TEMPLE_PLATFORM: usize = 511;

/// `DAT_1007e4030`: `ClosetTempleCouchLogic` on "Couch" of the display group "ClosetTempleDisplayGroup".
pub const COUCH: usize = 512;

/// `DAT_1007e4038`: `ClosetTempleClosetDoorLogic` on "SlidingDoor" of the display group "ClosetTempleDisplayGroup".
pub const SLIDING_DOOR: usize = 513;

/// `DAT_1007e4040`: `ClosetTempleThetanMeterLogic` on "ThetanMeter" of the display group "ClosetTempleDisplayGroup".
pub const THETAN_METER: usize = 514;

/// `DAT_1007e4048`: `ApocalypseTempleDoorLogic` on "Door1" of the display group "ClosetTempleDisplayGroup".
pub const CLOSET_TEMPLE_DOOR1: usize = 515;

/// `DAT_1007e4050`: `MultiModelLogic` on "SpaceBackground" of the display group "XenuTempleDisplayGroup".
pub const SPACE_BACKGROUND: usize = 516;

/// `DAT_1007e4058`: `XenuTempleLogic` on "Background" of the display group "XenuTempleDisplayGroup".
pub const XENU_TEMPLE_BACKGROUND: usize = 517;

/// `DAT_1007e4060`: `XenuLogic` on "Xenu" of the display group "XenuTempleDisplayGroup".
pub const XENU: usize = 518;

/// `DAT_1007e4068`: `ApocalypseTempleDoorLogic` on "Door1" of the display group "XenuTempleDisplayGroup".
pub const XENU_TEMPLE_DOOR1: usize = 519;

/// `DAT_1007e4070`: `BCButtonLogic` on "Symbol01Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL01_BUTTON: usize = 520;

/// `DAT_1007e4078`: `BCButtonLogic` on "Symbol02Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL02_BUTTON: usize = 521;

/// `DAT_1007e4080`: `BCButtonLogic` on "Symbol03Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL03_BUTTON: usize = 522;

/// `DAT_1007e4088`: `BCButtonLogic` on "Symbol04Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL04_BUTTON: usize = 523;

/// `DAT_1007e4090`: `BCButtonLogic` on "Symbol05Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL05_BUTTON: usize = 524;

/// `DAT_1007e4098`: `BCButtonLogic` on "Symbol06Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL06_BUTTON: usize = 525;

/// `DAT_1007e40a0`: `BCButtonLogic` on "Symbol07Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL07_BUTTON: usize = 526;

/// `DAT_1007e40a8`: `BCButtonLogic` on "Symbol08Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL08_BUTTON: usize = 527;

/// `DAT_1007e40b0`: `BCButtonLogic` on "Symbol09Button" of the display group "UnderwaterTempleButtonsDisplayGroup".
pub const SYMBOL09_BUTTON: usize = 528;

/// `DAT_1007e40b8`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape01".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE01: usize = 529;

/// `DAT_1007e40c0`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape02".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE02: usize = 530;

/// `DAT_1007e40c8`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape03".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE03: usize = 531;

/// `DAT_1007e40d0`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape04".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE04: usize = 532;

/// `DAT_1007e40d8`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape05".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE05: usize = 533;

/// `DAT_1007e40e0`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape06".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE06: usize = 534;

/// `DAT_1007e40e8`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape07".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE07: usize = 535;

/// `DAT_1007e40f0`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape08".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE08: usize = 536;

/// `DAT_1007e40f8`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterTempleSymbolShape09".
pub const UNDERWATER_TEMPLE_SYMBOL_SHAPE09: usize = 537;

/// `DAT_1007e4100`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol01".
pub const UNDERWATER_TEMPLE_SYMBOL01: usize = 538;

/// `DAT_1007e4108`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol02".
pub const UNDERWATER_TEMPLE_SYMBOL02: usize = 539;

/// `DAT_1007e4110`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol03".
pub const UNDERWATER_TEMPLE_SYMBOL03: usize = 540;

/// `DAT_1007e4118`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol04".
pub const UNDERWATER_TEMPLE_SYMBOL04: usize = 541;

/// `DAT_1007e4120`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol05".
pub const UNDERWATER_TEMPLE_SYMBOL05: usize = 542;

/// `DAT_1007e4128`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol06".
pub const UNDERWATER_TEMPLE_SYMBOL06: usize = 543;

/// `DAT_1007e4130`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol07".
pub const UNDERWATER_TEMPLE_SYMBOL07: usize = 544;

/// `DAT_1007e4138`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol08".
pub const UNDERWATER_TEMPLE_SYMBOL08: usize = 545;

/// `DAT_1007e4140`: `UnderwaterTempleSymbolLogic` on a new `BCMultiModel` "UnderwaterTempleSymbol09".
pub const UNDERWATER_TEMPLE_SYMBOL09: usize = 546;

/// `DAT_1007e4148`: `UnderwaterTempleLogic` on a new `BCMultiModel` "UnderwaterTemple".
pub const UNDERWATER_TEMPLE: usize = 547;

/// `DAT_1007e4150`: `MultiModelLogic` on a new `BCMultiModel` "UnderwaterCoralReefForeground".
pub const UNDERWATER_CORAL_REEF_FOREGROUND: usize = 548;

/// `DAT_1007e4158`: `BCButtonLogic` on "CloseButton" of the display group "MapContentDisplayGroup".
pub const MAP_CONTENT_CLOSE_BUTTON: usize = 549;

/// `DAT_1007e4160`: `AdBlockerLogic` on "AdBlocker" of the display group "MapContentDisplayGroup".
pub const AD_BLOCKER: usize = 550;

/// `DAT_1007e4168`: `BCButtonLogic` on "SandIslandButton" of the display group "MapContentDisplayGroup".
pub const SAND_ISLAND_BUTTON: usize = 551;

/// `DAT_1007e4170`: `BCButtonLogic` on "RockIslandButton" of the display group "MapContentDisplayGroup".
pub const ROCK_ISLAND_BUTTON: usize = 552;

/// `DAT_1007e4178`: `BCButtonLogic` on "GraveyardIslandButton" of the display group "MapContentDisplayGroup".
pub const GRAVEYARD_ISLAND_BUTTON: usize = 553;

/// `DAT_1007e4180`: `BCButtonLogic` on "IceIslandButton" of the display group "MapContentDisplayGroup".
pub const ICE_ISLAND_BUTTON: usize = 554;

/// `DAT_1007e4188`: `BCButtonLogic` on "ApeIslandButton" of the display group "MapContentDisplayGroup".
pub const APE_ISLAND_BUTTON: usize = 555;

/// `DAT_1007e4190`: `BCButtonLogic` on "MisfitIslandButton" of the display group "MapContentDisplayGroup".
pub const MISFIT_ISLAND_BUTTON: usize = 556;

/// `DAT_1007e4198`: `BCButtonLogic` on "GodIslandButton" of the display group "MapContentDisplayGroup".
pub const MAP_CONTENT_GOD_ISLAND_BUTTON: usize = 557;

/// `DAT_1007e41a0`: `BCButtonLogic` on "ApocalypseIslandButton" of the display group "MapContentDisplayGroup".
pub const APOCALYPSE_ISLAND_BUTTON: usize = 558;

/// `DAT_1007e41a8`: `BCButtonLogic` on "UnderwaterButton" of the display group "MapContentDisplayGroup".
pub const UNDERWATER_BUTTON: usize = 559;

/// `DAT_1007e41b0`: `BCButtonLogic` on "UnderwaterCoralReefButton" of the display group "MapContentDisplayGroup".
pub const UNDERWATER_CORAL_REEF_BUTTON: usize = 560;

/// `DAT_1007e41b8`: `BCButtonLogic` on "IslandLocationButton" of the display group "MenuDisplayGroup".
pub const ISLAND_LOCATION_BUTTON: usize = 561;

/// `DAT_1007e41c0`: `BCButtonLogic` on "IslandMenuButton" of the display group "MenuDisplayGroup".
pub const ISLAND_MENU_BUTTON: usize = 562;

/// `DAT_1007e41c8`: `BCButtonLogic` on "GlobalIslandMenuButton" of the display group "MenuDisplayGroup".
pub const GLOBAL_ISLAND_MENU_BUTTON: usize = 563;

/// `DAT_1007e41d0`: `BCButtonLogic` on "GodPowerMenuButton" of the display group "MenuDisplayGroup".
pub const GOD_POWER_MENU_BUTTON: usize = 564;

/// `DAT_1007e41d8`: `BCButtonLogic` on "CustomizeMenuButton" of the display group "MenuDisplayGroup".
pub const CUSTOMIZE_MENU_BUTTON: usize = 565;

/// `DAT_1007e41e0`: `BCButtonLogic` on "CommunityMenuButton" of the display group "MenuDisplayGroup".
pub const COMMUNITY_MENU_BUTTON: usize = 566;

/// `DAT_1007e41e8`: `BCButtonLogic` on "SettingsMenuButton" of the display group "MenuDisplayGroup".
pub const SETTINGS_MENU_BUTTON: usize = 567;

/// `DAT_1007e41f0`: `BCButtonLogic` on "HelpButton" of the display group "MenuDisplayGroup".
pub const MENU_HELP_BUTTON: usize = 568;

/// `DAT_1007e41f8`: `BCButtonLogic` on "DebugButton" of the display group "MenuDisplayGroup".
pub const DEBUG_BUTTON: usize = 569;

/// `DAT_1007e4200`: `BCButtonLogic` on "BoulderButton" of the display group "SandIslandMenuDisplayGroup".
pub const BOULDER_BUTTON: usize = 570;

/// `DAT_1007e4208`: `BCButtonLogic` on "FishButton" of the display group "SandIslandMenuDisplayGroup".
pub const SAND_ISLAND_MENU_FISH_BUTTON: usize = 571;

/// `DAT_1007e4210`: `BCButtonLogic` on "AntHillButton" of the display group "SandIslandMenuDisplayGroup".
pub const ANT_HILL_BUTTON: usize = 572;

/// `DAT_1007e4218`: `BCButtonLogic` on "WaterWingsButton" of the display group "SandIslandMenuDisplayGroup".
pub const SAND_ISLAND_MENU_WATER_WINGS_BUTTON: usize = 573;

/// `DAT_1007e4220`: `BCButtonLogic` on "FishButton" of the display group "RockIslandMenuDisplayGroup".
pub const ROCK_ISLAND_MENU_FISH_BUTTON: usize = 574;

/// `DAT_1007e4228`: `BCButtonLogic` on "OutHouseButton" of the display group "RockIslandMenuDisplayGroup".
pub const OUT_HOUSE_BUTTON: usize = 575;

/// `DAT_1007e4230`: `BCButtonLogic` on "TRexButton" of the display group "RockIslandMenuDisplayGroup".
pub const T_REX_BUTTON: usize = 576;

/// `DAT_1007e4238`: `BCButtonLogic` on "SpearButton" of the display group "RockIslandMenuDisplayGroup".
pub const ROCK_ISLAND_MENU_SPEAR_BUTTON: usize = 577;

/// `DAT_1007e4240`: `BCButtonLogic` on "WoodButton" of the display group "RockIslandMenuDisplayGroup".
pub const WOOD_BUTTON: usize = 578;

/// `DAT_1007e4248`: `BCButtonLogic` on "EggButton" of the display group "RockIslandMenuDisplayGroup".
pub const EGG_BUTTON: usize = 579;

/// `DAT_1007e4250`: `BCButtonLogic` on "ZombieGraveButton" of the display group "GraveyardIslandMenuDisplayGroup".
pub const ZOMBIE_GRAVE_BUTTON: usize = 580;

/// `DAT_1007e4258`: `BCButtonLogic` on "GhostButton" of the display group "GraveyardIslandMenuDisplayGroup".
pub const GHOST_BUTTON: usize = 581;

/// `DAT_1007e4260`: `BCButtonLogic` on "SpiderWebButton" of the display group "GraveyardIslandMenuDisplayGroup".
pub const SPIDER_WEB_BUTTON: usize = 582;

/// `DAT_1007e4268`: `BCButtonLogic` on "SpiderButton" of the display group "GraveyardIslandMenuDisplayGroup".
pub const SPIDER_BUTTON: usize = 583;

/// `DAT_1007e4270`: `BCButtonLogic` on "SpearButton" of the display group "GraveyardIslandMenuDisplayGroup".
pub const GRAVEYARD_ISLAND_MENU_SPEAR_BUTTON: usize = 584;

/// `DAT_1007e4278`: `BCButtonLogic` on "WaterWingsButton" of the display group "GraveyardIslandMenuDisplayGroup".
pub const GRAVEYARD_ISLAND_MENU_WATER_WINGS_BUTTON: usize = 585;

/// `DAT_1007e4280`: `BCButtonLogic` on "FishButton" of the display group "IceIslandMenuDisplayGroup".
pub const ICE_ISLAND_MENU_FISH_BUTTON: usize = 586;

/// `DAT_1007e4288`: `BCButtonLogic` on "WaterWingsButton" of the display group "IceIslandMenuDisplayGroup".
pub const ICE_ISLAND_MENU_WATER_WINGS_BUTTON: usize = 587;

/// `DAT_1007e4290`: `BCButtonLogic` on "IceMonsterButton" of the display group "IceIslandMenuDisplayGroup".
pub const ICE_MONSTER_BUTTON: usize = 588;

/// `DAT_1007e4298`: `BCButtonLogic` on "IglooButton" of the display group "IceIslandMenuDisplayGroup".
pub const IGLOO_BUTTON: usize = 589;

/// `DAT_1007e42a0`: `BCButtonLogic` on "IceHoleButton" of the display group "IceIslandMenuDisplayGroup".
pub const ICE_HOLE_BUTTON: usize = 590;

/// `DAT_1007e42a8`: `BCButtonLogic` on "IceSquidButton" of the display group "IceIslandMenuDisplayGroup".
pub const ICE_SQUID_BUTTON: usize = 591;

/// `DAT_1007e42b0`: `BCButtonLogic` on "ApeAltarButton" of the display group "ApeIslandMenuDisplayGroup".
pub const APE_ALTAR_BUTTON: usize = 592;

/// `DAT_1007e42b8`: `BCButtonLogic` on "ApeApeButton" of the display group "ApeIslandMenuDisplayGroup".
pub const APE_APE_BUTTON: usize = 593;

/// `DAT_1007e42c0`: `BCButtonLogic` on "ApeBananaTreeButton" of the display group "ApeIslandMenuDisplayGroup".
pub const APE_BANANA_TREE_BUTTON: usize = 594;

/// `DAT_1007e42c8`: `BCButtonLogic` on "ApeMonkeyButton" of the display group "ApeIslandMenuDisplayGroup".
pub const APE_MONKEY_BUTTON: usize = 595;

/// `DAT_1007e42d0`: `BCButtonLogic` on "ApeKonkeyDongButton" of the display group "ApeIslandMenuDisplayGroup".
pub const APE_KONKEY_DONG_BUTTON: usize = 596;

/// `DAT_1007e42d8`: `BCButtonLogic` on "MoronSnakeButton" of the display group "MisfitIslandMenuDisplayGroup".
pub const MORON_SNAKE_BUTTON: usize = 597;

/// `DAT_1007e42e0`: `BCButtonLogic` on "WaterWingsButton" of the display group "MisfitIslandMenuDisplayGroup".
pub const MISFIT_ISLAND_MENU_WATER_WINGS_BUTTON: usize = 598;

/// `DAT_1007e42e8`: `BCButtonLogic` on "MisfitMoronPestsButton" of the display group "MisfitIslandMenuDisplayGroup".
pub const MISFIT_MORON_PESTS_BUTTON: usize = 599;

/// `DAT_1007e42f0`: `BCButtonLogic` on "IdolSelectionButton" of the display group "GodIslandMenuDisplayGroup".
pub const IDOL_SELECTION_BUTTON: usize = 600;

/// `DAT_1007e42f8`: `BCButtonLogic` on "IdolsButton" of the display group "GodIslandMenuDisplayGroup".
pub const IDOLS_BUTTON: usize = 601;

/// `DAT_1007e4300`: `BCButtonLogic` on "WaterWingsButton" of the display group "GodIslandMenuDisplayGroup".
pub const GOD_ISLAND_MENU_WATER_WINGS_BUTTON: usize = 602;

/// `DAT_1007e4308`: `BCButtonLogic` on "WaterWingsButton" of the display group "ApocalypseIslandMenuDisplayGroup".
pub const APOCALYPSE_ISLAND_MENU_WATER_WINGS_BUTTON: usize = 603;

/// `DAT_1007e4310`: `BCButtonLogic` on "PiranhaButton" of the display group "UnderwaterMenuDisplayGroup".
pub const PIRANHA_BUTTON: usize = 604;

/// `DAT_1007e4318`: `BCButtonLogic` on "LaserSharkButton" of the display group "UnderwaterMenuDisplayGroup".
pub const LASER_SHARK_BUTTON: usize = 605;

/// `DAT_1007e4320`: `BCButtonLogic` on "TileDrainButton" of the display group "UnderwaterMenuDisplayGroup".
pub const TILE_DRAIN_BUTTON: usize = 606;

/// `DAT_1007e4328`: `BCButtonLogic` on "JellyfishButton" of the display group "UnderwaterCoralReefMenuDisplayGroup".
pub const JELLYFISH_BUTTON: usize = 607;

/// `DAT_1007e4330`: `BCButtonLogic` on "WeatherButton" of the display group "GlobalIslandMenuDisplayGroup".
pub const WEATHER_BUTTON: usize = 608;

/// `DAT_1007e4338`: `BCButtonLogic` on "SharkButton" of the display group "GlobalIslandMenuDisplayGroup".
pub const SHARK_BUTTON: usize = 609;

/// `DAT_1007e4340`: `BCButtonLogic` on "BirdButton" of the display group "GlobalIslandMenuDisplayGroup".
pub const BIRD_BUTTON: usize = 610;

/// `DAT_1007e4348`: `BCButtonLogic` on "CrackHoleButton" of the display group "GlobalIslandMenuDisplayGroup".
pub const CRACK_HOLE_BUTTON: usize = 611;

/// `DAT_1007e4350`: `BCButtonLogic` on "TheRunsButton" of the display group "GlobalIslandMenuDisplayGroup".
pub const THE_RUNS_BUTTON: usize = 612;

/// `DAT_1007e4358`: `BCButtonLogic` on "CharlieButton" of the display group "GlobalIslandMenuDisplayGroup".
pub const CHARLIE_BUTTON: usize = 613;

/// `DAT_1007e4360`: `BCButtonLogic` on "DrumButton" of the display group "GodPowerMenuDisplayGroup".
pub const DRUM_BUTTON: usize = 614;

/// `DAT_1007e4368`: `BCButtonLogic` on "TemperatureButton" of the display group "GodPowerMenuDisplayGroup".
pub const TEMPERATURE_BUTTON: usize = 615;

/// `DAT_1007e4370`: `BCButtonLogic` on "HurricaneButton" of the display group "GodPowerMenuDisplayGroup".
pub const HURRICANE_BUTTON: usize = 616;

/// `DAT_1007e4378`: `BCButtonLogic` on "RainbowButton" of the display group "GodPowerMenuDisplayGroup".
pub const RAINBOW_BUTTON: usize = 617;

/// `DAT_1007e4380`: `BCButtonLogic` on "SkinPackButton" of the display group "CustomizeMenuDisplayGroup".
pub const CUSTOMIZE_MENU_SKIN_PACK_BUTTON: usize = 618;

/// `DAT_1007e4388`: `BCButtonLogic` on "BonusPackButton" of the display group "CustomizeMenuDisplayGroup".
pub const BONUS_PACK_BUTTON: usize = 619;

/// `DAT_1007e4390`: `BCButtonLogic` on "StoryButton" of the display group "CustomizeMenuDisplayGroup".
pub const STORY_BUTTON: usize = 620;

/// `DAT_1007e4398`: `BCButtonLogic` on "StatsButton" of the display group "CustomizeMenuDisplayGroup".
pub const STATS_BUTTON: usize = 621;

/// `DAT_1007e43a0`: `BCButtonLogic` on "IdolsChallengeButton" of the display group "CustomizeMenuDisplayGroup".
pub const IDOLS_CHALLENGE_BUTTON: usize = 622;

/// `DAT_1007e43a8`: `BCButtonLogic` on "NewsButton" of the display group "CommunityMenuDisplayGroup".
pub const NEWS_BUTTON: usize = 623;

/// `DAT_1007e43b0`: `BCButtonLogic` on "OpenFeintButton" of the display group "CommunityMenuDisplayGroup" (missing in 1.48.2: created with a nil display object).
pub const OPEN_FEINT_BUTTON: usize = 624;

/// `DAT_1007e43b8`: `BCButtonLogic` on "ChallengeButton" of the display group "CommunityMenuDisplayGroup".
pub const CHALLENGE_BUTTON: usize = 625;

/// `DAT_1007e43c0`: `BCButtonLogic` on "StartScreenButton" of the display group "CommunityMenuDisplayGroup".
pub const START_SCREEN_BUTTON: usize = 626;

/// `DAT_1007e43c8`: `BCButtonLogic` on "MuteButton" of the display group "SettingsMenuDisplayGroup".
pub const MUTE_BUTTON: usize = 627;

/// `DAT_1007e43d0`: `BCButtonLogic` on "VibrateButton" of the display group "SettingsMenuDisplayGroup".
pub const VIBRATE_BUTTON: usize = 628;

/// `DAT_1007e43d8`: `BCButtonLogic` on "GravityButton" of the display group "SettingsMenuDisplayGroup".
pub const GRAVITY_BUTTON: usize = 629;

/// `DAT_1007e43e0`: `BCButtonLogic` on "NamesButton" of the display group "SettingsMenuDisplayGroup".
pub const NAMES_BUTTON: usize = 630;

/// `DAT_1007e43e8`: `BCButtonLogic` on "LabelsButton" of the display group "SettingsMenuDisplayGroup".
pub const LABELS_BUTTON: usize = 631;

/// Number of slots.
pub const COUNT: usize = 632;
