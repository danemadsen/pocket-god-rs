//! `IdolScreenLogic` (superclass `BCScreenLogic`).

use pg_assets::data::episodes::Episode;
use pg_engine::display::text;
use pg_engine::library::Library;

use crate::managers::idol_manager::{self, EpisodeNode, IdolManager};
use crate::prelude::*;

pub mod part_1;
pub use part_1::*;
pub mod part_2;
pub use part_2::*;

/// `IdolScreenLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IdolScreenLogic {
    /// `mIdolScreenLogicListener`: `@"<IdolScreenLogicListener>"`
    pub idol_screen_logic_listener: Option<Entity>,

    /// `mIdolPanelLogic`: `@"MultiModelLogic"`
    pub idol_panel_logic: Option<Entity>,

    /// `mIdolStartCheckBoxLogic`: `@"MultiModelLogic"`
    pub idol_start_check_box_logic: Option<Entity>,

    /// `mIdolIconCheckBoxLogic`: `@"MultiModelLogic"`
    pub idol_icon_check_box_logic: Option<Entity>,

    /// `mIdolBattleHelpCheckBoxLogic`: `@"MultiModelLogic"`
    pub idol_battle_help_check_box_logic: Option<Entity>,

    /// `mIdolDetailHeaderHintPanelLogic`: `@"MultiModelLogic"`
    pub idol_detail_header_hint_panel_logic: Option<Entity>,

    /// `mIdolDetailChallengeHeaderLogic`: `@"MultiModelLogic"`
    pub idol_detail_challenge_header_logic: Option<Entity>,

    /// `mIdolDetailChallengeHintItemLogic`: `@"MultiModelLogic"`
    pub idol_detail_challenge_hint_item_logic: Option<Entity>,

    /// `mIdolDetailLevelUpHeaderLogic`: `@"MultiModelLogic"`
    pub idol_detail_level_up_header_logic: Option<Entity>,

    /// `mIdolDetailHeaderLogic`: `@"MultiModelLogic"`
    pub idol_detail_header_logic: Option<Entity>,

    /// `mIdolDetailAttackTypeLogic`: `@"MultiModelLogic"`
    pub idol_detail_attack_type_logic: Option<Entity>,

    /// `mIdolDetailRealmTypeLogic`: `@"MultiModelLogic"`
    pub idol_detail_realm_type_logic: Option<Entity>,

    /// `mIdolAreYouSurePanelLogic`: `@"MultiModelLogic"`
    pub idol_are_you_sure_panel_logic: Option<Entity>,

    /// `mIdolSummaryAttackTableLogic`: `@"MultiModelLogic"`
    pub idol_summary_attack_table_logic: Option<Entity>,

    /// `mIdolSummaryRealmTableLogic`: `@"MultiModelLogic"`
    pub idol_summary_realm_table_logic: Option<Entity>,

    /// `mIdolBattleRealmEffectHighlightLogic`: `@"MultiModelLogic"`
    pub idol_battle_realm_effect_highlight_logic: Option<Entity>,

    /// `mIdolBattleResultWinnerTypeLogic`: `@"MultiModelLogic"`
    pub idol_battle_result_winner_type_logic: Option<Entity>,

    /// `mIdolBattleResultLoserTypeLogic`: `@"MultiModelLogic"`
    pub idol_battle_result_loser_type_logic: Option<Entity>,

    /// `mIdolBattleResultMessageLogic`: `@"MultiModelLogic"`
    pub idol_battle_result_message_logic: Option<Entity>,

    /// `mIdolIconListDisplayGroup`: `@"BCDisplayGroup"`
    pub idol_icon_list_display_group: Option<Entity>,

    /// `mIdolIconDetailDisplayGroup`: `@"BCDisplayGroup"`
    pub idol_icon_detail_display_group: Option<Entity>,

    /// `mIdolSelectionLibraryDisplayGroup`: `@"BCDisplayGroup"`
    pub idol_selection_library_display_group: Option<Entity>,

    /// `mIdolBattleRealmEffectDisplayGroup`: `@"BCDisplayGroup"`
    pub idol_battle_realm_effect_display_group: Option<Entity>,

    /// `mIdolDetailHintButtonsDisplayGroup`: `@"BCDisplayGroup"`
    pub idol_detail_hint_buttons_display_group: Option<Entity>,

    /// `mIdolDetailHintPanelButtonsDisplayGroup`: `@"BCDisplayGroup"`
    pub idol_detail_hint_panel_buttons_display_group: Option<Entity>,

    /// `mIdolAreYouSurePanelButtonsDisplayGroup`: `@"BCDisplayGroup"`
    pub idol_are_you_sure_panel_buttons_display_group: Option<Entity>,

    /// `mIdolScreenCloseButtonLogic`: `@"BCButtonLogic"`
    pub idol_screen_close_button_logic: Option<Entity>,

    /// `mIdolStartWhatsNewButtonLogic`: `@"BCButtonLogic"`
    pub idol_start_whats_new_button_logic: Option<Entity>,

    /// `mIdolStartOptionsButtonLogic`: `@"BCButtonLogic"`
    pub idol_start_options_button_logic: Option<Entity>,

    /// `mIdolStartChallengeButtonLogic`: `@"BCButtonLogic"`
    pub idol_start_challenge_button_logic: Option<Entity>,

    /// `mIdolStartCloseButtonLogic`: `@"BCButtonLogic"`
    pub idol_start_close_button_logic: Option<Entity>,

    /// `mIdolStartSkinPackButtonLogic`: `@"BCButtonLogic"`
    pub idol_start_skin_pack_button_logic: Option<Entity>,

    /// `mIdolWhatsNewBackButtonLogic`: `@"BCButtonLogic"`
    pub idol_whats_new_back_button_logic: Option<Entity>,

    /// `mIdolOptionsStartCheckBoxButtonLogic`: `@"BCButtonLogic"`
    pub idol_options_start_check_box_button_logic: Option<Entity>,

    /// `mIdolOptionsIconCheckBoxButtonLogic`: `@"BCButtonLogic"`
    pub idol_options_icon_check_box_button_logic: Option<Entity>,

    /// `mIdolOptionsResetButtonLogic`: `@"BCButtonLogic"`
    pub idol_options_reset_button_logic: Option<Entity>,

    /// `mIdolOptionsUnlockButtonLogic`: `@"BCButtonLogic"`
    pub idol_options_unlock_button_logic: Option<Entity>,

    /// `mIdolOptionsLevelUpButtonLogic`: `@"BCButtonLogic"`
    pub idol_options_level_up_button_logic: Option<Entity>,

    /// `mIdolOptionsBackButtonLogic`: `@"BCButtonLogic"`
    pub idol_options_back_button_logic: Option<Entity>,

    /// `mIdolDetailPanelBackButtonLogic`: `@"BCButtonLogic"`
    pub idol_detail_panel_back_button_logic: Option<Entity>,

    /// `mIdolDetailPanelChallengeButtonLogic`: `@"BCButtonLogic"`
    pub idol_detail_panel_challenge_button_logic: Option<Entity>,

    /// `mIdolDetailHeaderSummaryButtonLogic`: `@"BCButtonLogic"`
    pub idol_detail_header_summary_button_logic: Option<Entity>,

    /// `mIdolDetailHeaderHintButtonLogic`: `@"BCButtonLogic"`
    pub idol_detail_header_hint_button_logic: Option<Entity>,

    /// `mIdolDetailHintPanelCloseButtonLogic`: `@"BCButtonLogic"`
    pub idol_detail_hint_panel_close_button_logic: Option<Entity>,

    /// `mIdolAreYouSurePanelCloseButtonLogic`: `@"BCButtonLogic"`
    pub idol_are_you_sure_panel_close_button_logic: Option<Entity>,

    /// `mIdolAreYouSurePanelResetButtonLogic`: `@"BCButtonLogic"`
    pub idol_are_you_sure_panel_reset_button_logic: Option<Entity>,

    /// `mIdolSummaryBackButtonLogic`: `@"BCButtonLogic"`
    pub idol_summary_back_button_logic: Option<Entity>,

    /// `mIdolAwardCloseButtonLogic`: `@"BCButtonLogic"`
    pub idol_award_close_button_logic: Option<Entity>,

    /// `mIdolAwardGodIslandButtonLogic`: `@"BCButtonLogic"`
    pub idol_award_god_island_button_logic: Option<Entity>,

    /// `mIdolAwardChallengesButtonLogic`: `@"BCButtonLogic"`
    pub idol_award_challenges_button_logic: Option<Entity>,

    /// `mIdolSelectionChallengesButtonLogic`: `@"BCButtonLogic"`
    pub idol_selection_challenges_button_logic: Option<Entity>,

    /// `mIdolBattleSelectionExitButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_selection_exit_button_logic: Option<Entity>,

    /// `mIdolBattleSelectionAttackButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_selection_attack_button_logic: Option<Entity>,

    /// `mIdolBattleResultsWinBattleSelectButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_win_battle_select_button_logic: Option<Entity>,

    /// `mIdolBattleResultsWinRetryButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_win_retry_button_logic: Option<Entity>,

    /// `mIdolBattleResultsWinNextButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_win_next_button_logic: Option<Entity>,

    /// `mIdolBattleResultsLoseBattleSelectButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_lose_battle_select_button_logic: Option<Entity>,

    /// `mIdolBattleResultsLoseRetryButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_lose_retry_button_logic: Option<Entity>,

    /// `mIdolBattleResultsWinMultiplayerBattleSelectButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_win_multiplayer_battle_select_button_logic: Option<Entity>,

    /// `mIdolBattleResultsWinMultiplayerNewMatchButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_win_multiplayer_new_match_button_logic: Option<Entity>,

    /// `mIdolBattleResultsLoseMultiplayerBattleSelectButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_lose_multiplayer_battle_select_button_logic: Option<Entity>,

    /// `mIdolBattleResultsLoseMultiplayerNewMatchButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_lose_multiplayer_new_match_button_logic: Option<Entity>,

    /// `mIdolBattleResultsFinalBattleSelectButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_final_battle_select_button_logic: Option<Entity>,

    /// `mIdolBattleResultsFinalRetryButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_results_final_retry_button_logic: Option<Entity>,

    /// `mIdolBattleEnemyHelpButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_enemy_help_button_logic: Option<Entity>,

    /// `mIdolBattleEnemyMultiplayerButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_enemy_multiplayer_button_logic: Option<Entity>,

    /// `mIdolBattleHelpBattleButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_help_battle_button_logic: Option<Entity>,

    /// `mIdolBattleHelpCheckBoxButtonLogic`: `@"BCButtonLogic"`
    pub idol_battle_help_check_box_button_logic: Option<Entity>,

    /// `mIdolListBackButtonLogic`: `@"BCButtonLogic"`
    pub idol_list_back_button_logic: Option<Entity>,

    /// `mIdolIconLogicArray`: `@"NSMutableArray"`
    pub idol_icon_logic_array: Vec<Entity>,

    /// `mAvailableIdolIconLogicArray`: `@"NSMutableArray"`
    pub available_idol_icon_logic_array: Vec<Entity>,

    /// `mIdolSelectionIconLogicArray`: `@"NSMutableArray"`
    pub idol_selection_icon_logic_array: Vec<Entity>,

    /// `mAvailableIdolSelectionIconLogicArray`: `@"NSMutableArray"`
    pub available_idol_selection_icon_logic_array: Vec<Entity>,

    /// `mSelectedIdolSelectionIconLogicArray`: `@"NSMutableArray"`
    pub selected_idol_selection_icon_logic_array: Vec<Entity>,

    /// `mDraggedIdolSelectionIconLogic`: `@"IdolSelectionIconLogic"`
    pub dragged_idol_selection_icon_logic: Option<Entity>,

    /// `mChallengeItemLogicArray`: `@"NSMutableArray"`
    pub challenge_item_logic_array: Vec<Entity>,

    /// `mLevelUpItemLogicArray`: `@"NSMutableArray"`
    pub level_up_item_logic_array: Vec<Entity>,

    /// `mPlayerIdolBattleRealmEffectLogicArray`: `@"NSMutableArray"`
    pub player_idol_battle_realm_effect_logic_array: Vec<Entity>,

    /// `mEnemyIdolBattleRealmEffectLogicArray`: `@"NSMutableArray"`
    pub enemy_idol_battle_realm_effect_logic_array: Vec<Entity>,

    /// `mIdolBattleRealmEffectLogicArray`: `@"NSMutableArray"`
    pub idol_battle_realm_effect_logic_array: Vec<Entity>,

    /// `mActiveTouch`: `@"BCTouch"`
    pub active_touch: Option<Entity>,

    /// `mLastDisplayGroupTouchPos`: `{?="x"f"y"f"z"f"w"f}`
    pub last_display_group_touch_pos: Vec4,

    /// `mGameResultsPos`: `{?="x"f"y"f"z"f"w"f}`
    pub game_results_pos: Vec4,

    /// `mIdolScreenMode`: `i`
    pub idol_screen_mode: i32,

    /// `mRootIdolScreenMode`: `i`
    pub root_idol_screen_mode: i32,

    /// `mpStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub struct_xml_episode_node_def: Option<usize>,

    /// `mpActiveStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub active_struct_xml_episode_node_def: Option<usize>,

    /// `mpBattleStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub battle_struct_xml_episode_node_def: Option<usize>,

    /// `mIdolScreenAreaYouSureType`: `i`
    pub idol_screen_area_you_sure_type: i32,

    /// `mScrollDetailDisplayGroupMinYPos`: `f`
    pub scroll_detail_display_group_min_y_pos: f32,

    /// `mScrollDetailDisplayGroupMaxYPos`: `f`
    pub scroll_detail_display_group_max_y_pos: f32,

    /// `mScrollListDisplayGroupMinYPos`: `f`
    pub scroll_list_display_group_min_y_pos: f32,

    /// `mScrollListDisplayGroupMaxYPos`: `f`
    pub scroll_list_display_group_max_y_pos: f32,

    /// `mScrollSelectionDisplayGroupMinXPos`: `f`
    pub scroll_selection_display_group_min_x_pos: f32,

    /// `mScrollSelectionDisplayGroupMaxXPos`: `f`
    pub scroll_selection_display_group_max_x_pos: f32,

    /// `mIdolSelectionIconScrollXPos`: `f`
    pub idol_selection_icon_scroll_x_pos: f32,

    /// `mDragXPosVel`: `f`
    pub drag_x_pos_vel: f32,

    /// `mDragYPosVel`: `f`
    pub drag_y_pos_vel: f32,

    /// `mDragDeltaYPos`: `f`
    pub drag_delta_y_pos: f32,

    /// `mbDisableLibraryScroll`: `B`
    pub disable_library_scroll: bool,

    /// `mbDragPanel`: `B`
    pub drag_panel: bool,

    /// `mbSummaryOpen`: `B`
    pub summary_open: bool,

    /// `mbBattleHelpOpen`: `B`
    pub battle_help_open: bool,

    /// `mbShowingBattleRealmEffectResults`: `B`
    pub showing_battle_realm_effect_results: bool,

    /// `mbShowingPlayerBattleRealmEffectResults`: `B`
    pub showing_player_battle_realm_effect_results: bool,

    /// `mBattleRealmEffectResultIndex`: `i`
    pub battle_realm_effect_result_index: i32,

    /// `mTimeForNextRealmEffectResult`: `f`
    pub time_for_next_realm_effect_result: f32,

    /// `mTouchLogicArray`: `[1000@]`
    pub touch_logic_array: Vec<Entity>,
}

pub const IDOL_SCREEN_LOGIC: Class = Class {
    name: "IdolScreenLogic",
    superclass: Some(&pg_engine::logic::screen::BC_SCREEN_LOGIC),
    sequence_item,
    close_screen_now,
    on_game_frame,
    on_button_press,
    on_button_release,
    on_button_release_outside,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..pg_engine::logic::screen::BC_SCREEN_LOGIC
};

/// Sequence-item handlers implemented by `IdolScreenLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "showBattleRealmEffectResults" => seq_show_battle_realm_effect_results(world, this, call),
        "loadLocationTextureAtlas" => seq_load_location_texture_atlas(world, this, call),
        _ => return (pg_engine::logic::screen::BC_SCREEN_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[IdolScreenLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&IDOL_SCREEN_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `+[IdolScreenLogic instance]` @ 0x10018de50
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().idol_screen_logic
}

/// The side of a Battle of the Gods a `PlayerBattleState *` (the listener's `mPlayerBattleState`
/// or `mEnemyBattleState`) or a realm-effect logic array (`mPlayerIdolBattleRealmEffectLogicArray`
/// or `mEnemyIdolBattleRealmEffectLogicArray`) belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleSide {
    Player,
    Enemy,
}

/// `mIdolScreenLogicListener`: always the `IdolManager` (`initLogicListeners` wires it during
/// `loadGame`, before the screen can open).
fn screen_listener(world: &World, this: Entity) -> Entity {
    let listener = world.obj::<IdolScreenLogic>(this).idol_screen_logic_listener;
    listener.expect("initLogicListeners sets the IdolScreenLogic listener")
}

/// `[mIdolScreenLogicListener playerBattleState]` / `enemyBattleState`: a pointer to the
/// listener's battle state, which the original reads in place.
fn battle_state(world: &World, listener: Entity, side: BattleSide) -> &PlayerBattleState {
    let idol_manager = world.obj::<IdolManager>(listener);
    match side {
        BattleSide::Player => &idol_manager.player_battle_state,
        BattleSide::Enemy => &idol_manager.enemy_battle_state,
    }
}

/// The realm-effect logics shown on one side of the battle HUD.
fn realm_effect_logic_array(screen: &IdolScreenLogic, side: BattleSide) -> &Vec<Entity> {
    match side {
        BattleSide::Player => &screen.player_idol_battle_realm_effect_logic_array,
        BattleSide::Enemy => &screen.enemy_idol_battle_realm_effect_logic_array,
    }
}

/// [`realm_effect_logic_array`], mutably.
fn realm_effect_logic_array_mut(screen: &mut IdolScreenLogic, side: BattleSide) -> &mut Vec<Entity> {
    match side {
        BattleSide::Player => &mut screen.player_idol_battle_realm_effect_logic_array,
        BattleSide::Enemy => &mut screen.enemy_idol_battle_realm_effect_logic_array,
    }
}

/// The runtime members of the episode node `node` (a copy: release it before calling out).
fn episode_node(world: &mut World, node: usize) -> EpisodeNode {
    let table = idol_manager::episode_table(world);
    table.nodes[node].clone()
}

/// The data of the episode node `node` (`BCStructXmlEpisodeNodeDef` +0x08 … +0x80, +0xa8 …).
fn episode_def(world: &mut World, node: usize) -> &'static Episode {
    let table = idol_manager::episode_table(world);
    table.def(node)
}

/// A `BCStructXmlEpisodeNodeDef *` the original dereferences without a nil check.
fn required_node(node: Option<usize>) -> usize {
    node.expect("IdolScreenLogic: the episode node is NULL (the original dereferences it)")
}

/// `[[BCLibrary instance] getDynamicTextById:id]`.
fn dynamic_text(world: &World, id: &str) -> Option<Entity> {
    let library = world.resource::<Library>();
    library.dynamic_text(id)
}

/// `[dynamicText setText:text]` on a dynamic text that may be nil.
fn set_text(world: &mut World, dynamic_text: Option<Entity>, text: Option<&str>) {
    if let Some(dynamic_text) = dynamic_text {
        text::set_text(world, dynamic_text, text);
    }
}

/// `[[[BCLibrary instance] getDynamicTextById:id] setText:text]`.
fn set_dynamic_text(world: &mut World, id: &str, text: Option<&str>) {
    let dynamic_text = dynamic_text(world, id);
    set_text(world, dynamic_text, text);
}

/// `[mIdolScreenLogicListener attackName:attack]`.
fn listener_attack_name(world: &mut World, listener: Entity, attack: i32) -> Option<String> {
    let attack_name = idol_manager::attack_name(world, listener, attack);
    attack_name.map(str::to_owned)
}

/// `[mIdolScreenLogicListener realmName:realm]`.
fn listener_realm_name(world: &mut World, listener: Entity, realm: i32) -> Option<String> {
    let realm_name = idol_manager::realm_name(world, listener, realm);
    realm_name.map(str::to_owned)
}

/// `[mIdolScreenLogicListener fillPowerDescriptionText:text structXmlEpisodeNodeDef:node]`.
fn listener_fill_power_description_text(world: &mut World, listener: Entity, text: &str, node: Option<usize>) -> Option<String> {
    idol_manager::fill_power_description_text(world, listener, text, node)
}
