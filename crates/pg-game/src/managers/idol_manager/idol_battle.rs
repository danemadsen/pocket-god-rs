//! `IdolManager`: methods in this part of the class.
//!
//! The Battle of the Gods battle engine (`docs/re/managers-platform.md` §4.3–§4.4). A
//! `PlayerBattleState *` travels as [`PLAYER_BATTLE_STATE`] or [`ENEMY_BATTLE_STATE`], an
//! `IdolBattleState *` as [`idol_screen_logic::idol_battle_state_pointer`], and a
//! `BCStructXmlEpisodeNodeDef *` as an index into [`EpisodeTable::nodes`].

use pg_assets::data::episodes::RealmPower;

use crate::logic::battle::temple_door_logic;
use crate::logic::idol_screen::idol_screen_logic::{self, BattleSide};
use crate::prelude::*;

use super::*;

/// Idols per side (`PlayerBattleState.mIdolBattleStateArray[4]`).
const BATTLE_IDOL_COUNT: usize = 4;

/// `DAT_100729760`: the episode whose defeat also triggers `DefeatDeva`.
const EPISODE_40: &str = "Episode40";

/// `0x10029d46c`: battle experience by enemy level.
const BATTLE_EXPERIENCE: [f32; 20] = [
    500.0, 1000.0, 1500.0, 1900.0, 2300.0, 2600.0, 2900.0, 3100.0, 3300.0, 3400.0, 3500.0, 3600.0, 3700.0, 3800.0,
    3900.0, 4000.0, 4100.0, 4200.0, 4300.0, 4400.0,
];

/// `0x10029d35c`: the AI's weight of an idol, shared among the idols of its attack type.
const IDOL_CHOICE_SCORE: f32 = 100.0;

/// `0x10029d35c` again: `fillPowerDescriptionText:structXmlEpisodeNodeDef:` shows these powers'
/// values as percentages.
const PERCENT_FACTOR: f32 = 100.0;

/// `0x10029d468`: the AI's weight of an idol it throws the round with.
const THROW_ROUND_CHOICE_SCORE: f32 = 30000.0;

/// `0x10029d2e8`: the AI's strength factor of `IgnoreAttackDefense`.
const IGNORE_ATTACK_DEFENSE_STRENGTH: f32 = 0.4;

/// `0x10029d404`: the most a realm effect's damage is reduced by total defense.
const MAX_REALM_EFFECT_DEFENSE: f32 = 0.9;

/// `triggerEpisodeEvent:` ids of the battle.
const WIN_BATTLE_EVENT: &str = "WinBattle";
const DEFEAT_DEVA_EVENT: &str = "DefeatDeva";
const LOSE_BATTLE_EVENT: &str = "LoseBattle";
const DRAIN_HEALTH_EVENT: &str = "DrainHealth";
const REFLECT_DAMAGE_EVENT: &str = "ReflectDamage";
const TIE_DAMAGE_EVENT: &str = "TieDamage";
const RESURRECT_AFTER_DEATH_EVENT: &str = "ResurrectAfterDeath";
const FREEZE_IDOL_EVENT: &str = "FreezeIdol";
const REALM_DAMAGE_EVENT: &str = "RealmDamage";
const REALM_HEALING_EVENT: &str = "RealmHealing";
const BOOST_TOTAL_DAMAGE_EVENT: &str = "BoostTotalDamage";
const BOOST_ATTACK_DAMAGE_EVENT: &str = "BoostAttackDamage";
const ACTIVATE_ATTACK_DEFENSE_EVENT: &str = "ActivateAttackDefense";
const ACTIVATE_TOTAL_DEFENSE_EVENT: &str = "ActivateTotalDefense";
const DRAIN_LEVEL_POINTS_EVENT: &str = "DrainLevelPoints";
const POWER_UP_ALL_IDOLS_EVENT: &str = "PowerUpAllIdols";
const CANCEL_REALM_EFFECTS_EVENT: &str = "CancelRealmEffects";
const UNFREEZE_IDOLS_EVENT: &str = "UnfreezeIdols";

// ---- Pointers ------------------------------------------------------------------------------------

/// A pointer the original dereferences without a nil check (a crash there, a panic here).
fn required(pointer: Option<usize>) -> usize {
    pointer.expect("IdolManager: NULL pointer (the original dereferences it)")
}

/// The side a `PlayerBattleState *` points at, where the original dereferences it.
fn battle_side(battle_state: Option<usize>) -> BattleSide {
    let pointer = required(battle_state);
    match pointer {
        PLAYER_BATTLE_STATE => BattleSide::Player,
        ENEMY_BATTLE_STATE => BattleSide::Enemy,
        _ => panic!("IdolManager: {pointer} is not a PlayerBattleState pointer"),
    }
}

/// `battleState == &mPlayerBattleState`.
fn is_player_battle_state(battle_state: Option<usize>) -> bool {
    battle_state == Some(PLAYER_BATTLE_STATE)
}

/// `mPlayerBattleState` or `mEnemyBattleState`.
fn manager_side_state(manager: &IdolManager, side: BattleSide) -> &PlayerBattleState {
    match side {
        BattleSide::Player => &manager.player_battle_state,
        BattleSide::Enemy => &manager.enemy_battle_state,
    }
}

/// [`manager_side_state`], for writing.
fn manager_side_state_mut(manager: &mut IdolManager, side: BattleSide) -> &mut PlayerBattleState {
    match side {
        BattleSide::Player => &mut manager.player_battle_state,
        BattleSide::Enemy => &mut manager.enemy_battle_state,
    }
}

/// `*battleState`.
fn side_state(world: &World, this: Entity, side: BattleSide) -> &PlayerBattleState {
    let manager = world.obj::<IdolManager>(this);
    manager_side_state(manager, side)
}

/// `*battleState`, for writing.
fn side_state_mut(world: &mut World, this: Entity, side: BattleSide) -> Mut<'_, PlayerBattleState> {
    let manager = world.obj_mut::<IdolManager>(this);
    manager.map_unchanged(|manager| manager_side_state_mut(manager, side))
}

/// `battleState->mIdolBattleStateArray[slot]` (a copy).
fn idol_state(world: &World, this: Entity, side: BattleSide, slot: usize) -> IdolBattleState {
    side_state(world, this, side).idol_battle_state_array[slot]
}

/// `battleState->mIdolBattleStateArray[slot]`, for writing.
fn idol_state_mut(world: &mut World, this: Entity, side: BattleSide, slot: usize) -> Mut<'_, IdolBattleState> {
    let state = side_state_mut(world, this, side);
    state.map_unchanged(|state| &mut state.idol_battle_state_array[slot])
}

// ---- Episode nodes -------------------------------------------------------------------------------

/// The id (+0x28) of an episode node. A battle copy that was never filled is all zero in the
/// original, so its id is nil; every list episode has challenges, so only such a copy has none.
fn episode_node_id(world: &mut World, node: usize) -> Option<String> {
    let table = episode_table(world);
    let never_filled = table.nodes[node].challenges.is_empty();
    if never_filled {
        return None;
    }

    let def = table.def(node);
    Some(def.id.clone())
}

/// The enemy line-up of episode node `episode` (+0x08): for each of its enemy idols whose episode
/// id is in the list, while fewer than four have matched, that list episode and the enemy idol's
/// level.
fn enemy_line_up(world: &mut World, episode: usize) -> Vec<(usize, i32)> {
    let enemy_idols = &episode_table(world).def(episode).enemy_idols;

    let mut line_up = Vec::with_capacity(BATTLE_IDOL_COUNT);
    for enemy_idol in enemy_idols {
        if line_up.len() > 3 {
            continue;
        }
        let list_episode = episode_index(&enemy_idol.episode_id);
        if let Some(list_episode) = list_episode {
            line_up.push((list_episode, enemy_idol.level));
        }
    }
    line_up
}

/// `memcpy(copy, source, sizeof(BCStructXmlEpisodeNodeDef))`, then the copy's level and selection
/// slot.
fn copy_battle_episode(world: &mut World, source: usize, copy: usize, level: i32, slot: i32) {
    let mut table = episode_table_mut(world);
    let mut node = table.nodes[source].clone();
    node.level = level;
    node.selection_slot = slot;
    table.nodes[copy] = node;
}

// ---- Messages ------------------------------------------------------------------------------------

/// `[mIdolManagerListener challengePreferences]`.
fn listener_challenge_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = world.obj::<IdolManager>(this).idol_manager_listener;
    let listener = listener?;
    crate::controller::challenge_preferences(world, listener)
}

/// `[prefs setPreference:key asInt:value]`. A nil document ignores it; so does a nil key, which
/// only exists before `loadEpisodeState` (the original's dictionary would raise).
fn set_int_preference(world: &mut World, prefs: Option<Entity>, key: Option<&str>, value: i32) {
    let (Some(prefs), Some(key)) = (prefs, key) else {
        return;
    };
    pg_engine::prefs::set_preference_as_int(world, prefs, key, value);
}

/// `[[IdolManager instance] triggerEpisodeEvent:event]`.
fn trigger_battle_event(world: &mut World, event: &str) {
    let idol_manager = instance(world);
    if let Some(idol_manager) = idol_manager {
        trigger_episode_event(world, idol_manager, event);
    }
}

/// `mIdolScreenLogic`.
fn screen_logic(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<IdolManager>(this).idol_screen_logic
}

/// `[mIdolScreenLogic updateIdolBattleScreen]`.
fn update_battle_screen(world: &mut World, this: Entity) {
    let screen = screen_logic(world, this);
    if let Some(screen) = screen {
        idol_screen_logic::update_idol_battle_screen(world, screen);
    }
}

/// `x < 0 ? 0 : x`: a damage factor never goes below zero.
fn non_negative(value: f32) -> f32 {
    if value < 0.0 { 0.0 } else { value }
}

// ---- Accessors -----------------------------------------------------------------------------------

/// `-[IdolManager playerBattleState]` @ 0x10002c830
pub fn player_battle_state(_world: &mut World, _this: Entity) -> Option<usize> {
    Some(PLAYER_BATTLE_STATE)
}

/// `-[IdolManager enemyBattleState]` @ 0x10002c840
pub fn enemy_battle_state(_world: &mut World, _this: Entity) -> Option<usize> {
    Some(ENEMY_BATTLE_STATE)
}

/// `-[IdolManager playerName]` @ 0x10002c850
pub fn player_name(world: &mut World, this: Entity) -> Option<String> {
    world.obj::<IdolManager>(this).player_battle_state.name.clone()
}

/// `-[IdolManager enemyName]` @ 0x10002c860
pub fn enemy_name(world: &mut World, this: Entity) -> Option<String> {
    world.obj::<IdolManager>(this).enemy_battle_state.name.clone()
}

/// `-[IdolManager playerHealth]` @ 0x10002c870
pub fn player_health(world: &mut World, this: Entity) -> f32 {
    world.obj::<IdolManager>(this).player_battle_state.health
}

/// `-[IdolManager enemyHealth]` @ 0x10002c884
pub fn enemy_health(world: &mut World, this: Entity) -> f32 {
    world.obj::<IdolManager>(this).enemy_battle_state.health
}

/// `-[IdolManager playerMaxHealth]` @ 0x10002c898
pub fn player_max_health(world: &mut World, this: Entity) -> f32 {
    world.obj::<IdolManager>(this).player_battle_state.max_health
}

/// `-[IdolManager enemyMaxHealth]` @ 0x10002c8ac
pub fn enemy_max_health(world: &mut World, this: Entity) -> f32 {
    world.obj::<IdolManager>(this).enemy_battle_state.max_health
}

/// `-[IdolManager activeBattlePlayerEpisodeIdol]` @ 0x10002c8c0
pub fn active_battle_player_episode_idol(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<IdolManager>(this).player_battle_state.battle_struct_xml_episode_node_def
}

/// `-[IdolManager activeBattleEnemyEpisodeIdol]` @ 0x10002c8d4
pub fn active_battle_enemy_episode_idol(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<IdolManager>(this).enemy_battle_state.battle_struct_xml_episode_node_def
}

/// `-[IdolManager enemyEpisodeIdol]` @ 0x10002c8e8
pub fn enemy_episode_idol(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<IdolManager>(this).enemy_struct_xml_episode_node_def
}

/// `-[IdolManager unlockNextBattleIdol:]` @ 0x10002c8f8
///
/// Unlocks the list episode after `unlock_next_battle_idol` (the first one for nil).
pub fn unlock_next_battle_idol(world: &mut World, this: Entity, unlock_next_battle_idol: Option<usize>) {
    let challenge_preferences = listener_challenge_preferences(world, this);

    let mut previous = None;
    for episode in 0..EPISODE_LIST_LEN {
        if previous == unlock_next_battle_idol {
            let battle_unlocked = episode_table(world).nodes[episode].battle_unlocked;
            if battle_unlocked {
                return;
            }

            let id = &episode_table(world).def(episode).id;
            let key = format!("{id}BattleUnlock");
            set_int_preference(world, challenge_preferences, Some(key.as_str()), 1);
            episode_table_mut(world).nodes[episode].battle_unlocked = true;
            return;
        }
        previous = Some(episode);
    }
}

// ---- Names ---------------------------------------------------------------------------------------

/// `-[IdolManager attackName:]` @ 0x10002c9fc
pub fn attack_name(_world: &mut World, _this: Entity, attack_name: i32) -> Option<&'static str> {
    match attack_name {
        0 => Some("Strong"),
        1 => Some("Counter"),
        2 => Some("Fast"),
        _ => None,
    }
}

/// `-[IdolManager realmName:]` @ 0x10002ca30
pub fn realm_name(_world: &mut World, _this: Entity, realm_name: i32) -> Option<&'static str> {
    match realm_name {
        0 => Some("Oceanus"),
        1 => Some("Aeris"),
        2 => Some("Infernis"),
        3 => Some("Animus"),
        4 => Some("Pax"),
        _ => None,
    }
}

/// `-[IdolManager attackResultName:]` @ 0x10002caa8
pub fn attack_result_name(_world: &mut World, _this: Entity, attack_result_name: i32) -> Option<&'static str> {
    match attack_result_name {
        0 => Some("Attack Enemy Won"),
        1 => Some("Attack Player Won"),
        2 => Some("Attack Tie"),
        _ => None,
    }
}

/// `-[IdolManager realmResultName:]` @ 0x10002cadc
pub fn realm_result_name(_world: &mut World, _this: Entity, realm_result_name: i32) -> Option<&'static str> {
    match realm_result_name {
        0 => Some("Realm Enemy Won"),
        1 => Some("Realm Player Won"),
        2 => Some("Realm Tie"),
        _ => None,
    }
}

/// `-[IdolManager realmPowerName:]` @ 0x10002cb10
pub fn realm_power_name(_world: &mut World, _this: Entity, realm_power_name: i32) -> Option<&'static str> {
    match realm_power_name {
        0 => Some("Realm Damage"),
        1 => Some("Tie Damage"),
        2 => Some("Total Boost Damage"),
        3 => Some("Attack Boost Damage"),
        4 => Some("Attack Reflect Damage"),
        5 => Some("Attack Drain Health"),
        6 => Some("Attack Defense"),
        7 => Some("Total Defense"),
        8 => Some("Ignore Attack Defense"),
        9 => Some("Realm Heal"),
        10 => Some("Freeze Idol"),
        11 => Some("Resurrect After Death"),
        12 => Some("Power up all Idols"),
        13 => Some("Drain Level Points"),
        14 => Some("Cancel Enemy Powerups"),
        15 => Some("Unfreeze Idols"),
        _ => None,
    }
}

/// `-[IdolManager attackResultFromPlayerIdol:enemyIdol:]` @ 0x10002cc34
///
/// 0 enemy won, 1 player won, 2 tie: strong beats counter, counter beats fast, fast beats strong.
pub fn attack_result_from_player_idol(world: &mut World, _this: Entity, attack_result_from_player_idol: Option<usize>, enemy_idol: Option<usize>) -> i32 {
    let player_node = required(attack_result_from_player_idol);
    let player_attack = episode_table(world).def(player_node).attack as i32;
    let enemy_node = required(enemy_idol);
    let enemy_attack = episode_table(world).def(enemy_node).attack as i32;

    if player_attack == enemy_attack {
        return 2;
    }
    if (player_attack == 0 && enemy_attack == 1) || (player_attack == 2 && enemy_attack == 0) {
        return 1;
    }
    let player_won = player_attack == 1 && enemy_attack == 2;
    player_won as i32
}

/// `-[IdolManager realmResultFromPlayerIdol:enemyIdol:]` @ 0x10002cc84
///
/// 0 enemy won, 1 player won, 2 tie. The range tests are unsigned compares.
pub fn realm_result_from_player_idol(world: &mut World, _this: Entity, realm_result_from_player_idol: Option<usize>, enemy_idol: Option<usize>) -> i32 {
    let player_node = required(realm_result_from_player_idol);
    let player_realm = episode_table(world).def(player_node).realm as i32;
    let enemy_node = required(enemy_idol);
    let enemy_realm = episode_table(world).def(enemy_node).realm as i32;

    if player_realm == enemy_realm {
        return 2;
    }
    let enemy_realm_bits = enemy_realm as u32;
    if player_realm == 0 && enemy_realm_bits.wrapping_sub(1) < 2 {
        return 1;
    }

    let player_won = match player_realm {
        1 => enemy_realm == 2 || enemy_realm == 4,
        2 => enemy_realm_bits.wrapping_sub(3) < 2,
        3 => enemy_realm_bits < 2,
        4 => enemy_realm == 0 || enemy_realm == 3,
        _ => false,
    };
    player_won as i32
}

/// The text `fillPowerDescriptionText:structXmlEpisodeNodeDef:` shows for a realm power's value.
fn realm_power_value_text(power: RealmPower, value: f32) -> String {
    match power {
        RealmPower::TotalBoostDamage
        | RealmPower::AttackBoostDamage
        | RealmPower::AttackReflectDamage
        | RealmPower::AttackDrainHealth
        | RealmPower::AttackDefense
        | RealmPower::TotalDefense => {
            let percent = value * PERCENT_FACTOR;
            format!("{:.0}%", f64::from(percent))
        }
        RealmPower::IgnoreAttackDefense
        | RealmPower::FreezeIdol
        | RealmPower::ResurrectAfterDeath
        | RealmPower::PowerUpAllIdols
        | RealmPower::CancelEnemyPowerups
        | RealmPower::UnfreezeIdols => String::new(),
        RealmPower::RealmDamage | RealmPower::TieDamage | RealmPower::RealmHeal | RealmPower::DrainLevelPoints => {
            format!("{:.1}", f64::from(value))
        }
    }
}

/// `-[IdolManager fillPowerDescriptionText:structXmlEpisodeNodeDef:]` @ 0x10002cd20
///
/// The first realm power's value replaces `[PrimaryValue]`, the last later one
/// `[SecondaryValue]`, and the power's duration `[TurnCount]`.
pub fn fill_power_description_text(world: &mut World, _this: Entity, fill_power_description_text: &str, struct_xml_episode_node_def: Option<usize>) -> Option<String> {
    let node = required(struct_xml_episode_node_def);
    let def = episode_table(world).def(node);

    let mut primary_value: Option<String> = None;
    let mut secondary_value: Option<String> = None;
    for realm_power in &def.realm_powers {
        let level = episode_table(world).nodes[node].level;
        let value = realm_power.base_value + realm_power.value_per_level * level as f32;
        let value_text = realm_power_value_text(realm_power.power, value);

        if primary_value.is_none() {
            primary_value = Some(value_text);
        } else {
            secondary_value = Some(value_text);
        }
    }

    let power_turn_count = def.power_turn_count;
    let turn_count_text = match power_turn_count {
        1 => "1 turn".to_owned(),
        100 => "entire battle".to_owned(),
        _ => format!("{power_turn_count} turns"),
    };

    let mut description = fill_power_description_text.replace("[TurnCount]", &turn_count_text);
    if let Some(primary_value) = primary_value {
        description = description.replace("[PrimaryValue]", &primary_value);
    }
    if let Some(secondary_value) = secondary_value {
        description = description.replace("[SecondaryValue]", &secondary_value);
    }
    Some(description)
}

// ---- Realm powers --------------------------------------------------------------------------------

/// `-[IdolManager accumulateRealmPowers:opponentBattleState:]` @ 0x10002cf20
///
/// Sums the realm powers of the side's active idols into its battle state.
pub fn accumulate_realm_powers(world: &mut World, this: Entity, accumulate_realm_powers: Option<usize>, _opponent_battle_state: Option<usize>) {
    let side = battle_side(accumulate_realm_powers);
    let is_player = is_player_battle_state(accumulate_realm_powers);

    {
        let mut state = side_state_mut(world, this, side);
        state.attack_defense = 0.0;
        state.realm_health = 0.0;
        state.tie_damage = 0.0;
        state.total_defense = 0.0;
        state.attack_drain_health = 0.0;
        state.realm_damage = 0.0;
        state.attack_boost_damage = 0.0;
        state.attack_reflect_damage = 0.0;
        state.actual_drain_health = 0.0;
        state.total_boost_damage = 0.0;
        state.actual_attack_damage = 0.0;
        state.actual_reflect_damage = 0.0;
        state.unfreeze_idols = false;
        state.ignore_attack_defense = false;
        state.cancel_enemy_power_ups = false;
        state.drain_level_points = 0.0;
    }

    for slot in 0..BATTLE_IDOL_COUNT {
        let idol = {
            let mut idol = idol_state_mut(world, this, side, slot);
            idol.damage = 0.0;
            idol.health = 0.0;
            idol.freeze_idol = false;
            idol.resurrect = false;
            *idol
        };
        if idol.active_frame_count < 1 {
            continue;
        }

        let node = required(idol.struct_xml_episode_node_def);
        let realm_powers = &episode_table(world).def(node).realm_powers;
        for realm_power in realm_powers {
            let level = episode_table(world).nodes[node].level;
            let value = realm_power.base_value + realm_power.value_per_level * level as f32;
            let active_frame_count = idol_state(world, this, side, slot).active_frame_count;

            match realm_power.power {
                RealmPower::RealmDamage => {
                    side_state_mut(world, this, side).realm_damage += value;
                    idol_state_mut(world, this, side, slot).damage += value;
                }
                RealmPower::TieDamage => {
                    side_state_mut(world, this, side).tie_damage += value;
                }
                RealmPower::TotalBoostDamage => {
                    side_state_mut(world, this, side).total_boost_damage += value;
                    if is_player {
                        trigger_battle_event(world, BOOST_TOTAL_DAMAGE_EVENT);
                    }
                }
                RealmPower::AttackBoostDamage => {
                    side_state_mut(world, this, side).attack_boost_damage += value;
                    if is_player {
                        trigger_battle_event(world, BOOST_ATTACK_DAMAGE_EVENT);
                    }
                }
                RealmPower::AttackReflectDamage => {
                    side_state_mut(world, this, side).attack_reflect_damage += value;
                }
                RealmPower::AttackDrainHealth => {
                    side_state_mut(world, this, side).attack_drain_health += value;
                }
                RealmPower::AttackDefense => {
                    side_state_mut(world, this, side).attack_defense += value;
                    if is_player {
                        trigger_battle_event(world, ACTIVATE_ATTACK_DEFENSE_EVENT);
                    }
                }
                RealmPower::TotalDefense => {
                    side_state_mut(world, this, side).total_defense += value;
                    if is_player {
                        trigger_battle_event(world, ACTIVATE_TOTAL_DEFENSE_EVENT);
                    }
                }
                RealmPower::IgnoreAttackDefense => {
                    side_state_mut(world, this, side).ignore_attack_defense = true;
                }
                RealmPower::RealmHeal => {
                    side_state_mut(world, this, side).realm_health += value;
                    idol_state_mut(world, this, side, slot).health += value;
                }
                RealmPower::FreezeIdol => {
                    if active_frame_count > 1 {
                        idol_state_mut(world, this, side, slot).freeze_idol = true;
                    }
                }
                RealmPower::ResurrectAfterDeath => {
                    let mut idol = idol_state_mut(world, this, side, slot);
                    idol.resurrect = true;
                    idol.resurrect_health_per_level = value;
                }
                RealmPower::PowerUpAllIdols => {}
                RealmPower::DrainLevelPoints => {
                    {
                        let mut state = side_state_mut(world, this, side);
                        state.drain_level_points += value;
                        state.total_drain_level_points += value;
                    }
                    if is_player {
                        trigger_battle_event(world, DRAIN_LEVEL_POINTS_EVENT);
                    }
                }
                RealmPower::CancelEnemyPowerups => {
                    if active_frame_count > 1 {
                        side_state_mut(world, this, side).cancel_enemy_power_ups = true;
                    }
                }
                RealmPower::UnfreezeIdols => {
                    if active_frame_count > 1 {
                        side_state_mut(world, this, side).unfreeze_idols = true;
                    }
                }
            }
        }
    }
}

/// `-[IdolManager completeTurnForRealmPowers:]` @ 0x10002d284
pub fn complete_turn_for_realm_powers(world: &mut World, this: Entity, complete_turn_for_realm_powers: Option<usize>) {
    let side = battle_side(complete_turn_for_realm_powers);
    let mut state = side_state_mut(world, this, side);
    for idol in state.idol_battle_state_array.iter_mut() {
        if idol.active_frame_count > 0 {
            idol.active_frame_count -= 1;
        }
    }
}

/// `-[IdolManager activateRealmEffectInBattleState:idolBattleState:opponentBattleState:]` @ 0x10002d2ac
///
/// (Re)starts the idol's realm effect for its turn count; a newly active effect also applies its
/// one-shot powers.
pub fn activate_realm_effect_in_battle_state_idol_battle_state_opponent_battle_state(world: &mut World, this: Entity, activate_realm_effect_in_battle_state: Option<usize>, idol_battle_state: Option<usize>, opponent_battle_state: Option<usize>) {
    let idol_pointer = required(idol_battle_state);
    let (idol_side, idol_slot) = idol_screen_logic::idol_battle_state_location(idol_pointer);
    let idol = idol_state(world, this, idol_side, idol_slot);
    let previous_frame_count = idol.active_frame_count;

    let node = required(idol.struct_xml_episode_node_def);
    let def = episode_table(world).def(node);
    idol_state_mut(world, this, idol_side, idol_slot).active_frame_count = def.power_turn_count;
    if previous_frame_count > 0 {
        return;
    }

    let is_player = is_player_battle_state(activate_realm_effect_in_battle_state);
    for realm_power in &def.realm_powers {
        match realm_power.power {
            RealmPower::FreezeIdol => {
                let opponent_side = battle_side(opponent_battle_state);
                let opponent_unfreezes = side_state(world, this, opponent_side).unfreeze_idols;
                if opponent_unfreezes {
                    idol_state_mut(world, this, idol_side, idol_slot).active_frame_count = 0;
                }
            }
            RealmPower::PowerUpAllIdols => {
                activate_all_realm_effects_in_battle_state(world, this, activate_realm_effect_in_battle_state, opponent_battle_state);
                if is_player {
                    trigger_battle_event(world, POWER_UP_ALL_IDOLS_EVENT);
                }
            }
            RealmPower::CancelEnemyPowerups => {
                let side = battle_side(activate_realm_effect_in_battle_state);
                side_state_mut(world, this, side).cancel_enemy_power_ups = true;
                cancel_all_realm_effects_in_battle_state(world, this, opponent_battle_state);
                if is_player {
                    trigger_battle_event(world, CANCEL_REALM_EFFECTS_EVENT);
                }
            }
            RealmPower::UnfreezeIdols => {
                let side = battle_side(activate_realm_effect_in_battle_state);
                side_state_mut(world, this, side).unfreeze_idols = true;
                unfreeze_idols_in_battle_state(world, this, activate_realm_effect_in_battle_state, opponent_battle_state);
                if is_player {
                    trigger_battle_event(world, UNFREEZE_IDOLS_EVENT);
                }
            }
            _ => {}
        }
    }
}

/// `-[IdolManager cancelAllRealmEffectsInBattleState:]` @ 0x10002d488
pub fn cancel_all_realm_effects_in_battle_state(world: &mut World, this: Entity, cancel_all_realm_effects_in_battle_state: Option<usize>) {
    let side = battle_side(cancel_all_realm_effects_in_battle_state);
    let mut state = side_state_mut(world, this, side);
    for idol in state.idol_battle_state_array.iter_mut() {
        idol.active_frame_count = 0;
    }
}

/// `-[IdolManager unfreezeIdolsInBattleState:opponentBattleState:]` @ 0x10002d4a4
///
/// The opponent's freezing idols stop; the side's idols are no longer frozen.
pub fn unfreeze_idols_in_battle_state(world: &mut World, this: Entity, unfreeze_idols_in_battle_state: Option<usize>, opponent_battle_state: Option<usize>) {
    let opponent_side = battle_side(opponent_battle_state);
    {
        let mut opponent_state = side_state_mut(world, this, opponent_side);
        for idol in opponent_state.idol_battle_state_array.iter_mut() {
            if idol.freeze_idol {
                idol.active_frame_count = 0;
                idol.freeze_idol = false;
                idol.frozen_idol_battle_state = None;
            }
        }
    }

    let side = battle_side(unfreeze_idols_in_battle_state);
    let mut state = side_state_mut(world, this, side);
    for idol in state.idol_battle_state_array.iter_mut() {
        if idol.frozen_ref_count > 0 {
            idol.frozen_ref_count = 0;
        }
    }
}

/// `-[IdolManager activateAllRealmEffectsInBattleState:opponentBattleState:]` @ 0x10002d4f4
pub fn activate_all_realm_effects_in_battle_state(world: &mut World, this: Entity, activate_all_realm_effects_in_battle_state: Option<usize>, opponent_battle_state: Option<usize>) {
    let side = battle_side(activate_all_realm_effects_in_battle_state);
    for slot in 0..BATTLE_IDOL_COUNT {
        let idol_pointer = idol_screen_logic::idol_battle_state_pointer(side, slot);
        activate_realm_effect_in_battle_state_idol_battle_state_opponent_battle_state(world, this, activate_all_realm_effects_in_battle_state, Some(idol_pointer), opponent_battle_state);
    }
}

/// `-[IdolManager activateRealmEffectInBattleState:opponentBattleState:]` @ 0x10002d55c
///
/// Activates the realm effect of the side's battle idol.
pub fn activate_realm_effect_in_battle_state_opponent_battle_state(world: &mut World, this: Entity, activate_realm_effect_in_battle_state: Option<usize>, opponent_battle_state: Option<usize>) {
    let side = battle_side(activate_realm_effect_in_battle_state);
    for slot in 0..BATTLE_IDOL_COUNT {
        let state = side_state(world, this, side);
        let battle_node = state.battle_struct_xml_episode_node_def;
        let idol_node = state.idol_battle_state_array[slot].struct_xml_episode_node_def;
        if battle_node == idol_node {
            let idol_pointer = idol_screen_logic::idol_battle_state_pointer(side, slot);
            activate_realm_effect_in_battle_state_idol_battle_state_opponent_battle_state(world, this, activate_realm_effect_in_battle_state, Some(idol_pointer), opponent_battle_state);
            return;
        }
    }
}

/// `-[IdolManager calculateBattleStateResults]` @ 0x10002d5a0
///
/// Compares the two battle idols, counts their use, activates the realm winner's effect and sums
/// both sides' realm powers.
pub fn calculate_battle_state_results(world: &mut World, this: Entity) {
    let player_battle_node = side_state(world, this, BattleSide::Player).battle_struct_xml_episode_node_def;
    let enemy_battle_node = side_state(world, this, BattleSide::Enemy).battle_struct_xml_episode_node_def;
    let attack_result = attack_result_from_player_idol(world, this, player_battle_node, enemy_battle_node);
    world.obj_mut::<IdolManager>(this).attack_result = attack_result;
    let realm_result = realm_result_from_player_idol(world, this, player_battle_node, enemy_battle_node);
    world.obj_mut::<IdolManager>(this).realm_result = realm_result;

    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.player_battle_state.win_attack = false;
        manager.enemy_battle_state.win_attack = false;
        if attack_result == 1 {
            manager.player_battle_state.win_attack = true;
        } else if attack_result == 0 {
            manager.enemy_battle_state.win_attack = true;
        }

        for slot in 0..BATTLE_IDOL_COUNT {
            let enemy_state = &mut manager.enemy_battle_state;
            let enemy_battle_node = enemy_state.battle_struct_xml_episode_node_def;
            let enemy_idol = &mut enemy_state.idol_battle_state_array[slot];
            if enemy_battle_node == enemy_idol.struct_xml_episode_node_def {
                enemy_idol.used_count += 1;
            }

            let player_state = &mut manager.player_battle_state;
            let player_battle_node = player_state.battle_struct_xml_episode_node_def;
            let player_idol = &mut player_state.idol_battle_state_array[slot];
            if player_battle_node == player_idol.struct_xml_episode_node_def {
                player_idol.used_count += 1;
            }
        }

        manager.player_battle_state.win_realm = false;
        manager.enemy_battle_state.win_realm = false;
    }

    if realm_result == 1 {
        side_state_mut(world, this, BattleSide::Player).win_realm = true;
        let cancelled = side_state(world, this, BattleSide::Enemy).cancel_enemy_power_ups;
        if !cancelled {
            activate_realm_effect_in_battle_state_opponent_battle_state(world, this, Some(PLAYER_BATTLE_STATE), Some(ENEMY_BATTLE_STATE));
        }
    } else if realm_result == 0 {
        side_state_mut(world, this, BattleSide::Enemy).win_realm = true;
        let cancelled = side_state(world, this, BattleSide::Player).cancel_enemy_power_ups;
        if !cancelled {
            activate_realm_effect_in_battle_state_opponent_battle_state(world, this, Some(ENEMY_BATTLE_STATE), Some(PLAYER_BATTLE_STATE));
        }
    }

    accumulate_realm_powers(world, this, Some(PLAYER_BATTLE_STATE), Some(ENEMY_BATTLE_STATE));
    accumulate_realm_powers(world, this, Some(ENEMY_BATTLE_STATE), Some(PLAYER_BATTLE_STATE));
}

// ---- AI ------------------------------------------------------------------------------------------

/// `-[IdolManager aiEvaluateIdolStrength:idolBattleState:opponentBattleState:]` @ 0x10002d768
///
/// The idol's potential strength: its attack plus a weight of each of its realm powers.
pub fn ai_evaluate_idol_strength_idol_battle_state_opponent_battle_state(world: &mut World, this: Entity, ai_evaluate_idol_strength: Option<usize>, idol_battle_state: Option<usize>, opponent_battle_state: Option<usize>) {
    let idol_pointer = required(idol_battle_state);
    let (idol_side, idol_slot) = idol_screen_logic::idol_battle_state_location(idol_pointer);
    let node = idol_state(world, this, idol_side, idol_slot).struct_xml_episode_node_def;
    let node = required(node);
    let level = episode_table(world).nodes[node].level;
    let def = episode_table(world).def(node);

    let level_factor = (level + 1) as f32;
    let attack_strength = def.damage_per_level * level_factor;
    {
        let mut idol = idol_state_mut(world, this, idol_side, idol_slot);
        idol.potential_attack_strength = attack_strength;
        idol.potential_realm_strength = 0.0;
        idol.potential_strength = 0.0;
    }

    let mut realm_strength = 0.0;
    if !def.realm_powers.is_empty() {
        let side = battle_side(ai_evaluate_idol_strength);
        let opponent_side = battle_side(opponent_battle_state);
        let state = side_state(world, this, side);
        let average_damage = state.average_damage;
        let battle_level = state.level as f32;
        let opponent_state = side_state(world, this, opponent_side);
        let opponent_average_damage = opponent_state.average_damage;
        let opponent_has_freeze_idol = opponent_state.has_freeze_idol;

        let turn_count = def.power_turn_count as f32;
        let idol_level = level as f32;
        let ignore_defense_strength = turn_count * (average_damage * IGNORE_ATTACK_DEFENSE_STRENGTH);
        let turn_level_strength = turn_count * 10.0 * battle_level;
        let level_strength = battle_level * 10.0;

        for realm_power in &def.realm_powers {
            let value = realm_power.base_value + idol_level * realm_power.value_per_level;
            let power_strength = match realm_power.power {
                RealmPower::RealmDamage | RealmPower::RealmHeal => turn_count * value,
                RealmPower::TieDamage => (turn_count * value) / 3.0,
                RealmPower::TotalBoostDamage | RealmPower::AttackBoostDamage | RealmPower::AttackDrainHealth => {
                    turn_count * (average_damage * value)
                }
                RealmPower::AttackReflectDamage | RealmPower::AttackDefense | RealmPower::TotalDefense => {
                    turn_count * (opponent_average_damage * value)
                }
                RealmPower::IgnoreAttackDefense => ignore_defense_strength,
                RealmPower::FreezeIdol | RealmPower::CancelEnemyPowerups => turn_level_strength,
                RealmPower::ResurrectAfterDeath | RealmPower::PowerUpAllIdols => level_strength,
                RealmPower::DrainLevelPoints => 0.0,
                RealmPower::UnfreezeIdols => {
                    if opponent_has_freeze_idol { turn_level_strength } else { 0.0 }
                }
            };
            realm_strength = power_strength + realm_strength;
        }

        idol_state_mut(world, this, idol_side, idol_slot).potential_realm_strength = realm_strength;
    }

    idol_state_mut(world, this, idol_side, idol_slot).potential_strength = attack_strength + realm_strength;
}

/// `-[IdolManager aiEvaluateIdolStrength:opponentBattleState:]` @ 0x10002d8c4
pub fn ai_evaluate_idol_strength_opponent_battle_state(world: &mut World, this: Entity, ai_evaluate_idol_strength: Option<usize>, opponent_battle_state: Option<usize>) {
    let side = battle_side(ai_evaluate_idol_strength);
    for slot in 0..BATTLE_IDOL_COUNT {
        let idol_pointer = idol_screen_logic::idol_battle_state_pointer(side, slot);
        ai_evaluate_idol_strength_idol_battle_state_opponent_battle_state(world, this, ai_evaluate_idol_strength, Some(idol_pointer), opponent_battle_state);
    }
}

/// `-[IdolManager aiSelectEnemyIdol]` @ 0x10002d92c
///
/// Picks the enemy's battle idol at random, weighting idols of rarer attack types higher. Against
/// a stronger player the AI sometimes picks an idol that loses the attack on purpose; it never
/// picks a frozen idol unless the roll lands on zero.
pub fn ai_select_enemy_idol(world: &mut World, this: Entity) {
    {
        let mut enemy_state = side_state_mut(world, this, BattleSide::Enemy);
        enemy_state.attack_type_count_array = [0; 3];
        enemy_state.realm_type_count_array = [0; 5];
    }

    for slot in 0..BATTLE_IDOL_COUNT {
        let enemy_node = {
            let mut idol = idol_state_mut(world, this, BattleSide::Enemy, slot);
            idol.enemy_attack_lose_count = 0;
            idol.enemy_attack_win_count = 0;
            idol.enemy_attack_tie_count = 0;
            idol.enemy_realm_lose_count = 0;
            idol.enemy_realm_win_count = 0;
            idol.enemy_realm_tie_count = 0;
            idol.struct_xml_episode_node_def
        };

        let enemy_def = episode_table(world).def(required(enemy_node));
        let attack = enemy_def.attack as usize;
        let realm = enemy_def.realm as usize;
        {
            let mut enemy_state = side_state_mut(world, this, BattleSide::Enemy);
            let attack_type_index = enemy_state.attack_type_count_array[attack];
            let realm_type_index = enemy_state.realm_type_count_array[realm];
            let idol = &mut enemy_state.idol_battle_state_array[slot];
            idol.attack_type_index = attack_type_index;
            idol.realm_type_index = realm_type_index;
            enemy_state.attack_type_count_array[attack] += 1;
            enemy_state.realm_type_count_array[realm] += 1;
        }

        for player_slot in 0..BATTLE_IDOL_COUNT {
            let player_node = idol_state(world, this, BattleSide::Player, player_slot).struct_xml_episode_node_def;
            let attack_result = attack_result_from_player_idol(world, this, player_node, enemy_node);
            let realm_result = realm_result_from_player_idol(world, this, player_node, enemy_node);

            let mut idol = idol_state_mut(world, this, BattleSide::Enemy, slot);
            match attack_result {
                2 => idol.enemy_attack_tie_count += 1,
                1 => idol.enemy_attack_lose_count += 1,
                0 => idol.enemy_attack_win_count += 1,
                _ => {}
            }
            match realm_result {
                2 => idol.enemy_realm_tie_count += 1,
                1 => idol.enemy_realm_lose_count += 1,
                0 => idol.enemy_realm_win_count += 1,
                _ => {}
            }
        }
    }

    ai_evaluate_idol_strength_opponent_battle_state(world, this, Some(ENEMY_BATTLE_STATE), Some(PLAYER_BATTLE_STATE));
    ai_evaluate_idol_strength_opponent_battle_state(world, this, Some(PLAYER_BATTLE_STATE), Some(ENEMY_BATTLE_STATE));

    let player_level = side_state(world, this, BattleSide::Player).level;
    let enemy_level = side_state(world, this, BattleSide::Enemy).level;
    let mut throw_chance = 0;
    if player_level <= enemy_level {
        throw_chance = 20;
    }
    let base_throw_chance = throw_chance as f32;
    if enemy_level > 4 {
        throw_chance = (base_throw_chance * 0.75) as i32;
    }
    if enemy_level > 9 {
        throw_chance = (base_throw_chance * 0.5) as i32;
    }
    if enemy_level > 14 {
        throw_chance = (base_throw_chance * 0.25) as i32;
    }
    if enemy_level > 19 {
        throw_chance = 0;
    }

    let throw_roll = world.resource_mut::<Random>().arc4random_mod(101) as i32;
    let mut total_choice_score = 0.0;
    for slot in 0..BATTLE_IDOL_COUNT {
        let enemy_node = idol_state(world, this, BattleSide::Enemy, slot).struct_xml_episode_node_def;
        let attack = episode_table(world).def(required(enemy_node)).attack as usize;
        let attack_type_count = side_state(world, this, BattleSide::Enemy).attack_type_count_array[attack];
        let mut choice_score = IDOL_CHOICE_SCORE / attack_type_count as f32;

        if throw_roll < throw_chance {
            let player_battle_node = side_state(world, this, BattleSide::Player).battle_struct_xml_episode_node_def;
            let attack_result = attack_result_from_player_idol(world, this, player_battle_node, enemy_node);
            if attack_result == 1 {
                choice_score = THROW_ROUND_CHOICE_SCORE;
            }
        }

        let frozen_ref_count = idol_state(world, this, BattleSide::Enemy, slot).frozen_ref_count;
        if frozen_ref_count > 0 {
            choice_score = 0.0;
        }
        idol_state_mut(world, this, BattleSide::Enemy, slot).choice_score = choice_score;
        total_choice_score = total_choice_score + choice_score;
    }

    let choice_roll = world.resource_mut::<Random>().arc4random();
    let choice_range = (total_choice_score * 10.0) as i32;
    let choice_divisor = choice_range.wrapping_add(1) as u32;
    let choice_remainder = if choice_divisor == 0 {
        choice_roll
    } else {
        choice_roll % choice_divisor
    };
    let choice_remainder = choice_remainder as i32;
    let mut choice = choice_remainder as f32 / 10.0;

    let mut chosen_slot = 0;
    loop {
        let choice_score = idol_state(world, this, BattleSide::Enemy, chosen_slot).choice_score;
        choice = choice - choice_score;
        let reached = choice <= 0.0;
        if reached || chosen_slot == BATTLE_IDOL_COUNT - 1 {
            break;
        }
        chosen_slot += 1;
    }

    let chosen_node = idol_state(world, this, BattleSide::Enemy, chosen_slot).struct_xml_episode_node_def;
    side_state_mut(world, this, BattleSide::Enemy).battle_struct_xml_episode_node_def = chosen_node;
}

// ---- Rounds --------------------------------------------------------------------------------------

/// `-[IdolManager onBattleIdolSelected:]` @ 0x10002dce4
pub fn on_battle_idol_selected(world: &mut World, this: Entity, battle_idol_selected: Option<usize>) {
    side_state_mut(world, this, BattleSide::Player).battle_struct_xml_episode_node_def = battle_idol_selected;
    ai_select_enemy_idol(world, this);
    calculate_battle_state_results(world, this);

    let temple_door = temple_door_logic::instance(world);
    if let Some(temple_door) = temple_door {
        temple_door_logic::play_battle_sequence(world, temple_door);
    }
}

/// `-[IdolManager onGameDeath]` @ 0x10002dd58
///
/// A double KO triggers neither `WinBattle` nor `LoseBattle`.
pub fn on_game_death(world: &mut World, this: Entity) {
    let player_health = side_state(world, this, BattleSide::Player).health;
    if player_health <= 0.0 {
        let temple_door = temple_door_logic::instance(world);
        if let Some(temple_door) = temple_door {
            temple_door_logic::on_player_fatality(world, temple_door);
        }
    }

    let enemy_health = side_state(world, this, BattleSide::Enemy).health;
    if enemy_health <= 0.0 {
        let temple_door = temple_door_logic::instance(world);
        if let Some(temple_door) = temple_door {
            temple_door_logic::on_enemy_fatality(world, temple_door);
        }
    }

    update_battle_screen(world, this);

    let player_health = side_state(world, this, BattleSide::Player).health;
    if player_health > 0.0 {
        let multi_player_active = world.obj::<IdolManager>(this).multi_player_active;
        if !multi_player_active {
            trigger_battle_event(world, WIN_BATTLE_EVENT);

            let mut enemy_episode = world.obj::<IdolManager>(this).enemy_struct_xml_episode_node_def;
            let episode_40 = episode_index(EPISODE_40);
            if enemy_episode == episode_40 {
                trigger_battle_event(world, DEFEAT_DEVA_EVENT);
                enemy_episode = world.obj::<IdolManager>(this).enemy_struct_xml_episode_node_def;
            }
            unlock_next_battle_idol(world, this, enemy_episode);
        }
    }

    let enemy_health = side_state(world, this, BattleSide::Enemy).health;
    if enemy_health > 0.0 {
        trigger_battle_event(world, LOSE_BATTLE_EVENT);
    }
}

/// `-[IdolManager onBattleTurnComplete]` @ 0x10002df1c
pub fn on_battle_turn_complete(world: &mut World, this: Entity) {
    complete_turn_for_realm_powers(world, this, Some(PLAYER_BATTLE_STATE));
    complete_turn_for_realm_powers(world, this, Some(ENEMY_BATTLE_STATE));
    update_battle_screen(world, this);
}

/// `-[IdolManager calculateBattleAttackResults]` @ 0x10002df88
///
/// Applies the round's attack: tie damage both ways, or the winner's attack with the loser's
/// reflect and the winner's drain. Returns whether either side died.
pub fn calculate_battle_attack_results(world: &mut World, this: Entity) -> bool {
    let attack_result = world.obj::<IdolManager>(this).attack_result;
    if attack_result == 2 {
        let player_state = side_state(world, this, BattleSide::Player);
        let player_tie_damage = player_state.tie_damage;
        let player_total_boost_damage = player_state.total_boost_damage;
        let enemy_total_defense = side_state(world, this, BattleSide::Enemy).total_defense;
        let enemy_damage_factor = non_negative((player_total_boost_damage + 1.0) - enemy_total_defense);
        let enemy_damage = player_tie_damage * enemy_damage_factor;
        side_state_mut(world, this, BattleSide::Enemy).health -= enemy_damage;
        if enemy_damage > 0.0 {
            trigger_battle_event(world, TIE_DAMAGE_EVENT);
        }

        let enemy_state = side_state(world, this, BattleSide::Enemy);
        let enemy_tie_damage = enemy_state.tie_damage;
        let enemy_total_boost_damage = enemy_state.total_boost_damage;
        let player_state = side_state(world, this, BattleSide::Player);
        let player_total_defense = player_state.total_defense;
        let player_damage_factor = non_negative((enemy_total_boost_damage + 1.0) - player_total_defense);
        let player_health = player_state.health - enemy_tie_damage * player_damage_factor;
        side_state_mut(world, this, BattleSide::Player).health = player_health;
        if player_health <= 0.0 {
            return true;
        }

        let enemy_health = side_state(world, this, BattleSide::Enemy).health;
        return enemy_health <= 0.0;
    }

    let (winner, loser) = if attack_result == 0 {
        (BattleSide::Enemy, BattleSide::Player)
    } else {
        (BattleSide::Player, BattleSide::Enemy)
    };

    let winner_state = side_state(world, this, winner);
    let winner_node = required(winner_state.battle_struct_xml_episode_node_def);
    let winner_boost_damage = winner_state.total_boost_damage + winner_state.attack_boost_damage;
    let winner_ignores_defense = winner_state.ignore_attack_defense;
    let winner_total_defense = winner_state.total_defense;
    let winner_attack_drain_health = winner_state.attack_drain_health;
    let loser_state = side_state(world, this, loser);
    let loser_defense = loser_state.attack_defense + loser_state.total_defense;
    let loser_total_boost_damage = loser_state.total_boost_damage;
    let loser_attack_reflect_damage = loser_state.attack_reflect_damage;

    let winner_level = episode_table(world).nodes[winner_node].level;
    let winner_damage_per_level = episode_table(world).def(winner_node).damage_per_level;
    let idol_damage = winner_damage_per_level * (winner_level + 1) as f32;
    let defense = if winner_ignores_defense { 0.0 } else { loser_defense };
    let attack_factor = non_negative((winner_boost_damage + 1.0) - defense);
    let attack_damage = idol_damage * attack_factor;
    {
        let mut loser_state = side_state_mut(world, this, loser);
        loser_state.health -= attack_damage;
        loser_state.actual_attack_damage = attack_damage;
    }

    let reflect_factor = non_negative((loser_total_boost_damage + 1.0) - winner_total_defense);
    let reflect_damage = loser_attack_reflect_damage * attack_damage * reflect_factor;
    let drain_health = attack_damage * winner_attack_drain_health;
    {
        let mut winner_state = side_state_mut(world, this, winner);
        let winner_health = (winner_state.health - reflect_damage) + drain_health;
        winner_state.actual_reflect_damage = reflect_damage;
        winner_state.health = winner_health;
        winner_state.actual_drain_health = drain_health;
        if winner_health > winner_state.max_health {
            winner_state.health = winner_state.max_health;
        }
    }

    if winner == BattleSide::Player && drain_health > 0.0 {
        trigger_battle_event(world, DRAIN_HEALTH_EVENT);
    }
    if loser == BattleSide::Player && reflect_damage > 0.0 {
        trigger_battle_event(world, REFLECT_DAMAGE_EVENT);
    }

    let winner_health = side_state(world, this, winner).health;
    if winner_health <= 0.0 {
        return true;
    }
    let loser_health = side_state(world, this, loser).health;
    loser_health <= 0.0
}

/// `-[IdolManager onBattleDamageComplete]` @ 0x10002e214
pub fn on_battle_damage_complete(world: &mut World, this: Entity) {
    let someone_died = calculate_battle_attack_results(world, this);
    if !someone_died {
        let screen = screen_logic(world, this);
        if let Some(screen) = screen {
            idol_screen_logic::show_battle_attack_results(world, screen);
        }
        update_battle_screen(world, this);
        return;
    }

    on_game_death(world, this);
    let screen = screen_logic(world, this);
    if let Some(screen) = screen {
        idol_screen_logic::show_battle_attack_results(world, screen);
    }
}

/// `-[IdolManager performResurrection:]` @ 0x10002e29c
///
/// A dead side's first active idol with the resurrect power brings it back with a share of its
/// maximum health.
pub fn perform_resurrection_with_arg(world: &mut World, this: Entity, perform_resurrection: Option<usize>) {
    let side = battle_side(perform_resurrection);
    let is_player = is_player_battle_state(perform_resurrection);

    for slot in 0..BATTLE_IDOL_COUNT {
        let resurrected = {
            let mut state = side_state_mut(world, this, side);
            let idol = state.idol_battle_state_array[slot];
            let can_resurrect = idol.active_frame_count > 0 && state.health <= 0.0 && idol.resurrect;
            if can_resurrect {
                let max_health = state.max_health;
                let mut health = idol.resurrect_health_per_level * max_health;
                if health > max_health {
                    health = max_health;
                }
                state.health = health;

                let idol = &mut state.idol_battle_state_array[slot];
                idol.resurrect = false;
                idol.active_frame_count = 0;
            }
            can_resurrect
        };

        if resurrected && is_player {
            trigger_battle_event(world, RESURRECT_AFTER_DEATH_EVENT);
        }
    }
}

/// `-[IdolManager performResurrection]` @ 0x10002e378
pub fn perform_resurrection(world: &mut World, this: Entity) {
    let enemy_resurrects = match instance(world) {
        Some(idol_manager) => should_enemy_resurrect(world, idol_manager),
        None => false,
    };
    let player_resurrects = match instance(world) {
        Some(idol_manager) => should_player_resurrect(world, idol_manager),
        None => false,
    };

    if enemy_resurrects {
        perform_resurrection_with_arg(world, this, Some(ENEMY_BATTLE_STATE));
    }
    if player_resurrects {
        perform_resurrection_with_arg(world, this, Some(PLAYER_BATTLE_STATE));
    }

    let screen = screen_logic(world, this);
    if let Some(screen) = screen {
        idol_screen_logic::show_battle_resurrection_results_for_player(world, screen, player_resurrects, enemy_resurrects);
    }
    update_battle_screen(world, this);
}

/// `-[IdolManager showBattleRealmResults]` @ 0x10002e468
pub fn show_battle_realm_results(world: &mut World, this: Entity) {
    let screen = screen_logic(world, this);
    if let Some(screen) = screen {
        idol_screen_logic::show_battle_realm_results(world, screen);
    }
    update_battle_screen(world, this);
}

/// `-[IdolManager showBattleRealmResultsOnIdol]` @ 0x10002e4b0
pub fn show_battle_realm_results_on_idol(world: &mut World, this: Entity) {
    let screen = screen_logic(world, this);
    if let Some(screen) = screen {
        idol_screen_logic::show_battle_realm_results_on_idol(world, screen);
    }
    update_battle_screen(world, this);
}

// ---- Freezing ------------------------------------------------------------------------------------

/// `-[IdolManager countFrozenIdols:]` @ 0x10002e4f8
pub fn count_frozen_idols(world: &mut World, this: Entity, count_frozen_idols: Option<usize>) -> i32 {
    let side = battle_side(count_frozen_idols);
    let state = side_state(world, this, side);

    let mut frozen_count = 0;
    for idol in &state.idol_battle_state_array {
        if idol.frozen_ref_count > 0 {
            frozen_count += 1;
        }
    }
    frozen_count
}

/// `-[IdolManager chooseFrozenIdolBattleState:useRandomFrozenIdolIds:]` @ 0x10002e520
///
/// The idol of the side that a freeze power freezes: nil when three or more are frozen; in
/// single player (or without the agreed ids) a random unfrozen one; in multiplayer the one whose
/// episode id the players agreed on.
pub fn choose_frozen_idol_battle_state(world: &mut World, this: Entity, choose_frozen_idol_battle_state: Option<usize>, use_random_frozen_idol_ids: bool) -> Option<usize> {
    let frozen_count = count_frozen_idols(world, this, choose_frozen_idol_battle_state);
    if frozen_count > 2 {
        return None;
    }

    let candidate_count = (4 - frozen_count) as u32;
    let mut remaining = world.resource_mut::<Random>().arc4random_mod(candidate_count);
    let side = battle_side(choose_frozen_idol_battle_state);
    let is_player = is_player_battle_state(choose_frozen_idol_battle_state);

    for slot in 0..BATTLE_IDOL_COUNT {
        let idol = idol_state(world, this, side, slot);
        let multi_player_active = world.obj::<IdolManager>(this).multi_player_active;

        if !multi_player_active || !use_random_frozen_idol_ids {
            if idol.frozen_ref_count == 0 {
                if remaining == 0 {
                    let idol_pointer = idol_screen_logic::idol_battle_state_pointer(side, slot);
                    return Some(idol_pointer);
                }
                remaining -= 1;
            }
            continue;
        }

        let manager = world.obj::<IdolManager>(this);
        let random_frozen_idol_id = if is_player {
            manager.player_random_frozen_idol_id.clone()
        } else {
            manager.enemy_random_frozen_idol_id.clone()
        };
        let node = required(idol.struct_xml_episode_node_def);
        let idol_id = episode_node_id(world, node);
        let agreed = random_frozen_idol_id.is_some() && random_frozen_idol_id == idol_id;
        if agreed {
            let idol_pointer = idol_screen_logic::idol_battle_state_pointer(side, slot);
            return Some(idol_pointer);
        }
    }
    None
}

/// `-[IdolManager handleIdolFreeze:opponentBattleState:]` @ 0x10002e644
///
/// An expired freeze releases its target; an active freeze without a target picks one.
pub fn handle_idol_freeze(world: &mut World, this: Entity, handle_idol_freeze: Option<usize>, opponent_battle_state: Option<usize>) {
    let side = battle_side(handle_idol_freeze);
    let is_player = is_player_battle_state(handle_idol_freeze);

    for slot in 0..BATTLE_IDOL_COUNT {
        let idol = idol_state(world, this, side, slot);
        let target = idol.frozen_idol_battle_state;

        if !idol.freeze_idol {
            if let Some(target) = target {
                let (target_side, target_slot) = idol_screen_logic::idol_battle_state_location(target);
                idol_state_mut(world, this, target_side, target_slot).frozen_ref_count -= 1;
                idol_state_mut(world, this, side, slot).frozen_idol_battle_state = None;
            }
            continue;
        }
        if target.is_some() {
            continue;
        }

        let chosen = choose_frozen_idol_battle_state(world, this, opponent_battle_state, true);
        let Some(chosen) = chosen else {
            continue;
        };
        idol_state_mut(world, this, side, slot).frozen_idol_battle_state = Some(chosen);

        let mut frozen = chosen;
        if is_player {
            trigger_battle_event(world, FREEZE_IDOL_EVENT);
            let target = idol_state(world, this, side, slot).frozen_idol_battle_state;
            frozen = required(target);
        }
        let (frozen_side, frozen_slot) = idol_screen_logic::idol_battle_state_location(frozen);
        idol_state_mut(world, this, frozen_side, frozen_slot).frozen_ref_count += 1;
    }
}

/// `-[IdolManager showBattleRealmEffectResults]` @ 0x10002e750
pub fn show_battle_realm_effect_results(world: &mut World, this: Entity) {
    handle_idol_freeze(world, this, Some(PLAYER_BATTLE_STATE), Some(ENEMY_BATTLE_STATE));
    handle_idol_freeze(world, this, Some(ENEMY_BATTLE_STATE), Some(PLAYER_BATTLE_STATE));

    let screen = screen_logic(world, this);
    if let Some(screen) = screen {
        idol_screen_logic::show_battle_realm_effect_results(world, screen);
    }
    update_battle_screen(world, this);
}

/// `-[IdolManager calculateBattleRealmEffectResult:battleState:opponentBattleState:]` @ 0x10002e7e8
///
/// One active idol's realm effect: its damage to the opponent (boosted, and reduced by the
/// opponent's total defense up to 0.9) and its heal of its own side.
pub fn calculate_battle_realm_effect_result(world: &mut World, this: Entity, calculate_battle_realm_effect_result: Option<usize>, battle_state: Option<usize>, opponent_battle_state: Option<usize>) {
    let idol_pointer = required(calculate_battle_realm_effect_result);
    let (idol_side, idol_slot) = idol_screen_logic::idol_battle_state_location(idol_pointer);
    let idol_damage = idol_state(world, this, idol_side, idol_slot).damage;

    let opponent_side = battle_side(opponent_battle_state);
    let opponent_total_defense = side_state(world, this, opponent_side).total_defense;
    let defense = if opponent_total_defense > MAX_REALM_EFFECT_DEFENSE {
        MAX_REALM_EFFECT_DEFENSE
    } else {
        opponent_total_defense
    };

    let side = battle_side(battle_state);
    let total_boost_damage = side_state(world, this, side).total_boost_damage;
    let realm_damage = idol_damage * ((total_boost_damage + 1.0) - defense);
    side_state_mut(world, this, opponent_side).health -= realm_damage;

    let is_player = is_player_battle_state(battle_state);
    if is_player && realm_damage > 0.0 {
        trigger_battle_event(world, REALM_DAMAGE_EVENT);
    }

    let realm_health = idol_state(world, this, idol_side, idol_slot).health;
    {
        let mut state = side_state_mut(world, this, side);
        state.health = realm_health + state.health;
    }
    if is_player && realm_health > 0.0 {
        trigger_battle_event(world, REALM_HEALING_EVENT);
    }
}

/// `-[IdolManager showBattleRealmEffectResult:battleState:opponentBattleState:]` @ 0x10002e8fc
pub fn show_battle_realm_effect_result(world: &mut World, this: Entity, show_battle_realm_effect_result: Option<usize>, battle_state: Option<usize>, opponent_battle_state: Option<usize>) {
    calculate_battle_realm_effect_result(world, this, show_battle_realm_effect_result, battle_state, opponent_battle_state);
    update_battle_screen(world, this);
}

/// `-[IdolManager checkBattleRealmEffectDeath]` @ 0x10002e940
pub fn check_battle_realm_effect_death(world: &mut World, this: Entity) {
    for side in [BattleSide::Player, BattleSide::Enemy] {
        let mut state = side_state_mut(world, this, side);
        if state.health > state.max_health {
            state.health = state.max_health;
        }
    }

    let player_health = side_state(world, this, BattleSide::Player).health;
    let enemy_health = side_state(world, this, BattleSide::Enemy).health;
    if player_health <= 0.0 || enemy_health <= 0.0 {
        on_game_death(world, this);
    }
}

/// `-[IdolManager areBattleRealmEffectResultsComplete]` @ 0x10002e9bc
pub fn are_battle_realm_effect_results_complete(world: &mut World, this: Entity) -> bool {
    let screen = screen_logic(world, this);
    match screen {
        Some(screen) => idol_screen_logic::are_battle_realm_effect_results_complete(world, screen),
        None => false,
    }
}

/// `-[IdolManager onBattleAttackTie]` @ 0x10002e9d8
pub fn on_battle_attack_tie(world: &mut World, this: Entity) {
    on_battle_damage_complete(world, this);
}

// ---- Outcome -------------------------------------------------------------------------------------

/// `-[IdolManager isBattleComplete]` @ 0x10002e9e8
pub fn is_battle_complete(world: &mut World, this: Entity) -> bool {
    let enemy_health = side_state(world, this, BattleSide::Enemy).health;
    if enemy_health <= 0.0 {
        return true;
    }
    let player_health = side_state(world, this, BattleSide::Player).health;
    player_health <= 0.0
}

/// `-[IdolManager didPlayerWinLastBattle]` @ 0x10002ea24
pub fn did_player_win_last_battle(world: &mut World, this: Entity) -> bool {
    let enemy_health = side_state(world, this, BattleSide::Enemy).health;
    let enemy_dead = enemy_health <= 0.0;
    if !enemy_dead {
        return false;
    }
    let player_health = side_state(world, this, BattleSide::Player).health;
    player_health > 0.0
}

/// `-[IdolManager didPlayerTieLastBattle]` @ 0x10002ea60
pub fn did_player_tie_last_battle(world: &mut World, this: Entity) -> bool {
    let enemy_health = side_state(world, this, BattleSide::Enemy).health;
    let enemy_dead = enemy_health <= 0.0;
    if !enemy_dead {
        return false;
    }
    let player_health = side_state(world, this, BattleSide::Player).health;
    player_health <= 0.0
}

/// Whether the side is dead and one of its idols holds the resurrect power.
fn should_resurrect(world: &World, this: Entity, side: BattleSide) -> bool {
    let state = side_state(world, this, side);
    if state.health > 0.0 {
        return false;
    }

    for idol in &state.idol_battle_state_array {
        if idol.resurrect {
            return true;
        }
    }
    false
}

/// `-[IdolManager shouldPlayerResurrect]` @ 0x10002ea9c
pub fn should_player_resurrect(world: &mut World, this: Entity) -> bool {
    should_resurrect(world, this, BattleSide::Player)
}

/// `-[IdolManager shouldEnemyResurrect]` @ 0x10002eae4
pub fn should_enemy_resurrect(world: &mut World, this: Entity) -> bool {
    should_resurrect(world, this, BattleSide::Enemy)
}

/// `-[IdolManager applyBattleExperience]` @ 0x10002eb2c
///
/// A surviving player's idols earn experience by the enemy's level, more with drained level
/// points, as points on every level-up track of their list episodes.
pub fn apply_battle_experience(world: &mut World, this: Entity) {
    let challenge_preferences = listener_challenge_preferences(world, this);

    let manager = world.obj::<IdolManager>(this);
    let enemy_level = manager.enemy_battle_state.level;
    let total_drain_level_points = manager.player_battle_state.total_drain_level_points;
    let player_health = manager.player_battle_state.health;

    // PORT: the original indexes the table with any level; one outside 0–19 (only reachable
    // through corrupt match data) reads neighbouring data there and 0 here.
    let battle_experience = match usize::try_from(enemy_level) {
        Ok(index) => BATTLE_EXPERIENCE.get(index).copied().unwrap_or(0.0),
        Err(_) => 0.0,
    };
    let mut experience = battle_experience * 4.0 * (total_drain_level_points + 1.0) * 0.25;
    if player_health <= 0.0 {
        experience = 0.0;
    }

    let statics = world.get_resource::<IdolManagerStatics>();
    let level_thresholds = statics.map(|statics| statics.level_thresholds).unwrap_or_default();
    let cap_points = level_thresholds[19] as f32;

    for slot in 0..BATTLE_IDOL_COUNT {
        idol_state_mut(world, this, BattleSide::Player, slot).experience = experience;

        // The list episode with the idol's id (the last one when none matches).
        let node = idol_state(world, this, BattleSide::Player, slot).struct_xml_episode_node_def;
        let idol_id = episode_node_id(world, required(node));
        let mut episode = EPISODE_LIST_LEN - 1;
        for candidate in 0..EPISODE_LIST_LEN {
            let candidate_id = &episode_table(world).def(candidate).id;
            if idol_id.as_deref() == Some(candidate_id.as_str()) {
                episode = candidate;
                break;
            }
        }

        let level_ups = episode_table(world).nodes[episode].level_ups.clone();
        for level_up in level_ups {
            let max_points = episode_table(world).level_ups[level_up].def().max_points;
            let scale = max_points as f32 / cap_points;
            let experience_points = (experience * scale).ceil();

            let (points, key) = {
                let mut table = episode_table_mut(world);
                let state = &mut table.level_ups[level_up];
                let earned_points = (experience_points + state.points as f32) as i32;
                state.points = if earned_points > max_points { max_points } else { earned_points };
                (state.points, state.preference_key.clone())
            };
            set_int_preference(world, challenge_preferences, key.as_deref(), points);
        }

        update_episode_status(world, this, Some(episode), true);
    }
}

// ---- Set-up --------------------------------------------------------------------------------------

/// `-[IdolManager isLastBattleIdol:]` @ 0x10002ed08
pub fn is_last_battle_idol(_world: &mut World, _this: Entity, last_battle_idol: Option<usize>) -> bool {
    let last_episode = EPISODE_LIST_LEN - 1;
    last_battle_idol == Some(last_episode)
}

/// `-[IdolManager nextBattleIdol:]` @ 0x10002ed2c
///
/// The list episode after `next_battle_idol` (the first one for nil) if it is battle-unlocked.
pub fn next_battle_idol(world: &mut World, _this: Entity, next_battle_idol: Option<usize>) -> Option<usize> {
    let mut previous = None;
    for episode in 0..EPISODE_LIST_LEN {
        if previous == next_battle_idol {
            let battle_unlocked = episode_table(world).nodes[episode].battle_unlocked;
            if battle_unlocked {
                return Some(episode);
            }
            return None;
        }
        previous = Some(episode);
    }
    None
}

/// `-[IdolManager onBattleEnemySelected:]` @ 0x10002ed68
pub fn on_battle_enemy_selected(world: &mut World, this: Entity, battle_enemy_selected: Option<usize>) {
    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.multi_player_active = false;
        manager.enemy_struct_xml_episode_node_def = battle_enemy_selected;
    }
    init_battle_of_the_gods_enemy(world, this);

    let temple_door = temple_door_logic::instance(world);
    if let Some(temple_door) = temple_door {
        temple_door_logic::start_battle(world, temple_door);
    }
}

/// `-[IdolManager calculateEnemyLevel:]` @ 0x10002edc4
///
/// The enemy line-up's level: its idols' levels summed and divided by four.
pub fn calculate_enemy_level(world: &mut World, _this: Entity, calculate_enemy_level: Option<usize>) -> i32 {
    let episode = required(calculate_enemy_level);
    let line_up = enemy_line_up(world, episode);

    let mut level = 0;
    for (_, idol_level) in line_up {
        level += idol_level;
    }
    level / 4
}

/// `-[IdolManager calculateEnemyHealth:]` @ 0x10002ee7c
pub fn calculate_enemy_health(world: &mut World, _this: Entity, calculate_enemy_health: Option<usize>) -> f32 {
    let episode = required(calculate_enemy_health);
    let line_up = enemy_line_up(world, episode);

    let mut health = 0.0;
    for (list_episode, idol_level) in line_up {
        let health_per_level = episode_table(world).def(list_episode).health_per_level;
        health = health + health_per_level * (idol_level + 1) as f32;
    }
    health
}

/// `-[IdolManager calculatePlayerLevel]` @ 0x10002ef50
pub fn calculate_player_level(world: &mut World, this: Entity) -> i32 {
    let selected = world.obj::<IdolManager>(this).selected_struct_xml_episode_node_def_array;

    let mut level = 0;
    for node in selected {
        if let Some(node) = node {
            level += episode_table(world).nodes[node].level;
        }
    }
    level / 4
}

/// `-[IdolManager calculatePlayerHealth]` @ 0x10002ef94
pub fn calculate_player_health(world: &mut World, this: Entity) -> f32 {
    let selected = world.obj::<IdolManager>(this).selected_struct_xml_episode_node_def_array;

    let mut health = 0.0;
    for node in selected {
        if let Some(node) = node {
            let level = episode_table(world).nodes[node].level;
            let health_per_level = episode_table(world).def(node).health_per_level;
            health = health + health_per_level * (level + 1) as f32;
        }
    }
    health
}

/// `-[IdolManager initPlayerBattleState:]` @ 0x10002efd8
///
/// Points the side's idols at its battle copies and sums its level, health and average damage.
pub fn init_player_battle_state(world: &mut World, this: Entity, init_player_battle_state: Option<usize>) {
    let side = battle_side(init_player_battle_state);
    let copies = match side {
        BattleSide::Player => PLAYER_BATTLE_EPISODE_COPIES,
        BattleSide::Enemy => ENEMY_BATTLE_EPISODE_COPIES,
    };

    {
        let mut state = side_state_mut(world, this, side);
        state.battle_struct_xml_episode_node_def = None;
        state.level = 0;
        state.health = 0.0;
        state.total_drain_level_points = 0.0;
        state.cancel_enemy_power_ups = false;
        state.unfreeze_idols = false;
        state.average_damage = 0.0;
        state.has_freeze_idol = false;
    }

    for slot in 0..BATTLE_IDOL_COUNT {
        let copy = copies + slot;
        let copy_level = episode_table(world).nodes[copy].level;
        let def = episode_table(world).def(copy);
        let level_factor = (copy_level + 1) as f32;

        let mut state = side_state_mut(world, this, side);
        {
            let idol = &mut state.idol_battle_state_array[slot];
            idol.used_count = 0;
            idol.active_frame_count = 0;
            idol.struct_xml_episode_node_def = Some(copy);
            idol.damage = 0.0;
            idol.health = 0.0;
            idol.experience = 0.0;
            idol.frozen_ref_count = 0;
            idol.freeze_idol = false;
            idol.frozen_idol_battle_state = None;
            idol.resurrect = false;
            idol.resurrect_health_per_level = 0.0;
        }

        state.health = state.health + def.health_per_level * level_factor;
        state.level += copy_level;
        state.average_damage = state.average_damage + def.damage_per_level * level_factor;
        for realm_power in &def.realm_powers {
            if realm_power.power == RealmPower::FreezeIdol {
                state.has_freeze_idol = true;
            }
        }
    }

    let mut state = side_state_mut(world, this, side);
    state.max_health = state.health;
    state.level /= 4;
    state.average_damage = state.average_damage * 0.25;
}

/// `-[IdolManager initBattleOfTheGodsEnemy]` @ 0x10002f0fc
///
/// Copies the enemy episode's line-up and the player's four selected idols into the battle
/// copies, names both sides and sets up their battle states.
pub fn init_battle_of_the_gods_enemy(world: &mut World, this: Entity) {
    let enemy_episode = world.obj::<IdolManager>(this).enemy_struct_xml_episode_node_def;
    let line_up = enemy_line_up(world, required(enemy_episode));
    for (slot, (list_episode, idol_level)) in line_up.into_iter().enumerate() {
        let copy = ENEMY_BATTLE_EPISODE_COPIES + slot;
        copy_battle_episode(world, list_episode, copy, idol_level, slot as i32);
    }

    let selected = world.obj::<IdolManager>(this).selected_struct_xml_episode_node_def_array;
    for (slot, node) in selected.into_iter().enumerate() {
        let node = node.expect("initBattleOfTheGodsEnemy: every selection slot holds an idol (memcpy from NULL)");
        let mut table = episode_table_mut(world);
        let copy = table.nodes[node].clone();
        table.nodes[PLAYER_BATTLE_EPISODE_COPIES + slot] = copy;
    }

    let temple_door = temple_door_logic::instance(world);
    let battle_pygmy = match temple_door {
        Some(temple_door) => temple_door_logic::battle_pygmy_logic(world, temple_door),
        None => None,
    };
    let player_name = match battle_pygmy {
        Some(battle_pygmy) => crate::pygmy::pygmy_name_label(world, battle_pygmy),
        None => None,
    };
    side_state_mut(world, this, BattleSide::Player).name = player_name;
    init_player_battle_state(world, this, Some(PLAYER_BATTLE_STATE));

    let enemy_episode = world.obj::<IdolManager>(this).enemy_struct_xml_episode_node_def;
    let enemy_name = episode_table(world).def(required(enemy_episode)).idol_name.clone();
    side_state_mut(world, this, BattleSide::Enemy).name = Some(enemy_name);
    init_player_battle_state(world, this, Some(ENEMY_BATTLE_STATE));
}
