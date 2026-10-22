//! `IdolManager` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod idol_battle;
pub use idol_battle::*;
pub mod idol_episodes;
pub use idol_episodes::*;
pub mod idol_multiplayer;
pub use idol_multiplayer::*;

/// `IdolManager` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IdolManager {
    /// `mIdolManagerListener`: `@"<IdolManagerListener>"`
    pub idol_manager_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mbIdolScreenActive`: `B`
    pub idol_screen_active: bool,

    /// `mIdolScreenLogic`: `@"IdolScreenLogic"`
    pub idol_screen_logic: Option<Entity>,

    /// `mActivatedHintCount`: `i`
    pub activated_hint_count: i32,

    /// `mEpisodeAlertLogic`: `@"EpisodeAlertLogic"`
    pub episode_alert_logic: Option<Entity>,

    /// `mBattleAlertLogic`: `@"BattleAlertLogic"`
    pub battle_alert_logic: Option<Entity>,

    /// `mGemIdDictionary`: `@"NSMutableDictionary"` (event, sacrifice or gift id → gem id)
    pub gem_id_dictionary: HashMap<String, i32>,

    /// `mDelayedEpisodeEventArray`: `@"NSMutableArray"` (event ids)
    pub delayed_episode_event_array: Vec<String>,

    /// `mDelayedEpisodeGiftEventArray`: `@"NSMutableArray"` (gift ids)
    pub delayed_episode_gift_event_array: Vec<String>,

    /// `mDelayedEpisodeSacrificeEventArray`: `@"NSMutableArray"` (sacrifice ids)
    pub delayed_episode_sacrifice_event_array: Vec<String>,

    /// `mDelayedAwardScreenArray`: `@"NSMutableArray"` (`NSValue` episode node pointers)
    pub delayed_award_screen_array: Vec<Option<usize>>,

    /// `mpActiveStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub active_struct_xml_episode_node_def: Option<usize>,

    /// `mSelectedStructXmlEpisodeNodeDefArray`: `[4^{BCStructXmlEpisodeNodeDef}]`
    pub selected_struct_xml_episode_node_def_array: [Option<usize>; 4],

    /// `mpEnemyStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub enemy_struct_xml_episode_node_def: Option<usize>,

    /// `mAttackResult`: `i`
    pub attack_result: i32,

    /// `mRealmResult`: `i`
    pub realm_result: i32,

    /// `mPlayerBattleState`: `{PlayerBattleState="mName"@"NSString""mLevel"i"mHealth"f"mMaxHealth"f"mpPlayerStructXmlEpisodeNodeDef"^{BCStructXmlEpisodeNodeDef}"mpBattleStructXmlEpisodeNodeDef"^{BCStructXmlEpisodeNodeDef}"mbWinAttack"B"mbWinRealm"B"mActualAttackDamage"f"mActualReflectDamage"f"mActualDrainHealth"f"mTotalBoostDamage"f"mAttackBoostDamage"f"mAttackReflectDamage"f"mAttackDrainHealth"f"mRealmDamage"f"mTieDamage"f"mTotalDefense"f"mAttackDefense"f"mRealmHealth"f"mTotalDrainLevelPoints"f"mDrainLevelPoints"f"mbIgnoreAttackDefense"B"mbCancelEnemyPowerUps"B"mbUnfreezeIdols"B"mAverageDamage"f"mbHasFreezeIdol"B"mAttackTypeCountArray"[3i]"mRealmTypeCountArray"[5i]"mIdolBattleStateArray"[4{IdolBattleState="mActiveFrameCount"i"mpStructXmlEpisodeNodeDef"^{BCStructXmlEpisodeNodeDef}"mIdolBattleRealmEffectLogic"@"IdolBattleRealmEffectLogic""mDamage"f"mHealth"f"mUsedCount"i"mExperience"f"mFrozenRefCount"i"mbFreezeIdol"B"mpFrozenIdolBattleState"^{IdolBattleState}"mbResurrect"B"mResurrectHealthPerLevel"f"mPotentialAttackStrength"f"mPotentialRealmStrength"f"mPotentialStrength"f"mEnemyAttackLoseCount"i"mEnemyAttackWinCount"i"mEnemyAttackTieCount"i"mEnemyRealmLoseCount"i"mEnemyRealmWinCount"i"mEnemyRealmTieCount"i"mAttackTypeIndex"i"mRealmTypeIndex"i"mChoiceScore"f}]}`
    pub player_battle_state: PlayerBattleState,

    /// `mEnemyBattleState`: `{PlayerBattleState="mName"@"NSString""mLevel"i"mHealth"f"mMaxHealth"f"mpPlayerStructXmlEpisodeNodeDef"^{BCStructXmlEpisodeNodeDef}"mpBattleStructXmlEpisodeNodeDef"^{BCStructXmlEpisodeNodeDef}"mbWinAttack"B"mbWinRealm"B"mActualAttackDamage"f"mActualReflectDamage"f"mActualDrainHealth"f"mTotalBoostDamage"f"mAttackBoostDamage"f"mAttackReflectDamage"f"mAttackDrainHealth"f"mRealmDamage"f"mTieDamage"f"mTotalDefense"f"mAttackDefense"f"mRealmHealth"f"mTotalDrainLevelPoints"f"mDrainLevelPoints"f"mbIgnoreAttackDefense"B"mbCancelEnemyPowerUps"B"mbUnfreezeIdols"B"mAverageDamage"f"mbHasFreezeIdol"B"mAttackTypeCountArray"[3i]"mRealmTypeCountArray"[5i]"mIdolBattleStateArray"[4{IdolBattleState="mActiveFrameCount"i"mpStructXmlEpisodeNodeDef"^{BCStructXmlEpisodeNodeDef}"mIdolBattleRealmEffectLogic"@"IdolBattleRealmEffectLogic""mDamage"f"mHealth"f"mUsedCount"i"mExperience"f"mFrozenRefCount"i"mbFreezeIdol"B"mpFrozenIdolBattleState"^{IdolBattleState}"mbResurrect"B"mResurrectHealthPerLevel"f"mPotentialAttackStrength"f"mPotentialRealmStrength"f"mPotentialStrength"f"mEnemyAttackLoseCount"i"mEnemyAttackWinCount"i"mEnemyAttackTieCount"i"mEnemyRealmLoseCount"i"mEnemyRealmWinCount"i"mEnemyRealmTieCount"i"mAttackTypeIndex"i"mRealmTypeIndex"i"mChoiceScore"f}]}`
    pub enemy_battle_state: PlayerBattleState,

    /// `mPlayerRandomFrozenIdolId`: `@"NSString"`
    pub player_random_frozen_idol_id: Option<String>,

    /// `mEnemyRandomFrozenIdolId`: `@"NSString"`
    pub enemy_random_frozen_idol_id: Option<String>,

    /// `mActiveMatch`: `@"GKTurnBasedMatch"`
    pub active_match: Option<Entity>,

    /// `mMatchPreferences`: `@"BCPreferences"`
    pub match_preferences: Option<Entity>,

    /// `mbLoadActiveMatchData`: `B`
    pub load_active_match_data: bool,

    /// `mbMultiPlayerActive`: `B`
    pub multi_player_active: bool,

    /// `mbMultiplayerMyTurn`: `B`
    pub multiplayer_my_turn: bool,

    /// `mbIAmChallenger`: `B`
    pub i_am_challenger: bool,

    /// `mPlayerPreferencePrefix`: `@"NSString"`
    pub player_preference_prefix: Option<String>,

    /// `mEnemyPreferencePrefix`: `@"NSString"`
    pub enemy_preference_prefix: Option<String>,

    /// `mActivePlayerId`: `@"NSString"`
    pub active_player_id: Option<String>,

    /// `mbGameCenterAuthenticationComplete`: `B`
    pub game_center_authentication_complete: bool,

    /// `mbGameCenterNotAvailable`: `B`
    pub game_center_not_available: bool,

    /// `mbOpenMatchPanelOnAuthenticationComplete`: `B`
    pub open_match_panel_on_authentication_complete: bool,
}

pub const IDOL_MANAGER: Class = Class {
    name: "IdolManager",
    superclass: None,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `IdolManager` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "waitForScreenClosed" => seq_wait_for_screen_closed(world, this, call),
        "checkAutoUnlockChallenges" => seq_check_auto_unlock_challenges(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
