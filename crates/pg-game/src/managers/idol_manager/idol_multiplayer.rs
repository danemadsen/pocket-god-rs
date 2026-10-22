//! `IdolManager`: methods in this part of the class.
//!
//! Battle of the Gods multiplayer over Game Center turn-based matches
//! (`docs/re/managers-platform.md` §4.5). GameKit is the stand-in of
//! `crate::managers::challenge_manager` ([`GkLocalPlayer`], [`GkTurnBasedMatch`],
//! [`GameKitService`] …). Without a match service the local player is never authenticated:
//! `openMatchPanel` shows "Multiplayer Not Available:" and nothing else here runs.

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

use crate::controller::BUNDLE_VERSION;
use crate::logic::battle::{battle_alert_logic, temple_door_logic};
use crate::logic::idol_screen::idol_screen_logic::{self, BattleSide, EPISODE_IDOL_SLOT_BITS};
use crate::managers::challenge_manager::{
    ErrorHandler, GameKitService, GkLocalPlayer, GkMatchRequest, GkTurnBasedMatch, GkTurnBasedMatchmakerViewController,
    GkTurnBasedParticipant, MatchDataHandler,
};
use crate::managers::game_server;

use super::*;

/// `GKTurnBasedMatchStatus`.
const MATCH_STATUS_UNKNOWN: i64 = 0;
const MATCH_STATUS_OPEN: i64 = 1;
const MATCH_STATUS_ENDED: i64 = 2;
const MATCH_STATUS_MATCHING: i64 = 3;

/// `GKTurnBasedMatchOutcome`.
const MATCH_OUTCOME_QUIT: i64 = 1;
const MATCH_OUTCOME_WON: i64 = 2;
const MATCH_OUTCOME_LOST: i64 = 3;
const MATCH_OUTCOME_TIED: i64 = 4;

/// `GKMatchRequest playerGroup` of Battle of the Gods matches.
const BATTLE_OF_THE_GODS_PLAYER_GROUP: u64 = 2;

/// Players of a match (`minPlayers` and `maxPlayers`).
const MATCH_PLAYER_COUNT: u64 = 2;

/// `-[IdolScreenLogic idolScreenMode]` while the battle screen is open.
const IDOL_SCREEN_MODE_BATTLE: i32 = 7;

/// Idols of each side.
const BATTLE_IDOL_COUNT: usize = 4;

/// `mPlayerPreferencePrefix` / `mEnemyPreferencePrefix` values: the side's key prefix in the
/// match data.
const CHALLENGER_PREFIX: &str = "Challenger";
const RECIPIENT_PREFIX: &str = "Recipient";

/// Match-data keys (`docs/re/audio-prefs-text.md` §2.6.5).
const STATE_SAVED_KEY: &str = "StateSaved";
const GAME_CATEGORY_KEY: &str = "GameCategory";
const GAME_VERSION_KEY: &str = "GameVersion";
const CHALLENGER_ALIAS_KEY: &str = "ChallengerAlias";
const CHALLENGER_ID_KEY: &str = "ChallengerId";
const RECIPIENT_ALIAS_KEY: &str = "RecipientAlias";
const RECIPIENT_ID_KEY: &str = "RecipientId";
const CHALLENGER_IDOL_ID_KEYS: [&str; BATTLE_IDOL_COUNT] =
    ["ChallengerIdol0Id", "ChallengerIdol1Id", "ChallengerIdol2Id", "ChallengerIdol3Id"];
const CHALLENGER_IDOL_LEVEL_KEYS: [&str; BATTLE_IDOL_COUNT] =
    ["ChallengerIdol0Level", "ChallengerIdol1Level", "ChallengerIdol2Level", "ChallengerIdol3Level"];
const RECIPIENT_IDOL_ID_KEYS: [&str; BATTLE_IDOL_COUNT] =
    ["RecipientIdol0Id", "RecipientIdol1Id", "RecipientIdol2Id", "RecipientIdol3Id"];
const RECIPIENT_IDOL_LEVEL_KEYS: [&str; BATTLE_IDOL_COUNT] =
    ["RecipientIdol0Level", "RecipientIdol1Level", "RecipientIdol2Level", "RecipientIdol3Level"];

/// `GameCategory` of a Battle of the Gods match.
const BATTLE_OF_THE_GODS_CATEGORY: &str = "BattleOfTheGods";

/// `[match setMessage:]` of a Battle of the Gods match.
const BATTLE_OF_THE_GODS_MATCH_MESSAGE: &str = "Battle of the Gods";

/// `GKMatchRequest inviteMessage`.
const INVITE_MESSAGE: &str = "Want to play Pocket God: Battle of the Gods?";

/// `showEnemyPanelMultiplayerAlert:message:` texts.
const BATTLE_COMPLETE_TITLE: &str = "Battle Complete:";
const WAITING_TO_VIEW_RESULTS: &str = "Waiting for other player to view results.";
const BATTLE_IS_COMPLETE: &str = "The Battle is complete.";
const UNKNOWN_RESULT_TITLE: &str = "Unknown Result:";
const TRY_AGAIN: &str = "Try again..";
const INVITE_SENT_TITLE: &str = "Invite Sent:";
const WAITING_TO_ACCEPT_INVITE: &str = "Waiting for other player to accept invite.";
const PLEASE_UPDATE_TITLE: &str = "Please Update Pocket God:";
const OPPONENT_HAS_NEWER_VERSION: &str =
    "Opponent has a newer version of Pocket God\nthat is incompatible with your version.";
const NOT_BATTLE_OF_THE_GODS_TITLE: &str = "Not Battle of The Gods:";
const NOT_BATTLE_OF_THE_GODS_MATCH: &str =
    "This match is not a Battle of the Gods Match,\nopen it in the Challenge friends screen.";
const PLAYER_QUIT_TITLE: &str = "Player Quit:";
const OTHER_PLAYER_QUIT: &str = "Other player quit game.";
const MULTIPLAYER_NOT_AVAILABLE_TITLE: &str = "Multiplayer Not Available:";
const LOG_INTO_GAME_CENTER: &str = "Please log into Game Center or upgrade to iOS5.";
const ERROR_OCCURED_TITLE: &str = "Error Occured:";
const ERROR_LOADING_MATCH: &str = "An error occured trying to load the match.";

// ---- GameKit -------------------------------------------------------------------------------------

/// The `GKTurnBasedMatch` requests only `leaveMatch` makes, which [`GameKitService`] does not
/// serve. A match service installs this next to [`GameKitService`]; without it the requests never
/// complete.
#[derive(Resource, Clone, Copy)]
pub struct GameKitLeaveMatchService {
    /// `-[GKTurnBasedMatch matchData]`
    pub match_data: fn(&World, Entity) -> Option<Vec<u8>>,

    /// `-[GKTurnBasedMatch participantQuitInTurnWithOutcome:nextParticipant:matchData:completionHandler:]`
    pub participant_quit_in_turn: fn(&mut World, Entity, i64, Entity, Option<Vec<u8>>, ErrorHandler),

    /// `-[GKTurnBasedMatch participantQuitOutOfTurnWithOutcome:withCompletionHandler:]`
    pub participant_quit_out_of_turn: fn(&mut World, Entity, i64, ErrorHandler),
}

/// `[[GKLocalPlayer localPlayer] isAuthenticated]`.
fn local_player_is_authenticated(world: &World) -> bool {
    match world.get_resource::<GkLocalPlayer>() {
        Some(local_player) => local_player.authenticated,
        None => false,
    }
}

/// `[[GKLocalPlayer localPlayer] playerID]`.
fn local_player_id(world: &World) -> Option<String> {
    let local_player = world.get_resource::<GkLocalPlayer>()?;
    local_player.player_id.clone()
}

/// `[[GKLocalPlayer localPlayer] alias]`.
fn local_player_alias(world: &World) -> Option<String> {
    let local_player = world.get_resource::<GkLocalPlayer>()?;
    local_player.alias.clone()
}

/// `[match status]`.
fn match_status(world: &World, game_match: Entity) -> i64 {
    match world.get::<GkTurnBasedMatch>(game_match) {
        Some(state) => state.status,
        None => MATCH_STATUS_UNKNOWN,
    }
}

/// `[match matchID]`.
fn match_id(world: &World, game_match: Entity) -> Option<String> {
    let state = world.get::<GkTurnBasedMatch>(game_match)?;
    state.match_id.clone()
}

/// `[match participants]` (nil for a nil match), copied for enumeration.
fn match_participants(world: &World, game_match: Option<Entity>) -> Vec<Entity> {
    let Some(game_match) = game_match else {
        return Vec::new();
    };
    match world.get::<GkTurnBasedMatch>(game_match) {
        Some(state) => state.participants.clone(),
        None => Vec::new(),
    }
}

/// `[match currentParticipant]` (nil for a nil match).
fn match_current_participant(world: &World, game_match: Option<Entity>) -> Option<Entity> {
    let state = world.get::<GkTurnBasedMatch>(game_match?)?;
    state.current_participant
}

/// `[match setMessage:message]`.
fn set_match_message(world: &mut World, game_match: Entity, message: &str) {
    if let Some(mut state) = world.get_mut::<GkTurnBasedMatch>(game_match) {
        state.message = Some(message.to_owned());
    }
}

/// `[participant playerID]` (nil for a nil participant).
fn participant_player_id(world: &World, participant: Option<Entity>) -> Option<String> {
    let state = world.get::<GkTurnBasedParticipant>(participant?)?;
    state.player_id.clone()
}

/// `[participant matchOutcome]`.
fn participant_match_outcome(world: &World, participant: Entity) -> i64 {
    match world.get::<GkTurnBasedParticipant>(participant) {
        Some(state) => state.match_outcome,
        None => 0,
    }
}

/// `[participant setMatchOutcome:outcome]`.
fn set_participant_match_outcome(world: &mut World, participant: Entity, outcome: i64) {
    if let Some(mut state) = world.get_mut::<GkTurnBasedParticipant>(participant) {
        state.match_outcome = outcome;
    }
}

/// `[match loadMatchDataWithCompletionHandler:handler]`; never completes without a service.
fn load_match_data(world: &mut World, game_match: Entity, handler: MatchDataHandler) {
    let service = world.get_resource::<GameKitService>().copied();
    if let Some(service) = service {
        (service.load_match_data)(world, game_match, handler);
    }
}

/// `[match endTurnWithNextParticipant:next matchData:data completionHandler:handler]`.
fn end_turn_with_next_participant(world: &mut World, game_match: Entity, next: Entity, data: Option<Vec<u8>>, handler: ErrorHandler) {
    let service = world.get_resource::<GameKitService>().copied();
    if let Some(service) = service {
        (service.end_turn_with_next_participant)(world, game_match, next, data, handler);
    }
}

/// `[match endMatchInTurnWithMatchData:data completionHandler:handler]`.
fn end_match_in_turn_with_match_data(world: &mut World, game_match: Entity, data: Option<Vec<u8>>, handler: ErrorHandler) {
    let service = world.get_resource::<GameKitService>().copied();
    if let Some(service) = service {
        (service.end_match_in_turn_with_match_data)(world, game_match, data, handler);
    }
}

/// `[match matchData]`: nil without a service.
fn match_data(world: &World, game_match: Entity) -> Option<Vec<u8>> {
    let service = world.get_resource::<GameKitLeaveMatchService>()?;
    (service.match_data)(world, game_match)
}

/// `[match participantQuitInTurnWithOutcome:outcome nextParticipant:next matchData:data
/// completionHandler:handler]`.
fn participant_quit_in_turn(world: &mut World, game_match: Entity, outcome: i64, next: Entity, data: Option<Vec<u8>>, handler: ErrorHandler) {
    let service = world.get_resource::<GameKitLeaveMatchService>().copied();
    if let Some(service) = service {
        (service.participant_quit_in_turn)(world, game_match, outcome, next, data, handler);
    }
}

/// `[match participantQuitOutOfTurnWithOutcome:outcome withCompletionHandler:handler]`.
fn participant_quit_out_of_turn(world: &mut World, game_match: Entity, outcome: i64, handler: ErrorHandler) {
    let service = world.get_resource::<GameKitLeaveMatchService>().copied();
    if let Some(service) = service {
        (service.participant_quit_out_of_turn)(world, game_match, outcome, handler);
    }
}

/// `[[GKTurnBasedMatchmakerViewController alloc] initWithMatchRequest:request]`, then
/// `setTurnBasedMatchmakerDelegate:delegate`.
fn spawn_matchmaker_view_controller(world: &mut World, match_request: GkMatchRequest, delegate: Entity) -> Entity {
    let view_controller = GkTurnBasedMatchmakerViewController {
        match_request,
        turn_based_matchmaker_delegate: Some(delegate),
    };
    world.spawn(view_controller).id()
}

// ---- Main-thread calls ---------------------------------------------------------------------------

/// Which `showMatchAlert…:` selector `showMatchAlert:`'s block performs.
#[derive(Clone, Copy, Debug)]
enum MatchAlert {
    QuitMatch,
    MatchComplete,
    Invite,
    YourTurn,
}

/// A `performSelectorOnMainThread:withObject:waitUntilDone:NO` of this class that carries an
/// object (the GameKit completion handlers' calls). `ShowMatchAlert` carries the array
/// `[match, alias]` as its two objects.
#[derive(Clone, Debug)]
enum MainThreadCall {
    ParseMatchData(Option<Vec<u8>>),
    SetActiveMatch(Option<Entity>),
    ShowMatchAlert(MatchAlert, Entity, Option<String>),
}

/// The objects of the queued [`MainThreadCall`]s, oldest first. Each has its entry in the
/// controller's main-thread queue (`crate::controller::perform_selector_on_main_thread`), so every
/// main-thread call of the game keeps one order.
#[derive(Resource, Default)]
struct IdolManagerMainThreadCalls {
    calls: VecDeque<MainThreadCall>,
}

/// `[self performSelectorOnMainThread:selector withObject:object waitUntilDone:NO]`.
fn perform_on_main_thread(world: &mut World, this: Entity, call: MainThreadCall) {
    let mut main_thread_calls = world.get_resource_or_insert_with(IdolManagerMainThreadCalls::default);
    main_thread_calls.calls.push_back(call);
    crate::controller::perform_selector_on_main_thread(world, this, run_main_thread_call);
}

/// Runs the oldest queued [`MainThreadCall`].
fn run_main_thread_call(world: &mut World, this: Entity) {
    let call = match world.get_resource_mut::<IdolManagerMainThreadCalls>() {
        Some(mut main_thread_calls) => main_thread_calls.calls.pop_front(),
        None => None,
    };
    let Some(call) = call else {
        return;
    };

    match call {
        MainThreadCall::ParseMatchData(data) => parse_match_data(world, this, data.as_deref()),
        MainThreadCall::SetActiveMatch(game_match) => set_active_match(world, this, game_match),
        MainThreadCall::ShowMatchAlert(alert, game_match, alias) => {
            let alias = alias.as_deref();
            match alert {
                MatchAlert::QuitMatch => show_match_alert_quit_match(world, this, game_match, alias),
                MatchAlert::MatchComplete => show_match_alert_match_complete(world, this, game_match, alias),
                MatchAlert::Invite => show_match_alert_invite(world, this, game_match, alias),
                MatchAlert::YourTurn => show_match_alert_your_turn(world, this, game_match, alias),
            }
        }
    }
}

// ---- Helpers ---------------------------------------------------------------------------------------

/// `[a isEqualToString:b]`: NO when either string is nil.
fn ns_string_equal(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// An object as `%@` formats it (`(null)` for nil).
fn ns_description(value: Option<&str>) -> &str {
    value.unwrap_or("(null)")
}

/// A pointer the original dereferences without a nil check (a crash there, a panic here).
fn required(pointer: Option<usize>) -> usize {
    pointer.expect("IdolManager: NULL pointer (the original dereferences it)")
}

// ---- Battle-state pointers -------------------------------------------------------------------------

/// `&mPlayerBattleState` as a `PlayerBattleState *`, the `Option<usize>` battle-state parameter of
/// `IdolManager`'s battle methods: side 0 of the `IdolBattleState *` encoding
/// ([`idol_screen_logic::idol_battle_state_pointer`]).
pub const PLAYER_BATTLE_STATE: usize = 0;

/// `&mEnemyBattleState` as a `PlayerBattleState *`: side 1.
pub const ENEMY_BATTLE_STATE: usize = 1;

/// The side a `PlayerBattleState *` points at.
fn battle_side(battle_state_pointer: usize) -> BattleSide {
    match battle_state_pointer {
        PLAYER_BATTLE_STATE => BattleSide::Player,
        ENEMY_BATTLE_STATE => BattleSide::Enemy,
        _ => panic!("IdolManager: {battle_state_pointer} is not a PlayerBattleState pointer"),
    }
}

/// `&battleState->mIdolBattleStateArray[slot]` as an `IdolBattleState *`.
fn idol_battle_state_pointer_in(battle_state_pointer: usize, slot: usize) -> usize {
    let side = battle_side(battle_state_pointer);
    idol_screen_logic::idol_battle_state_pointer(side, slot)
}

/// `mPlayerBattleState` or `mEnemyBattleState`.
fn side_battle_state(manager: &IdolManager, side: BattleSide) -> &PlayerBattleState {
    match side {
        BattleSide::Player => &manager.player_battle_state,
        BattleSide::Enemy => &manager.enemy_battle_state,
    }
}

/// [`side_battle_state`], for writing.
fn side_battle_state_mut(manager: &mut IdolManager, side: BattleSide) -> &mut PlayerBattleState {
    match side {
        BattleSide::Player => &mut manager.player_battle_state,
        BattleSide::Enemy => &mut manager.enemy_battle_state,
    }
}

/// `*battleState` for a `PlayerBattleState *`.
fn battle_state_at(world: &World, this: Entity, battle_state_pointer: usize) -> &PlayerBattleState {
    let manager = world.obj::<IdolManager>(this);
    let side = battle_side(battle_state_pointer);
    side_battle_state(manager, side)
}

/// `*battleState`, for writing.
fn battle_state_at_mut(world: &mut World, this: Entity, battle_state_pointer: usize) -> Mut<'_, PlayerBattleState> {
    let manager = world.obj_mut::<IdolManager>(this);
    let side = battle_side(battle_state_pointer);
    manager.map_unchanged(|manager| side_battle_state_mut(manager, side))
}

/// `*idolBattleState` for an `IdolBattleState *`.
fn idol_battle_state_at(world: &World, this: Entity, idol_battle_state_pointer: usize) -> &IdolBattleState {
    let manager = world.obj::<IdolManager>(this);
    let (side, slot) = idol_screen_logic::idol_battle_state_location(idol_battle_state_pointer);
    &side_battle_state(manager, side).idol_battle_state_array[slot]
}

/// `*idolBattleState`, for writing.
fn idol_battle_state_at_mut(world: &mut World, this: Entity, idol_battle_state_pointer: usize) -> Mut<'_, IdolBattleState> {
    let manager = world.obj_mut::<IdolManager>(this);
    let (side, slot) = idol_screen_logic::idol_battle_state_location(idol_battle_state_pointer);
    manager.map_unchanged(|manager| &mut side_battle_state_mut(manager, side).idol_battle_state_array[slot])
}

// ---- Episode nodes -----------------------------------------------------------------------------------

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

/// `memcpy(copy, source, sizeof(BCStructXmlEpisodeNodeDef))`, then the copy's level and selection
/// slot.
fn copy_battle_episode(world: &mut World, source: usize, copy: usize, level: i32, slot: i32) {
    let mut table = episode_table_mut(world);
    let mut node = table.nodes[source].clone();
    node.level = level;
    node.selection_slot = slot;
    table.nodes[copy] = node;
}

/// `FUN_100038648(slotBits, node->mIdolTexture, @"raw")`: loads the idol image of episode node
/// `node` into the episode idol slot `idol_slot` (0–7: `EpisodeIdolSlot1` … `8`).
fn activate_idol_slot_texture(world: &mut World, idol_slot: usize, node: usize) {
    let idol_texture = episode_table(world).def(node).idol_texture.clone();
    let skin_texture_bits_def_array = EPISODE_IDOL_SLOT_BITS[idol_slot];
    idol_screen_logic::activate_skin_texture_bits_def_array(world, skin_texture_bits_def_array, &idol_texture);
}

/// `idolBattleState->mpStructXmlEpisodeNodeDef->mId` for an `IdolBattleState *`.
fn idol_battle_state_episode_id(world: &mut World, this: Entity, idol_battle_state_pointer: usize) -> Option<String> {
    let node = idol_battle_state_at(world, this, idol_battle_state_pointer).struct_xml_episode_node_def;
    let node = required(node);
    episode_node_id(world, node)
}

/// `mPlayerPreferencePrefix` and `mEnemyPreferencePrefix` as `%@` formats them.
fn preference_prefixes(world: &World, this: Entity) -> (String, String) {
    let manager = world.obj::<IdolManager>(this);
    let player_prefix = ns_description(manager.player_preference_prefix.as_deref()).to_owned();
    let enemy_prefix = ns_description(manager.enemy_preference_prefix.as_deref()).to_owned();
    (player_prefix, enemy_prefix)
}

/// `[mMatchPreferences getStringPreference:key]` (nil without a document).
fn match_string_preference(world: &World, this: Entity, key: &str) -> Option<String> {
    let match_preferences = world.obj::<IdolManager>(this).match_preferences?;
    pg_engine::prefs::get_string_preference(world, match_preferences, key)
}

/// `[mMatchPreferences getIntPreference:key]`.
fn match_int_preference(world: &World, this: Entity, key: &str) -> i32 {
    match world.obj::<IdolManager>(this).match_preferences {
        Some(match_preferences) => pg_engine::prefs::get_int_preference(world, match_preferences, key),
        None => 0,
    }
}

/// `[mMatchPreferences getFloatPreference:key]`.
fn match_float_preference(world: &World, this: Entity, key: &str) -> f32 {
    match world.obj::<IdolManager>(this).match_preferences {
        Some(match_preferences) => pg_engine::prefs::get_float_preference(world, match_preferences, key),
        None => 0.0,
    }
}

/// `[mMatchPreferences setPreference:key asString:value]`.
fn set_match_string_preference(world: &mut World, this: Entity, key: &str, value: Option<&str>) {
    let match_preferences = world.obj::<IdolManager>(this).match_preferences;
    let Some(match_preferences) = match_preferences else {
        return;
    };

    // PORT: a nil value would raise in `setObject:forKey:`. The values are episode ids and the
    // authenticated local player's alias and id, which are never nil; nothing is stored instead.
    if let Some(value) = value {
        pg_engine::prefs::set_preference_as_string(world, match_preferences, key, value);
    }
}

/// `[mMatchPreferences setPreference:key asInt:value]`.
fn set_match_int_preference(world: &mut World, this: Entity, key: &str, value: i32) {
    let match_preferences = world.obj::<IdolManager>(this).match_preferences;
    if let Some(match_preferences) = match_preferences {
        pg_engine::prefs::set_preference_as_int(world, match_preferences, key, value);
    }
}

/// `[mMatchPreferences setPreference:key asFloat:value]`.
fn set_match_float_preference(world: &mut World, this: Entity, key: &str, value: f32) {
    let match_preferences = world.obj::<IdolManager>(this).match_preferences;
    if let Some(match_preferences) = match_preferences {
        pg_engine::prefs::set_preference_as_float(world, match_preferences, key, value);
    }
}

/// `[mMatchPreferences removePreference:key]`.
fn remove_match_preference(world: &mut World, this: Entity, key: &str) {
    let match_preferences = world.obj::<IdolManager>(this).match_preferences;
    if let Some(match_preferences) = match_preferences {
        pg_engine::prefs::remove_preference(world, match_preferences, key);
    }
}

/// `[mMatchPreferences dataFromPreferences]`.
fn match_preferences_data(world: &World, this: Entity) -> Option<Vec<u8>> {
    let match_preferences = world.obj::<IdolManager>(this).match_preferences?;
    pg_engine::prefs::data_from_preferences(world, match_preferences)
}

/// `-[NSString compare:options:NSNumericSearch]`: runs of digits compare by value, any other
/// character by itself.
fn compare_numerically(a: &str, b: &str) -> Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();
    loop {
        let a_next = a_chars.peek().copied();
        let b_next = b_chars.peek().copied();
        let (a_char, b_char) = match (a_next, b_next) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a_char), Some(b_char)) => (a_char, b_char),
        };

        if a_char.is_ascii_digit() && b_char.is_ascii_digit() {
            let a_digits = take_digits(&mut a_chars);
            let b_digits = take_digits(&mut b_chars);
            let digits_order = compare_digit_runs(&a_digits, &b_digits);
            if digits_order != Ordering::Equal {
                return digits_order;
            }
            continue;
        }

        if a_char != b_char {
            return a_char.cmp(&b_char);
        }
        a_chars.next();
        b_chars.next();
    }
}

/// The run of ASCII digits at the front of `chars`.
fn take_digits(chars: &mut Peekable<Chars>) -> String {
    let mut digits = String::new();
    while let Some(digit) = chars.next_if(char::is_ascii_digit) {
        digits.push(digit);
    }
    digits
}

/// Two runs of digits by their values.
fn compare_digit_runs(a: &str, b: &str) -> Ordering {
    let a_value = a.trim_start_matches('0');
    let b_value = b.trim_start_matches('0');
    let length_order = a_value.len().cmp(&b_value.len());
    if length_order != Ordering::Equal {
        return length_order;
    }
    a_value.cmp(b_value)
}

/// `[CFBundleVersion compare:[mMatchPreferences getStringPreference:@"GameVersion"]
/// options:NSNumericSearch] == NSOrderedAscending`: the opponent plays a newer version.
fn is_game_version_newer(world: &World, this: Entity) -> bool {
    let game_version = match_string_preference(world, this, GAME_VERSION_KEY);

    // PORT: `compare:` with a nil string is undefined in Foundation; it counts as "not newer"
    // here. The challenger writes GameVersion before anyone else can open the match.
    let Some(game_version) = game_version else {
        return false;
    };
    compare_numerically(BUNDLE_VERSION, &game_version) == Ordering::Less
}

/// `mIdolManagerListener` (the view controller, set right after `init` in `loadGame`).
fn idol_manager_listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<IdolManager>(this).idol_manager_listener
}

/// `[mIdolManagerListener dismissModalViewController]`.
fn dismiss_listener_modal_view_controller(world: &mut World, this: Entity) {
    let listener = idol_manager_listener(world, this);
    if let Some(listener) = listener {
        crate::controller::dismiss_modal_view_controller(world, listener);
    }
}

/// `[mIdolScreenLogic idolScreenMode]` (0 for a nil screen logic).
fn idol_screen_mode(world: &mut World, this: Entity) -> i32 {
    let idol_screen_logic = world.obj::<IdolManager>(this).idol_screen_logic;
    match idol_screen_logic {
        Some(idol_screen_logic) => idol_screen_logic::idol_screen_mode(world, idol_screen_logic),
        None => 0,
    }
}

/// `[mIdolScreenLogic closeScreenNow]` (dispatched).
fn close_idol_screen_now(world: &mut World, this: Entity) {
    let idol_screen_logic = world.obj::<IdolManager>(this).idol_screen_logic;
    if let Some(idol_screen_logic) = idol_screen_logic {
        let class = isa(world, idol_screen_logic);
        (class.close_screen_now)(world, idol_screen_logic);
    }
}

/// `[mIdolScreenLogic showEnemyPanelMultiplayerAlert:title message:message]`.
fn show_enemy_panel_multiplayer_alert(world: &mut World, this: Entity, title: &str, message: &str) {
    let idol_screen_logic = world.obj::<IdolManager>(this).idol_screen_logic;
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::show_enemy_panel_multiplayer_alert(world, idol_screen_logic, title, message);
    }
}

/// `[[TempleDoorLogic instance] quitGame]`.
fn quit_temple_door_game(world: &mut World) {
    let temple_door = temple_door_logic::instance(world);
    if let Some(temple_door) = temple_door {
        temple_door_logic::quit_game(world, temple_door);
    }
}

/// `[[TempleDoorLogic instance] playBattleSequence]`.
fn play_temple_door_battle_sequence(world: &mut World) {
    let temple_door = temple_door_logic::instance(world);
    if let Some(temple_door) = temple_door {
        temple_door_logic::play_battle_sequence(world, temple_door);
    }
}

/// `mbMultiPlayerActive = NO`.
fn stop_multiplayer(world: &mut World, this: Entity) {
    world.obj_mut::<IdolManager>(this).multi_player_active = false;
}

// ---- Turn state ------------------------------------------------------------------------------------

/// `-[IdolManager quitBattleOfTheGods]` @ 0x1000302d8
pub fn quit_battle_of_the_gods(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.multi_player_active = false;
        manager.load_active_match_data = false;
    }
    set_active_match(world, this, None);
}

/// `-[IdolManager onIdolBattleSelectionPanelOpened]` @ 0x100030304
pub fn on_idol_battle_selection_panel_opened(world: &mut World, this: Entity) {
    let manager = world.obj::<IdolManager>(this);
    let opponents_turn = manager.multi_player_active && !manager.multiplayer_my_turn;
    if opponents_turn {
        load_active_match_data(world, this);
    }
}

/// `-[IdolManager isMultiplayerActive]` @ 0x100030338
pub fn is_multiplayer_active(world: &mut World, this: Entity) -> bool {
    world.obj::<IdolManager>(this).multi_player_active
}

/// `-[IdolManager isItMyTurn]` @ 0x100030348
pub fn is_it_my_turn(world: &mut World, this: Entity) -> bool {
    world.obj::<IdolManager>(this).multiplayer_my_turn
}

/// `-[IdolManager saveTurnIdolBattleState:preferencePrefix:]` @ 0x100030358
///
/// `idol_battle_state` is an `IdolBattleState *` ([`idol_screen_logic::idol_battle_state_pointer`]).
pub fn save_turn_idol_battle_state(world: &mut World, this: Entity, idol_battle_state: Option<usize>, preference_prefix: &str) {
    let idol_battle_state_pointer = required(idol_battle_state);
    let state = *idol_battle_state_at(world, this, idol_battle_state_pointer);

    let active_frame_count_key = format!("{preference_prefix}ActiveFrameCount");
    set_match_int_preference(world, this, &active_frame_count_key, state.active_frame_count);

    let used_count_key = format!("{preference_prefix}UsedCount");
    set_match_int_preference(world, this, &used_count_key, state.used_count);

    let frozen_ref_count_key = format!("{preference_prefix}FrozenRefCount");
    set_match_int_preference(world, this, &frozen_ref_count_key, state.frozen_ref_count);

    let frozen_idol_id_key = format!("{preference_prefix}FrozenIdolId");
    remove_match_preference(world, this, &frozen_idol_id_key);

    let Some(frozen_idol_battle_state) = state.frozen_idol_battle_state else {
        return;
    };
    let frozen_idol_id = idol_battle_state_episode_id(world, this, frozen_idol_battle_state);
    set_match_string_preference(world, this, &frozen_idol_id_key, frozen_idol_id.as_deref());
}

/// `-[IdolManager loadTurnIdolBattleState:preferencePrefix:opponentBattleState:]` @ 0x1000304e4
///
/// `idol_battle_state` is an `IdolBattleState *`; `opponent_battle_state` a `PlayerBattleState *`.
/// The frozen target becomes the opponent's idol with the saved episode id (the last one, should
/// two match).
pub fn load_turn_idol_battle_state(world: &mut World, this: Entity, idol_battle_state: Option<usize>, preference_prefix: &str, opponent_battle_state: Option<usize>) {
    let idol_battle_state_pointer = required(idol_battle_state);

    let active_frame_count_key = format!("{preference_prefix}ActiveFrameCount");
    let active_frame_count = match_int_preference(world, this, &active_frame_count_key);
    idol_battle_state_at_mut(world, this, idol_battle_state_pointer).active_frame_count = active_frame_count;

    let used_count_key = format!("{preference_prefix}UsedCount");
    let used_count = match_int_preference(world, this, &used_count_key);
    idol_battle_state_at_mut(world, this, idol_battle_state_pointer).used_count = used_count;

    let frozen_ref_count_key = format!("{preference_prefix}FrozenRefCount");
    let frozen_ref_count = match_int_preference(world, this, &frozen_ref_count_key);
    idol_battle_state_at_mut(world, this, idol_battle_state_pointer).frozen_ref_count = frozen_ref_count;

    let frozen_idol_id_key = format!("{preference_prefix}FrozenIdolId");
    let frozen_idol_id = match_string_preference(world, this, &frozen_idol_id_key);
    idol_battle_state_at_mut(world, this, idol_battle_state_pointer).frozen_idol_battle_state = None;

    let Some(frozen_idol_id) = frozen_idol_id else {
        return;
    };
    let opponent_battle_state_pointer = required(opponent_battle_state);
    for slot in 0..BATTLE_IDOL_COUNT {
        let opponent_idol_pointer = idol_battle_state_pointer_in(opponent_battle_state_pointer, slot);
        let opponent_idol_id = idol_battle_state_episode_id(world, this, opponent_idol_pointer);
        if !ns_string_equal(opponent_idol_id.as_deref(), Some(&frozen_idol_id)) {
            continue;
        }
        idol_battle_state_at_mut(world, this, idol_battle_state_pointer).frozen_idol_battle_state = Some(opponent_idol_pointer);
    }
}

/// `-[IdolManager saveTurnPlayerBattleState:preferencePrefix:]` @ 0x100030668
///
/// `battle_state` is a `PlayerBattleState *` ([`PLAYER_BATTLE_STATE`] or [`ENEMY_BATTLE_STATE`]).
pub fn save_turn_player_battle_state(world: &mut World, this: Entity, battle_state: Option<usize>, preference_prefix: &str) {
    let battle_state_pointer = required(battle_state);
    let state = battle_state_at(world, this, battle_state_pointer);
    let health = state.health;
    let max_health = state.max_health;
    let total_drain_level_points = state.total_drain_level_points;

    let health_key = format!("{preference_prefix}Health");
    set_match_float_preference(world, this, &health_key, health);

    let max_health_key = format!("{preference_prefix}MaxHealth");
    set_match_float_preference(world, this, &max_health_key, max_health);

    let total_drain_level_points_key = format!("{preference_prefix}TotalDrainLevelPoints");
    set_match_float_preference(world, this, &total_drain_level_points_key, total_drain_level_points);

    for slot in 0..BATTLE_IDOL_COUNT {
        let idol_prefix = format!("{preference_prefix}Idol{slot}");
        let idol_battle_state_pointer = idol_battle_state_pointer_in(battle_state_pointer, slot);
        save_turn_idol_battle_state(world, this, Some(idol_battle_state_pointer), &idol_prefix);
    }
}

/// `-[IdolManager loadTurnPlayerBattleState:preferencePrefix:opponentBattleState:]` @ 0x1000307b4
///
/// `battle_state` and `opponent_battle_state` are `PlayerBattleState *`s.
pub fn load_turn_player_battle_state(world: &mut World, this: Entity, battle_state: Option<usize>, preference_prefix: &str, opponent_battle_state: Option<usize>) {
    let battle_state_pointer = required(battle_state);

    let health_key = format!("{preference_prefix}Health");
    let health = match_float_preference(world, this, &health_key);
    battle_state_at_mut(world, this, battle_state_pointer).health = health;

    let max_health_key = format!("{preference_prefix}MaxHealth");
    let max_health = match_float_preference(world, this, &max_health_key);
    battle_state_at_mut(world, this, battle_state_pointer).max_health = max_health;

    let total_drain_level_points_key = format!("{preference_prefix}TotalDrainLevelPoints");
    let total_drain_level_points = match_float_preference(world, this, &total_drain_level_points_key);
    battle_state_at_mut(world, this, battle_state_pointer).total_drain_level_points = total_drain_level_points;

    for slot in 0..BATTLE_IDOL_COUNT {
        let idol_prefix = format!("{preference_prefix}Idol{slot}");
        let idol_battle_state_pointer = idol_battle_state_pointer_in(battle_state_pointer, slot);
        load_turn_idol_battle_state(world, this, Some(idol_battle_state_pointer), &idol_prefix, opponent_battle_state);
    }
}

/// `-[IdolManager saveTurnGameState]` @ 0x100030910
pub fn save_turn_game_state(world: &mut World, this: Entity) {
    set_match_int_preference(world, this, STATE_SAVED_KEY, 1);

    let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
    let player_random_frozen_idol_id_key = format!("{player_prefix}RandomFrozenIdolId");
    let enemy_random_frozen_idol_id_key = format!("{enemy_prefix}RandomFrozenIdolId");
    remove_match_preference(world, this, &player_random_frozen_idol_id_key);
    remove_match_preference(world, this, &enemy_random_frozen_idol_id_key);

    let player_random_frozen_idol_id = world.obj::<IdolManager>(this).player_random_frozen_idol_id.clone();
    if let Some(player_random_frozen_idol_id) = player_random_frozen_idol_id {
        set_match_string_preference(world, this, &player_random_frozen_idol_id_key, Some(&player_random_frozen_idol_id));
    }

    let enemy_random_frozen_idol_id = world.obj::<IdolManager>(this).enemy_random_frozen_idol_id.clone();
    if let Some(enemy_random_frozen_idol_id) = enemy_random_frozen_idol_id {
        set_match_string_preference(world, this, &enemy_random_frozen_idol_id_key, Some(&enemy_random_frozen_idol_id));
    }

    save_turn_player_battle_state(world, this, Some(PLAYER_BATTLE_STATE), &player_prefix);
    save_turn_player_battle_state(world, this, Some(ENEMY_BATTLE_STATE), &enemy_prefix);
}

/// `-[IdolManager loadTurnGameState]` @ 0x100030ae0
pub fn load_turn_game_state(world: &mut World, this: Entity) {
    let state_saved = match_int_preference(world, this, STATE_SAVED_KEY);
    if state_saved != 1 {
        return;
    }

    let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
    let player_random_frozen_idol_id_key = format!("{player_prefix}RandomFrozenIdolId");
    let player_random_frozen_idol_id = match_string_preference(world, this, &player_random_frozen_idol_id_key);
    let enemy_random_frozen_idol_id_key = format!("{enemy_prefix}RandomFrozenIdolId");
    let enemy_random_frozen_idol_id = match_string_preference(world, this, &enemy_random_frozen_idol_id_key);
    set_player_random_frozen_idol_id(world, this, player_random_frozen_idol_id.as_deref(), enemy_random_frozen_idol_id.as_deref());

    load_turn_player_battle_state(world, this, Some(PLAYER_BATTLE_STATE), &player_prefix, Some(ENEMY_BATTLE_STATE));
    load_turn_player_battle_state(world, this, Some(ENEMY_BATTLE_STATE), &enemy_prefix, Some(PLAYER_BATTLE_STATE));
}

// ---- The battle --------------------------------------------------------------------------------------

/// `-[IdolManager initBattleOfTheGodsMultiPlayer]` @ 0x100030c60
///
/// Both sides' idols come from the match data: each list episode named by `%@Idol%dId` is copied
/// into the side's battle copy with level `%@Idol%dLevel`, and its image goes into
/// `EpisodeIdolSlot1`–`4` (player) or `5`–`8` (enemy).
pub fn init_battle_of_the_gods_multi_player(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.i_am_challenger = false;
        manager.player_preference_prefix = Some(RECIPIENT_PREFIX.to_owned());
        manager.enemy_preference_prefix = Some(CHALLENGER_PREFIX.to_owned());
    }

    let challenger_id = match_string_preference(world, this, CHALLENGER_ID_KEY);
    let player_id = local_player_id(world);
    if ns_string_equal(challenger_id.as_deref(), player_id.as_deref()) {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.i_am_challenger = true;
        manager.player_preference_prefix = Some(CHALLENGER_PREFIX.to_owned());
        manager.enemy_preference_prefix = Some(RECIPIENT_PREFIX.to_owned());
    }

    let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
    for slot in 0..BATTLE_IDOL_COUNT {
        let player_idol_id_key = format!("{player_prefix}Idol{slot}Id");
        let player_idol_id = match_string_preference(world, this, &player_idol_id_key);
        let player_idol_level_key = format!("{player_prefix}Idol{slot}Level");
        let player_idol_level = match_int_preference(world, this, &player_idol_level_key);
        let enemy_idol_id_key = format!("{enemy_prefix}Idol{slot}Id");
        let enemy_idol_id = match_string_preference(world, this, &enemy_idol_id_key);
        let enemy_idol_level_key = format!("{enemy_prefix}Idol{slot}Level");
        let enemy_idol_level = match_int_preference(world, this, &enemy_idol_level_key);

        for episode in 0..EPISODE_LIST_LEN {
            let episode_id = episode_node_id(world, episode);

            if ns_string_equal(episode_id.as_deref(), player_idol_id.as_deref()) {
                let player_copy = PLAYER_BATTLE_EPISODE_COPIES + slot;
                copy_battle_episode(world, episode, player_copy, player_idol_level, slot as i32);
                activate_idol_slot_texture(world, slot, episode);
            }

            if ns_string_equal(episode_id.as_deref(), enemy_idol_id.as_deref()) {
                let enemy_copy = ENEMY_BATTLE_EPISODE_COPIES + slot;
                copy_battle_episode(world, episode, enemy_copy, enemy_idol_level, slot as i32);
                activate_idol_slot_texture(world, BATTLE_IDOL_COUNT + slot, episode);
            }
        }
    }

    let player_alias_key = format!("{player_prefix}Alias");
    let player_alias = match_string_preference(world, this, &player_alias_key);
    world.obj_mut::<IdolManager>(this).player_battle_state.name = player_alias;
    init_player_battle_state(world, this, Some(PLAYER_BATTLE_STATE));

    let enemy_alias_key = format!("{enemy_prefix}Alias");
    let enemy_alias = match_string_preference(world, this, &enemy_alias_key);
    world.obj_mut::<IdolManager>(this).enemy_battle_state.name = enemy_alias;
    init_player_battle_state(world, this, Some(ENEMY_BATTLE_STATE));
}

/// `-[IdolManager canShowIdolBattleMultiplayerResult]` @ 0x1000310b4
pub fn can_show_idol_battle_multiplayer_result(world: &mut World, this: Entity) -> bool {
    let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
    let player_battle_idol_id_key = format!("{player_prefix}BattleIdolId");
    let player_battle_idol_id = match_string_preference(world, this, &player_battle_idol_id_key);
    let enemy_battle_idol_id_key = format!("{enemy_prefix}BattleIdolId");
    let enemy_battle_idol_id = match_string_preference(world, this, &enemy_battle_idol_id_key);

    if player_battle_idol_id.is_none() || enemy_battle_idol_id.is_none() {
        return false;
    }
    world.obj::<IdolManager>(this).multiplayer_my_turn
}

/// `-[IdolManager loadMultiplayerBattleIdols]` @ 0x1000311b0
///
/// Each side's battle idol is its battle copy whose id is the side's `%@BattleIdolId`.
pub fn load_multiplayer_battle_idols(world: &mut World, this: Entity) {
    let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
    let player_battle_idol_id_key = format!("{player_prefix}BattleIdolId");
    let player_battle_idol_id = match_string_preference(world, this, &player_battle_idol_id_key);
    let enemy_battle_idol_id_key = format!("{enemy_prefix}BattleIdolId");
    let enemy_battle_idol_id = match_string_preference(world, this, &enemy_battle_idol_id_key);

    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.enemy_battle_state.battle_struct_xml_episode_node_def = None;
        manager.player_battle_state.battle_struct_xml_episode_node_def = None;
    }

    for slot in 0..BATTLE_IDOL_COUNT {
        let enemy_copy = ENEMY_BATTLE_EPISODE_COPIES + slot;
        let enemy_copy_id = episode_node_id(world, enemy_copy);
        if ns_string_equal(enemy_copy_id.as_deref(), enemy_battle_idol_id.as_deref()) {
            world.obj_mut::<IdolManager>(this).enemy_battle_state.battle_struct_xml_episode_node_def = Some(enemy_copy);
        }

        let player_copy = PLAYER_BATTLE_EPISODE_COPIES + slot;
        let player_copy_id = episode_node_id(world, player_copy);
        if ns_string_equal(player_copy_id.as_deref(), player_battle_idol_id.as_deref()) {
            world.obj_mut::<IdolManager>(this).player_battle_state.battle_struct_xml_episode_node_def = Some(player_copy);
        }
    }
}

/// `-[IdolManager calculateIdolBattleMultiplayerResult]` @ 0x100031314
pub fn calculate_idol_battle_multiplayer_result(world: &mut World, this: Entity) {
    let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
    let player_battle_idol_id_key = format!("{player_prefix}BattleIdolId");
    remove_match_preference(world, this, &player_battle_idol_id_key);
    let enemy_battle_idol_id_key = format!("{enemy_prefix}BattleIdolId");
    remove_match_preference(world, this, &enemy_battle_idol_id_key);

    calculate_battle_state_results(world, this);
}

/// The realm effects of one side's active idols on the other side, as
/// `updateBattleOfTheGodsMultiPlayer` replays them (both arguments are `PlayerBattleState *`s).
fn calculate_side_realm_effect_results(world: &mut World, this: Entity, battle_state_pointer: usize, opponent_battle_state_pointer: usize) {
    for slot in 0..BATTLE_IDOL_COUNT {
        let idol_battle_state_pointer = idol_battle_state_pointer_in(battle_state_pointer, slot);
        let active_frame_count = idol_battle_state_at(world, this, idol_battle_state_pointer).active_frame_count;
        if active_frame_count > 0 {
            calculate_battle_realm_effect_result(world, this, Some(idol_battle_state_pointer), Some(battle_state_pointer), Some(opponent_battle_state_pointer));
        }
    }
}

/// `-[IdolManager updateBattleOfTheGodsMultiPlayer]` @ 0x1000313f4
///
/// Entering: set up both sides from the match data and, on the opponent's turn, replay the
/// round the opponent resolved; then start the battle. Already in, on my turn: reload the round
/// and show its result or refresh the selection panel.
pub fn update_battle_of_the_gods_multi_player(world: &mut World, this: Entity) {
    let multiplayer_active = world.obj::<IdolManager>(this).multi_player_active;
    if multiplayer_active {
        let my_turn = world.obj::<IdolManager>(this).multiplayer_my_turn;
        if !my_turn {
            return;
        }

        load_multiplayer_battle_idols(world, this);
        load_turn_game_state(world, this);

        let idol_screen_logic = world.obj::<IdolManager>(this).idol_screen_logic;
        if let Some(idol_screen_logic) = idol_screen_logic {
            idol_screen_logic::update_idol_battle_screen(world, idol_screen_logic);
        }

        let can_show_result = can_show_idol_battle_multiplayer_result(world, this);
        if can_show_result {
            show_idol_battle_multiplayer_result(world, this);
            let idol_screen_logic = world.obj::<IdolManager>(this).idol_screen_logic;
            if let Some(idol_screen_logic) = idol_screen_logic {
                idol_screen_logic::close_idol_battle_selection_panel(world, idol_screen_logic);
            }
        } else {
            let idol_screen_logic = world.obj::<IdolManager>(this).idol_screen_logic;
            if let Some(idol_screen_logic) = idol_screen_logic {
                idol_screen_logic::refresh_idol_battle_selection_panel(world, idol_screen_logic);
            }
        }
        return;
    }

    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.multi_player_active = true;
        manager.enemy_struct_xml_episode_node_def = None;
    }
    init_battle_of_the_gods_multi_player(world, this);
    load_multiplayer_battle_idols(world, this);
    load_turn_game_state(world, this);

    let my_turn = world.obj::<IdolManager>(this).multiplayer_my_turn;
    if !my_turn {
        let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
        let player_battle_idol_id_key = format!("{player_prefix}BattleIdolId");
        let player_battle_idol_id = match_string_preference(world, this, &player_battle_idol_id_key);
        let enemy_battle_idol_id_key = format!("{enemy_prefix}BattleIdolId");
        let enemy_battle_idol_id = match_string_preference(world, this, &enemy_battle_idol_id_key);

        if player_battle_idol_id.is_some() && enemy_battle_idol_id.is_some() {
            calculate_idol_battle_multiplayer_result(world, this);
            let someone_died = calculate_battle_attack_results(world, this);
            if !someone_died {
                handle_idol_freeze(world, this, Some(PLAYER_BATTLE_STATE), Some(ENEMY_BATTLE_STATE));
                handle_idol_freeze(world, this, Some(ENEMY_BATTLE_STATE), Some(PLAYER_BATTLE_STATE));
                calculate_side_realm_effect_results(world, this, PLAYER_BATTLE_STATE, ENEMY_BATTLE_STATE);
                calculate_side_realm_effect_results(world, this, ENEMY_BATTLE_STATE, PLAYER_BATTLE_STATE);
            }

            let battle_complete = is_battle_complete(world, this);
            if battle_complete {
                let enemy_resurrects = match instance(world) {
                    Some(idol_manager) => should_enemy_resurrect(world, idol_manager),
                    None => false,
                };
                let player_resurrects = match instance(world) {
                    Some(idol_manager) => should_player_resurrect(world, idol_manager),
                    None => false,
                };

                if !player_resurrects && !enemy_resurrects {
                    stop_multiplayer(world, this);
                    show_enemy_panel_multiplayer_alert(world, this, BATTLE_COMPLETE_TITLE, WAITING_TO_VIEW_RESULTS);
                    return;
                }
                if enemy_resurrects {
                    perform_resurrection_with_arg(world, this, Some(ENEMY_BATTLE_STATE));
                }
                if player_resurrects {
                    perform_resurrection_with_arg(world, this, Some(PLAYER_BATTLE_STATE));
                }
            }
        }
    }

    close_idol_screen_now(world, this);
    let temple_door = temple_door_logic::instance(world);
    if let Some(temple_door) = temple_door {
        temple_door_logic::start_battle(world, temple_door);
    }
}

/// `-[IdolManager showIdolBattleMultiplayerResult]` @ 0x100031848
pub fn show_idol_battle_multiplayer_result(world: &mut World, this: Entity) {
    calculate_idol_battle_multiplayer_result(world, this);
    play_temple_door_battle_sequence(world);
}

/// `-[IdolManager setPlayerRandomFrozenIdolId:enemyRandomFrozenIdolId:]` @ 0x10003188c
pub fn set_player_random_frozen_idol_id(world: &mut World, this: Entity, player_random_frozen_idol_id: Option<&str>, enemy_random_frozen_idol_id: Option<&str>) {
    let mut manager = world.obj_mut::<IdolManager>(this);
    manager.player_random_frozen_idol_id = player_random_frozen_idol_id.map(str::to_owned);
    manager.enemy_random_frozen_idol_id = enemy_random_frozen_idol_id.map(str::to_owned);
}

/// The episode id of the idol `chooseFrozenIdolBattleState:useRandomFrozenIdolIds:` picked.
fn chosen_frozen_idol_id(world: &mut World, this: Entity, chosen: Option<usize>) -> Option<String> {
    // PORT: with three or more frozen idols on a side the choice is NULL and the original
    // dereferences it (a crash); the id is nil instead (docs/re/managers-platform.md §20.6).
    let chosen = chosen?;
    idol_battle_state_episode_id(world, this, chosen)
}

/// `-[IdolManager onBattleIdolSelectedMultiplayer:]` @ 0x100031920
///
/// `battle_idol` is the chosen idol's episode node. Returns whether the round resolves here (the
/// opponent had already chosen).
pub fn on_battle_idol_selected_multiplayer(world: &mut World, this: Entity, battle_idol: Option<usize>) -> bool {
    world.obj_mut::<IdolManager>(this).player_battle_state.battle_struct_xml_episode_node_def = battle_idol;

    let (player_prefix, enemy_prefix) = preference_prefixes(world, this);
    let player_battle_idol_id_key = format!("{player_prefix}BattleIdolId");
    let enemy_battle_idol_id_key = format!("{enemy_prefix}BattleIdolId");
    let enemy_battle_idol_id = match_string_preference(world, this, &enemy_battle_idol_id_key);

    let Some(enemy_battle_idol_id) = enemy_battle_idol_id else {
        let my_turn = world.obj::<IdolManager>(this).multiplayer_my_turn;
        if my_turn {
            world.obj_mut::<IdolManager>(this).multiplayer_my_turn = false;
            let player_battle_node = world.obj::<IdolManager>(this).player_battle_state.battle_struct_xml_episode_node_def;
            let player_battle_node = required(player_battle_node);
            let player_battle_idol_id = episode_node_id(world, player_battle_node);
            set_match_string_preference(world, this, &player_battle_idol_id_key, player_battle_idol_id.as_deref());
            save_turn_game_state(world, this);
            end_turn(world, this);
        }
        return false;
    };

    let player_battle_node = world.obj::<IdolManager>(this).player_battle_state.battle_struct_xml_episode_node_def;
    let player_battle_node = required(player_battle_node);
    let player_battle_idol_id = episode_node_id(world, player_battle_node);
    set_match_string_preference(world, this, &player_battle_idol_id_key, player_battle_idol_id.as_deref());

    let player_frozen_choice = choose_frozen_idol_battle_state(world, this, Some(PLAYER_BATTLE_STATE), false);
    let enemy_frozen_choice = choose_frozen_idol_battle_state(world, this, Some(ENEMY_BATTLE_STATE), false);
    let player_random_frozen_idol_id = chosen_frozen_idol_id(world, this, player_frozen_choice);
    let enemy_random_frozen_idol_id = chosen_frozen_idol_id(world, this, enemy_frozen_choice);
    set_player_random_frozen_idol_id(world, this, player_random_frozen_idol_id.as_deref(), enemy_random_frozen_idol_id.as_deref());

    world.obj_mut::<IdolManager>(this).multiplayer_my_turn = false;
    save_turn_game_state(world, this);
    end_turn(world, this);

    for slot in 0..BATTLE_IDOL_COUNT {
        let enemy_copy = ENEMY_BATTLE_EPISODE_COPIES + slot;
        let enemy_copy_id = episode_node_id(world, enemy_copy);
        if ns_string_equal(enemy_copy_id.as_deref(), Some(&enemy_battle_idol_id)) {
            world.obj_mut::<IdolManager>(this).enemy_battle_state.battle_struct_xml_episode_node_def = Some(enemy_copy);
        }
    }

    remove_match_preference(world, this, &player_battle_idol_id_key);
    remove_match_preference(world, this, &enemy_battle_idol_id_key);
    calculate_battle_state_results(world, this);
    play_temple_door_battle_sequence(world);
    true
}

/// `FUN_100031ed0`: the finished match ended.
fn game_over_match_ended(world: &mut World, this: Entity, _error: Option<String>) {
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(None));
}

/// `-[IdolManager onMultiplayerGameOver]` @ 0x100031c48
///
/// On my turn the match ends here: the enemy alive, I lost; the enemy dead, I won, or tied when
/// I am dead too.
pub fn on_multiplayer_game_over(world: &mut World, this: Entity) {
    stop_multiplayer(world, this);

    let manager = world.obj::<IdolManager>(this);
    if !manager.multiplayer_my_turn {
        return;
    }
    let player_health = manager.player_battle_state.health;
    let enemy_health = manager.enemy_battle_state.health;
    let active_match = manager.active_match;

    let (local_outcome, other_outcome) = if enemy_health > 0.0 {
        (MATCH_OUTCOME_LOST, MATCH_OUTCOME_WON)
    } else if player_health > 0.0 {
        (MATCH_OUTCOME_WON, MATCH_OUTCOME_LOST)
    } else {
        (MATCH_OUTCOME_TIED, MATCH_OUTCOME_TIED)
    };

    let participants = match_participants(world, active_match);
    for participant in participants {
        let player_id = local_player_id(world);
        let participant_id = participant_player_id(world, Some(participant));
        let outcome = if ns_string_equal(player_id.as_deref(), participant_id.as_deref()) {
            local_outcome
        } else {
            other_outcome
        };
        set_participant_match_outcome(world, participant, outcome);
    }

    let match_data = match_preferences_data(world, this);
    let Some(game_match) = active_match else {
        return;
    };
    let completion: ErrorHandler = Box::new(move |world: &mut World, error: Option<String>| {
        game_over_match_ended(world, this, error);
    });
    end_match_in_turn_with_match_data(world, game_match, match_data, completion);
}

// ---- The match -----------------------------------------------------------------------------------

/// `FUN_100032130`: the quit match ended.
fn cancelled_match_ended(world: &mut World, this: Entity, _error: Option<String>) {
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(None));
}

/// `-[IdolManager cancelMatch:]` @ 0x100031f80
///
/// Every participant quits, and the match ends with the current data.
pub fn cancel_match(world: &mut World, this: Entity, game_match: Entity) {
    let participants = match_participants(world, Some(game_match));
    for participant in participants {
        set_participant_match_outcome(world, participant, MATCH_OUTCOME_QUIT);
    }

    // `[mMatchPreferences dataFromPreferences] ?: [NSData data]`.
    let match_data = match_preferences_data(world, this);
    let match_data = match_data.unwrap_or_default();
    let completion: ErrorHandler = Box::new(move |world: &mut World, error: Option<String>| {
        cancelled_match_ended(world, this, error);
    });
    end_match_in_turn_with_match_data(world, game_match, Some(match_data), completion);
}

/// How `setActiveMatch:` ends for an open Battle of the Gods match.
enum OpenMatchOutcome {
    /// `updateBattleOfTheGodsMultiPlayer` ran.
    Playing,

    /// The opponent has not accepted yet: the "Invite Sent" alert, the match released.
    InviteSent,

    /// The opponent plays a newer version: the "Please Update" alert, the match released.
    UpdateRequired,
}

/// `setActiveMatch:` on the opponent's turn: watch the opponent's round once they accepted.
fn set_active_match_their_turn(world: &mut World, this: Entity) -> OpenMatchOutcome {
    let recipient_alias = match_string_preference(world, this, RECIPIENT_ALIAS_KEY);
    if recipient_alias.is_none() {
        return OpenMatchOutcome::InviteSent;
    }
    if is_game_version_newer(world, this) {
        return OpenMatchOutcome::UpdateRequired;
    }

    world.obj_mut::<IdolManager>(this).multiplayer_my_turn = false;
    update_battle_of_the_gods_multi_player(world, this);
    OpenMatchOutcome::Playing
}

/// The local player's four selected idols, written under `id_keys` / `level_keys`.
fn write_selected_idols(world: &mut World, this: Entity, id_keys: &[&str; BATTLE_IDOL_COUNT], level_keys: &[&str; BATTLE_IDOL_COUNT]) {
    for slot in 0..BATTLE_IDOL_COUNT {
        let selected = world.obj::<IdolManager>(this).selected_struct_xml_episode_node_def_array[slot];
        let selected = required(selected);

        let selected_id = episode_node_id(world, selected);
        set_match_string_preference(world, this, id_keys[slot], selected_id.as_deref());

        let selected_level = episode_table(world).nodes[selected].level;
        set_match_int_preference(world, this, level_keys[slot], selected_level);
    }
}

/// `setActiveMatch:` on my turn: create the match, accept it, or play my round.
fn set_active_match_my_turn(world: &mut World, this: Entity, game_match: Entity) -> OpenMatchOutcome {
    set_match_message(world, game_match, BATTLE_OF_THE_GODS_MATCH_MESSAGE);

    let challenger_alias = match_string_preference(world, this, CHALLENGER_ALIAS_KEY);
    if challenger_alias.is_none() {
        // A new match: I challenge.
        set_match_string_preference(world, this, GAME_VERSION_KEY, Some(BUNDLE_VERSION));
        set_match_string_preference(world, this, GAME_CATEGORY_KEY, Some(BATTLE_OF_THE_GODS_CATEGORY));
        let alias = local_player_alias(world);
        let player_id = local_player_id(world);
        set_match_string_preference(world, this, CHALLENGER_ALIAS_KEY, alias.as_deref());
        set_match_string_preference(world, this, CHALLENGER_ID_KEY, player_id.as_deref());
        write_selected_idols(world, this, &CHALLENGER_IDOL_ID_KEYS, &CHALLENGER_IDOL_LEVEL_KEYS);
        end_turn(world, this);
        return OpenMatchOutcome::InviteSent;
    }

    let recipient_alias = match_string_preference(world, this, RECIPIENT_ALIAS_KEY);
    if recipient_alias.is_none() {
        // I accept the challenge. The match keeps the higher version.
        if is_game_version_newer(world, this) {
            return OpenMatchOutcome::UpdateRequired;
        }
        set_match_string_preference(world, this, GAME_VERSION_KEY, Some(BUNDLE_VERSION));
        let alias = local_player_alias(world);
        let player_id = local_player_id(world);
        set_match_string_preference(world, this, RECIPIENT_ALIAS_KEY, alias.as_deref());
        set_match_string_preference(world, this, RECIPIENT_ID_KEY, player_id.as_deref());
        write_selected_idols(world, this, &RECIPIENT_IDOL_ID_KEYS, &RECIPIENT_IDOL_LEVEL_KEYS);
    }

    if is_game_version_newer(world, this) {
        return OpenMatchOutcome::UpdateRequired;
    }

    world.obj_mut::<IdolManager>(this).multiplayer_my_turn = true;
    update_battle_of_the_gods_multi_player(world, this);
    OpenMatchOutcome::Playing
}

/// `setActiveMatch:` for an open match.
fn set_open_active_match(world: &mut World, this: Entity, game_match: Entity) {
    let participants = match_participants(world, Some(game_match));
    let mut nobody_quit = true;
    for participant in &participants {
        let outcome = participant_match_outcome(world, *participant);
        nobody_quit &= outcome != MATCH_OUTCOME_QUIT;
    }

    if participants.len() != 2 || !nobody_quit {
        let multiplayer_active = world.obj::<IdolManager>(this).multi_player_active;
        if multiplayer_active {
            stop_multiplayer(world, this);
            close_idol_screen_now(world, this);
            quit_temple_door_game(world);
        }

        // Original quirk, kept: quitGame ends in `setActiveMatch:nil`, so after a running
        // battle the match read here is nil and is not cancelled.
        let active_match = world.obj::<IdolManager>(this).active_match;
        if let Some(active_match) = active_match {
            cancel_match(world, this, active_match);
        }
        show_enemy_panel_multiplayer_alert(world, this, PLAYER_QUIT_TITLE, OTHER_PLAYER_QUIT);
        return;
    }

    let game_category = match_string_preference(world, this, GAME_CATEGORY_KEY);
    let battle_of_the_gods = match game_category.as_deref() {
        Some(game_category) => game_category == BATTLE_OF_THE_GODS_CATEGORY,
        None => true,
    };
    if !battle_of_the_gods {
        show_enemy_panel_multiplayer_alert(world, this, NOT_BATTLE_OF_THE_GODS_TITLE, NOT_BATTLE_OF_THE_GODS_MATCH);
        return;
    }

    let current_participant = match_current_participant(world, Some(game_match));
    let current_player_id = participant_player_id(world, current_participant);
    let player_id = local_player_id(world);
    let my_turn = ns_string_equal(current_player_id.as_deref(), player_id.as_deref());
    let outcome = if my_turn {
        set_active_match_my_turn(world, this, game_match)
    } else {
        set_active_match_their_turn(world, this)
    };

    let (title, message) = match outcome {
        OpenMatchOutcome::Playing => return,
        OpenMatchOutcome::InviteSent => (INVITE_SENT_TITLE, WAITING_TO_ACCEPT_INVITE),
        OpenMatchOutcome::UpdateRequired => (PLEASE_UPDATE_TITLE, OPPONENT_HAS_NEWER_VERSION),
    };
    stop_multiplayer(world, this);
    show_enemy_panel_multiplayer_alert(world, this, title, message);
    world.obj_mut::<IdolManager>(this).active_match = None;
}

/// `-[IdolManager setActiveMatch:]` @ 0x1000321e0
///
/// The alerts of statuses 0, 2 and 3 keep the match.
pub fn set_active_match(world: &mut World, this: Entity, active_match: Option<Entity>) {
    world.obj_mut::<IdolManager>(this).active_match = active_match;
    let Some(game_match) = active_match else {
        return;
    };

    let status = match_status(world, game_match);
    match status {
        MATCH_STATUS_UNKNOWN => {
            stop_multiplayer(world, this);
            show_enemy_panel_multiplayer_alert(world, this, UNKNOWN_RESULT_TITLE, TRY_AGAIN);
        }
        MATCH_STATUS_OPEN => set_open_active_match(world, this, game_match),
        MATCH_STATUS_ENDED => {
            stop_multiplayer(world, this);
            show_enemy_panel_multiplayer_alert(world, this, BATTLE_COMPLETE_TITLE, BATTLE_IS_COMPLETE);
        }
        MATCH_STATUS_MATCHING => {
            stop_multiplayer(world, this);
            show_enemy_panel_multiplayer_alert(world, this, INVITE_SENT_TITLE, WAITING_TO_ACCEPT_INVITE);
        }
        _ => {}
    }
}

/// `-[IdolManager parseMatchData:]` @ 0x100032d0c
pub fn parse_match_data(world: &mut World, this: Entity, data: Option<&[u8]>) {
    let previous_preferences = world.obj::<IdolManager>(this).match_preferences;
    if let Some(previous_preferences) = previous_preferences {
        world.despawn(previous_preferences);
    }

    let match_preferences = pg_engine::prefs::spawn_empty(world);
    world.obj_mut::<IdolManager>(this).match_preferences = Some(match_preferences);
    pg_engine::prefs::load_preferences_from_data(world, match_preferences, data);
}

/// `FUN_100032e00`: `loadActiveMatchData`'s match data arrived. The match handed on is the one
/// active when the data arrives.
fn active_match_data_loaded(world: &mut World, this: Entity, match_data: Option<Vec<u8>>) {
    perform_on_main_thread(world, this, MainThreadCall::ParseMatchData(match_data));
    let active_match = world.obj::<IdolManager>(this).active_match;
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(active_match));
}

/// `-[IdolManager loadActiveMatchData]` @ 0x100032d8c
pub fn load_active_match_data(world: &mut World, this: Entity) {
    world.obj_mut::<IdolManager>(this).load_active_match_data = false;

    let active_match = world.obj::<IdolManager>(this).active_match;
    let Some(active_match) = active_match else {
        return;
    };
    let completion: MatchDataHandler = Box::new(move |world: &mut World, match_data: Option<Vec<u8>>, _error: Option<String>| {
        active_match_data_loaded(world, this, match_data);
    });
    load_match_data(world, active_match, completion);
}

/// `-[IdolManager handleInviteFromGameCenter:]` @ 0x100032e8c
pub fn handle_invite_from_game_center(_world: &mut World, _this: Entity, _invite: Entity) {}

// ---- Match alerts (never shown in 1.48.2: GameServer does not call the inactive-match handlers) ----

/// `[array objectAtIndex:1]` of `showMatchAlert:`'s `[match, alias]` array.
fn match_alert_alias(opponent_alias: Option<&str>) -> &str {
    // Original bug, kept: `arrayWithObjects:match, alias, nil` stops at a nil alias, so
    // `objectAtIndex:1` raises NSRangeException.
    opponent_alias.expect("IdolManager: objectAtIndex:1 of a one-object match alert array")
}

/// `[mBattleAlertLogic activateMatchAlert:match message:message]`.
fn activate_match_alert(world: &mut World, this: Entity, game_match: Entity, message: &str) {
    let battle_alert_logic = world.obj::<IdolManager>(this).battle_alert_logic;
    if let Some(battle_alert_logic) = battle_alert_logic {
        battle_alert_logic::activate_match_alert(world, battle_alert_logic, game_match, message);
    }
}

/// `-[IdolManager showMatchAlertQuitMatch:]` @ 0x100032e90
///
/// The argument is the array `[match, opponent alias]`.
pub fn show_match_alert_quit_match(world: &mut World, this: Entity, game_match: Entity, opponent_alias: Option<&str>) {
    let alias = match_alert_alias(opponent_alias);
    let message = format!("{alias} quit the match");
    activate_match_alert(world, this, game_match, &message);
}

/// `-[IdolManager showMatchAlertMatchComplete:]` @ 0x100032f40
///
/// The argument is the array `[match, opponent alias]`.
pub fn show_match_alert_match_complete(world: &mut World, this: Entity, game_match: Entity, opponent_alias: Option<&str>) {
    let alias = match_alert_alias(opponent_alias);
    let message = format!("Match against {alias} is complete");
    activate_match_alert(world, this, game_match, &message);
}

/// `-[IdolManager showMatchAlertInvite:]` @ 0x100032ff0
///
/// The argument is the array `[match, opponent alias]`.
pub fn show_match_alert_invite(world: &mut World, this: Entity, game_match: Entity, opponent_alias: Option<&str>) {
    let alias = match_alert_alias(opponent_alias);
    let message = format!("{alias} invited you to a match");
    activate_match_alert(world, this, game_match, &message);
}

/// `-[IdolManager showMatchAlertYourTurn:]` @ 0x1000330a0
///
/// The argument is the array `[match, opponent alias]`.
pub fn show_match_alert_your_turn(world: &mut World, this: Entity, game_match: Entity, opponent_alias: Option<&str>) {
    let alias = match_alert_alias(opponent_alias);
    let message = format!("Your turn against {alias}");
    activate_match_alert(world, this, game_match, &message);
}

/// `FUN_100033344`: `showMatchAlert:`'s match data arrived. The alias is the opponent's; the alert
/// says who quit, that the match is complete, that it is my turn, or (no recipient yet) that I am
/// invited.
fn match_alert_data_loaded(world: &mut World, this: Entity, game_match: Entity, someone_quit: bool, match_complete: bool, match_data: Option<Vec<u8>>, error: Option<String>) {
    let preferences = pg_engine::prefs::spawn_empty(world);
    pg_engine::prefs::load_preferences_from_data(world, preferences, match_data.as_deref());
    let recipient_alias = pg_engine::prefs::get_string_preference(world, preferences, RECIPIENT_ALIAS_KEY);
    let challenger_alias = pg_engine::prefs::get_string_preference(world, preferences, CHALLENGER_ALIAS_KEY);
    let challenger_id = pg_engine::prefs::get_string_preference(world, preferences, CHALLENGER_ID_KEY);
    let player_id = local_player_id(world);
    let i_am_challenger = ns_string_equal(challenger_id.as_deref(), player_id.as_deref());
    world.despawn(preferences);

    if error.is_some() {
        return;
    }

    let opponent_alias = if i_am_challenger {
        recipient_alias.clone()
    } else {
        challenger_alias
    };
    let alert = if someone_quit {
        MatchAlert::QuitMatch
    } else if match_complete {
        MatchAlert::MatchComplete
    } else if recipient_alias.is_some() {
        MatchAlert::YourTurn
    } else {
        MatchAlert::Invite
    };
    perform_on_main_thread(world, this, MainThreadCall::ShowMatchAlert(alert, game_match, opponent_alias));
}

/// `-[IdolManager showMatchAlert:]` @ 0x100033150
pub fn show_match_alert(world: &mut World, this: Entity, game_match: Entity) {
    let participants = match_participants(world, Some(game_match));
    let mut someone_quit = false;
    let mut match_complete = false;
    for participant in participants {
        let outcome = participant_match_outcome(world, participant);
        someone_quit |= outcome == MATCH_OUTCOME_QUIT;
        if outcome == MATCH_OUTCOME_TIED || outcome == MATCH_OUTCOME_WON || outcome == MATCH_OUTCOME_LOST {
            match_complete = true;
        }
    }

    let completion: MatchDataHandler = Box::new(move |world: &mut World, match_data: Option<Vec<u8>>, error: Option<String>| {
        match_alert_data_loaded(world, this, game_match, someone_quit, match_complete, match_data, error);
    });
    load_match_data(world, game_match, completion);
}

// ---- Turn events ---------------------------------------------------------------------------------

/// `-[IdolManager handleTurnEventForActiveMatch:]` @ 0x1000335c4
///
/// A turn of the active match: reload it now in the battle screen, else only remember it.
pub fn handle_turn_event_for_active_match(world: &mut World, this: Entity, game_match: Entity) -> bool {
    let active_match = world.obj::<IdolManager>(this).active_match;
    let Some(active_match) = active_match else {
        return false;
    };

    let active_match_id = match_id(world, active_match);
    let event_match_id = match_id(world, game_match);
    if !ns_string_equal(active_match_id.as_deref(), event_match_id.as_deref()) {
        return false;
    }

    let screen_mode = idol_screen_mode(world, this);
    if screen_mode == IDOL_SCREEN_MODE_BATTLE {
        load_active_match_data(world, this);
        return true;
    }
    world.obj_mut::<IdolManager>(this).load_active_match_data = true;
    true
}

/// `-[IdolManager handleTurnEventNotificationForInactiveMatch:]` @ 0x1000336cc
pub fn handle_turn_event_notification_for_inactive_match(world: &mut World, this: Entity, game_match: Entity) -> bool {
    show_match_alert(world, this, game_match);
    true
}

/// `-[IdolManager handleMatchEndedForActiveMatch:]` @ 0x100033748
pub fn handle_match_ended_for_active_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// `-[IdolManager handleMatchEndedNotificationForInactiveMatch:]` @ 0x1000337a8
pub fn handle_match_ended_notification_for_inactive_match(world: &mut World, this: Entity, game_match: Entity) -> bool {
    show_match_alert(world, this, game_match);
    true
}

// ---- Authentication --------------------------------------------------------------------------------

/// `-[IdolManager onGameServerAuthenticationStarted]` @ 0x100033830
pub fn on_game_server_authentication_started(world: &mut World, this: Entity) {
    let mut manager = world.obj_mut::<IdolManager>(this);
    manager.game_center_authentication_complete = false;
    manager.game_center_not_available = false;
}

/// `-[IdolManager onGameServerAuthenticationComplete]` @ 0x10003384c
///
/// A player change (or no player) ends a running multiplayer battle; a queued `openMatchPanel`
/// follows.
pub fn on_game_server_authentication_complete(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.game_center_authentication_complete = false;
        manager.game_center_not_available = false;
    }

    let game_server = game_server::instance(world);
    let multiplayer_available = match game_server {
        Some(game_server) => game_server::is_turn_based_multiplayer_available(world, game_server),
        None => false,
    };
    if !multiplayer_available {
        world.obj_mut::<IdolManager>(this).game_center_not_available = true;
        return;
    }

    let same_user = if local_player_is_authenticated(world) {
        world.obj_mut::<IdolManager>(this).game_center_authentication_complete = true;
        let player_id = local_player_id(world);
        let active_player_id = world.obj::<IdolManager>(this).active_player_id.clone();
        let same_user = ns_string_equal(active_player_id.as_deref(), player_id.as_deref());
        if !same_user {
            world.obj_mut::<IdolManager>(this).active_player_id = player_id;
        }
        same_user
    } else {
        world.obj_mut::<IdolManager>(this).game_center_not_available = true;
        false
    };

    if !same_user {
        crate::controller::perform_selector_on_main_thread(world, this, on_game_center_authentication_changed);
    }
    crate::controller::perform_selector_on_main_thread(world, this, finish_game_center_authentication);

    let open_match_panel_queued = world.obj::<IdolManager>(this).open_match_panel_on_authentication_complete;
    if !open_match_panel_queued {
        return;
    }
    world.obj_mut::<IdolManager>(this).open_match_panel_on_authentication_complete = false;
    crate::controller::perform_selector_on_main_thread(world, this, open_match_panel);
}

/// `-[IdolManager finishGameCenterAuthentication]` @ 0x100033a3c
pub fn finish_game_center_authentication(world: &mut World, this: Entity) {
    let screen_mode = idol_screen_mode(world, this);
    if screen_mode != IDOL_SCREEN_MODE_BATTLE {
        return;
    }

    let manager = world.obj::<IdolManager>(this);
    let opponents_turn = manager.multi_player_active && !manager.multiplayer_my_turn;
    if opponents_turn {
        load_active_match_data(world, this);
    }
}

/// `-[IdolManager onGameCenterAuthenticationChanged]` @ 0x100033ab8
pub fn on_game_center_authentication_changed(world: &mut World, this: Entity) {
    let multiplayer_active = world.obj::<IdolManager>(this).multi_player_active;
    if !multiplayer_active {
        return;
    }
    close_idol_screen_now(world, this);
    quit_temple_door_game(world);
}

/// `-[IdolManager onGameServerAuthenticationViewControllerClosed]` @ 0x100033b20
pub fn on_game_server_authentication_view_controller_closed(_world: &mut World, _this: Entity) {}

// ---- Matchmaking -----------------------------------------------------------------------------------

/// `-[IdolManager openMatchPanel]` @ 0x100033b24
///
/// The idol screen's multiplayer button: the matchmaker sheet once Game Center is ready.
pub fn open_match_panel(world: &mut World, this: Entity) {
    let manager = world.obj::<IdolManager>(this);
    let game_center_not_available = manager.game_center_not_available;
    let game_center_authentication_complete = manager.game_center_authentication_complete;

    if game_center_not_available {
        show_enemy_panel_multiplayer_alert(world, this, MULTIPLAYER_NOT_AVAILABLE_TITLE, LOG_INTO_GAME_CENTER);
        return;
    }

    if !game_center_authentication_complete {
        world.obj_mut::<IdolManager>(this).open_match_panel_on_authentication_complete = true;
        return;
    }

    set_active_match(world, this, None);

    // `[request respondsToSelector:@selector(inviteMessage)]`: GameKit of iOS 5 or later
    // (docs/re/managers-platform.md §18.1).
    let match_request = GkMatchRequest {
        player_group: BATTLE_OF_THE_GODS_PLAYER_GROUP,
        max_players: MATCH_PLAYER_COUNT,
        min_players: MATCH_PLAYER_COUNT,
        invite_message: Some(INVITE_MESSAGE.to_owned()),
    };
    let view_controller = spawn_matchmaker_view_controller(world, match_request, this);

    let listener = idol_manager_listener(world, this);
    if let Some(listener) = listener {
        crate::controller::present_modal_view_controller(world, listener, view_controller);
    }
}

/// `FUN_100033de4`: the match picked in the matchmaker sheet has its data.
fn found_match_data_loaded(world: &mut World, this: Entity, found_match: Entity, match_data: Option<Vec<u8>>) {
    perform_on_main_thread(world, this, MainThreadCall::ParseMatchData(match_data));
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(Some(found_match)));
}

/// `-[IdolManager turnBasedMatchmakerViewController:didFindMatch:]` @ 0x100033d44
pub fn turn_based_matchmaker_view_controller_did_find_match(world: &mut World, this: Entity, _view_controller: Entity, found_match: Entity) {
    let completion: MatchDataHandler = Box::new(move |world: &mut World, match_data: Option<Vec<u8>>, _error: Option<String>| {
        found_match_data_loaded(world, this, found_match, match_data);
    });
    load_match_data(world, found_match, completion);

    dismiss_listener_modal_view_controller(world, this);
    // Then `[viewController release]`: the sheet's entity stays with the match service.
}

/// `-[IdolManager turnBasedMatchmakerViewController:didFailWithError:]` @ 0x100033eb4
pub fn turn_based_matchmaker_view_controller_did_fail_with_error(world: &mut World, this: Entity, _view_controller: Entity, _error: Entity) {
    set_active_match(world, this, None);
    show_enemy_panel_multiplayer_alert(world, this, ERROR_OCCURED_TITLE, ERROR_LOADING_MATCH);
    dismiss_listener_modal_view_controller(world, this);
}

/// `-[IdolManager turnBasedMatchmakerViewController:playerQuitForMatch:]` @ 0x100033f40
pub fn turn_based_matchmaker_view_controller_player_quit_for_match(world: &mut World, this: Entity, _view_controller: Entity, game_match: Entity) {
    set_active_match(world, this, None);
    cancel_match(world, this, game_match);
    show_enemy_panel_multiplayer_alert(world, this, PLAYER_QUIT_TITLE, OTHER_PLAYER_QUIT);
    dismiss_listener_modal_view_controller(world, this);
}

/// `-[IdolManager turnBasedMatchmakerViewControllerWasCancelled:]` @ 0x100033ff0
pub fn turn_based_matchmaker_view_controller_was_cancelled(world: &mut World, this: Entity, _view_controller: Entity) {
    set_active_match(world, this, None);
    dismiss_listener_modal_view_controller(world, this);
}

/// `FUN_10003424c`: the turn passed to the opponent; a failure ends the battle.
fn end_turn_completed(world: &mut World, this: Entity, error: Option<String>) {
    if error.is_none() {
        return;
    }
    close_idol_screen_now(world, this);
    quit_temple_door_game(world);
    show_enemy_panel_multiplayer_alert(world, this, ERROR_OCCURED_TITLE, ERROR_LOADING_MATCH);
}

/// `-[IdolManager endTurn]` @ 0x100034050
///
/// The turn and the match data go to the participant that is not the local player.
pub fn end_turn(world: &mut World, this: Entity) {
    let player_id = local_player_id(world);
    let active_match = world.obj::<IdolManager>(this).active_match;
    let Some(game_match) = active_match else {
        return;
    };

    let participants = match_participants(world, Some(game_match));
    for participant in participants {
        let participant_id = participant_player_id(world, Some(participant));
        if ns_string_equal(player_id.as_deref(), participant_id.as_deref()) {
            continue;
        }

        let match_data = match_preferences_data(world, this);
        let completion: ErrorHandler = Box::new(move |world: &mut World, error: Option<String>| {
            end_turn_completed(world, this, error);
        });
        end_turn_with_next_participant(world, game_match, participant, match_data, completion);
    }
}

/// `FUN_1000345bc`: the local player left the match.
fn left_match(world: &mut World, this: Entity, _error: Option<String>) {
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(None));
}

/// `-[IdolManager leaveMatch]` @ 0x100034354
///
/// Nothing sends this selector in 1.48.2 (docs/re/managers-platform.md §20.7).
pub fn leave_match(world: &mut World, this: Entity) {
    let active_match = world.obj::<IdolManager>(this).active_match;
    let current_participant = match_current_participant(world, active_match);
    let current_player_id = participant_player_id(world, current_participant);
    let player_id = local_player_id(world);
    let Some(game_match) = active_match else {
        return;
    };

    if !ns_string_equal(current_player_id.as_deref(), player_id.as_deref()) {
        let completion: ErrorHandler = Box::new(move |world: &mut World, error: Option<String>| {
            left_match(world, this, error);
        });
        participant_quit_out_of_turn(world, game_match, MATCH_OUTCOME_LOST, completion);
        return;
    }

    let participants = match_participants(world, Some(game_match));
    for participant in participants {
        let participant_id = participant_player_id(world, Some(participant));
        if ns_string_equal(player_id.as_deref(), participant_id.as_deref()) {
            continue;
        }

        let match_data = match_data(world, game_match);
        let completion: ErrorHandler = Box::new(move |world: &mut World, error: Option<String>| {
            left_match(world, this, error);
        });
        participant_quit_in_turn(world, game_match, MATCH_OUTCOME_LOST, participant, match_data, completion);
    }
}
