//! `IdolScreenLogic`: methods in this part of the class.

use pg_engine::animation;
use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::text;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::logic::screen;
use pg_engine::prefs;

use crate::logic::idol_ui_widgets::{idol_battle_realm_effect_logic, idol_icon_logic, idol_selection_icon_logic};
use crate::logic_table;
use crate::managers::game_ui_manager;
use crate::managers::idol_manager::idol_episodes::SkinTextureBitsDef;
use crate::managers::idol_manager::{self, EPISODE_LIST_LEN};
use crate::managers::skin_pack_manager;

use super::*;

/// `PTR_PTR_100569ac8` … `PTR_PTR_100569c48`: the `BCSkinTextureBitsDef` arrays of the episode
/// icon slots 1–9, as indices into [`SKIN_TEXTURE_BITS_DEF_ARRAYS`].
pub const EPISODE_ICON_SLOT_BITS: [usize; 9] = [2, 3, 4, 5, 6, 7, 8, 9, 10];

/// `PTR_PTR_100569dc8` … `PTR_PTR_100569c78`: the `BCSkinTextureBitsDef` arrays of the episode
/// idol slots 1–8, as indices into [`SKIN_TEXTURE_BITS_DEF_ARRAYS`].
pub const EPISODE_IDOL_SLOT_BITS: [usize; 8] = [18, 17, 16, 15, 14, 13, 12, 11];

/// The static one-record `BCSkinTextureBitsDef` arrays `PTR_PTR_100569a68` … `PTR_PTR_100569dc8`
/// (0x30 bytes apart) in address order, as `textures.json` `bits_slots` lists them. A
/// `BCSkinTextureBitsDef *` (`setSkinTextureBitsDefArray:`) is passed around as an index into
/// this table; [`activate_skin_texture_bits_def_array`] uploads an image into one.
pub(crate) static SKIN_TEXTURE_BITS_DEF_ARRAYS: [SkinTextureBitsDef; 19] = [
    SkinTextureBitsDef { texture: "EpisodeAlertIconSlot", x: 128, y: 164, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeAlertIdolSlot", x: 0, y: 66, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot1", x: 0, y: 0, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot2", x: 84, y: 0, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot3", x: 168, y: 0, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot4", x: 0, y: 84, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot5", x: 84, y: 84, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot6", x: 168, y: 84, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot7", x: 0, y: 168, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot8", x: 84, y: 168, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIconSlot9", x: 168, y: 168, width: 84, height: 84 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot8", x: 0, y: 0, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot7", x: 128, y: 0, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot6", x: 0, y: 128, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot5", x: 128, y: 128, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot4", x: 0, y: 0, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot3", x: 128, y: 0, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot2", x: 0, y: 128, width: 128, height: 128 },
    SkinTextureBitsDef { texture: "EpisodeIdolSlot1", x: 128, y: 128, width: 128, height: 128 },
];

/// `IdolDetailChallengeItem1` … `80`: the challenge rows of the detail panel.
const CHALLENGE_ITEM_COUNT: usize = 80;

/// `IdolDetailLevelUpItem1` … `9`: the level-up rows of the detail panel.
const LEVEL_UP_ITEM_COUNT: usize = 9;

/// `IdolIconButton1` … `9`: the icons of one idol list page.
const IDOL_ICON_COUNT: usize = 9;

/// `IdolBattleRealmEffectSlot1` … `8`.
const REALM_EFFECT_SLOT_COUNT: usize = 8;

/// Home x of the four god-idol slots of the selection panel (`setHomeXPos:yPos:`, y = 252).
const SELECTED_ICON_HOME_X_POS: [f32; 4] = [40.0, 141.0, 242.0, 343.0];

/// Home x of the player's four idols on the battle panels (`setBattleHomeXPos:yPos:`, y = 115).
const SELECTED_ICON_BATTLE_HOME_X_POS: [f32; 4] = [50.0, 128.0, 206.0, 284.0];

/// `FUN_100038648(records, file, @"raw")` (`activateSkinTextureBitsBatchDef`) on the
/// `BCSkinTextureBitsDef` array `skin_texture_bits_def_array` (an index into
/// [`SKIN_TEXTURE_BITS_DEF_ARRAYS`]): uploads `<file>.raw` into its slot.
pub(crate) fn activate_skin_texture_bits_def_array(world: &mut World, skin_texture_bits_def_array: usize, file: &str) {
    let record = &SKIN_TEXTURE_BITS_DEF_ARRAYS[skin_texture_bits_def_array];
    idol_manager::activate_episode_bits_slot(world, record, file);
}

/// `Some([[BCLibrary instance] getLogic:index])`, for an ivar.
fn table_logic(world: &World, index: usize) -> Option<Entity> {
    let logic = library::logic(world, index);
    Some(logic)
}

/// An ivar `initWithDisplayObject:` fills from the logic table: never nil afterwards.
fn wired(logic: Option<Entity>) -> Entity {
    logic.expect("initWithDisplayObject: wires every part of the IdolScreenLogic")
}

/// A display group the original reaches through `[group pos]`: a nil group is a NULL pointer
/// dereference (a crash) there, a panic here.
fn required_display_group(display_group: Option<Entity>) -> Entity {
    display_group.expect("IdolScreenLogic: the display group is nil (the original dereferences [nil pos])")
}

/// `self->mpStructXmlEpisodeNodeDef`, which the original dereferences.
fn detail_node(world: &World, this: Entity) -> usize {
    let node = world.obj::<IdolScreenLogic>(this).struct_xml_episode_node_def;
    required_node(node)
}

/// `[logic setBehavior:behavior]` on a logic that may be nil.
fn set_logic_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    if let Some(logic) = logic {
        logic::set_behavior(world, logic, Some(behavior));
    }
}

/// `[[logic displayObject] removeFromDisplayGroup]` on a logic that may be nil.
fn remove_logic_display_object(world: &mut World, logic: Option<Entity>) {
    if let Some(logic) = logic {
        let display_object = logic::display_object(world, logic);
        display::remove_from_display_group(world, display_object);
    }
}

/// `[displayGroup removeFromDisplayGroup]` on a group that may be nil.
fn remove_display_group(world: &mut World, display_group: Option<Entity>) {
    if let Some(display_group) = display_group {
        display::remove_from_display_group(world, display_group);
    }
}

/// `[displayObject pos]` written as `(x, y)` (z and w kept).
fn set_display_object_xy(world: &mut World, display_object: Entity, x_pos: f32, y_pos: f32) {
    let mut object = display::object_mut(world, display_object);
    object.pos.x = x_pos;
    object.pos.y = y_pos;
}

/// `[displayObject show]` / `hide`.
fn set_display_object_shown(world: &mut World, display_object: Entity, shown: bool) {
    display::object_mut(world, display_object).hidden = !shown;
}

/// `[[SkinPackManager instance] loadTextureAtlasChannel:channel]`.
fn load_skin_pack_texture_atlas_channel(world: &mut World, channel: i32) {
    let skin_pack_manager = skin_pack_manager::instance(world);
    if let Some(skin_pack_manager) = skin_pack_manager {
        skin_pack_manager::load_texture_atlas_channel(world, skin_pack_manager, channel);
    }
}

/// `[preferences getIntPreference:key]` (0 for a nil document).
fn int_preference(world: &World, preferences: Option<Entity>, key: &str) -> i32 {
    match preferences {
        Some(preferences) => prefs::get_int_preference(world, preferences, key),
        None => 0,
    }
}

/// `[displayGroup getDisplayObject:id]` on a group that may be nil.
fn display_group_object(world: &World, display_group: Option<Entity>, id: &str) -> Option<Entity> {
    let display_group = display_group?;
    group::get_display_object(world, display_group, id)
}

/// `[[GameUiManager instance] showMessage:message sequenceId:sequence_id pos:mGameResultsPos
/// messageLogic:mIdolBattleResultMessageLogic]`.
fn show_battle_result_message(world: &mut World, this: Entity, message: &str, sequence_id: &str) {
    let game_ui_manager = game_ui_manager::instance(world);
    let Some(game_ui_manager) = game_ui_manager else {
        return;
    };

    let screen_logic = world.obj::<IdolScreenLogic>(this);
    let game_results_pos = screen_logic.game_results_pos;
    let message_logic = wired(screen_logic.idol_battle_result_message_logic);
    game_ui_manager::show_message_sequence_id_pos_message_logic(world, game_ui_manager, message, sequence_id, game_results_pos, message_logic);
}

/// `setSequenceId:battleEnemySequenceId:battleEnemyHighlightSequenceId:battleEnemyLockedSequenceId:
/// battleSequenceId:battleHighlightSequenceId:battleFrozenSequenceId:` with the behaviours of
/// selection icon `number` (1–8).
fn set_selection_icon_sequence_ids(world: &mut World, idol_selection_icon: Entity, number: usize) {
    let sequence_id = format!("IdolSelectionIconSlot{number}");
    let battle_enemy_sequence_id = format!("IdolBattleEnemySelectionIconSlot{number}");
    let battle_enemy_highlight_sequence_id = format!("IdolBattleEnemyHighlightIconSlot{number}");
    let battle_enemy_locked_sequence_id = format!("IdolBattleEnemyLockedIconSlot{number}");
    let battle_sequence_id = format!("IdolBattleSelectionIconSlot{number}");
    let battle_highlight_sequence_id = format!("IdolBattleSelectionIconHighlightSlot{number}");
    let battle_frozen_sequence_id = format!("IdolBattleSelectionIconFrozenSlot{number}");

    idol_selection_icon_logic::set_sequence_id(
        world,
        idol_selection_icon,
        &sequence_id,
        &battle_enemy_sequence_id,
        &battle_enemy_highlight_sequence_id,
        &battle_enemy_locked_sequence_id,
        &battle_sequence_id,
        &battle_highlight_sequence_id,
        &battle_frozen_sequence_id,
    );
}

/// `-[IdolScreenLogic initWithDisplayObject:]` @ 0x10018de5c
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IdolScreenLogic::default());
    screen::init_with_display_object(world, this, display_object);

    world.resource_mut::<Singletons>().idol_screen_logic = Some(this);

    // `getLogic:` and `getDisplayGroupById:` have no side effects, so the parts are looked up
    // in the original's order and stored together.
    let wiring = IdolScreenLogic {
        game_results_pos: Vec4::new(240.0, 200.0, 0.0, 0.0),
        idol_panel_logic: table_logic(world, logic_table::IDOL_PANEL),
        idol_screen_close_button_logic: table_logic(world, logic_table::IDOL_SCREEN_BUTTONS_CLOSE_BUTTON),
        idol_start_whats_new_button_logic: table_logic(world, logic_table::WHATS_NEW_BUTTON),
        idol_start_options_button_logic: table_logic(world, logic_table::OPTIONS_BUTTON),
        idol_start_challenge_button_logic: table_logic(world, logic_table::IDOL_START_BUTTONS_CHALLENGES_BUTTON),
        idol_start_close_button_logic: table_logic(world, logic_table::IDOL_START_BUTTONS_CLOSE_BUTTON),
        idol_start_skin_pack_button_logic: table_logic(world, logic_table::IDOL_START_BUTTONS_SKIN_PACK_BUTTON),
        idol_start_check_box_logic: table_logic(world, logic_table::IDOL_START_CHECK_BOX),
        idol_icon_check_box_logic: table_logic(world, logic_table::IDOL_ICON_CHECK_BOX),
        idol_battle_help_check_box_logic: table_logic(world, logic_table::IDOL_BATTLE_HELP_CHECK_BOX),
        idol_whats_new_back_button_logic: table_logic(world, logic_table::IDOL_WHATS_NEW_BUTTONS_BACK_BUTTON),
        idol_options_start_check_box_button_logic: table_logic(world, logic_table::START_CHECK_BOX_BUTTON),
        idol_options_icon_check_box_button_logic: table_logic(world, logic_table::ICON_CHECK_BOX_BUTTON),
        idol_options_reset_button_logic: table_logic(world, logic_table::IDOL_OPTIONS_BUTTONS_RESET_BUTTON),
        idol_options_unlock_button_logic: table_logic(world, logic_table::UNLOCK_BUTTON),
        idol_options_level_up_button_logic: table_logic(world, logic_table::LEVEL_UP_BUTTON),
        idol_options_back_button_logic: table_logic(world, logic_table::IDOL_OPTIONS_BUTTONS_BACK_BUTTON),
        idol_detail_panel_back_button_logic: table_logic(world, logic_table::IDOL_DETAIL_BACK_BUTTON),
        idol_detail_panel_challenge_button_logic: table_logic(world, logic_table::IDOL_DETAIL_CHALLENGE_BUTTON),
        idol_detail_header_logic: table_logic(world, logic_table::IDOL_DETAIL_HEADER),
        idol_detail_challenge_header_logic: table_logic(world, logic_table::IDOL_DETAIL_CHALLENGE_HEADER),
        idol_detail_challenge_hint_item_logic: table_logic(world, logic_table::IDOL_DETAIL_CHALLENGE_HINT_ITEM),
        idol_detail_level_up_header_logic: table_logic(world, logic_table::IDOL_DETAIL_LEVEL_UP_HEADER),
        idol_detail_header_summary_button_logic: table_logic(world, logic_table::IDOL_SUMMARY_BUTTON),
        idol_detail_header_hint_button_logic: table_logic(world, logic_table::HINT_BUTTON),
        idol_detail_hint_panel_close_button_logic: table_logic(world, logic_table::IDOL_DETAIL_HINT_PANEL_BUTTONS_CLOSE_BUTTON),
        idol_detail_hint_buttons_display_group: library::display_group(world, "IdolDetailHintButtons"),
        idol_detail_hint_panel_buttons_display_group: library::display_group(world, "IdolDetailHintPanelButtons"),
        idol_are_you_sure_panel_buttons_display_group: library::display_group(world, "IdolAreYouSurePanelButtons"),
        idol_are_you_sure_panel_close_button_logic: table_logic(world, logic_table::IDOL_ARE_YOU_SURE_PANEL_BUTTONS_CLOSE_BUTTON),
        idol_are_you_sure_panel_reset_button_logic: table_logic(world, logic_table::IDOL_ARE_YOU_SURE_PANEL_BUTTONS_RESET_BUTTON),
        idol_detail_header_hint_panel_logic: table_logic(world, logic_table::IDOL_DETAIL_HINT_PANEL),
        idol_detail_attack_type_logic: table_logic(world, logic_table::IDOL_DETAIL_ATTACK_TYPE),
        idol_detail_realm_type_logic: table_logic(world, logic_table::IDOL_DETAIL_REALM_TYPE),
        idol_are_you_sure_panel_logic: table_logic(world, logic_table::IDOL_ARE_YOU_SURE_PANEL),
        idol_summary_attack_table_logic: table_logic(world, logic_table::IDOL_SUMMARY_ATTACK_TABLE),
        idol_summary_realm_table_logic: table_logic(world, logic_table::IDOL_SUMMARY_REALM_TABLE),
        idol_battle_realm_effect_highlight_logic: table_logic(world, logic_table::IDOL_BATTLE_REALM_EFFECT_HIGHLIGHT),
        idol_battle_result_winner_type_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULT_WINNER_TYPE),
        idol_battle_result_loser_type_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULT_LOSER_TYPE),
        idol_battle_result_message_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULT_MESSAGE),
        idol_summary_back_button_logic: table_logic(world, logic_table::IDOL_SUMMARY_BUTTONS_BACK_BUTTON),
        idol_award_close_button_logic: table_logic(world, logic_table::IDOL_AWARD_BUTTONS_CLOSE_BUTTON),
        idol_award_god_island_button_logic: table_logic(world, logic_table::IDOL_AWARD_BUTTONS_GOD_ISLAND_BUTTON),
        idol_award_challenges_button_logic: table_logic(world, logic_table::IDOL_AWARD_BUTTONS_CHALLENGES_BUTTON),
        idol_selection_challenges_button_logic: table_logic(world, logic_table::IDOL_SELECTION_BUTTONS_CHALLENGES_BUTTON),
        idol_battle_selection_exit_button_logic: table_logic(world, logic_table::EXIT_BUTTON),
        idol_battle_selection_attack_button_logic: table_logic(world, logic_table::ATTACK_BUTTON),
        idol_battle_results_win_battle_select_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_WIN_BUTTONS_BATTLE_SELECT_BUTTON),
        idol_battle_results_win_retry_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_WIN_BUTTONS_RETRY_BUTTON),
        idol_battle_results_win_next_button_logic: table_logic(world, logic_table::NEXT_BUTTON),
        idol_battle_results_lose_battle_select_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_LOSE_BUTTONS_BATTLE_SELECT_BUTTON),
        idol_battle_results_lose_retry_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_LOSE_BUTTONS_RETRY_BUTTON),
        idol_battle_results_win_multiplayer_battle_select_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_WIN_MULTIPLAYER_BUTTONS_BATTLE_SELECT_BUTTON),
        idol_battle_results_win_multiplayer_new_match_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_WIN_MULTIPLAYER_BUTTONS_NEW_MATCH_BUTTON),
        idol_battle_results_lose_multiplayer_battle_select_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_LOSE_MULTIPLAYER_BUTTONS_BATTLE_SELECT_BUTTON),
        idol_battle_results_lose_multiplayer_new_match_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_LOSE_MULTIPLAYER_BUTTONS_NEW_MATCH_BUTTON),
        idol_battle_results_final_battle_select_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_FINAL_BUTTONS_BATTLE_SELECT_BUTTON),
        idol_battle_results_final_retry_button_logic: table_logic(world, logic_table::IDOL_BATTLE_RESULTS_FINAL_BUTTONS_RETRY_BUTTON),
        idol_battle_enemy_help_button_logic: table_logic(world, logic_table::IDOL_BATTLE_ENEMY_BUTTONS_HELP_BUTTON),
        idol_battle_enemy_multiplayer_button_logic: table_logic(world, logic_table::MULTIPLAYER_BUTTON),
        idol_battle_help_battle_button_logic: table_logic(world, logic_table::BATTLE_BUTTON),
        idol_battle_help_check_box_button_logic: table_logic(world, logic_table::CHECK_BOX_BUTTON),
        idol_list_back_button_logic: table_logic(world, logic_table::IDOL_LIST_BUTTONS_BACK_BUTTON),
        ..Default::default()
    };
    let idol_panel_logic = wired(wiring.idol_panel_logic);
    *world.obj_mut::<IdolScreenLogic>(this) = wiring;

    let idol_icon_list_display_group = library::display_group(world, "IdolIconListDisplayGroup");
    world.obj_mut::<IdolScreenLogic>(this).idol_icon_list_display_group = idol_icon_list_display_group;
    let idol_panel_animation = logic::animation(world, idol_panel_logic);
    animation::set_tweenable(world, idol_panel_animation, idol_icon_list_display_group, 2);

    let idol_icon_detail_display_group = library::display_group(world, "IdolIconDetailDisplayGroup");
    world.obj_mut::<IdolScreenLogic>(this).idol_icon_detail_display_group = idol_icon_detail_display_group;
    let idol_panel_animation = logic::animation(world, idol_panel_logic);
    animation::set_tweenable(world, idol_panel_animation, idol_icon_detail_display_group, 3);

    let idol_detail_header_buttons = library::display_group(world, "IdolDetailHeaderButtons");
    if let Some(idol_detail_header_buttons) = idol_detail_header_buttons {
        display::move_to_display_group(world, idol_detail_header_buttons, idol_icon_detail_display_group);
    }

    let idol_selection_library_display_group = library::display_group(world, "IdolSelectionLibraryDisplayGroup");
    world.obj_mut::<IdolScreenLogic>(this).idol_selection_library_display_group = idol_selection_library_display_group;
    let idol_panel_animation = logic::animation(world, idol_panel_logic);
    animation::set_tweenable(world, idol_panel_animation, idol_selection_library_display_group, 9);

    // This channel is on the screen's own player, not the panel's.
    let idol_battle_realm_effect_display_group = library::display_group(world, "IdolBattleRealmEffectDisplayGroup");
    world.obj_mut::<IdolScreenLogic>(this).idol_battle_realm_effect_display_group = idol_battle_realm_effect_display_group;
    let own_animation = logic::animation(world, this);
    animation::set_tweenable(world, own_animation, idol_battle_realm_effect_display_group, 11);

    for slot in 0..IDOL_ICON_COUNT {
        let idol_icon = library::logic(world, logic_table::IDOL_ICON_BUTTON1 + slot);
        idol_icon_logic::set_skin_texture_bits_def_array(world, idol_icon, Some(EPISODE_ICON_SLOT_BITS[slot]));

        let halo = library::logic(world, logic_table::IDOL_ICON_HALO1 + slot);
        idol_icon_logic::set_halo_logic(world, idol_icon, halo);

        let check_mark = library::logic(world, logic_table::IDOL_ICON_CHECK_MARK1 + slot);
        idol_icon_logic::set_check_mark_logic(world, idol_icon, check_mark);

        let label_id = format!("IdolIconSlot{}Label", slot + 1);
        idol_icon_logic::set_label_id(world, idol_icon, &label_id);

        world.obj_mut::<IdolScreenLogic>(this).idol_icon_logic_array.push(idol_icon);
    }

    let idol_icon_logic_array = world.obj::<IdolScreenLogic>(this).idol_icon_logic_array.clone();
    for idol_icon in idol_icon_logic_array {
        let icon_display_object = logic::display_object(world, idol_icon);
        let list_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_list_display_group;
        display::move_to_display_group(world, icon_display_object, list_display_group);
    }

    for item in 0..CHALLENGE_ITEM_COUNT {
        let challenge_item = library::logic(world, logic_table::IDOL_DETAIL_CHALLENGE_ITEM1 + item);
        world.obj_mut::<IdolScreenLogic>(this).challenge_item_logic_array.push(challenge_item);
    }

    for item in 0..LEVEL_UP_ITEM_COUNT {
        let level_up_item = library::logic(world, logic_table::IDOL_DETAIL_LEVEL_UP_ITEM1 + item);
        world.obj_mut::<IdolScreenLogic>(this).level_up_item_logic_array.push(level_up_item);
    }

    for slot in 0..SELECTED_ICON_HOME_X_POS.len() {
        let number = slot + 1;
        let idol_selection_icon = library::logic(world, logic_table::IDOL_SELECTION_ICON1 + slot);
        idol_selection_icon_logic::set_skin_texture_bits_def_array(world, idol_selection_icon, Some(EPISODE_IDOL_SLOT_BITS[slot]));

        let label_id = format!("IdolSelectionIcon{number}LevelNum");
        idol_selection_icon_logic::set_label_id(world, idol_selection_icon, &label_id);

        let attack_type = library::logic(world, logic_table::IDOL_SELECTION_ICON1_ATTACK_TYPE + 3 * slot);
        idol_selection_icon_logic::set_attack_type_logic(world, idol_selection_icon, attack_type);

        let realm_type = library::logic(world, logic_table::IDOL_SELECTION_ICON1_REALM_TYPE + 3 * slot);
        idol_selection_icon_logic::set_realm_type_logic(world, idol_selection_icon, realm_type);

        let battle_selected = library::logic(world, logic_table::IDOL_SELECTION_ICON1_SELECTED + 3 * slot);
        idol_selection_icon_logic::set_battle_selected_logic(world, idol_selection_icon, battle_selected);

        idol_selection_icon_logic::set_idol_selection_icon_type(world, idol_selection_icon, 1);
        idol_selection_icon_logic::set_home_x_pos(world, idol_selection_icon, SELECTED_ICON_HOME_X_POS[slot], 252.0);
        idol_selection_icon_logic::set_battle_home_x_pos(world, idol_selection_icon, SELECTED_ICON_BATTLE_HOME_X_POS[slot], 115.0);
        set_selection_icon_sequence_ids(world, idol_selection_icon, number);

        world.obj_mut::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array.push(idol_selection_icon);
    }

    for library_slot in 0..4 {
        let number = library_slot + 5;
        let idol_selection_icon = library::logic(world, logic_table::LIBRARY_IDOL_SELECTION_ICON1 + library_slot);
        idol_selection_icon_logic::set_skin_texture_bits_def_array(world, idol_selection_icon, Some(EPISODE_IDOL_SLOT_BITS[4 + library_slot]));

        let label_id = format!("IdolSelectionIcon{number}LevelNum");
        idol_selection_icon_logic::set_label_id(world, idol_selection_icon, &label_id);

        let name_label_id = format!("IdolSelectionIcon{number}Name");
        let stats_label_id = format!("IdolSelectionIcon{number}Stats");
        idol_selection_icon_logic::set_battle_name_label_id(world, idol_selection_icon, &name_label_id, &stats_label_id);

        let attack_type = library::logic(world, logic_table::IDOL_SELECTION_ICON5_ATTACK_TYPE + 2 * library_slot);
        idol_selection_icon_logic::set_attack_type_logic(world, idol_selection_icon, attack_type);

        let realm_type = library::logic(world, logic_table::IDOL_SELECTION_ICON5_REALM_TYPE + 2 * library_slot);
        idol_selection_icon_logic::set_realm_type_logic(world, idol_selection_icon, realm_type);

        idol_selection_icon_logic::set_idol_selection_icon_type(world, idol_selection_icon, 0);
        set_selection_icon_sequence_ids(world, idol_selection_icon, number);
        idol_selection_icon_logic::set_is_enemy_idol(world, idol_selection_icon, true);

        world.obj_mut::<IdolScreenLogic>(this).idol_selection_icon_logic_array.push(idol_selection_icon);
    }

    for slot in 0..REALM_EFFECT_SLOT_COUNT {
        let number = slot + 1;
        let realm_effect = library::logic(world, logic_table::IDOL_BATTLE_REALM_EFFECT_SLOT1 + slot);
        let buff_sequence_id = format!("IdolBattleRealmEffectBuffSlot{number}");
        let debuff_sequence_id = format!("IdolBattleRealmEffectDebuffSlot{number}");
        idol_battle_realm_effect_logic::set_buff_sequence_id(world, realm_effect, &buff_sequence_id, &debuff_sequence_id);

        let realm_effect_display_object = logic::display_object(world, realm_effect);
        let realm_effect_display_group = world.obj::<IdolScreenLogic>(this).idol_battle_realm_effect_display_group;
        display::move_to_display_group(world, realm_effect_display_object, realm_effect_display_group);

        world.obj_mut::<IdolScreenLogic>(this).idol_battle_realm_effect_logic_array.push(realm_effect);
    }

    screen::set_screen_layer_logic(world, this, 0, Some(this));
    screen::set_screen_layer_logic(world, this, 1, Some(idol_panel_logic));

    let own_display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, own_display_object);
        object.is_selectable = true;
        object.col_x_min = 0.0;
        object.col_x_max = 480.0;
        object.col_y_min = 0.0;
        object.col_y_max = 320.0;
    }

    logic::start_game_frame(world, this);
}

/// `-[IdolScreenLogic dealloc]` @ 0x1001910b0
pub fn dealloc(world: &mut World, this: Entity) {
    // The arrays and the retained listener go with the component; `BCScreenLogic` and
    // `MultiModelLogic` release nothing else.
    logic::dealloc(world, this);
}

/// `-[IdolScreenLogic setIdolScreenLogicListener:]` @ 0x1001911e0
pub fn set_idol_screen_logic_listener(world: &mut World, this: Entity, idol_screen_logic_listener: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_logic_listener = Some(idol_screen_logic_listener);
}

/// `-[IdolScreenLogic idolScreenMode]` @ 0x10019123c
pub fn idol_screen_mode(world: &mut World, this: Entity) -> i32 {
    world.obj::<IdolScreenLogic>(this).idol_screen_mode
}

/// `-[IdolScreenLogic updateIdolIconLogics]` @ 0x10019124c
pub fn update_idol_icon_logics(world: &mut World, this: Entity) {
    let idol_icon_list_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_list_display_group;
    let list_display_group = required_display_group(idol_icon_list_display_group);
    let list_y_pos = display::object(world, list_display_group).pos.y;
    let top_limit = list_y_pos + -320.0 + 22.0;
    let bottom_limit = list_y_pos + -22.0;

    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_node(world, episode);
        let Some(idol_icon) = node.idol_icon_logic else {
            continue;
        };

        // An icon whose position is NaN counts as below the list, as with the original's
        // `b.le`.
        let icon_bottom = -100.0 - node.idol_icon_pos.y;
        let icon_top = -node.idol_icon_pos.y;
        let reaches_list = icon_bottom <= bottom_limit;
        let above_list = top_limit > icon_top;
        if !reaches_list || above_list {
            world.obj_mut::<IdolScreenLogic>(this).available_idol_icon_logic_array.push(idol_icon);
            idol_icon_logic::deactivate(world, idol_icon);
            idol_manager::episode_table_mut(world).nodes[episode].idol_icon_logic = None;
        }
    }

    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_node(world, episode);
        if node.idol_icon_logic.is_some() {
            continue;
        }

        let icon_bottom = -100.0 - node.idol_icon_pos.y;
        let icon_top = -node.idol_icon_pos.y;
        let in_list = top_limit <= icon_top && icon_bottom <= bottom_limit;
        if !in_list {
            continue;
        }

        let available_idol_icon = world.obj::<IdolScreenLogic>(this).available_idol_icon_logic_array.first().copied();
        let Some(idol_icon) = available_idol_icon else {
            continue;
        };

        idol_manager::episode_table_mut(world).nodes[episode].idol_icon_logic = Some(idol_icon);
        let active_node = world.obj::<IdolScreenLogic>(this).active_struct_xml_episode_node_def;
        let selected = active_node == Some(episode);
        idol_icon_logic::activate(world, idol_icon, Some(episode), selected);
        world.obj_mut::<IdolScreenLogic>(this).available_idol_icon_logic_array.remove(0);
    }
}

/// `-[IdolScreenLogic updateIdolSelectionIconLogics]` @ 0x10019140c
pub fn update_idol_selection_icon_logics(world: &mut World, this: Entity) {
    let scroll_x_pos = world.obj::<IdolScreenLogic>(this).idol_selection_icon_scroll_x_pos;
    let left_limit = 22.0 - scroll_x_pos;
    let right_limit = 458.0 - scroll_x_pos;

    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_node(world, episode);
        if !node.complete {
            continue;
        }
        let Some(idol_selection_icon) = node.idol_selection_icon_logic else {
            continue;
        };

        let icon_x_pos = node.idol_selection_icon_pos.x;
        let icon_right = icon_x_pos + 88.0;
        let out_of_view = icon_x_pos > right_limit || icon_right < left_limit;
        if out_of_view {
            world.obj_mut::<IdolScreenLogic>(this).available_idol_selection_icon_logic_array.push(idol_selection_icon);
            idol_selection_icon_logic::deactivate(world, idol_selection_icon);
            idol_manager::episode_table_mut(world).nodes[episode].idol_selection_icon_logic = None;
        }
    }

    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_node(world, episode);
        if node.complete && node.idol_selection_icon_logic.is_none() {
            let icon_x_pos = node.idol_selection_icon_pos.x;
            let icon_right = icon_x_pos + 88.0;
            let in_view = icon_right >= left_limit && icon_x_pos <= right_limit;
            let available_icon = world.obj::<IdolScreenLogic>(this).available_idol_selection_icon_logic_array.first().copied();
            if in_view && let Some(idol_selection_icon) = available_icon {
                idol_manager::episode_table_mut(world).nodes[episode].idol_selection_icon_logic = Some(idol_selection_icon);
                idol_selection_icon_logic::activate(world, idol_selection_icon, Some(episode));
                world.obj_mut::<IdolScreenLogic>(this).available_idol_selection_icon_logic_array.remove(0);
            }
        }

        let idol_selection_icon = episode_node(world, episode).idol_selection_icon_logic;
        if let Some(idol_selection_icon) = idol_selection_icon {
            let scroll_x_pos = world.obj::<IdolScreenLogic>(this).idol_selection_icon_scroll_x_pos;
            idol_selection_icon_logic::set_scroll_x_pos(world, idol_selection_icon, scroll_x_pos);
        }
    }
}

/// `-[IdolScreenLogic updateIdolBattleEnemySelectionIconLogics]` @ 0x1001915c4
pub fn update_idol_battle_enemy_selection_icon_logics(world: &mut World, this: Entity) {
    let scroll_x_pos = world.obj::<IdolScreenLogic>(this).idol_selection_icon_scroll_x_pos;
    let left_limit = 22.0 - scroll_x_pos;
    let right_limit = 458.0 - scroll_x_pos;

    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_node(world, episode);
        let Some(idol_selection_icon) = node.idol_selection_icon_logic else {
            continue;
        };

        let icon_x_pos = node.idol_selection_icon_pos.x;
        let icon_right = icon_x_pos + 88.0;
        let out_of_view = icon_x_pos > right_limit || icon_right < left_limit;
        if out_of_view {
            world.obj_mut::<IdolScreenLogic>(this).available_idol_selection_icon_logic_array.push(idol_selection_icon);
            idol_selection_icon_logic::deactivate(world, idol_selection_icon);
            idol_manager::episode_table_mut(world).nodes[episode].idol_selection_icon_logic = None;
        }
    }

    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_node(world, episode);
        if node.idol_selection_icon_logic.is_none() {
            let icon_x_pos = node.idol_selection_icon_pos.x;
            let icon_right = icon_x_pos + 88.0;
            let in_view = icon_right >= left_limit && icon_x_pos <= right_limit;
            let available_icon = world.obj::<IdolScreenLogic>(this).available_idol_selection_icon_logic_array.first().copied();
            if in_view && let Some(idol_selection_icon) = available_icon {
                idol_manager::episode_table_mut(world).nodes[episode].idol_selection_icon_logic = Some(idol_selection_icon);

                let listener = screen_listener(world, this);
                let enemy_level = idol_manager::calculate_enemy_level(world, listener, Some(episode));
                let enemy_health = idol_manager::calculate_enemy_health(world, listener, Some(episode));
                idol_selection_icon_logic::set_enemy_stats_level(world, idol_selection_icon, enemy_level, enemy_health);
                idol_selection_icon_logic::activate_for_battle_enemy(world, idol_selection_icon, Some(episode));
                world.obj_mut::<IdolScreenLogic>(this).available_idol_selection_icon_logic_array.remove(0);
            }
        }

        let idol_selection_icon = episode_node(world, episode).idol_selection_icon_logic;
        if let Some(idol_selection_icon) = idol_selection_icon {
            let scroll_x_pos = world.obj::<IdolScreenLogic>(this).idol_selection_icon_scroll_x_pos;
            idol_selection_icon_logic::set_scroll_x_pos(world, idol_selection_icon, scroll_x_pos);
        }
    }
}

/// `-[IdolScreenLogic removeDetailItems]` @ 0x1001917ec
pub fn remove_detail_items(world: &mut World, this: Entity) {
    let challenge_header = world.obj::<IdolScreenLogic>(this).idol_detail_challenge_header_logic;
    remove_logic_display_object(world, challenge_header);

    let challenge_hint_item = world.obj::<IdolScreenLogic>(this).idol_detail_challenge_hint_item_logic;
    remove_logic_display_object(world, challenge_hint_item);

    let level_up_header = world.obj::<IdolScreenLogic>(this).idol_detail_level_up_header_logic;
    remove_logic_display_object(world, level_up_header);

    let hint_buttons = world.obj::<IdolScreenLogic>(this).idol_detail_hint_buttons_display_group;
    remove_display_group(world, hint_buttons);

    let hint_panel_buttons = world.obj::<IdolScreenLogic>(this).idol_detail_hint_panel_buttons_display_group;
    remove_display_group(world, hint_panel_buttons);

    let are_you_sure_panel_buttons = world.obj::<IdolScreenLogic>(this).idol_are_you_sure_panel_buttons_display_group;
    remove_display_group(world, are_you_sure_panel_buttons);

    let hint_panel = world.obj::<IdolScreenLogic>(this).idol_detail_header_hint_panel_logic;
    set_logic_behavior(world, hint_panel, "IdolDetailHintHide");

    let challenge_items = world.obj::<IdolScreenLogic>(this).challenge_item_logic_array.clone();
    for challenge_item in challenge_items {
        remove_logic_display_object(world, Some(challenge_item));
    }

    let level_up_items = world.obj::<IdolScreenLogic>(this).level_up_item_logic_array.clone();
    for level_up_item in level_up_items {
        remove_logic_display_object(world, Some(level_up_item));
    }
}

/// `-[IdolScreenLogic updateRealmTypeLogic:idolRealmType:]` @ 0x100191a70
pub fn update_realm_type_logic(world: &mut World, _this: Entity, update_realm_type_logic: Entity, idol_realm_type: i32) {
    let behavior = match idol_realm_type {
        0 => "IdolRealmTypeOceanus",
        1 => "IdolRealmTypeAeris",
        2 => "IdolRealmTypeInfernis",
        3 => "IdolRealmTypeAnimus",
        4 => "IdolRealmTypePax",
        _ => return,
    };
    logic::set_behavior(world, update_realm_type_logic, Some(behavior));
}

/// `-[IdolScreenLogic updateAttackTypeLogic:idolAttackType:]` @ 0x100191b28
pub fn update_attack_type_logic(world: &mut World, _this: Entity, update_attack_type_logic: Entity, idol_attack_type: i32) {
    let behavior = match idol_attack_type {
        2 => "IdolAttackTypeFast",
        1 => "IdolAttackTypeCounter",
        0 => "IdolAttackTypeStrong",
        _ => return,
    };
    logic::set_behavior(world, update_attack_type_logic, Some(behavior));
}

/// `-[IdolScreenLogic openIdolDetailPanel:]` @ 0x100191b90
pub fn open_idol_detail_panel(world: &mut World, this: Entity, open_idol_detail_panel: Option<usize>) {
    let node = required_node(open_idol_detail_panel);
    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.idol_screen_mode = 2;
        screen_logic.struct_xml_episode_node_def = open_idol_detail_panel;
    }

    let node_def = episode_def(world, node);
    activate_skin_texture_bits_def_array(world, EPISODE_ICON_SLOT_BITS[0], &node_def.icon_texture);
    let detail = detail_node(world, this);
    let detail_def = episode_def(world, detail);
    activate_skin_texture_bits_def_array(world, EPISODE_IDOL_SLOT_BITS[0], &detail_def.idol_texture);

    let icon_detail_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_detail_display_group;
    let detail_display_group = required_display_group(icon_detail_display_group);
    display::object_mut(world, detail_display_group).pos.y = 320.0;

    remove_detail_items(world, this);

    let sandbox = library::display_group(world, "DisplayObjectSandbox");

    let detail = detail_node(world, this);
    let detail_complete = episode_node(world, detail).complete;
    let header_behavior = if detail_complete { "IdolDetailHeaderUnlocked" } else { "IdolDetailHeaderLocked" };
    let detail_header = world.obj::<IdolScreenLogic>(this).idol_detail_header_logic;
    set_logic_behavior(world, detail_header, header_behavior);

    let episode_number = format!("Episode {}", node_def.episode_number);
    set_dynamic_text(world, "IdolDetailEpisodeNumber", Some(&episode_number));
    set_dynamic_text(world, "IdolDetailEpisodeName", Some(&node_def.episode_title));
    let node_level = episode_node(world, node).level;
    let idol_level = format!("L{:02}", node_level.wrapping_add(1));
    set_dynamic_text(world, "IdolDetailIdolLevel", Some(&idol_level));
    set_dynamic_text(world, "IdolDetailIdolName", Some(&node_def.idol_name));

    let attack_type_logic = wired(world.obj::<IdolScreenLogic>(this).idol_detail_attack_type_logic);
    let detail = detail_node(world, this);
    let detail_attack = episode_def(world, detail).attack as i32;
    update_attack_type_logic(world, this, attack_type_logic, detail_attack);

    let realm_type_logic = wired(world.obj::<IdolScreenLogic>(this).idol_detail_realm_type_logic);
    let detail = detail_node(world, this);
    let detail_realm = episode_def(world, detail).realm as i32;
    update_realm_type_logic(world, this, realm_type_logic, detail_realm);

    let mut row_y_pos: f32;
    let detail = detail_node(world, this);
    let detail_state = episode_node(world, detail);
    if !detail_state.complete {
        row_y_pos = -127.0;
    } else {
        let level_up_header = wired(world.obj::<IdolScreenLogic>(this).idol_detail_level_up_header_logic);
        let level_up_header_display_object = logic::display_object(world, level_up_header);
        set_display_object_xy(world, level_up_header_display_object, 34.0, -127.0);
        let icon_detail_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_detail_display_group;
        display::move_to_display_group(world, level_up_header_display_object, icon_detail_display_group);

        row_y_pos = -167.0;
        let detail = detail_node(world, this);
        let level_ups = episode_node(world, detail).level_ups;
        for (index, level_up) in level_ups.enumerate() {
            let level_up_item = world.obj::<IdolScreenLogic>(this).level_up_item_logic_array[index];
            let level_up_item_display_object = logic::display_object(world, level_up_item);
            set_display_object_xy(world, level_up_item_display_object, 34.0, row_y_pos);
            let icon_detail_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_detail_display_group;
            display::move_to_display_group(world, level_up_item_display_object, icon_detail_display_group);

            let level_up_state = idol_manager::episode_table(world).level_ups[level_up].clone();
            let item_number = index + 1;

            let name_id = format!("IdolDetailLevelUpItemName{item_number}");
            let name_text = display_group_object(world, sandbox, &name_id);
            set_text(world, name_text, Some(&level_up_state.def().description));

            let level_id = format!("IdolDetailLevelUpItemLevel{item_number}");
            let level_text = display_group_object(world, sandbox, &level_id);
            let level = format!("Level {}", level_up_state.level.wrapping_add(1));
            set_text(world, level_text, Some(&level));

            let level_up_bar = library::logic(world, logic_table::IDOL_DETAIL_LEVEL_UP_BAR1 + index);
            let level_points = level_up_state.points.wrapping_sub(level_up_state.level_start_points) as f32;
            let level_span = level_up_state.next_level_points.wrapping_sub(level_up_state.level_start_points) as f32;
            let level_fraction = level_points / level_span;
            let bar_percent = if level_points != 0.0 && level_fraction < 0.01 { 0.01 } else { level_fraction };
            let level_up_bar_animation = logic::animation(world, level_up_bar);
            animation::set_frame_percent(world, level_up_bar_animation, bar_percent);

            row_y_pos += -40.0;
        }
    }

    let challenge_header = wired(world.obj::<IdolScreenLogic>(this).idol_detail_challenge_header_logic);
    let challenge_header_display_object = logic::display_object(world, challenge_header);
    set_display_object_xy(world, challenge_header_display_object, 34.0, row_y_pos);
    let icon_detail_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_detail_display_group;
    display::move_to_display_group(world, challenge_header_display_object, icon_detail_display_group);
    row_y_pos += -40.0;

    let detail = detail_node(world, this);
    let challenges = episode_node(world, detail).challenges;
    let mut shown_challenge_count: i32 = 0;
    for challenge in challenges {
        let challenge_state = idol_manager::episode_table(world).challenges[challenge].clone();
        if !challenge_state.completed {
            continue;
        }

        let challenge_item = world.obj::<IdolScreenLogic>(this).challenge_item_logic_array[shown_challenge_count as usize];
        let challenge_item_display_object = logic::display_object(world, challenge_item);
        set_display_object_xy(world, challenge_item_display_object, 34.0, row_y_pos);
        let icon_detail_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_detail_display_group;
        display::move_to_display_group(world, challenge_item_display_object, icon_detail_display_group);

        shown_challenge_count += 1;
        let name_id = format!("IdolDetailChallengeItemName{shown_challenge_count}");
        let name_text = display_group_object(world, sandbox, &name_id);
        set_text(world, name_text, Some(&challenge_state.def().description));

        row_y_pos += -40.0;
    }

    let detail = detail_node(world, this);
    let detail_complete = episode_node(world, detail).complete;
    if !detail_complete {
        let challenge_hint_item = wired(world.obj::<IdolScreenLogic>(this).idol_detail_challenge_hint_item_logic);
        let hint_item_display_object = logic::display_object(world, challenge_hint_item);
        set_display_object_xy(world, hint_item_display_object, 34.0, row_y_pos);
        let icon_detail_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_detail_display_group;
        display::move_to_display_group(world, hint_item_display_object, icon_detail_display_group);

        let node_state = episode_node(world, node);
        let unlocked = format!("{} of {} Unlocked", node_state.completed_challenge_count, node_state.challenge_count);
        set_dynamic_text(world, "TaskHintDescription", Some(&unlocked));

        let hint_buttons = world.obj::<IdolScreenLogic>(this).idol_detail_hint_buttons_display_group;
        let icon_detail_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_detail_display_group;
        if let Some(hint_buttons) = hint_buttons {
            display::move_to_display_group(world, hint_buttons, icon_detail_display_group);
        }
        let hint_buttons = required_display_group(hint_buttons);
        let hint_item_pos = display::object(world, hint_item_display_object).pos;
        display::object_mut(world, hint_buttons).pos = hint_item_pos;

        row_y_pos += -40.0;
    }

    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.scroll_detail_display_group_min_y_pos = 320.0;
        screen_logic.scroll_detail_display_group_max_y_pos = 160.0 - row_y_pos;
    }
    screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolDetailPanel"));
}

/// `-[IdolScreenLogic openIdolStartPanel]` @ 0x100192408
pub fn open_idol_start_panel(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 0;

    let hints_used = dynamic_text(world, "HintsUsed");
    let listener = screen_listener(world, this);
    let activated_hint_count = idol_manager::activated_hint_count(world, listener);
    let hint_count = format!("{activated_hint_count}");
    set_text(world, hints_used, Some(&hint_count));

    let option_preferences = idol_manager::option_preferences(world, listener);

    let show_idol_start_screen = int_preference(world, option_preferences, "ShowIdolStartScreen");
    let start_check_box_behavior = if show_idol_start_screen == 0 { "HideIdolStartCheckBox" } else { "ShowIdolStartCheckBox" };
    let start_check_box = world.obj::<IdolScreenLogic>(this).idol_start_check_box_logic;
    set_logic_behavior(world, start_check_box, start_check_box_behavior);

    let show_episode_icon = int_preference(world, option_preferences, "ShowEpisodeIcon");
    let icon_check_box_behavior = if show_episode_icon == 0 { "HideIdolStartCheckBox" } else { "ShowIdolStartCheckBox" };
    let icon_check_box = world.obj::<IdolScreenLogic>(this).idol_icon_check_box_logic;
    set_logic_behavior(world, icon_check_box, icon_check_box_behavior);

    let show_battle_help = int_preference(world, option_preferences, "ShowBattleHelp");
    let battle_help_check_box_behavior = if show_battle_help == 0 { "HideIdolStartCheckBox" } else { "ShowIdolStartCheckBox" };
    let battle_help_check_box = world.obj::<IdolScreenLogic>(this).idol_battle_help_check_box_logic;
    set_logic_behavior(world, battle_help_check_box, battle_help_check_box_behavior);
}

/// `-[IdolScreenLogic openIdolListPanel:]` @ 0x1001925c4
pub fn open_idol_list_panel(world: &mut World, this: Entity, open_idol_list_panel: Option<usize>) {
    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.idol_screen_mode = 1;
        screen_logic.active_struct_xml_episode_node_def = open_idol_list_panel;
        screen_logic.struct_xml_episode_node_def = open_idol_list_panel;
    }
    screen::set_screen_layer_behavior(world, this, 0, Some("OpenIdolScreen"));
    load_skin_pack_texture_atlas_channel(world, 0);

    let idol_icon_list_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_list_display_group;
    let list_display_group = required_display_group(idol_icon_list_display_group);
    display::object_mut(world, list_display_group).pos.y = 320.0;

    let mut row: i32 = 0;
    let mut column: i32 = 0;
    for episode in 0..EPISODE_LIST_LEN {
        let icon_x_pos = column as f32 * 100.0 + 90.0;
        let icon_y_pos = row as f32 * -126.0 + -230.0;
        let idol_icon = {
            let mut table = idol_manager::episode_table_mut(world);
            let node = &mut table.nodes[episode];
            node.idol_icon_pos.x = icon_x_pos;
            node.idol_icon_pos.y = icon_y_pos;
            node.idol_icon_logic
        };

        if let Some(idol_icon) = idol_icon {
            idol_icon_logic::deactivate(world, idol_icon);
            idol_manager::episode_table_mut(world).nodes[episode].idol_icon_logic = None;
        }

        if column < 2 {
            column += 1;
        } else {
            row += 1;
            column = 0;
        }
    }
    let list_bottom = row as f32 * -126.0 + -280.0;

    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.scroll_list_display_group_min_y_pos = 320.0;
        screen_logic.scroll_list_display_group_max_y_pos = -list_bottom;

        let idol_icon_logic_array = screen_logic.idol_icon_logic_array.clone();
        screen_logic.available_idol_icon_logic_array.clear();
        screen_logic.available_idol_icon_logic_array.extend(idol_icon_logic_array);
    }

    update_idol_icon_logics(world, this);
}

/// `-[IdolScreenLogic openIdolSummaryPanel]` @ 0x1001927b0
pub fn open_idol_summary_panel(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).summary_open = true;

    let summary_node = detail_node(world, this);
    let summary_def = episode_def(world, summary_node);
    set_dynamic_text(world, "IdolSummaryName", Some(&summary_def.idol_name));

    let summary_description = dynamic_text(world, "IdolSummaryDescription");
    let listener = screen_listener(world, this);
    let summary_node = detail_node(world, this);
    let summary_def = episode_def(world, summary_node);
    let power_description = listener_fill_power_description_text(world, listener, &summary_def.power_description, Some(summary_node));
    set_text(world, summary_description, power_description.as_deref());
    if let Some(summary_description) = summary_description {
        text::set_wrap_width(world, summary_description, 250.0);
    }

    let summary_node = detail_node(world, this);
    let summary_level = episode_node(world, summary_node).level;
    let level = format!("{}", summary_level.wrapping_add(1));
    set_dynamic_text(world, "IdolSummaryLevel", Some(&level));

    let summary_node = detail_node(world, this);
    let summary_def = episode_def(world, summary_node);
    let summary_level = episode_node(world, summary_node).level;
    let health = summary_def.health_per_level * summary_level.wrapping_add(1) as f32;
    let health_text = format!("{:.0}", f64::from(health));
    set_dynamic_text(world, "IdolSummaryHealth", Some(&health_text));

    let summary_node = detail_node(world, this);
    let summary_def = episode_def(world, summary_node);
    let attack_table_behavior = match summary_def.attack as i32 {
        2 => Some("IdolAttackTableFast"),
        1 => Some("IdolAttackTableCounter"),
        0 => Some("IdolAttackTableStrong"),
        _ => None,
    };
    if let Some(attack_table_behavior) = attack_table_behavior {
        let attack_table = world.obj::<IdolScreenLogic>(this).idol_summary_attack_table_logic;
        set_logic_behavior(world, attack_table, attack_table_behavior);
    }

    let summary_node = detail_node(world, this);
    let summary_def = episode_def(world, summary_node);
    let realm_table_behavior = match summary_def.realm as i32 {
        0 => Some("IdolRealmTableOceanus"),
        1 => Some("IdolRealmTableAeris"),
        2 => Some("IdolRealmTableInfernis"),
        3 => Some("IdolRealmTableAnimus"),
        4 => Some("IdolRealmTablePax"),
        _ => None,
    };
    if let Some(realm_table_behavior) = realm_table_behavior {
        let realm_table = world.obj::<IdolScreenLogic>(this).idol_summary_realm_table_logic;
        set_logic_behavior(world, realm_table, realm_table_behavior);
    }

    screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolSummaryPanel"));
}

/// `-[IdolScreenLogic openIdolSelectionPanel]` @ 0x100192b28
pub fn open_idol_selection_panel(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 4;
    load_skin_pack_texture_atlas_channel(world, 1);

    let mut earned_idol_count: i32 = 0;
    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_node(world, episode);
        if node.complete {
            let home_x_pos = earned_idol_count as f32 * 130.0 + 39.0;
            {
                let mut table = idol_manager::episode_table_mut(world);
                let node = &mut table.nodes[episode];
                node.idol_selection_icon_pos.x = home_x_pos;
                node.idol_selection_icon_pos.y = 112.0;
            }
            if let Some(idol_selection_icon) = node.idol_selection_icon_logic {
                idol_selection_icon_logic::deactivate(world, idol_selection_icon);
            }
            earned_idol_count += 1;
        }
        idol_manager::episode_table_mut(world).nodes[episode].idol_selection_icon_logic = None;
    }

    let screen_behavior = if earned_idol_count != 0 { "OpenIdolSelectionScreen" } else { "OpenIdolSelectionEmptyScreen" };
    screen::set_screen_layer_behavior(world, this, 0, Some(screen_behavior));

    let player_level_text = dynamic_text(world, "PlayerLevel");
    let player_health_text = dynamic_text(world, "PlayerHealth");
    let listener = screen_listener(world, this);
    let player_level = idol_manager::calculate_player_level(world, listener);
    let level = format!("Level: {}", player_level.wrapping_add(1));
    set_text(world, player_level_text, Some(&level));
    let player_health = idol_manager::calculate_player_health(world, listener);
    let health = format!("Health: {:.0}", f64::from(player_health));
    set_text(world, player_health_text, Some(&health));

    {
        let library_right = earned_idol_count as f32 * 130.0 + 89.0 + -480.0;
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.scroll_selection_display_group_min_x_pos = -library_right;
        screen_logic.scroll_selection_display_group_max_x_pos = 0.0;
        screen_logic.idol_selection_icon_scroll_x_pos = 0.0;

        let idol_selection_icon_logic_array = screen_logic.idol_selection_icon_logic_array.clone();
        screen_logic.available_idol_selection_icon_logic_array.clear();
        screen_logic.available_idol_selection_icon_logic_array.extend(idol_selection_icon_logic_array);
    }

    for slot in 0..4 {
        let selected_idol = idol_manager::selected_episode_idol_in_slot(world, listener, slot);
        let idol_selection_icon = world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array[slot as usize];
        match selected_idol {
            Some(selected_idol) => idol_selection_icon_logic::activate(world, idol_selection_icon, Some(selected_idol)),
            None => idol_selection_icon_logic::deactivate(world, idol_selection_icon),
        }
    }

    update_idol_selection_icon_logics(world, this);
}

/// `-[IdolScreenLogic openIdolBattleEnemyPanel]` @ 0x100192e9c
pub fn open_idol_battle_enemy_panel(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 5;
    load_skin_pack_texture_atlas_channel(world, 1);

    let hint_panel = world.obj::<IdolScreenLogic>(this).idol_detail_header_hint_panel_logic;
    set_logic_behavior(world, hint_panel, "IdolDetailHintHide");
    let hint_panel_buttons = world.obj::<IdolScreenLogic>(this).idol_detail_hint_panel_buttons_display_group;
    remove_display_group(world, hint_panel_buttons);

    let mut last_battle_unlocked: Option<usize> = None;
    let mut icon_count: i32 = 0;
    for episode in 0..EPISODE_LIST_LEN {
        let home_x_pos = icon_count as f32 * 130.0 + 39.0;
        let node = {
            let mut table = idol_manager::episode_table_mut(world);
            let node = &mut table.nodes[episode];
            node.idol_selection_icon_pos.x = home_x_pos;
            node.idol_selection_icon_pos.y = 250.0;
            node.clone()
        };

        if let Some(idol_selection_icon) = node.idol_selection_icon_logic {
            idol_selection_icon_logic::deactivate(world, idol_selection_icon);
            idol_manager::episode_table_mut(world).nodes[episode].idol_selection_icon_logic = None;
        }
        if node.battle_unlocked {
            last_battle_unlocked = Some(episode);
        }
        icon_count += 1;
    }
    let library_right = icon_count as f32 * 130.0 + 89.0 + -480.0;

    screen::set_screen_layer_behavior(world, this, 0, Some("OpenIdolBattleEnemyScreen"));
    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.scroll_selection_display_group_min_x_pos = -library_right;
        screen_logic.scroll_selection_display_group_max_x_pos = 0.0;
    }

    if let Some(last_battle_unlocked) = last_battle_unlocked {
        let unlocked_x_pos = episode_node(world, last_battle_unlocked).idol_selection_icon_pos.x;
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.idol_selection_icon_scroll_x_pos = -(unlocked_x_pos + -240.0);
        if screen_logic.scroll_selection_display_group_min_x_pos > screen_logic.idol_selection_icon_scroll_x_pos {
            screen_logic.idol_selection_icon_scroll_x_pos = screen_logic.scroll_selection_display_group_min_x_pos;
        }
        if screen_logic.idol_selection_icon_scroll_x_pos > screen_logic.scroll_selection_display_group_max_x_pos {
            screen_logic.idol_selection_icon_scroll_x_pos = screen_logic.scroll_selection_display_group_max_x_pos;
        }
    }

    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        let idol_selection_icon_logic_array = screen_logic.idol_selection_icon_logic_array.clone();
        screen_logic.available_idol_selection_icon_logic_array.clear();
        screen_logic.available_idol_selection_icon_logic_array.extend(idol_selection_icon_logic_array);
    }

    let player_level_text = dynamic_text(world, "PlayerLevel");
    let player_health_text = dynamic_text(world, "PlayerHealth");
    let listener = screen_listener(world, this);
    let player_level = idol_manager::calculate_player_level(world, listener);
    let level = format!("Level: {}", player_level.wrapping_add(1));
    set_text(world, player_level_text, Some(&level));
    let player_health = idol_manager::calculate_player_health(world, listener);
    let health = format!("Health: {:.0}", f64::from(player_health));
    set_text(world, player_health_text, Some(&health));

    for slot in 0..4 {
        let selected_idol = idol_manager::selected_episode_idol_in_slot(world, listener, slot);
        let idol_selection_icon = world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array[slot as usize];
        match selected_idol {
            Some(selected_idol) => idol_selection_icon_logic::activate_for_battle_enemy(world, idol_selection_icon, Some(selected_idol)),
            None => idol_selection_icon_logic::deactivate(world, idol_selection_icon),
        }
    }

    update_idol_battle_enemy_selection_icon_logics(world, this);
}

/// `-[IdolScreenLogic openIdolBattleResultsPanel]` @ 0x100193268
pub fn open_idol_battle_results_panel(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 8;
    load_skin_pack_texture_atlas_channel(world, 1);

    let listener = screen_listener(world, this);
    let enemy_episode_idol = idol_manager::enemy_episode_idol(world, listener);
    let player_won = idol_manager::did_player_win_last_battle(world, listener);
    let player_tied = idol_manager::did_player_tie_last_battle(world, listener);
    let last_battle_idol = idol_manager::is_last_battle_idol(world, listener, enemy_episode_idol);

    let results_exp_texts = [
        dynamic_text(world, "IdolResultsExp1"),
        dynamic_text(world, "IdolResultsExp2"),
        dynamic_text(world, "IdolResultsExp3"),
        dynamic_text(world, "IdolResultsExp4"),
    ];
    for (slot, results_exp_text) in results_exp_texts.into_iter().enumerate() {
        let idol_battle_state = battle_state(world, listener, BattleSide::Player).idol_battle_state_array[slot];
        let experience = f64::from(idol_battle_state.experience);
        let results = format!("Used: x{}\nLevel up: +{:.0}", idol_battle_state.used_count, experience);
        set_text(world, results_exp_text, Some(&results));
    }

    let multiplayer_active = idol_manager::is_multiplayer_active(world, listener);
    let screen_behavior = if !multiplayer_active {
        if player_tied {
            "OpenIdolBattleResultsTieScreen"
        } else if !player_won {
            "OpenIdolBattleResultsLoseScreen"
        } else if !last_battle_idol {
            "OpenIdolBattleResultsWinScreen"
        } else {
            "OpenIdolBattleResultsFinalScreen"
        }
    } else if player_tied {
        "OpenIdolBattleResultsTieMultiplayerScreen"
    } else if !player_won {
        "OpenIdolBattleResultsLoseMultiplayerScreen"
    } else {
        "OpenIdolBattleResultsWinMultiplayerScreen"
    };
    screen::set_screen_layer_behavior(world, this, 0, Some(screen_behavior));
}

/// `-[IdolScreenLogic closeIdolBattleSelectionPanel]` @ 0x1001935a8
pub fn close_idol_battle_selection_panel(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 6;
    screen::set_screen_layer_behavior(world, this, 1, Some("CloseIdolBattleSelectionPanel"));
}

/// `-[IdolScreenLogic refreshIdolBattleSelectionPanel]` @ 0x1001935d4
pub fn refresh_idol_battle_selection_panel(world: &mut World, this: Entity) {
    let listener = screen_listener(world, this);

    let mut battle_idol_highlighted = false;
    for slot in 0..4 {
        let idol_battle_state = battle_state(world, listener, BattleSide::Player).idol_battle_state_array[slot];
        let idol_selection_icon = world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array[slot];
        let Some(node) = idol_battle_state.struct_xml_episode_node_def else {
            continue;
        };

        let frozen = 0 < idol_battle_state.frozen_ref_count;
        idol_selection_icon_logic::activate_for_battle(world, idol_selection_icon, Some(node), frozen);
        let battle_node = world.obj::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def;
        if Some(node) == battle_node {
            idol_selection_icon_logic::highlight_for_battle(world, idol_selection_icon);
            battle_idol_highlighted = true;
        }
    }

    for slot in 0..4 {
        let idol_battle_state = battle_state(world, listener, BattleSide::Enemy).idol_battle_state_array[slot];
        let idol_selection_icon = world.obj::<IdolScreenLogic>(this).idol_selection_icon_logic_array[slot];
        let Some(node) = idol_battle_state.struct_xml_episode_node_def else {
            continue;
        };

        let frozen = 0 < idol_battle_state.frozen_ref_count;
        idol_selection_icon_logic::activate_for_battle(world, idol_selection_icon, Some(node), frozen);
        let battle_node = world.obj::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def;
        if Some(node) == battle_node {
            idol_selection_icon_logic::highlight_for_battle(world, idol_selection_icon);
        }
    }

    let attack_button = wired(world.obj::<IdolScreenLogic>(this).idol_battle_selection_attack_button_logic);
    let attack_button_display_object = logic::display_object(world, attack_button);
    display::object_mut(world, attack_button_display_object).is_selectable = false;

    let attack_button_skin = library::logic(world, logic_table::IDOL_BATTLE_SELECTION_ATTACK_BUTTON_SKIN);
    let attack_button_skin_display_object = logic::display_object(world, attack_button_skin);
    let panel_text = dynamic_text(world, "PlayerBattlePanelText");

    let waiting_for_opponent = idol_manager::is_multiplayer_active(world, listener) && !idol_manager::is_it_my_turn(world, listener);
    if waiting_for_opponent {
        let player_battle_node = battle_state(world, listener, BattleSide::Player).battle_struct_xml_episode_node_def;
        let enemy_battle_node = battle_state(world, listener, BattleSide::Enemy).battle_struct_xml_episode_node_def;
        if player_battle_node.is_some() && enemy_battle_node.is_none() {
            let selected_icons = world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array.clone();
            for idol_selection_icon in selected_icons {
                let icon_node = idol_selection_icon_logic::struct_xml_episode_node_def(world, idol_selection_icon);
                let player_battle_node = battle_state(world, listener, BattleSide::Player).battle_struct_xml_episode_node_def;
                if icon_node == player_battle_node {
                    idol_selection_icon_logic::select_for_battle(world, idol_selection_icon);
                }
            }
        }

        set_text(world, panel_text, Some(""));
        logic::set_behavior(world, attack_button_skin, Some("IdolBattleSelectionButtonWaiting"));
        set_display_object_shown(world, attack_button_skin_display_object, true);
        return;
    }

    set_text(world, panel_text, Some("Tap Idol to select, then Go!"));
    logic::set_behavior(world, attack_button_skin, Some("IdolBattleSelectionButtonAttack"));
    if battle_idol_highlighted {
        display::object_mut(world, attack_button_display_object).is_selectable = true;
    }
    set_display_object_shown(world, attack_button_skin_display_object, battle_idol_highlighted);
}

/// `-[IdolScreenLogic openIdolBattleSelectionPanel]` @ 0x100193a78
pub fn open_idol_battle_selection_panel(world: &mut World, this: Entity) {
    let idol_screen_mode = world.obj::<IdolScreenLogic>(this).idol_screen_mode;
    if idol_screen_mode != 7 {
        world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 7;
        screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolBattleSelectionPanel"));
        world.obj_mut::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def = None;
    }

    refresh_idol_battle_selection_panel(world, this);

    let listener = screen_listener(world, this);
    idol_manager::on_idol_battle_selection_panel_opened(world, listener);
}

/// `-[IdolScreenLogic openIdolStartScreen]` @ 0x100193b08
pub fn open_idol_start_screen(world: &mut World, this: Entity) {
    screen::open_screen(world, this);
    world.obj_mut::<IdolScreenLogic>(this).root_idol_screen_mode = 0;
    logic::set_binary_frame_def(world, this, None, None);
    open_idol_start_panel(world, this);
    screen::set_screen_layer_behavior(world, this, 0, Some("OpenIdolStartScreen"));
}

/// `-[IdolScreenLogic openIdolListScreen:]` @ 0x100193bac
pub fn open_idol_list_screen(world: &mut World, this: Entity, open_idol_list_screen: Option<usize>) {
    screen::open_screen(world, this);
    world.obj_mut::<IdolScreenLogic>(this).root_idol_screen_mode = 0;
    logic::set_binary_frame_def(world, this, None, None);
    open_idol_list_panel(world, this, open_idol_list_screen);
}

/// `-[IdolScreenLogic openIdolDetailScreen:]` @ 0x100193c38
pub fn open_idol_detail_screen(world: &mut World, this: Entity, open_idol_detail_screen: Option<usize>) {
    screen::open_screen(world, this);
    world.obj_mut::<IdolScreenLogic>(this).root_idol_screen_mode = 0;
    logic::set_binary_frame_def(world, this, None, None);
    open_idol_list_panel(world, this, open_idol_detail_screen);
    open_idol_detail_panel(world, this, open_idol_detail_screen);

    let idol_icon_list_display_group = world.obj::<IdolScreenLogic>(this).idol_icon_list_display_group;
    let list_display_group = required_display_group(idol_icon_list_display_group);
    let node = required_node(open_idol_detail_screen);
    let icon_y_pos = episode_node(world, node).idol_icon_pos.y;
    let mut list_y_pos = -(icon_y_pos + -110.0);
    display::object_mut(world, list_display_group).pos.y = list_y_pos;

    let min_y_pos = world.obj::<IdolScreenLogic>(this).scroll_list_display_group_min_y_pos;
    if min_y_pos > list_y_pos {
        display::object_mut(world, list_display_group).pos.y = min_y_pos;
        list_y_pos = min_y_pos;
    }
    let max_y_pos = world.obj::<IdolScreenLogic>(this).scroll_list_display_group_max_y_pos;
    if list_y_pos > max_y_pos {
        display::object_mut(world, list_display_group).pos.y = max_y_pos;
    }

    screen::set_screen_layer_behavior(world, this, 0, Some("OpenIdolDetailScreen"));
}

/// `-[IdolScreenLogic openIdolSelectionScreen]` @ 0x100193d64
pub fn open_idol_selection_screen(world: &mut World, this: Entity) {
    screen::open_screen(world, this);
    world.obj_mut::<IdolScreenLogic>(this).root_idol_screen_mode = 4;
    logic::set_binary_frame_def(world, this, None, None);
    open_idol_selection_panel(world, this);
}

/// `-[IdolScreenLogic manageIdolBattleRealmEffectLogics:buffIdolBattleRealmEffectLogicArray:debuffIdolBattleRealmEffectLogicArray:]` @ 0x100193dec
///
/// The first side picks the listener's `PlayerBattleState` (read in place), the other two pick
/// this screen's realm-effect logic arrays.
pub fn manage_idol_battle_realm_effect_logics(world: &mut World, this: Entity, manage_idol_battle_realm_effect_logics: BattleSide, buff_idol_battle_realm_effect_logic_array: BattleSide, debuff_idol_battle_realm_effect_logic_array: BattleSide) {
    let listener = screen_listener(world, this);

    for slot in 0..4 {
        let idol_battle_state = battle_state(world, listener, manage_idol_battle_realm_effect_logics).idol_battle_state_array[slot];
        let realm_effect = idol_battle_state.idol_battle_realm_effect_logic;

        if idol_battle_state.active_frame_count < 1 {
            remove_realm_effect(world, this, debuff_idol_battle_realm_effect_logic_array, realm_effect);
            remove_realm_effect(world, this, buff_idol_battle_realm_effect_logic_array, realm_effect);
            continue;
        }

        let Some(node) = idol_battle_state.struct_xml_episode_node_def else {
            continue;
        };
        let realm_powers = &episode_def(world, node).realm_powers;
        if realm_powers.is_empty() {
            continue;
        }

        // Realm damage, freeze idol and cancel enemy power-ups hurt the opponent (debuff); the
        // other powers help the idol's own side (buff).
        let mut has_buff_power = false;
        let mut has_debuff_power = false;
        for realm_power in realm_powers {
            let power = realm_power.power as u32;
            if power >= 0x10 {
                continue;
            }

            let power_bit = 1u32 << power;
            if power_bit & 0xbbfe != 0 {
                has_buff_power = true;
            } else if power_bit & 0x4401 != 0 {
                has_debuff_power = true;
            }
        }

        if has_debuff_power {
            add_realm_effect(world, this, debuff_idol_battle_realm_effect_logic_array, realm_effect, node, false);
        } else if has_buff_power {
            add_realm_effect(world, this, buff_idol_battle_realm_effect_logic_array, realm_effect, node, true);
        }
    }
}

/// `if ([array containsObject:logic]) { [array removeObject:logic]; [logic deactivate]; }` on a
/// realm-effect logic array.
fn remove_realm_effect(world: &mut World, this: Entity, side: BattleSide, realm_effect: Option<Entity>) {
    let Some(realm_effect) = realm_effect else {
        return;
    };
    let screen_logic = world.obj::<IdolScreenLogic>(this);
    let contains = realm_effect_logic_array(screen_logic, side).contains(&realm_effect);
    if !contains {
        return;
    }

    let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
    realm_effect_logic_array_mut(&mut screen_logic, side).retain(|logic| *logic != realm_effect);
    idol_battle_realm_effect_logic::deactivate(world, realm_effect);
}

/// `if (![array containsObject:logic]) { [array addObject:logic]; [logic activate:node
/// isBuffEffect:buff]; }` on a realm-effect logic array.
fn add_realm_effect(world: &mut World, this: Entity, side: BattleSide, realm_effect: Option<Entity>, node: usize, buff_effect: bool) {
    // Every idol's realm-effect logic is set by `openIdolBattleScreen` before the HUD updates
    // (`[array addObject:nil]` would raise).
    let Some(realm_effect) = realm_effect else {
        return;
    };
    let screen_logic = world.obj::<IdolScreenLogic>(this);
    let contains = realm_effect_logic_array(screen_logic, side).contains(&realm_effect);
    if contains {
        return;
    }

    let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
    realm_effect_logic_array_mut(&mut screen_logic, side).push(realm_effect);
    idol_battle_realm_effect_logic::activate(world, realm_effect, Some(node), buff_effect);
}

/// `-[IdolScreenLogic updateIdolBattleScreen]` @ 0x100193fe8
pub fn update_idol_battle_screen(world: &mut World, this: Entity) {
    let player_name_text = dynamic_text(world, "IdolBattlePlayerName");
    let enemy_name_text = dynamic_text(world, "IdolBattleEnemyName");
    let listener = screen_listener(world, this);

    // `playerName` / `enemyName` return the battle states' `mName`.
    let player_name = battle_state(world, listener, BattleSide::Player).name.clone();
    set_text(world, player_name_text, player_name.as_deref());
    let enemy_name = battle_state(world, listener, BattleSide::Enemy).name.clone();
    set_text(world, enemy_name_text, enemy_name.as_deref());

    let player_health_bar = library::logic(world, logic_table::BATTLE_HEALTH_BAR1);
    let enemy_health_bar = library::logic(world, logic_table::BATTLE_HEALTH_BAR2);

    let player_health = idol_manager::player_health(world, listener);
    let player_max_health = idol_manager::player_max_health(world, listener);
    let mut player_health_percent = player_health / player_max_health;
    if player_health_percent < 0.0 {
        player_health_percent = 0.0;
    }
    if player_health_percent > 1.0 {
        player_health_percent = 1.0;
    }

    let enemy_health = idol_manager::enemy_health(world, listener);
    let enemy_max_health = idol_manager::enemy_max_health(world, listener);
    let mut enemy_health_percent = enemy_health / enemy_max_health;
    if enemy_health_percent < 0.0 {
        enemy_health_percent = 0.0;
    }
    if enemy_health_percent > 1.0 {
        enemy_health_percent = 1.0;
    }

    let player_health_bar_animation = logic::animation(world, player_health_bar);
    animation::set_frame_percent(world, player_health_bar_animation, player_health_percent);
    let enemy_health_bar_animation = logic::animation(world, enemy_health_bar);
    animation::set_frame_percent(world, enemy_health_bar_animation, enemy_health_percent);

    manage_idol_battle_realm_effect_logics(world, this, BattleSide::Player, BattleSide::Player, BattleSide::Enemy);
    manage_idol_battle_realm_effect_logics(world, this, BattleSide::Enemy, BattleSide::Enemy, BattleSide::Player);

    let player_realm_effects = world.obj::<IdolScreenLogic>(this).player_idol_battle_realm_effect_logic_array.clone();
    let mut target_x_pos: f32 = 20.0;
    for realm_effect in player_realm_effects {
        idol_battle_realm_effect_logic::set_target_x_pos(world, realm_effect, target_x_pos, 260.0);
        target_x_pos += 40.0;
    }

    let enemy_realm_effects = world.obj::<IdolScreenLogic>(this).enemy_idol_battle_realm_effect_logic_array.clone();
    let mut target_x_pos: f32 = 440.0;
    for realm_effect in enemy_realm_effects {
        idol_battle_realm_effect_logic::set_target_x_pos(world, realm_effect, target_x_pos, 260.0);
        target_x_pos += -40.0;
    }
}

/// `-[IdolScreenLogic setBattleAttackResult:textId:iconLogic:]` @ 0x100194438
pub fn set_battle_attack_result(world: &mut World, this: Entity, battle_attack_result: Option<usize>, text_id: &str, icon_logic: Entity) {
    let node = required_node(battle_attack_result);
    let attack = episode_def(world, node).attack as i32;
    let listener = screen_listener(world, this);
    let attack_name = listener_attack_name(world, listener, attack);
    update_attack_type_logic(world, this, icon_logic, attack);
    set_dynamic_text(world, text_id, attack_name.as_deref());
}

/// `-[IdolScreenLogic setBattleRealmResult:textId:iconLogic:]` @ 0x1001944ec
pub fn set_battle_realm_result(world: &mut World, this: Entity, battle_realm_result: Option<usize>, text_id: &str, icon_logic: Entity) {
    let node = required_node(battle_realm_result);
    let realm = episode_def(world, node).realm as i32;
    let listener = screen_listener(world, this);
    let realm_name = listener_realm_name(world, listener, realm);
    update_realm_type_logic(world, this, icon_logic, realm);
    set_dynamic_text(world, text_id, realm_name.as_deref());
}

/// `-[IdolScreenLogic showBattleRealmResults]` @ 0x1001945a0
pub fn show_battle_realm_results(world: &mut World, this: Entity) {
    let listener = screen_listener(world, this);
    let special_effect_name = dynamic_text(world, "SpecialEffectName");
    let results_text = dynamic_text(world, "ResultsText");

    let player_won_realm = battle_state(world, listener, BattleSide::Player).win_realm;
    let enemy_won_realm = battle_state(world, listener, BattleSide::Enemy).win_realm;
    let player_battle_node = battle_state(world, listener, BattleSide::Player).battle_struct_xml_episode_node_def;
    let enemy_battle_node = battle_state(world, listener, BattleSide::Enemy).battle_struct_xml_episode_node_def;
    let winner_type = wired(world.obj::<IdolScreenLogic>(this).idol_battle_result_winner_type_logic);
    let loser_type = wired(world.obj::<IdolScreenLogic>(this).idol_battle_result_loser_type_logic);

    if !player_won_realm && !enemy_won_realm {
        // A tie lists the enemy's idol as the player's and the player's as the opponent's,
        // as the original does.
        set_battle_realm_result(world, this, enemy_battle_node, "PlayerName", winner_type);
        set_battle_realm_result(world, this, player_battle_node, "OpponentName", loser_type);
        show_battle_result_message(world, this, "TIES", "BattleRealmResultTie");
        set_text(world, results_text, Some("TIES"));
        return;
    }

    let (realm_winner_node, message) = if player_won_realm { (player_battle_node, "BEATS") } else { (enemy_battle_node, "LOSES TO") };
    let realm_winner = required_node(realm_winner_node);
    let realm_winner_def = episode_def(world, realm_winner);
    let power_short_description = listener_fill_power_description_text(world, listener, &realm_winner_def.power_short_description, realm_winner_node);
    set_text(world, special_effect_name, power_short_description.as_deref());

    set_battle_realm_result(world, this, player_battle_node, "PlayerName", winner_type);
    set_battle_realm_result(world, this, enemy_battle_node, "OpponentName", loser_type);
    show_battle_result_message(world, this, message, "BattleRealmResult");
    set_text(world, results_text, Some(message));
}

/// `-[IdolScreenLogic showBattleRealmResultsOnIdol]` @ 0x1001948f4
pub fn show_battle_realm_results_on_idol(world: &mut World, this: Entity) {
    let listener = screen_listener(world, this);
    let player_won_realm = battle_state(world, listener, BattleSide::Player).win_realm;
    let enemy_won_realm = battle_state(world, listener, BattleSide::Enemy).win_realm;

    let message_x_pos = if player_won_realm {
        70.0
    } else if enemy_won_realm {
        420.0
    } else {
        return;
    };

    let game_ui_manager = game_ui_manager::instance(world);
    let message = String::from("power up");
    let message_pos = Vec4::new(message_x_pos, 150.0, 0.0, 0.0);
    if let Some(game_ui_manager) = game_ui_manager {
        game_ui_manager::show_message_sequence_id_pos(world, game_ui_manager, &message, "BattleIdolWinRealm", message_pos);
    }
}

/// `-[IdolScreenLogic showBattleAttackStart]` @ 0x100194a3c
pub fn show_battle_attack_start(world: &mut World, this: Entity) {
    show_battle_result_message(world, this, "BEATS", "BattleAttackStart");
}
