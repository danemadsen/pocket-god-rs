//! `ChallengeManager` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod challenge_match;
pub use challenge_match::*;
pub mod challenge_scoring;
pub use challenge_scoring::*;

/// `ChallengeManager` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct ChallengeManager {
    /// `mChallengeManagerListener`: `@"<ChallengeManagerListener>"`
    pub challenge_manager_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mbGameCenterAuthenticationComplete`: `B`
    pub game_center_authentication_complete: bool,

    /// `mbGameCenterNotAvailable`: `B`
    pub game_center_not_available: bool,

    /// `mActiveMatch`: `@"GKTurnBasedMatch"`
    pub active_match: Option<Entity>,

    /// `mActivePlayerId`: `@"NSString"`
    pub active_player_id: Option<String>,

    /// `mMatchPreferences`: `@"BCPreferences"`
    pub match_preferences: Option<Entity>,

    /// `mbOpenMatchPanelOnAuthenticationComplete`: `B`
    pub open_match_panel_on_authentication_complete: bool,

    /// `mbGameActive`: `B`
    pub game_active: bool,

    /// `mChallengeScoreType`: `i`
    pub challenge_score_type: i32,

    /// `mActiveGameChallengeType`: `i`
    pub active_game_challenge_type: i32,

    /// `mActiveGameScore`: `i`
    pub active_game_score: i32,

    /// `mBestChallengeGameScore`: `i`
    pub best_challenge_game_score: i32,

    /// `mOpponentPlayerAlias`: `@"NSString"`
    pub opponent_player_alias: Option<String>,

    /// `mChallengeType`: `i`
    pub challenge_type: i32,

    /// `mbError`: `B`
    pub error: bool,

    /// `mbSavingMatchData`: `B`
    pub saving_match_data: bool,

    /// `mbSubmittingChallenge`: `B`
    pub submitting_challenge: bool,

    /// `mbSubmittingChallengeResult`: `B`
    pub submitting_challenge_result: bool,

    /// `mbStartingMatchFromViewController`: `B`
    pub starting_match_from_view_controller: bool,

    /// `mNewChallengeType`: `i`
    pub new_challenge_type: i32,

    /// `mNewChallengeCount`: `i`
    pub new_challenge_count: i32,

    /// `mLogicArray`: `@"NSMutableArray"`
    pub logic_array: Vec<Entity>,

    /// `mChallengeUiLogicArray`: `@"NSMutableArray"`
    pub challenge_ui_logic_array: Vec<Entity>,

    /// `mChallengeCreateUiLogicArray`: `@"NSMutableArray"`
    pub challenge_create_ui_logic_array: Vec<Entity>,

    /// `mbChallengeSendAttemptActive`: `B`
    pub challenge_send_attempt_active: bool,

    /// `mbChallengeRespondAttemptActive`: `B`
    pub challenge_respond_attempt_active: bool,

    /// `mbChallengeScreenActive`: `B`
    pub challenge_screen_active: bool,

    /// `mbChallengeStartSendScreenActive`: `B`
    pub challenge_start_send_screen_active: bool,

    /// `mbChallengeConfirmSendScreenActive`: `B`
    pub challenge_confirm_send_screen_active: bool,

    /// `mbChallengeStartRespondScreenActive`: `B`
    pub challenge_start_respond_screen_active: bool,

    /// `mbChallengeResultsScreenActive`: `B`
    pub challenge_results_screen_active: bool,

    /// `mbChallengeSystemErrorActive`: `B`
    pub challenge_system_error_active: bool,

    /// `mbChallengeSystemErrorDuringResultScreen`: `B`
    pub challenge_system_error_during_result_screen: bool,

    /// `mbChallengeSystemErrorDuringSendChallenge`: `B`
    pub challenge_system_error_during_send_challenge: bool,

    /// `mbChallengeErrorActive`: `B`
    pub challenge_error_active: bool,

    /// `mbChallengeTimerActive`: `B`
    pub challenge_timer_active: bool,

    /// `mAchievementMessageLogic`: `@"MessageLogic"`
    pub achievement_message_logic: Option<Entity>,

    /// `mCountingMessageLogic`: `@"MessageLogic"`
    pub counting_message_logic: Option<Entity>,

    /// `mChallengeTimerLogic`: `@"MessageLogic"`
    pub challenge_timer_logic: Option<Entity>,

    /// `mChallengeAlertLogic`: `@"MessageLogic"`
    pub challenge_alert_logic: Option<Entity>,

    /// `mChallengeInProgressLogic`: `@"MessageLogic"`
    pub challenge_in_progress_logic: Option<Entity>,

    /// `mChallengeCloseButtonLogic`: `@"BCButtonLogic"`
    pub challenge_close_button_logic: Option<Entity>,

    /// `mChallengeCreateNextChallengeButtonLogic`: `@"BCButtonLogic"`
    pub challenge_create_next_challenge_button_logic: Option<Entity>,

    /// `mChallengeCreatePrevChallengeButtonLogic`: `@"BCButtonLogic"`
    pub challenge_create_prev_challenge_button_logic: Option<Entity>,

    /// `mChallengeCreateChallengeButtonLogic`: `@"BCButtonLogic"`
    pub challenge_create_challenge_button_logic: Option<Entity>,

    /// `mChallengeStartSendCancelButtonLogic`: `@"BCButtonLogic"`
    pub challenge_start_send_cancel_button_logic: Option<Entity>,

    /// `mChallengeStartSendGoButtonLogic`: `@"BCButtonLogic"`
    pub challenge_start_send_go_button_logic: Option<Entity>,

    /// `mChallengeConfirmSendCloseButtonLogic`: `@"BCButtonLogic"`
    pub challenge_confirm_send_close_button_logic: Option<Entity>,

    /// `mChallengeStartRespondCancelButtonLogic`: `@"BCButtonLogic"`
    pub challenge_start_respond_cancel_button_logic: Option<Entity>,

    /// `mChallengeStartRespondGoButtonLogic`: `@"BCButtonLogic"`
    pub challenge_start_respond_go_button_logic: Option<Entity>,

    /// `mChallengeResultsCloseButtonLogic`: `@"BCButtonLogic"`
    pub challenge_results_close_button_logic: Option<Entity>,

    /// `mChallengeResultsRechallengeButtonLogic`: `@"BCButtonLogic"`
    pub challenge_results_rechallenge_button_logic: Option<Entity>,

    /// `mChallengeErrorCloseButtonLogic`: `@"BCButtonLogic"`
    pub challenge_error_close_button_logic: Option<Entity>,

    /// `mChallengeSystemErrorCloseButtonLogic`: `@"BCButtonLogic"`
    pub challenge_system_error_close_button_logic: Option<Entity>,
}

pub const CHALLENGE_MANAGER: Class = Class {
    name: "ChallengeManager",
    superclass: None,
    sequence_item,
    id,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    on_message_complete,
    ..Class::NS_OBJECT
};

/// Sequence-item handlers implemented by `ChallengeManager` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "waitForSaveMatchDataComplete" => seq_wait_for_save_match_data_complete(world, this, call),
        "submitChallenge" => seq_submit_challenge(world, this, call),
        "submitChallengeResult" => seq_submit_challenge_result(world, this, call),
        "updateChallengeCreatePanel" => seq_update_challenge_create_panel(world, this, call),
        "updateChallengeStartSendScreen" => seq_update_challenge_start_send_screen(world, this, call),
        "showChallengeConfirmSendScreen" => seq_show_challenge_confirm_send_screen(world, this, call),
        "updateChallengeStartRespondScreen" => seq_update_challenge_start_respond_screen(world, this, call),
        "showChallengeResultsScreen" => seq_show_challenge_results_screen(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        _ => return (Class::NS_OBJECT.sequence_item)(world, this, call),
    };
    Some(result)
}
