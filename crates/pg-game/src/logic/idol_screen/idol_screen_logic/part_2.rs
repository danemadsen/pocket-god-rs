//! `IdolScreenLogic`: methods in this part of the class.

use pg_engine::display;
use pg_engine::display::group;
use pg_engine::display::text;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::logic::screen;
use pg_engine::prefs;
use pg_engine::sound;
use pg_engine::touch::{BcTouch, all_touches};

use crate::logic::battle::temple_door_logic;
use crate::logic::idol_ui_widgets::{episode_alert_logic, idol_battle_realm_effect_logic, idol_icon_logic, idol_selection_icon_logic};
use crate::logic_table;
use crate::managers::game_ui_manager;
use crate::managers::idol_manager::{self, EPISODE_LIST_LEN, IdolManager};
use crate::send;

use super::*;

/// Idols per side of a Battle of the Gods (`PlayerBattleState.mIdolBattleStateArray[4]`).
const BATTLE_IDOL_COUNT: usize = 4;

/// `0.5`: the pause before the next realm-effect result (`mTimeForNextRealmEffectResult`).
const REALM_EFFECT_RESULT_DELAY: f32 = 0.5;

/// Movement in screen points before a tap becomes a panel drag.
const PANEL_DRAG_THRESHOLD: f32 = 10.0;

/// An `IdolBattleState *` as the `Option<usize>` the generated signatures carry
/// (`setIdolBattleState:`, `idolBattleState`, `showBattleRealmEffectResult:battleState:
/// opponentBattleState:`): `&battleState->mIdolBattleStateArray[slot]` is `4 * side + slot`, side
/// 0 being the listener's `mPlayerBattleState` and side 1 its `mEnemyBattleState`. That is also
/// the order of `mIdolBattleRealmEffectLogicArray`, which `openIdolBattleScreen` pairs with them.
pub fn idol_battle_state_pointer(side: BattleSide, slot: usize) -> usize {
    assert!(slot < BATTLE_IDOL_COUNT, "IdolBattleState slot {slot} is outside mIdolBattleStateArray[4]");
    match side {
        BattleSide::Player => slot,
        BattleSide::Enemy => BATTLE_IDOL_COUNT + slot,
    }
}

/// The battle state and slot an [`idol_battle_state_pointer`] value points into.
pub fn idol_battle_state_location(pointer: usize) -> (BattleSide, usize) {
    if pointer < BATTLE_IDOL_COUNT {
        return (BattleSide::Player, pointer);
    }

    let slot = pointer - BATTLE_IDOL_COUNT;
    assert!(slot < BATTLE_IDOL_COUNT, "IdolBattleState pointer {pointer} is outside the two battle states");
    (BattleSide::Enemy, slot)
}

/// `*idolBattleState` for an [`idol_battle_state_pointer`] value, read in place in the
/// listener's battle state (a copy).
fn idol_battle_state_at(world: &World, listener: Entity, pointer: usize) -> IdolBattleState {
    let (side, slot) = idol_battle_state_location(pointer);
    battle_state(world, listener, side).idol_battle_state_array[slot]
}

/// The other side of the battle.
fn opponent_side(side: BattleSide) -> BattleSide {
    match side {
        BattleSide::Player => BattleSide::Enemy,
        BattleSide::Enemy => BattleSide::Player,
    }
}

/// An ivar `initWithDisplayObject:` fills from the logic table: never nil afterwards.
fn wired(logic: Option<Entity>) -> Entity {
    logic.expect("initWithDisplayObject: wires every part of the IdolScreenLogic")
}

/// A display group the original messages where nil would crash it (`[nil pos]` dereferenced,
/// `addLogicsToArray:` returning NULL): a panic here.
fn required_display_group(display_group: Option<Entity>) -> Entity {
    display_group.expect("IdolScreenLogic: the display group is nil (the original dereferences it)")
}

/// `[logic setBehavior:behavior]` on a logic that may be nil.
fn set_logic_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    if let Some(logic) = logic {
        logic::set_behavior(world, logic, Some(behavior));
    }
}

/// `[displayGroup removeFromDisplayGroup]` on a group that may be nil.
fn remove_display_group(world: &mut World, display_group: Option<Entity>) {
    if let Some(display_group) = display_group {
        display::remove_from_display_group(world, display_group);
    }
}

/// `[mDisplayObject displayGroup]`: the group the screen is drawn in.
fn screen_display_group(world: &World, this: Entity) -> Option<Entity> {
    let own_display_object = logic::display_object(world, this);
    display::object(world, own_display_object).display_group
}

/// `[displayGroup moveToDisplayGroup:[mDisplayObject displayGroup]]` on a group that may be nil.
fn move_to_screen_display_group(world: &mut World, this: Entity, display_group: Option<Entity>) {
    let Some(display_group) = display_group else {
        return;
    };

    let screen_group = screen_display_group(world, this);
    display::move_to_display_group(world, display_group, screen_group);
}

/// `[[mDisplayObject displayGroup] viewToDisplayObject:[touch pos]]`: the touch in the screen
/// group's space (a message to a nil group returns the zero vector).
fn touch_pos_in_screen_group(world: &World, this: Entity, touch: Entity) -> Vec4 {
    let screen_group = screen_display_group(world, this);
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    match screen_group {
        Some(screen_group) => display::view_to_display_object(world, screen_group, touch_pos),
        None => Vec4::ZERO,
    }
}

/// `[preferences getIntPreference:key]` (0 for a nil document).
fn int_preference(world: &World, preferences: Option<Entity>, key: &str) -> i32 {
    match preferences {
        Some(preferences) => prefs::get_int_preference(world, preferences, key),
        None => 0,
    }
}

/// `[preferences setPreference:key asInt:value]` on a document that may be nil.
fn set_int_preference(world: &mut World, preferences: Option<Entity>, key: &str, value: i32) {
    if let Some(preferences) = preferences {
        prefs::set_preference_as_int(world, preferences, key, value);
    }
}

/// `[[GameUiManager instance] showMessage:message sequenceId:sequence_id pos:pos]`.
fn show_message(world: &mut World, message: &str, sequence_id: &str, pos: Vec4) {
    let game_ui_manager = game_ui_manager::instance(world);
    if let Some(game_ui_manager) = game_ui_manager {
        game_ui_manager::show_message_sequence_id_pos(world, game_ui_manager, message, sequence_id, pos);
    }
}

/// `[[GameUiManager instance] showMessage:message sequenceId:sequence_id pos:mGameResultsPos]`.
fn show_game_results_message(world: &mut World, this: Entity, message: &str, sequence_id: &str) {
    let game_results_pos = world.obj::<IdolScreenLogic>(this).game_results_pos;
    show_message(world, message, sequence_id, game_results_pos);
}

/// `[NSString stringWithFormat:@"-%.0f", value]`.
fn negative_value_text(value: f32) -> String {
    format!("-{:.0}", f64::from(value))
}

/// `[NSString stringWithFormat:@"+%.0f", value]`.
fn positive_value_text(value: f32) -> String {
    format!("+{:.0}", f64::from(value))
}

/// `%@` of a battle state's `mName`: a nil name prints as "(null)".
fn battle_name(world: &World, listener: Entity, side: BattleSide) -> String {
    let name = battle_state(world, listener, side).name.clone();
    name.unwrap_or_else(|| String::from("(null)"))
}

/// `[[[TempleDoorLogic instance] battlePygmyLogic] displayObject]` (or `battleEnemyPygmyLogic`):
/// nil along a nil chain.
fn battle_pygmy_display_object(world: &mut World, side: BattleSide) -> Option<Entity> {
    let temple_door = temple_door_logic::instance(world)?;
    let battle_pygmy = match side {
        BattleSide::Player => temple_door_logic::battle_pygmy_logic(world, temple_door),
        BattleSide::Enemy => temple_door_logic::battle_enemy_pygmy_logic(world, temple_door),
    };
    let battle_pygmy = battle_pygmy?;
    let display_object = logic::display_object(world, battle_pygmy);
    Some(display_object)
}

/// `pos->x` through the `[displayObject pos]` pointer of a battle pygmy: a nil display object
/// gives a NULL `pos`, which the original dereferences (a crash there, a panic here).
fn battle_pygmy_x_pos(world: &World, display_object: Option<Entity>) -> f32 {
    let display_object = display_object.expect("showBattleAttackResults: the battle pygmy is nil (the original dereferences its pos)");
    display::object(world, display_object).pos.x
}

/// `setBattleAttackResult:` for both battle idols, then the result message's behaviour and the
/// `ResultsText` text: the shared part of every attack outcome.
fn set_battle_attack_results(world: &mut World, this: Entity, listener: Entity, behavior: &str, results_text: Option<Entity>, result: &str) {
    let player_battle_node = battle_state(world, listener, BattleSide::Player).battle_struct_xml_episode_node_def;
    let winner_type = wired(world.obj::<IdolScreenLogic>(this).idol_battle_result_winner_type_logic);
    set_battle_attack_result(world, this, player_battle_node, "PlayerName", winner_type);

    let enemy_battle_node = battle_state(world, listener, BattleSide::Enemy).battle_struct_xml_episode_node_def;
    let loser_type = wired(world.obj::<IdolScreenLogic>(this).idol_battle_result_loser_type_logic);
    set_battle_attack_result(world, this, enemy_battle_node, "OpponentName", loser_type);

    let message_logic = world.obj::<IdolScreenLogic>(this).idol_battle_result_message_logic;
    set_logic_behavior(world, message_logic, behavior);
    set_text(world, results_text, Some(result));
}

/// The attack winner's reflect-damage (`-`, at y = 100) and drain-health (`+`, below it when
/// both show) messages at `x_pos`.
fn show_winner_power_results(world: &mut World, listener: Entity, winner: BattleSide, x_pos: f32) {
    let mut drain_y_pos: f32 = 100.0;

    let reflect_damage = battle_state(world, listener, winner).actual_reflect_damage;
    if reflect_damage != 0.0 {
        let message = negative_value_text(reflect_damage);
        let reflect_pos = Vec4::new(x_pos, 100.0, 0.0, 0.0);
        show_message(world, &message, "BattleNegativeResult", reflect_pos);
        drain_y_pos = 60.0;
    }

    let drain_health = battle_state(world, listener, winner).actual_drain_health;
    if drain_health == 0.0 {
        return;
    }

    let message = positive_value_text(drain_health);
    let drain_pos = Vec4::new(x_pos, drain_y_pos, 0.0, 0.0);
    show_message(world, &message, "BattlePositiveResult", drain_pos);
}

/// `-[IdolScreenLogic showBattleAttackResults]` @ 0x100194aac
pub fn show_battle_attack_results(world: &mut World, this: Entity) {
    let listener = screen_listener(world, this);
    let player_pygmy = battle_pygmy_display_object(world, BattleSide::Player);
    let enemy_pygmy = battle_pygmy_display_object(world, BattleSide::Enemy);
    let results_text = dynamic_text(world, "ResultsText");

    let player_won_attack = battle_state(world, listener, BattleSide::Player).win_attack;
    if player_won_attack {
        let mut power_x_pos = battle_pygmy_x_pos(world, player_pygmy);
        let enemy_health = battle_state(world, listener, BattleSide::Enemy).health;
        let damage_pos = if enemy_health > 0.0 {
            power_x_pos += -128.0;
            Vec4::new(410.0, 50.0, 0.0, 0.0)
        } else {
            Vec4::new(430.0, 100.0, 0.0, 0.0)
        };
        if power_x_pos < 30.0 {
            power_x_pos = 30.0;
        }

        set_battle_attack_results(world, this, listener, "BattleAttackResult", results_text, "BEATS");

        let enemy_damage = battle_state(world, listener, BattleSide::Enemy).actual_attack_damage;
        let message = negative_value_text(enemy_damage);
        show_message(world, &message, "BattleNegativeResult", damage_pos);

        show_winner_power_results(world, listener, BattleSide::Player, power_x_pos);
        return;
    }

    let enemy_won_attack = battle_state(world, listener, BattleSide::Enemy).win_attack;
    if enemy_won_attack {
        let mut power_x_pos = battle_pygmy_x_pos(world, enemy_pygmy);
        let player_health = battle_state(world, listener, BattleSide::Player).health;
        let damage_pos = if player_health > 0.0 {
            power_x_pos += 128.0;
            Vec4::new(70.0, 50.0, 0.0, 0.0)
        } else {
            Vec4::new(50.0, 100.0, 0.0, 0.0)
        };
        if power_x_pos > 450.0 {
            power_x_pos = 450.0;
        }

        // The player's idol still goes in the winner slot, as in the original.
        set_battle_attack_results(world, this, listener, "BattleAttackResult", results_text, "LOSES TO");

        let player_damage = battle_state(world, listener, BattleSide::Player).actual_attack_damage;
        let message = negative_value_text(player_damage);
        show_message(world, &message, "BattleNegativeResult", damage_pos);

        show_winner_power_results(world, listener, BattleSide::Enemy, power_x_pos);
        return;
    }

    set_battle_attack_results(world, this, listener, "BattleAttackResultTie", results_text, "TIES");

    // A tie shows each side's tie damage over the other side's pygmy.
    let enemy_x_pos = battle_pygmy_x_pos(world, enemy_pygmy);
    let enemy_tie_damage = battle_state(world, listener, BattleSide::Enemy).tie_damage;
    if enemy_tie_damage > 0.0 {
        let player_x_pos = battle_pygmy_x_pos(world, player_pygmy);
        let message = negative_value_text(enemy_tie_damage);
        let message_pos = Vec4::new(player_x_pos, 100.0, 0.0, 0.0);
        show_message(world, &message, "BattleNegativeResult", message_pos);
    }

    let player_tie_damage = battle_state(world, listener, BattleSide::Player).tie_damage;
    if player_tie_damage > 0.0 {
        let message = negative_value_text(player_tie_damage);
        let message_pos = Vec4::new(enemy_x_pos, 100.0, 0.0, 0.0);
        show_message(world, &message, "BattleNegativeResult", message_pos);
    }
}

/// `-[IdolScreenLogic showBattleRealmEffectResultsForPlayer:]` @ 0x1001951e0
///
/// The parameter is an `NSArray` of `IdolBattleRealmEffectLogic`s. Nothing in the binary sends
/// this selector.
pub fn show_battle_realm_effect_results_for_player(world: &mut World, this: Entity, show_battle_realm_effect_results_for_player: &[Entity]) {
    let listener = screen_listener(world, this);

    for &realm_effect in show_battle_realm_effect_results_for_player {
        let idol_battle_state = idol_battle_realm_effect_logic::idol_battle_state(world, realm_effect);
        let idol_battle_state = idol_battle_state.expect("showBattleRealmEffectResultsForPlayer: the realm effect has no IdolBattleState");
        let realm_effect_display_object = logic::display_object(world, realm_effect);
        let effect_pos = display::object(world, realm_effect_display_object).pos;

        let state = idol_battle_state_at(world, listener, idol_battle_state);
        if state.damage > 0.0 {
            let message = negative_value_text(state.damage);
            show_message(world, &message, "BattleNegativeRealmEffectResult", effect_pos);
        }

        let state = idol_battle_state_at(world, listener, idol_battle_state);
        if state.health > 0.0 {
            // Original bug, kept: the positive message shows the damage, not the health.
            let message = positive_value_text(state.damage);
            show_message(world, &message, "BattlePositiveRealmEffectResult", effect_pos);
        }
    }
}

/// `-[IdolScreenLogic showBattleRealmEffectResults:::]` @ 0x100195448
pub fn seq_show_battle_realm_effect_results(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        world.obj_mut::<IdolScreenLogic>(this).time_for_next_realm_effect_result = REALM_EFFECT_RESULT_DELAY;

        let game_ui_manager = game_ui_manager::instance(world);
        if let Some(game_ui_manager) = game_ui_manager {
            let screen_logic = world.obj::<IdolScreenLogic>(this);
            let game_results_pos = screen_logic.game_results_pos;
            let message_logic = wired(screen_logic.idol_battle_result_message_logic);
            game_ui_manager::show_message_sequence_id_pos_message_logic(world, game_ui_manager, "BEATS", "BattleEffectsStart", game_results_pos, message_logic);
        }
    }

    let delta_time = call.delta_time;
    let time_for_next_result = {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.time_for_next_realm_effect_result -= delta_time;
        screen_logic.time_for_next_realm_effect_result
    };
    if time_for_next_result > 0.0 {
        return ItemResult::Pending;
    }

    // The player's side first: its own buffs and the enemy's debuffs, then the enemy's side.
    let listener = screen_listener(world, this);
    let player_battle_state = idol_manager::player_battle_state(world, listener);
    let enemy_battle_state = idol_manager::enemy_battle_state(world, listener);
    let showing_player = world.obj::<IdolScreenLogic>(this).showing_player_battle_realm_effect_results;
    let (side, battle_state_pointer, opponent_battle_state_pointer) = if showing_player {
        (BattleSide::Player, player_battle_state, enemy_battle_state)
    } else {
        (BattleSide::Enemy, enemy_battle_state, player_battle_state)
    };

    let screen_logic = world.obj::<IdolScreenLogic>(this);
    let realm_effects = realm_effect_logic_array(screen_logic, side);
    let realm_effect = match usize::try_from(screen_logic.battle_realm_effect_result_index) {
        Ok(result_index) => realm_effects.get(result_index).copied(),
        Err(_) => None,
    };

    if let Some(realm_effect) = realm_effect {
        let idol_battle_state = idol_battle_realm_effect_logic::idol_battle_state(world, realm_effect);
        let realm_effect_display_object = logic::display_object(world, realm_effect);
        let effect_pos = display::object(world, realm_effect_display_object).pos;

        // A buff acts for this side against the opponent; a debuff is the opponent's idol
        // acting against this side.
        let buff_effect = idol_battle_realm_effect_logic::is_buff_effect(world, realm_effect);
        let (acting_battle_state, target_battle_state) = if buff_effect {
            (battle_state_pointer, opponent_battle_state_pointer)
        } else {
            (opponent_battle_state_pointer, battle_state_pointer)
        };
        idol_manager::show_battle_realm_effect_result(world, listener, idol_battle_state, acting_battle_state, target_battle_state);

        let idol_battle_state = idol_battle_state.expect("showBattleRealmEffectResults: the realm effect has no IdolBattleState");
        let state = idol_battle_state_at(world, listener, idol_battle_state);
        let showed_damage = state.damage > 0.0;
        if showed_damage {
            let total_defense = battle_state(world, listener, side).total_defense;
            let total_boost_damage = battle_state(world, listener, opponent_side(side)).total_boost_damage;
            let mut damage_factor = total_boost_damage + 1.0 - total_defense;
            if damage_factor < 0.0 {
                damage_factor = 0.0;
            }

            let message = negative_value_text(state.damage * damage_factor);
            show_message(world, &message, "BattleNegativeRealmEffectResult", effect_pos);
        }

        let state = idol_battle_state_at(world, listener, idol_battle_state);
        let showed_health = state.health > 0.0;
        if showed_health {
            let message = positive_value_text(state.health);
            show_message(world, &message, "BattlePositiveRealmEffectResult", effect_pos);
        }

        if showed_damage || showed_health {
            world.obj_mut::<IdolScreenLogic>(this).time_for_next_realm_effect_result = REALM_EFFECT_RESULT_DELAY;
        }
    }

    // `showBattleRealmEffectResult:` updates the battle screen, which may add or drop realm
    // effects, so the count is taken again.
    let next_index = {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.battle_realm_effect_result_index = screen_logic.battle_realm_effect_result_index.wrapping_add(1);
        screen_logic.battle_realm_effect_result_index
    };
    let screen_logic = world.obj::<IdolScreenLogic>(this);
    let realm_effect_count = realm_effect_logic_array(screen_logic, side).len();
    let more_results = match usize::try_from(next_index) {
        Ok(next_index) => next_index < realm_effect_count,
        Err(_) => false,
    };
    if more_results {
        return ItemResult::Pending;
    }

    let showing_player = {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.battle_realm_effect_result_index = 0;
        screen_logic.showing_player_battle_realm_effect_results
    };
    if showing_player {
        world.obj_mut::<IdolScreenLogic>(this).showing_player_battle_realm_effect_results = false;
        return ItemResult::Pending;
    }

    world.obj_mut::<IdolScreenLogic>(this).showing_battle_realm_effect_results = false;
    let message_logic = world.obj::<IdolScreenLogic>(this).idol_battle_result_message_logic;
    set_logic_behavior(world, message_logic, "BattleEffectsEnd");

    let listener = screen_listener(world, this);
    idol_manager::check_battle_realm_effect_death(world, listener);
    ItemResult::Done
}

/// `-[IdolScreenLogic showBattleRealmEffectResults]` @ 0x100195874
pub fn show_battle_realm_effect_results(world: &mut World, this: Entity) {
    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.battle_realm_effect_result_index = 0;
        screen_logic.showing_battle_realm_effect_results = true;
        screen_logic.showing_player_battle_realm_effect_results = true;
    }
    logic::set_behavior(world, this, Some("IdolScreenShowRealmEffectResults"));
}

/// `-[IdolScreenLogic areBattleRealmEffectResultsComplete]` @ 0x1001958b4
pub fn are_battle_realm_effect_results_complete(world: &mut World, this: Entity) -> bool {
    !world.obj::<IdolScreenLogic>(this).showing_battle_realm_effect_results
}

/// `-[IdolScreenLogic showBattleResurrectionResultsForPlayer:forEnemy:]` @ 0x1001958cc
pub fn show_battle_resurrection_results_for_player(world: &mut World, this: Entity, show_battle_resurrection_results_for_player: bool, for_enemy: bool) {
    let listener = screen_listener(world, this);
    let player_resurrects = show_battle_resurrection_results_for_player;

    let message = if player_resurrects && for_enemy {
        let player_name = battle_name(world, listener, BattleSide::Player);
        let enemy_name = battle_name(world, listener, BattleSide::Enemy);
        format!("Both {player_name} and {enemy_name}\ncome back from the dead!")
    } else if player_resurrects {
        let player_name = battle_name(world, listener, BattleSide::Player);
        format!("{player_name}\ncomes back from the dead!")
    } else if for_enemy {
        let enemy_name = battle_name(world, listener, BattleSide::Enemy);
        format!("{enemy_name}\ncomes back from the dead!")
    } else {
        return;
    };

    show_game_results_message(world, this, &message, "BattleResult");
}

/// `-[IdolScreenLogic openIdolBattleEnemyScreen]` @ 0x100195ad0
pub fn open_idol_battle_enemy_screen(world: &mut World, this: Entity) {
    screen::open_screen(world, this);
    world.obj_mut::<IdolScreenLogic>(this).root_idol_screen_mode = 5;
    logic::set_binary_frame_def(world, this, None, None);
    open_idol_battle_enemy_panel(world, this);

    let listener = screen_listener(world, this);
    let option_preferences = idol_manager::option_preferences(world, listener);
    let show_battle_help = int_preference(world, option_preferences, "ShowBattleHelp");
    if show_battle_help != 0 {
        world.obj_mut::<IdolScreenLogic>(this).battle_help_open = true;
        screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolBattleHelpScreen"));
    }
}

/// `-[IdolScreenLogic openIdolBattleScreen]` @ 0x100195bc0
///
/// Realm-effect logics 1–4 go with the player's idols and 5–8 with the enemy's: each logic gets
/// its `IdolBattleState *` and the state keeps the logic (`mIdolBattleRealmEffectLogic`).
pub fn open_idol_battle_screen(world: &mut World, this: Entity) {
    screen::open_screen(world, this);
    logic::set_binary_frame_def(world, this, None, None);
    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.root_idol_screen_mode = 6;
        screen_logic.idol_screen_mode = 6;
    }

    let listener = screen_listener(world, this);
    let sides = [BattleSide::Player, BattleSide::Enemy];
    for (side_index, side) in sides.into_iter().enumerate() {
        for slot in 0..BATTLE_IDOL_COUNT {
            let realm_effect_index = side_index * BATTLE_IDOL_COUNT + slot;
            let realm_effect = world.obj::<IdolScreenLogic>(this).idol_battle_realm_effect_logic_array[realm_effect_index];
            let idol_battle_state = idol_battle_state_pointer(side, slot);
            idol_battle_realm_effect_logic::set_idol_battle_state(world, realm_effect, Some(idol_battle_state));

            let mut idol_manager = world.obj_mut::<IdolManager>(listener);
            let battle_state = match side {
                BattleSide::Player => &mut idol_manager.player_battle_state,
                BattleSide::Enemy => &mut idol_manager.enemy_battle_state,
            };
            battle_state.idol_battle_state_array[slot].idol_battle_realm_effect_logic = Some(realm_effect);
        }
    }

    update_idol_battle_screen(world, this);
    screen::set_screen_layer_behavior(world, this, 0, Some("OpenIdolBattleScreen"));
}

/// `-[IdolScreenLogic openIdolBattleResultsScreen]` @ 0x100195d60
pub fn open_idol_battle_results_screen(world: &mut World, this: Entity) {
    screen::open_screen(world, this);
    logic::set_binary_frame_def(world, this, None, None);
    world.obj_mut::<IdolScreenLogic>(this).root_idol_screen_mode = 8;
    open_idol_battle_results_panel(world, this);
}

/// `-[IdolScreenLogic openIdolAwardScreen:]` @ 0x100195de8
pub fn open_idol_award_screen(world: &mut World, this: Entity, open_idol_award_screen: Option<usize>) {
    screen::open_screen(world, this);
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 3;
    logic::set_binary_frame_def(world, this, None, None);
    world.obj_mut::<IdolScreenLogic>(this).struct_xml_episode_node_def = open_idol_award_screen;

    let award_description = dynamic_text(world, "IdolAwardDescription");
    let node = required_node(open_idol_award_screen);
    let node_def = episode_def(world, node);
    let description = format!(
        "You unlocked the {} idol!\nChoose another challenge or continue to level this idol up for the big battle!",
        node_def.idol_name
    );
    set_text(world, award_description, Some(&description));
    if let Some(award_description) = award_description {
        text::set_wrap_width(world, award_description, 195.0);
        text::set_line_count(world, award_description, 3);
    }

    screen::set_screen_layer_behavior(world, this, 0, Some("OpenIdolAwardScreen"));
}

/// `-[IdolScreenLogic closeScreen]` @ 0x100195f40
pub fn close_screen(world: &mut World, this: Entity) {
    screen::set_screen_layer_behavior(world, this, 0, Some("CloseIdolScreen"));
}

/// `-[IdolScreenLogic closeScreenNow]` @ 0x100195f5c
pub fn close_screen_now(world: &mut World, this: Entity) {
    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.summary_open = false;
        screen_logic.battle_help_open = false;
    }
    screen::close_screen_now(world, this);

    let idol_screen_mode = world.obj::<IdolScreenLogic>(this).idol_screen_mode;
    let listener = screen_listener(world, this);
    if idol_screen_mode == 3 {
        idol_manager::on_idol_award_screen_closed(world, listener);
    } else {
        idol_manager::on_idol_screen_closed(world, listener);
    }
}

/// The panels `onGameFrame:` and the touches scroll vertically.
#[derive(Clone, Copy)]
enum ScrollPanel {
    /// The idol detail panel (mode 2).
    Detail,

    /// The idol list (every other mode outside the selection panels).
    List,
}

/// `onGameFrame:`'s inertia for a vertically scrolled panel: while no finger holds it, it keeps
/// moving by `mDragYPosVel` (×0.9 per frame) and eases halfway back into its bounds.
fn scroll_panel_with_inertia(world: &mut World, this: Entity, panel: ScrollPanel) {
    let screen_logic = world.obj::<IdolScreenLogic>(this);
    if screen_logic.active_touch.is_some() {
        return;
    }

    let (display_group, max_y_pos, min_y_pos) = match panel {
        ScrollPanel::Detail => (
            screen_logic.idol_icon_detail_display_group,
            screen_logic.scroll_detail_display_group_max_y_pos,
            screen_logic.scroll_detail_display_group_min_y_pos,
        ),
        ScrollPanel::List => (
            screen_logic.idol_icon_list_display_group,
            screen_logic.scroll_list_display_group_max_y_pos,
            screen_logic.scroll_list_display_group_min_y_pos,
        ),
    };
    let mut drag_y_pos_vel = screen_logic.drag_y_pos_vel;
    let display_group = required_display_group(display_group);

    let mut y_pos = display::object(world, display_group).pos.y;
    y_pos = drag_y_pos_vel + y_pos;
    drag_y_pos_vel *= 0.9;
    if y_pos < min_y_pos {
        drag_y_pos_vel = 0.0;
        y_pos = y_pos + (min_y_pos - y_pos) * 0.5;
    }
    if max_y_pos < y_pos {
        drag_y_pos_vel = 0.0;
        y_pos = y_pos + (max_y_pos - y_pos) * 0.5;
    }

    display::object_mut(world, display_group).pos.y = y_pos;
    world.obj_mut::<IdolScreenLogic>(this).drag_y_pos_vel = drag_y_pos_vel;
}

/// `onGameFrame:`'s inertia for the horizontally scrolled idol library of the selection and
/// battle-enemy panels (`mIdolSelectionIconScrollXPos`, `mDragXPosVel`).
fn scroll_library_with_inertia(world: &mut World, this: Entity) {
    let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
    if screen_logic.active_touch.is_some() {
        return;
    }

    let mut scroll_x_pos = screen_logic.drag_x_pos_vel + screen_logic.idol_selection_icon_scroll_x_pos;
    let mut drag_x_pos_vel = screen_logic.drag_x_pos_vel * 0.9;

    let min_x_pos = screen_logic.scroll_selection_display_group_min_x_pos;
    if scroll_x_pos < min_x_pos {
        drag_x_pos_vel = 0.0;
        scroll_x_pos = scroll_x_pos + (min_x_pos - scroll_x_pos) * 0.5;
    }

    let max_x_pos = screen_logic.scroll_selection_display_group_max_x_pos;
    if max_x_pos < scroll_x_pos {
        drag_x_pos_vel = 0.0;
        scroll_x_pos = scroll_x_pos + (max_x_pos - scroll_x_pos) * 0.5;
    }

    screen_logic.idol_selection_icon_scroll_x_pos = scroll_x_pos;
    screen_logic.drag_x_pos_vel = drag_x_pos_vel;
}

/// `-[IdolScreenLogic onGameFrame:]` @ 0x100195ff8
pub fn on_game_frame(world: &mut World, this: Entity, _dt: f32) {
    let shutdown = logic::logic(world, this).shutdown;
    if shutdown {
        return;
    }

    let idol_screen_mode = world.obj::<IdolScreenLogic>(this).idol_screen_mode;
    if idol_screen_mode == 2 {
        scroll_panel_with_inertia(world, this, ScrollPanel::Detail);
    } else if matches!(idol_screen_mode, 4 | 5) {
        scroll_library_with_inertia(world, this);

        let idol_screen_mode = world.obj::<IdolScreenLogic>(this).idol_screen_mode;
        if idol_screen_mode == 5 {
            update_idol_battle_enemy_selection_icon_logics(world, this);
        } else {
            update_idol_selection_icon_logics(world, this);
        }
        return;
    } else {
        scroll_panel_with_inertia(world, this, ScrollPanel::List);
    }

    let idol_screen_mode = world.obj::<IdolScreenLogic>(this).idol_screen_mode;
    if idol_screen_mode == 1 {
        update_idol_icon_logics(world, this);
    }
}

/// `-[IdolScreenLogic loadLocationTextureAtlas:::]` @ 0x100196238
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = screen_listener(world, this);
    idol_manager::on_load_location_texture_atlas(world, listener);
    ItemResult::Done
}

/// `-[IdolScreenLogic showEnemyPanelMultiplayerAlert:message:]` @ 0x100196268
pub fn show_enemy_panel_multiplayer_alert(world: &mut World, this: Entity, show_enemy_panel_multiplayer_alert: &str, message: &str) {
    set_dynamic_text(world, "IdolDetailHintHeader", Some(show_enemy_panel_multiplayer_alert));
    set_dynamic_text(world, "IdolDetailHintText", Some(message));

    let hint_panel = world.obj::<IdolScreenLogic>(this).idol_detail_header_hint_panel_logic;
    set_logic_behavior(world, hint_panel, "IdolDetailHintShow");

    let hint_panel_buttons = world.obj::<IdolScreenLogic>(this).idol_detail_hint_panel_buttons_display_group;
    move_to_screen_display_group(world, this, hint_panel_buttons);
}

/// What `onButtonPress:touch:` does for each of the screen's buttons.
#[derive(Clone, Copy)]
enum IdolScreenButton {
    Close,
    OpenWhatsNewPanel,
    OpenOptionsPanel,
    OpenActiveEpisodeList,
    CloseScreen,
    LaunchStoreScreen,
    OpenStartPanel,
    StartScreenCheckBox,
    EpisodeIconCheckBox,
    ResetChallenges,
    UnlockChallenges,
    LevelUpIdols,
    DetailPanelBack,
    ActivateEpisode,
    OpenSummary,
    SummaryBack,
    GoToGodIsland,
    OpenChallenges,
    ShowHint,
    CloseHintPanel,
    CloseAreYouSurePanel,
    ConfirmAreYouSure,
    ExitBattle,
    Attack,
    OpenBattleEnemyPanel,
    RetryBattle,
    OpenMatchPanel,
    NextBattle,
    OpenBattleHelp,
    CloseBattleHelp,
    BattleHelpCheckBox,
    ListBack,
}

/// The action of `button`: the original compares it with the button ivars in this order and
/// acts on the first match.
fn pressed_button(screen_logic: &IdolScreenLogic, button: Entity) -> Option<IdolScreenButton> {
    let buttons = [
        (screen_logic.idol_screen_close_button_logic, IdolScreenButton::Close),
        (screen_logic.idol_start_whats_new_button_logic, IdolScreenButton::OpenWhatsNewPanel),
        (screen_logic.idol_start_options_button_logic, IdolScreenButton::OpenOptionsPanel),
        (screen_logic.idol_start_challenge_button_logic, IdolScreenButton::OpenActiveEpisodeList),
        (screen_logic.idol_start_close_button_logic, IdolScreenButton::CloseScreen),
        (screen_logic.idol_start_skin_pack_button_logic, IdolScreenButton::LaunchStoreScreen),
        (screen_logic.idol_whats_new_back_button_logic, IdolScreenButton::OpenStartPanel),
        (screen_logic.idol_options_start_check_box_button_logic, IdolScreenButton::StartScreenCheckBox),
        (screen_logic.idol_options_icon_check_box_button_logic, IdolScreenButton::EpisodeIconCheckBox),
        (screen_logic.idol_options_reset_button_logic, IdolScreenButton::ResetChallenges),
        (screen_logic.idol_options_unlock_button_logic, IdolScreenButton::UnlockChallenges),
        (screen_logic.idol_options_level_up_button_logic, IdolScreenButton::LevelUpIdols),
        (screen_logic.idol_options_back_button_logic, IdolScreenButton::OpenStartPanel),
        (screen_logic.idol_detail_panel_back_button_logic, IdolScreenButton::DetailPanelBack),
        (screen_logic.idol_detail_panel_challenge_button_logic, IdolScreenButton::ActivateEpisode),
        (screen_logic.idol_detail_header_summary_button_logic, IdolScreenButton::OpenSummary),
        (screen_logic.idol_summary_back_button_logic, IdolScreenButton::SummaryBack),
        (screen_logic.idol_award_close_button_logic, IdolScreenButton::CloseScreen),
        (screen_logic.idol_award_god_island_button_logic, IdolScreenButton::GoToGodIsland),
        (screen_logic.idol_award_challenges_button_logic, IdolScreenButton::OpenChallenges),
        (screen_logic.idol_detail_header_hint_button_logic, IdolScreenButton::ShowHint),
        (screen_logic.idol_detail_hint_panel_close_button_logic, IdolScreenButton::CloseHintPanel),
        (screen_logic.idol_are_you_sure_panel_close_button_logic, IdolScreenButton::CloseAreYouSurePanel),
        (screen_logic.idol_are_you_sure_panel_reset_button_logic, IdolScreenButton::ConfirmAreYouSure),
        (screen_logic.idol_selection_challenges_button_logic, IdolScreenButton::OpenActiveEpisodeList),
        (screen_logic.idol_battle_selection_exit_button_logic, IdolScreenButton::ExitBattle),
        (screen_logic.idol_battle_selection_attack_button_logic, IdolScreenButton::Attack),
        (screen_logic.idol_battle_results_win_battle_select_button_logic, IdolScreenButton::OpenBattleEnemyPanel),
        (screen_logic.idol_battle_results_lose_battle_select_button_logic, IdolScreenButton::OpenBattleEnemyPanel),
        (screen_logic.idol_battle_results_final_battle_select_button_logic, IdolScreenButton::OpenBattleEnemyPanel),
        (screen_logic.idol_battle_results_win_multiplayer_battle_select_button_logic, IdolScreenButton::OpenBattleEnemyPanel),
        (screen_logic.idol_battle_results_lose_multiplayer_battle_select_button_logic, IdolScreenButton::OpenBattleEnemyPanel),
        (screen_logic.idol_battle_results_win_retry_button_logic, IdolScreenButton::RetryBattle),
        (screen_logic.idol_battle_results_lose_retry_button_logic, IdolScreenButton::RetryBattle),
        (screen_logic.idol_battle_results_final_retry_button_logic, IdolScreenButton::RetryBattle),
        (screen_logic.idol_battle_results_win_multiplayer_new_match_button_logic, IdolScreenButton::OpenMatchPanel),
        (screen_logic.idol_battle_results_lose_multiplayer_new_match_button_logic, IdolScreenButton::OpenMatchPanel),
        (screen_logic.idol_battle_results_win_next_button_logic, IdolScreenButton::NextBattle),
        (screen_logic.idol_battle_enemy_help_button_logic, IdolScreenButton::OpenBattleHelp),
        (screen_logic.idol_battle_enemy_multiplayer_button_logic, IdolScreenButton::OpenMatchPanel),
        (screen_logic.idol_battle_help_battle_button_logic, IdolScreenButton::CloseBattleHelp),
        (screen_logic.idol_battle_help_check_box_button_logic, IdolScreenButton::BattleHelpCheckBox),
        (screen_logic.idol_list_back_button_logic, IdolScreenButton::ListBack),
    ];

    for (button_logic, action) in buttons {
        if button_logic == Some(button) {
            return Some(action);
        }
    }
    None
}

/// `[[TempleDoorLogic instance] cancelBattle]`.
fn cancel_temple_door_battle(world: &mut World) {
    let temple_door = temple_door_logic::instance(world);
    if let Some(temple_door) = temple_door {
        temple_door_logic::cancel_battle(world, temple_door);
    }
}

/// `[[TempleDoorLogic instance] quitGame]`.
fn quit_temple_door_game(world: &mut World) {
    let temple_door = temple_door_logic::instance(world);
    if let Some(temple_door) = temple_door {
        temple_door_logic::quit_game(world, temple_door);
    }
}

/// `[[[BCLibrary instance] getDynamicTextById:@"HintsUsed"] setText:@"%d" activatedHintCount]`.
fn update_hints_used_text(world: &mut World, this: Entity) {
    let hints_used = dynamic_text(world, "HintsUsed");
    let listener = screen_listener(world, this);
    let activated_hint_count = idol_manager::activated_hint_count(world, listener);
    let hint_count = format!("{activated_hint_count}");
    set_text(world, hints_used, Some(&hint_count));
}

/// The close button: leaves a battle screen (cancelling the temple door's battle), goes back to
/// the battle selection from the battle screen, or closes the screen.
fn press_close_button(world: &mut World, this: Entity) {
    let root_idol_screen_mode = world.obj::<IdolScreenLogic>(this).root_idol_screen_mode;
    if root_idol_screen_mode == 5 || root_idol_screen_mode == 8 {
        cancel_temple_door_battle(world);
        close_screen(world, this);
        return;
    }

    if root_idol_screen_mode == 6 {
        {
            let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
            screen_logic.summary_open = false;
            screen_logic.battle_help_open = false;
        }
        open_idol_battle_selection_panel(world, this);
        return;
    }

    close_screen(world, this);
}

/// The options panel's check boxes: plays `Button`, flips the `Options` preference `key` and
/// shows the new state on `check_box_logic`; the episode-icon box also tells the episode alert.
fn toggle_option_check_box(world: &mut World, this: Entity, key: &str, check_box_logic: Option<Entity>, notify_episode_alert: bool) {
    sound::play_sound(world, "Button");

    let listener = screen_listener(world, this);
    let option_preferences = idol_manager::option_preferences(world, listener);
    let was_checked = int_preference(world, option_preferences, key) != 0;
    let checked = !was_checked;
    set_int_preference(world, option_preferences, key, i32::from(checked));

    if notify_episode_alert {
        let episode_alert = idol_manager::episode_alert_icon(world, listener);
        if let Some(episode_alert) = episode_alert {
            episode_alert_logic::on_episode_icon_pref_changed(world, episode_alert, checked);
        }
    }

    let behavior = if checked { "ShowIdolStartCheckBox" } else { "HideIdolStartCheckBox" };
    set_logic_behavior(world, check_box_logic, behavior);
}

/// The reset, unlock and level-up buttons of the options panel: the are-you-sure panel with its
/// question; `mIdolScreenAreaYouSureType` remembers which action OK confirms.
fn show_are_you_sure_panel(world: &mut World, this: Entity, are_you_sure_type: i32, question: &str) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_area_you_sure_type = are_you_sure_type;
    set_dynamic_text(world, "IdolAreYouSureText", Some(question));

    let are_you_sure_panel = world.obj::<IdolScreenLogic>(this).idol_are_you_sure_panel_logic;
    set_logic_behavior(world, are_you_sure_panel, "IdolAreYouSureShow");

    let are_you_sure_buttons = world.obj::<IdolScreenLogic>(this).idol_are_you_sure_panel_buttons_display_group;
    move_to_screen_display_group(world, this, are_you_sure_buttons);
}

/// Hides the are-you-sure panel and its buttons.
fn hide_are_you_sure_panel(world: &mut World, this: Entity) {
    let are_you_sure_panel = world.obj::<IdolScreenLogic>(this).idol_are_you_sure_panel_logic;
    set_logic_behavior(world, are_you_sure_panel, "IdolAreYouSureHide");

    let are_you_sure_buttons = world.obj::<IdolScreenLogic>(this).idol_are_you_sure_panel_buttons_display_group;
    remove_display_group(world, are_you_sure_buttons);
}

/// OK on the are-you-sure panel: level every idol up to level 20, unlock every challenge, or
/// reset every challenge (and the hint count), then hide the panel.
fn confirm_are_you_sure_panel(world: &mut World, this: Entity) {
    let are_you_sure_type = world.obj::<IdolScreenLogic>(this).idol_screen_area_you_sure_type;
    match are_you_sure_type {
        2 => {
            let idol_manager = idol_manager::instance(world);
            if let Some(idol_manager) = idol_manager {
                idol_manager::set_idol_levels(world, idol_manager, 20);
            }
        }
        1 => {
            let idol_manager = idol_manager::instance(world);
            if let Some(idol_manager) = idol_manager {
                idol_manager::unlock_idol_challenges(world, idol_manager);
            }
        }
        0 => {
            let listener = screen_listener(world, this);
            idol_manager::reset_all_challenges(world, listener);
            update_hints_used_text(world, this);
        }
        _ => {}
    }

    hide_are_you_sure_panel(world, this);
}

/// The detail panel's back button: back to the idol list, with every listed icon's texture and
/// halo refreshed.
fn press_detail_panel_back_button(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 1;

    for episode in 0..EPISODE_LIST_LEN {
        let idol_icon = idol_manager::episode_table(world).nodes[episode].idol_icon_logic;
        let Some(idol_icon) = idol_icon else {
            continue;
        };
        idol_icon_logic::update_idol_icon_texture(world, idol_icon);

        let idol_icon = idol_manager::episode_table(world).nodes[episode].idol_icon_logic;
        let active_node = world.obj::<IdolScreenLogic>(this).active_struct_xml_episode_node_def;
        let is_active = active_node == Some(episode);
        if let Some(idol_icon) = idol_icon {
            idol_icon_logic::update_halo(world, idol_icon, is_active);
        }
    }

    screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolDetailList"));
}

/// The detail panel's challenge button: makes the shown episode the active one and closes the
/// screen.
fn press_activate_episode_button(world: &mut World, this: Entity) {
    let detail_node = {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.active_struct_xml_episode_node_def = screen_logic.struct_xml_episode_node_def;
        screen_logic.struct_xml_episode_node_def
    };

    let listener = screen_listener(world, this);
    idol_manager::on_episode_activated(world, listener, detail_node);
    close_screen(world, this);
}

/// The summary panel's back button: back to the battle selection, the battle-enemy or idol
/// selection panel (restoring the first selected idol's texture), or the detail panel.
fn press_summary_back_button(world: &mut World, this: Entity) {
    world.obj_mut::<IdolScreenLogic>(this).summary_open = false;

    let root_idol_screen_mode = world.obj::<IdolScreenLogic>(this).root_idol_screen_mode;
    if root_idol_screen_mode == 6 {
        open_idol_battle_selection_panel(world, this);
        return;
    }

    let idol_screen_mode = world.obj::<IdolScreenLogic>(this).idol_screen_mode;
    let panel_behavior = match idol_screen_mode {
        5 => "OpenIdolBattleEnemyPanel",
        4 => "OpenIdolSelectionPanel",
        _ => "OpenIdolDetailPanel",
    };
    if idol_screen_mode == 5 || idol_screen_mode == 4 {
        let first_selected_icon = world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array[0];
        idol_selection_icon_logic::update_idol_icon_texture(world, first_selected_icon);
    }

    screen::set_screen_layer_behavior(world, this, 1, Some(panel_behavior));
}

/// The detail header's hint button: activates the hint of the first challenge not yet
/// completed (shown in the hint panel), then shows the hint count and the hint panel.
fn press_show_hint_button(world: &mut World, this: Entity) {
    let detail_node = world.obj::<IdolScreenLogic>(this).struct_xml_episode_node_def;
    let detail_node = required_node(detail_node);
    let challenges = episode_node(world, detail_node).challenges;

    for challenge in challenges {
        let completed = idol_manager::episode_table(world).challenges[challenge].completed;
        if completed {
            continue;
        }

        let listener = screen_listener(world, this);
        idol_manager::on_hint_activated(world, listener, Some(challenge));
        set_dynamic_text(world, "IdolDetailHintHeader", Some("Hint:"));

        let hint_text = dynamic_text(world, "IdolDetailHintText");
        let hint = &idol_manager::episode_table(world).challenges[challenge].def().hint;
        set_text(world, hint_text, Some(hint));
        if let Some(hint_text) = hint_text {
            text::set_wrap_width(world, hint_text, 380.0);
            text::set_line_count(world, hint_text, 5);
        }
        break;
    }

    update_hints_used_text(world, this);

    let hint_panel = world.obj::<IdolScreenLogic>(this).idol_detail_header_hint_panel_logic;
    set_logic_behavior(world, hint_panel, "IdolDetailHintShow");

    let hint_panel_buttons = world.obj::<IdolScreenLogic>(this).idol_detail_hint_panel_buttons_display_group;
    move_to_screen_display_group(world, this, hint_panel_buttons);
}

/// The battle selection's attack button: starts the turn with the highlighted idol; in a
/// multiplayer match a refused turn leaves the idol selected and the button waiting.
fn press_attack_button(world: &mut World, this: Entity) {
    let battle_node = world.obj::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def;
    if battle_node.is_none() {
        return;
    }

    let listener = screen_listener(world, this);
    let multiplayer_active = idol_manager::is_multiplayer_active(world, listener);
    if !multiplayer_active {
        world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 6;
        screen::set_screen_layer_behavior(world, this, 1, Some("CloseIdolBattleSelectionPanel"));

        let battle_node = world.obj::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def;
        idol_manager::on_battle_idol_selected(world, listener, battle_node);
        return;
    }

    let turn_taken = idol_manager::on_battle_idol_selected_multiplayer(world, listener, battle_node);
    if turn_taken {
        world.obj_mut::<IdolScreenLogic>(this).idol_screen_mode = 6;
        screen::set_screen_layer_behavior(world, this, 1, Some("CloseIdolBattleSelectionPanel"));
        return;
    }

    let attack_button = world.obj::<IdolScreenLogic>(this).idol_battle_selection_attack_button_logic;
    if let Some(attack_button) = attack_button {
        let attack_button_display_object = logic::display_object(world, attack_button);
        display::object_mut(world, attack_button_display_object).is_selectable = false;
    }

    let attack_button_skin = library::logic(world, logic_table::IDOL_BATTLE_SELECTION_ATTACK_BUTTON_SKIN);
    logic::set_behavior(world, attack_button_skin, Some("IdolBattleSelectionButtonWaiting"));
    set_dynamic_text(world, "PlayerBattlePanelText", Some(""));

    let selected_icons = world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array.clone();
    for selected_icon in selected_icons {
        let icon_node = idol_selection_icon_logic::struct_xml_episode_node_def(world, selected_icon);
        let battle_node = world.obj::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def;
        if icon_node == battle_node {
            idol_selection_icon_logic::select_for_battle(world, selected_icon);
        }
    }
}

/// The results panel's retry buttons: the same enemy again.
fn press_retry_battle_button(world: &mut World, this: Entity) {
    close_screen_now(world, this);

    let listener = screen_listener(world, this);
    let enemy_episode_idol = idol_manager::enemy_episode_idol(world, listener);
    idol_manager::on_battle_enemy_selected(world, listener, enemy_episode_idol);
}

/// The results panel's next button: the next enemy, or the same one when there is none.
fn press_next_battle_button(world: &mut World, this: Entity) {
    close_screen_now(world, this);

    let listener = screen_listener(world, this);
    let enemy_episode_idol = idol_manager::enemy_episode_idol(world, listener);
    let next_battle_idol = idol_manager::next_battle_idol(world, listener, enemy_episode_idol);
    let selected_enemy = if next_battle_idol.is_some() { next_battle_idol } else { enemy_episode_idol };
    idol_manager::on_battle_enemy_selected(world, listener, selected_enemy);
}

/// The idol list's back button: back to the selection panel, or to the start panel.
fn press_list_back_button(world: &mut World, this: Entity) {
    let root_idol_screen_mode = world.obj::<IdolScreenLogic>(this).root_idol_screen_mode;
    if root_idol_screen_mode == 4 {
        open_idol_selection_panel(world, this);
        return;
    }

    open_idol_start_panel(world, this);
    screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolStartPanel"));
}

/// `-[IdolScreenLogic onButtonPress:touch:]` @ 0x100196380
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let screen_logic = world.obj::<IdolScreenLogic>(this);
    let Some(pressed) = pressed_button(screen_logic, button) else {
        return;
    };

    match pressed {
        IdolScreenButton::Close => press_close_button(world, this),
        IdolScreenButton::OpenWhatsNewPanel => {
            screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolWhatsNewPanel"));
        }
        IdolScreenButton::OpenOptionsPanel => {
            hide_are_you_sure_panel(world, this);
            screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolOptionsPanel"));
        }
        IdolScreenButton::OpenActiveEpisodeList => {
            let listener = screen_listener(world, this);
            let active_episode = idol_manager::active_episode_node_def(world, listener);
            open_idol_list_panel(world, this, active_episode);
        }
        IdolScreenButton::CloseScreen => close_screen(world, this),
        IdolScreenButton::LaunchStoreScreen => {
            close_screen(world, this);
            let listener = screen_listener(world, this);
            idol_manager::on_idol_screen_launch_store_screen(world, listener);
        }
        IdolScreenButton::OpenStartPanel => {
            screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolStartPanel"));
        }
        IdolScreenButton::StartScreenCheckBox => {
            let check_box = world.obj::<IdolScreenLogic>(this).idol_start_check_box_logic;
            toggle_option_check_box(world, this, "ShowIdolStartScreen", check_box, false);
        }
        IdolScreenButton::EpisodeIconCheckBox => {
            let check_box = world.obj::<IdolScreenLogic>(this).idol_icon_check_box_logic;
            toggle_option_check_box(world, this, "ShowEpisodeIcon", check_box, true);
        }
        IdolScreenButton::ResetChallenges => {
            show_are_you_sure_panel(world, this, 0, "Pressing OK will reset all your challenge and battle data.");
        }
        IdolScreenButton::UnlockChallenges => {
            show_are_you_sure_panel(world, this, 1, "Pressing OK will unlock all your challenges.");
        }
        IdolScreenButton::LevelUpIdols => {
            show_are_you_sure_panel(world, this, 2, "Pressing OK will level up all your idols to level 20.");
        }
        IdolScreenButton::DetailPanelBack => press_detail_panel_back_button(world, this),
        IdolScreenButton::ActivateEpisode => press_activate_episode_button(world, this),
        IdolScreenButton::OpenSummary => {
            let detail_node = world.obj::<IdolScreenLogic>(this).struct_xml_episode_node_def;
            let detail_node = required_node(detail_node);
            let idol_earned = episode_node(world, detail_node).complete;
            if idol_earned {
                open_idol_summary_panel(world, this);
            }
        }
        IdolScreenButton::SummaryBack => press_summary_back_button(world, this),
        IdolScreenButton::GoToGodIsland => {
            close_screen(world, this);
            let listener = screen_listener(world, this);
            idol_manager::on_idol_award_screen_change_island_location_state(world, listener, 7);
        }
        IdolScreenButton::OpenChallenges => {
            close_screen_now(world, this);
            let listener = screen_listener(world, this);
            idol_manager::on_idol_award_screen_open_challenges(world, listener);
        }
        IdolScreenButton::ShowHint => press_show_hint_button(world, this),
        IdolScreenButton::CloseHintPanel => {
            let hint_panel = world.obj::<IdolScreenLogic>(this).idol_detail_header_hint_panel_logic;
            set_logic_behavior(world, hint_panel, "IdolDetailHintHide");
            let hint_panel_buttons = world.obj::<IdolScreenLogic>(this).idol_detail_hint_panel_buttons_display_group;
            remove_display_group(world, hint_panel_buttons);
        }
        IdolScreenButton::CloseAreYouSurePanel => hide_are_you_sure_panel(world, this),
        IdolScreenButton::ConfirmAreYouSure => confirm_are_you_sure_panel(world, this),
        IdolScreenButton::ExitBattle => {
            close_screen_now(world, this);
            quit_temple_door_game(world);
        }
        IdolScreenButton::Attack => press_attack_button(world, this),
        IdolScreenButton::OpenBattleEnemyPanel => open_idol_battle_enemy_panel(world, this),
        IdolScreenButton::RetryBattle => press_retry_battle_button(world, this),
        IdolScreenButton::OpenMatchPanel => {
            let listener = screen_listener(world, this);
            idol_manager::open_match_panel(world, listener);
        }
        IdolScreenButton::NextBattle => press_next_battle_button(world, this),
        IdolScreenButton::OpenBattleHelp => {
            world.obj_mut::<IdolScreenLogic>(this).battle_help_open = true;
            screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolBattleHelpPanel"));
        }
        IdolScreenButton::CloseBattleHelp => {
            world.obj_mut::<IdolScreenLogic>(this).battle_help_open = false;
            screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolBattleEnemyPanel"));
        }
        IdolScreenButton::BattleHelpCheckBox => {
            let check_box = world.obj::<IdolScreenLogic>(this).idol_battle_help_check_box_logic;
            toggle_option_check_box(world, this, "ShowBattleHelp", check_box, false);
        }
        IdolScreenButton::ListBack => press_list_back_button(world, this),
    }
}

/// `-[IdolScreenLogic onButtonRelease:touch:]` @ 0x1001977d0
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[IdolScreenLogic onButtonReleaseOutside:touch:]` @ 0x1001977d4
pub fn on_button_release_outside(world: &mut World, this: Entity, button: Entity, touch: Entity) {
    on_button_release(world, this, button, touch);
}

/// `-[IdolScreenLogic highlightIdolBattleResults:]` @ 0x1001977e4
///
/// The battle selection's details panel for a tapped selection icon: its idol's stats, how
/// long its realm power stays active and its power description; the attack button for the
/// player's own unfrozen idols; and the icons' highlights.
pub fn highlight_idol_battle_results(world: &mut World, this: Entity, highlight_idol_battle_results: Entity) {
    let tapped_icon = highlight_idol_battle_results;
    let is_enemy_idol = idol_selection_icon_logic::is_enemy_idol(world, tapped_icon);
    screen::set_screen_layer_behavior(world, this, 1, Some("OpenIdolBattleSelectionDetailsPanel"));

    let icon_node = idol_selection_icon_logic::struct_xml_episode_node_def(world, tapped_icon);
    let node = required_node(icon_node);
    let node_def = episode_def(world, node);
    set_dynamic_text(world, "IdolBattleDetailsIdolName", Some(&node_def.idol_name));

    let idol_level = episode_node(world, node).level;
    let level = format!("Level: {}", idol_level.wrapping_add(1));
    set_dynamic_text(world, "IdolBattleLevel", Some(&level));

    let level_factor = idol_level.wrapping_add(1) as f32;
    let health_value = (node_def.health_per_level * level_factor) as i32;
    let health = format!("Health: {health_value}");
    set_dynamic_text(world, "IdolBattleHealth", Some(&health));

    let damage_value = (node_def.damage_per_level * level_factor) as i32;
    let damage = format!("Damage: {damage_value}");
    set_dynamic_text(world, "IdolBattleDamage", Some(&damage));

    let active_text = dynamic_text(world, "IdolBattleActive");
    let listener = screen_listener(world, this);
    let side = if is_enemy_idol { BattleSide::Enemy } else { BattleSide::Player };
    let idol_battle_states = battle_state(world, listener, side).idol_battle_state_array;
    let mut idol_battle_state = None;
    for state in idol_battle_states {
        if state.struct_xml_episode_node_def == Some(node) {
            idol_battle_state = Some(state);
            break;
        }
    }

    let mut frozen = false;
    let active = match idol_battle_state {
        Some(state) => {
            frozen = 0 < state.frozen_ref_count;
            match state.active_frame_count {
                1 => format!("Active: {} Turn", 1),
                0 => String::from("Not Active"),
                _ if node_def.power_turn_count == 100 => String::from("Active: Entire Battle"),
                active_frame_count => format!("Active: {active_frame_count} Turns"),
            }
        }
        None => String::from("Not Active"),
    };
    set_text(world, active_text, Some(&active));

    let details_text = dynamic_text(world, "IdolBattleDetails");
    let power_description = listener_fill_power_description_text(world, listener, &node_def.power_description, Some(node));
    set_text(world, details_text, power_description.as_deref());
    if let Some(details_text) = details_text {
        text::set_wrap_width(world, details_text, 200.0);
    }

    let attack_button_skin = library::logic(world, logic_table::IDOL_BATTLE_SELECTION_ATTACK_BUTTON_SKIN);
    let attack_button_skin_display_object = logic::display_object(world, attack_button_skin);
    let attack_button = world.obj::<IdolScreenLogic>(this).idol_battle_selection_attack_button_logic;
    let attack_button_display_object = match attack_button {
        Some(attack_button) => Some(logic::display_object(world, attack_button)),
        None => None,
    };

    let battle_node = idol_selection_icon_logic::struct_xml_episode_node_def(world, tapped_icon);
    world.obj_mut::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def = battle_node;

    let multiplayer_active = idol_manager::is_multiplayer_active(world, listener);
    let can_attack_now = !multiplayer_active || idol_manager::is_it_my_turn(world, listener);
    if !can_attack_now {
        logic::set_behavior(world, attack_button_skin, Some("IdolBattleSelectionButtonWaiting"));
        set_display_object_selectable(world, attack_button_display_object, false);
        display::object_mut(world, attack_button_skin_display_object).hidden = false;
    } else if is_enemy_idol || frozen {
        display::object_mut(world, attack_button_skin_display_object).hidden = true;
        set_display_object_selectable(world, attack_button_display_object, false);
        world.obj_mut::<IdolScreenLogic>(this).battle_struct_xml_episode_node_def = None;
    } else {
        display::object_mut(world, attack_button_skin_display_object).hidden = false;
        logic::set_behavior(world, attack_button_skin, Some("IdolBattleSelectionButtonAttack"));
        set_display_object_selectable(world, attack_button_display_object, true);
    }

    // The tapped side's icons highlight the tapped one; the other side's show how they fare
    // against its attack.
    let selected_icons = world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array.clone();
    for selected_icon in selected_icons {
        if is_enemy_idol {
            let attack_node = idol_selection_icon_logic::struct_xml_episode_node_def(world, tapped_icon);
            idol_selection_icon_logic::highlight_based_on_attack_episode_node_def(world, selected_icon, attack_node);
        } else if selected_icon == tapped_icon {
            idol_selection_icon_logic::highlight_for_battle(world, selected_icon);
        } else {
            idol_selection_icon_logic::unhighlight_for_battle(world, selected_icon);
        }
    }

    let enemy_icons = world.obj::<IdolScreenLogic>(this).idol_selection_icon_logic_array.clone();
    for enemy_icon in enemy_icons {
        if !is_enemy_idol {
            let attack_node = idol_selection_icon_logic::struct_xml_episode_node_def(world, tapped_icon);
            idol_selection_icon_logic::highlight_based_on_attack_episode_node_def(world, enemy_icon, attack_node);
        } else if enemy_icon == tapped_icon {
            idol_selection_icon_logic::highlight_for_battle(world, enemy_icon);
        } else {
            idol_selection_icon_logic::unhighlight_for_battle(world, enemy_icon);
        }
    }
}

/// `[displayObject setIsSelectable:selectable]` on a display object that may be nil.
fn set_display_object_selectable(world: &mut World, display_object: Option<Entity>, selectable: bool) {
    if let Some(display_object) = display_object {
        display::object_mut(world, display_object).is_selectable = selectable;
    }
}

/// `-[IdolScreenLogic onIdolSelectionIconSelect:episodeNodeDef:]` @ 0x100197f54
pub fn on_idol_selection_icon_select(world: &mut World, this: Entity, idol_selection_icon_select: Entity, episode_node_def: Option<usize>) {
    let selection_icon = idol_selection_icon_select;
    let root_idol_screen_mode = world.obj::<IdolScreenLogic>(this).root_idol_screen_mode;
    if root_idol_screen_mode == 5 || root_idol_screen_mode == 8 {
        let is_enemy_idol = idol_selection_icon_logic::is_enemy_idol(world, selection_icon);
        if is_enemy_idol {
            let node = required_node(episode_node_def);
            let battle_unlocked = episode_node(world, node).battle_unlocked;
            if !battle_unlocked {
                return;
            }

            close_screen_now(world, this);
            let listener = screen_listener(world, this);
            idol_manager::on_battle_enemy_selected(world, listener, episode_node_def);
            return;
        }
    }

    let root_idol_screen_mode = world.obj::<IdolScreenLogic>(this).root_idol_screen_mode;
    if root_idol_screen_mode == 6 {
        highlight_idol_battle_results(world, this, selection_icon);
        return;
    }

    let Some(node) = episode_node_def else {
        return;
    };
    world.obj_mut::<IdolScreenLogic>(this).struct_xml_episode_node_def = episode_node_def;

    let idol_texture = &episode_def(world, node).idol_texture;
    activate_skin_texture_bits_def_array(world, EPISODE_IDOL_SLOT_BITS[0], idol_texture);
    open_idol_summary_panel(world, this);
}

/// `-[IdolScreenLogic onIdolIconSelect:]` @ 0x100198068
pub fn on_idol_icon_select(world: &mut World, this: Entity, idol_icon_select: Option<usize>) {
    open_idol_detail_panel(world, this, idol_icon_select);
}

/// `-[IdolScreenLogic onIdolBattleRealmEffectSelect:episodeNodeDef:]` @ 0x100198078
pub fn on_idol_battle_realm_effect_select(_world: &mut World, _this: Entity, _idol_battle_realm_effect_select: Entity, _episode_node_def: Option<usize>) {}

/// `[displayGroup addLogicsToArray:mTouchLogicArray]` (refilled from its start), then
/// `[displayGroup setDisplayGroup:[mDisplayObject displayGroup]]` so the hit tests see the group
/// where the screen draws it. Returns the logics, frontmost last.
fn collect_touch_logics(world: &mut World, this: Entity, display_group: Entity) -> Vec<Entity> {
    let mut touch_logics = Vec::new();
    group::add_logics_to_array(world, display_group, &mut touch_logics);
    world.obj_mut::<IdolScreenLogic>(this).touch_logic_array = touch_logics.clone();

    let screen_group = screen_display_group(world, this);
    display::object_mut(world, display_group).display_group = screen_group;
    touch_logics
}

/// `[logic doesPointIntersect:[touch pos] checkSelect:NO]`.
fn does_touch_intersect_logic(world: &World, touch_logic: Entity, touch: Entity) -> bool {
    let touch_pos = world.obj::<BcTouch>(touch).pos;
    let touch_logic_display_object = logic::display_object(world, touch_logic);
    display::does_point_intersect(world, touch_logic_display_object, touch_pos, false)
}

/// `-[IdolScreenLogic checkScrollDisplayGroupTouch:touch:allTouches:]` @ 0x10019807c
///
/// A tap on a scrolled panel: every logic under the finger, frontmost first, gets
/// `touchBeganInside:` (the engine's touch slots take no `allTouches`).
pub fn check_scroll_display_group_touch(world: &mut World, this: Entity, check_scroll_display_group_touch: Entity, touch: Entity, _all_touches: &[Entity]) {
    let display_group = check_scroll_display_group_touch;
    let touch_logics = collect_touch_logics(world, this, display_group);

    for touch_logic in touch_logics.into_iter().rev() {
        let inside = does_touch_intersect_logic(world, touch_logic, touch);
        if inside {
            let class = isa(world, touch_logic);
            (class.touch_began_inside)(world, touch_logic, touch);
        }
    }

    display::object_mut(world, display_group).display_group = None;
}

/// `-[IdolScreenLogic getIntersectingIdolSelectionIconLogic:touch:]` @ 0x1001981a8
pub fn get_intersecting_idol_selection_icon_logic(world: &mut World, this: Entity, intersecting_idol_selection_icon_logic: Entity, touch: Entity) -> Option<Entity> {
    let display_group = intersecting_idol_selection_icon_logic;
    let touch_logics = collect_touch_logics(world, this, display_group);

    let mut intersecting_logic = None;
    for touch_logic in touch_logics.into_iter().rev() {
        let active = send::is_active(world, Some(touch_logic));
        if !active {
            continue;
        }

        let inside = does_touch_intersect_logic(world, touch_logic, touch);
        if inside {
            intersecting_logic = Some(touch_logic);
            break;
        }
    }

    display::object_mut(world, display_group).display_group = None;
    intersecting_logic
}

/// `-[IdolScreenLogic getIntersectingSlot:]` @ 0x1001982ec
///
/// The god-idol slot (0–3) of the selection panel under the touch, −1 for none.
pub fn get_intersecting_slot(world: &mut World, this: Entity, intersecting_slot: Entity) -> i32 {
    let touch_pos = touch_pos_in_screen_group(world, this, intersecting_slot);
    let x_pos = touch_pos.x;
    let y_pos = touch_pos.y;

    if y_pos < 164.0 || y_pos > 264.0 {
        return -1;
    }

    if x_pos > 28.0 && x_pos < 128.0 {
        0
    } else if x_pos > 134.0 && x_pos < 234.0 {
        1
    } else if x_pos > 240.0 && x_pos < 340.0 {
        2
    } else if x_pos > 347.0 && x_pos < 447.0 {
        3
    } else {
        -1
    }
}

/// `-[IdolScreenLogic touchBeganInside:allTouches:]` @ 0x100198420
pub fn touch_began_inside(world: &mut World, this: Entity, touch: Entity) {
    let screen_logic = world.obj::<IdolScreenLogic>(this);
    let idol_screen_mode = screen_logic.idol_screen_mode;
    let summary_open = screen_logic.summary_open;
    let battle_help_open = screen_logic.battle_help_open;

    // The battle selection panel's icons are tapped, not dragged.
    if idol_screen_mode == 7 {
        if !summary_open && !battle_help_open {
            let library_display_group = world.obj::<IdolScreenLogic>(this).idol_selection_library_display_group;
            let library_display_group = required_display_group(library_display_group);
            let all_touches = all_touches(world);
            check_scroll_display_group_touch(world, this, library_display_group, touch, &all_touches);
        }
        return;
    }

    // The list, detail, selection and battle-enemy panels scroll with the finger.
    let scrolled_mode = matches!(idol_screen_mode, 1 | 2 | 4 | 5);
    let active_touch = world.obj::<IdolScreenLogic>(this).active_touch;
    if active_touch.is_some() || !scrolled_mode || summary_open || battle_help_open {
        return;
    }

    let last_display_group_touch_pos = touch_pos_in_screen_group(world, this, touch);
    {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.active_touch = Some(touch);
        screen_logic.last_display_group_touch_pos = last_display_group_touch_pos;
        screen_logic.drag_y_pos_vel = 0.0;
        screen_logic.drag_x_pos_vel = 0.0;
        screen_logic.drag_panel = false;
        screen_logic.disable_library_scroll = false;
        screen_logic.drag_delta_y_pos = 0.0;
    }

    let idol_screen_mode = world.obj::<IdolScreenLogic>(this).idol_screen_mode;
    if idol_screen_mode != 4 {
        return;
    }

    let library_display_group = world.obj::<IdolScreenLogic>(this).idol_selection_library_display_group;
    let library_display_group = required_display_group(library_display_group);
    let dragged_icon = get_intersecting_idol_selection_icon_logic(world, this, library_display_group, touch);
    let Some(dragged_icon) = dragged_icon else {
        return;
    };

    world.obj_mut::<IdolScreenLogic>(this).dragged_idol_selection_icon_logic = Some(dragged_icon);
    let dragged_icon_display_object = logic::display_object(world, dragged_icon);
    let library_display_group = world.obj::<IdolScreenLogic>(this).idol_selection_library_display_group;
    display::move_to_display_group(world, dragged_icon_display_object, library_display_group);
}

/// `-[IdolScreenLogic touchBeganOutside:allTouches:]` @ 0x10019862c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolScreenLogic touchMovedEntered:allTouches:]` @ 0x100198630
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolScreenLogic touchMovedExited:allTouches:]` @ 0x100198634
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolScreenLogic touchMovedInside:allTouches:]` @ 0x100198638
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    touch_moved_outside(world, this, touch);
}

/// The selection panel's drag. Until a dragged icon has moved more than 30 points vertically,
/// the library scrolls with the finger (unless the icon is a god-idol slot icon) and a dragged
/// library icon follows the finger vertically only; from then on the dragged icon follows it
/// freely and the library stays put.
fn drag_selection_panel(world: &mut World, this: Entity, touch_pos: Vec4) {
    let last_touch_pos = world.obj::<IdolScreenLogic>(this).last_display_group_touch_pos;
    let delta_y_pos = touch_pos.y - last_touch_pos.y;
    let dragged_icon = {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.drag_delta_y_pos = delta_y_pos + screen_logic.drag_delta_y_pos;
        let dragged_far = screen_logic.drag_delta_y_pos.abs() > 30.0;
        if screen_logic.dragged_idol_selection_icon_logic.is_some() && dragged_far {
            screen_logic.disable_library_scroll = true;
            screen_logic.drag_x_pos_vel = 0.0;
        }
        screen_logic.dragged_idol_selection_icon_logic
    };

    let mut delta_x_pos = touch_pos.x - last_touch_pos.x;
    let disable_library_scroll = world.obj::<IdolScreenLogic>(this).disable_library_scroll;
    if !disable_library_scroll {
        let drags_slot_icon = match dragged_icon {
            Some(dragged_icon) => idol_selection_icon_logic::idol_selection_icon_type(world, dragged_icon) == 1,
            None => false,
        };
        if !drags_slot_icon {
            let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
            screen_logic.drag_x_pos_vel = screen_logic.drag_x_pos_vel + (delta_x_pos - screen_logic.drag_x_pos_vel) * 0.5;
            screen_logic.idol_selection_icon_scroll_x_pos = delta_x_pos + screen_logic.idol_selection_icon_scroll_x_pos;
        }
    }

    let Some(dragged_icon) = dragged_icon else {
        return;
    };

    let mut dragged_icon = dragged_icon;
    if !disable_library_scroll {
        let icon_type = idol_selection_icon_logic::idol_selection_icon_type(world, dragged_icon);
        if icon_type != 1 {
            delta_x_pos = 0.0;
        }

        let current_dragged_icon = world.obj::<IdolScreenLogic>(this).dragged_idol_selection_icon_logic;
        let Some(current_dragged_icon) = current_dragged_icon else {
            return;
        };
        dragged_icon = current_dragged_icon;
    }
    idol_selection_icon_logic::drag_x_pos(world, dragged_icon, delta_x_pos, delta_y_pos);
}

/// `-[IdolScreenLogic touchMovedOutside:allTouches:]` @ 0x100198648
pub fn touch_moved_outside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<IdolScreenLogic>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    let touch_pos = touch_pos_in_screen_group(world, this, touch);
    let screen_logic = world.obj::<IdolScreenLogic>(this);
    if !screen_logic.drag_panel {
        // Android reports small movements during taps. Keep the initial position until a
        // drag starts so several small moves still accumulate into a deliberate scroll.
        let touch_delta = touch_pos - screen_logic.last_display_group_touch_pos;
        let distance_squared = touch_delta.x * touch_delta.x + touch_delta.y * touch_delta.y;
        if distance_squared <= PANEL_DRAG_THRESHOLD * PANEL_DRAG_THRESHOLD {
            return;
        }
    }

    let idol_screen_mode = screen_logic.idol_screen_mode;
    match idol_screen_mode {
        5 => {
            let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
            let delta_x_pos = touch_pos.x - screen_logic.last_display_group_touch_pos.x;
            screen_logic.drag_x_pos_vel = screen_logic.drag_x_pos_vel + (delta_x_pos - screen_logic.drag_x_pos_vel) * 0.5;
            screen_logic.idol_selection_icon_scroll_x_pos = delta_x_pos + screen_logic.idol_selection_icon_scroll_x_pos;
        }
        4 => drag_selection_panel(world, this, touch_pos),
        _ => {
            let screen_logic = world.obj::<IdolScreenLogic>(this);
            let display_group = if idol_screen_mode == 2 {
                screen_logic.idol_icon_detail_display_group
            } else {
                screen_logic.idol_icon_list_display_group
            };
            let display_group = required_display_group(display_group);

            let delta_y_pos = touch_pos.y - screen_logic.last_display_group_touch_pos.y;
            let drag_y_pos_vel = screen_logic.drag_y_pos_vel + (delta_y_pos - screen_logic.drag_y_pos_vel) * 0.5;
            world.obj_mut::<IdolScreenLogic>(this).drag_y_pos_vel = drag_y_pos_vel;

            let mut display_group_object = display::object_mut(world, display_group);
            display_group_object.pos.y = display_group_object.pos.y + delta_y_pos;
        }
    }

    let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
    screen_logic.last_display_group_touch_pos = touch_pos;
    screen_logic.drag_panel = true;
}

/// `[mSelectedIdolSelectionIconLogicArray objectAtIndex:slot]` (an index outside the four
/// god-idol slots raises `NSRangeException`).
fn selected_icon_in_slot(world: &World, this: Entity, slot: i32) -> Entity {
    let slot_index = usize::try_from(slot).expect("objectAtIndex: with a negative slot raises NSRangeException");
    world.obj::<IdolScreenLogic>(this).selected_idol_selection_icon_logic_array[slot_index]
}

/// Drops the dragged selection icon: a library icon dropped on a god-idol slot takes it (the
/// icon of the slot its idol held before is cleared); a slot icon moves to another slot or, off
/// the slots, is deselected. Then the player's level and health are shown again.
fn drop_dragged_icon(world: &mut World, this: Entity, dragged_icon: Entity, touch: Entity) {
    idol_selection_icon_logic::stop_drag(world, dragged_icon);
    let slot = get_intersecting_slot(world, this, touch);
    let icon_node = idol_selection_icon_logic::struct_xml_episode_node_def(world, dragged_icon);
    let icon_type = idol_selection_icon_logic::idol_selection_icon_type(world, dragged_icon);
    let listener = screen_listener(world, this);

    if icon_type == 0 {
        if slot != -1 {
            let node = required_node(icon_node);
            let previous_slot = episode_node(world, node).selection_slot;
            if previous_slot != -1 {
                let previous_slot_icon = selected_icon_in_slot(world, this, previous_slot);
                idol_selection_icon_logic::deactivate(world, previous_slot_icon);
            }

            idol_manager::on_episode_idol_selected(world, listener, icon_node, slot);
            let slot_icon = selected_icon_in_slot(world, this, slot);
            idol_selection_icon_logic::activate(world, slot_icon, icon_node);
        }
    } else {
        let node = required_node(icon_node);
        let previous_slot = episode_node(world, node).selection_slot;
        if slot != previous_slot {
            if slot == -1 {
                idol_manager::on_episode_idol_deselected(world, listener, icon_node);
            } else {
                idol_manager::on_episode_idol_selected(world, listener, icon_node, slot);
                let slot_icon = selected_icon_in_slot(world, this, slot);
                idol_selection_icon_logic::activate(world, slot_icon, icon_node);
            }
            idol_selection_icon_logic::deactivate(world, dragged_icon);
        }
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

    world.obj_mut::<IdolScreenLogic>(this).dragged_idol_selection_icon_logic = None;
}

/// `-[IdolScreenLogic touchEndedInside:allTouches:]` @ 0x100198924
pub fn touch_ended_inside(world: &mut World, this: Entity, touch: Entity) {
    let active_touch = world.obj::<IdolScreenLogic>(this).active_touch;
    if active_touch != Some(touch) {
        return;
    }

    // A touch that did not drag is a tap on the panel's contents.
    let tapped_panel = {
        let mut screen_logic = world.obj_mut::<IdolScreenLogic>(this);
        screen_logic.active_touch = None;
        let tapped = !screen_logic.summary_open && !screen_logic.battle_help_open && !screen_logic.drag_panel;
        match screen_logic.idol_screen_mode {
            _ if !tapped => None,
            4 | 5 => Some(screen_logic.idol_selection_library_display_group),
            2 => Some(screen_logic.idol_icon_detail_display_group),
            1 => Some(screen_logic.idol_icon_list_display_group),
            _ => None,
        }
    };
    if let Some(tapped_panel) = tapped_panel {
        let tapped_panel = required_display_group(tapped_panel);
        let all_touches = all_touches(world);
        check_scroll_display_group_touch(world, this, tapped_panel, touch, &all_touches);
    }

    let dragged_icon = world.obj::<IdolScreenLogic>(this).dragged_idol_selection_icon_logic;
    if let Some(dragged_icon) = dragged_icon {
        drop_dragged_icon(world, this, dragged_icon, touch);
    }
}

/// `-[IdolScreenLogic touchEndedOutside:allTouches:]` @ 0x100198cb8
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    touch_ended_inside(world, this, touch);
}
