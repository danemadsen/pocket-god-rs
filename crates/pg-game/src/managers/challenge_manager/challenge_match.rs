//! `ChallengeManager`: methods in this part of the class.

use std::collections::VecDeque;

use pg_engine::display::group::get_display_object;
use pg_engine::library::Library;

use super::*;

/// `-[ChallengeManager id]`.
const CLASS_ID: &str = "ChallengeManager";

/// `getChallengeType:` for a `GameId` it does not know.
pub const CHALLENGE_TYPE_NOT_RECOGNIZED: i32 = 17;

/// The last challenge type: `updateChallengeType:` keeps the type in 0…15.
const LAST_CHALLENGE_TYPE: i32 = 15;

/// What the per-type string getters answer for a type outside 0…15.
const UNDEFINED: &str = "Undefined";

/// `getChallengeId:` (CFStrings `0x100386ed0 + 0x20·i`): the `GameId` stored in the match data.
const CHALLENGE_IDS: [&str; 16] = [
    "1", "11", "21", "31", "41", "51", "933", "2333", "2503", "3793", "6733", "10064", "18482", "20753", "30903",
    "40000",
];

/// `getChallengeIconId:`: the icon texture of each type.
const CHALLENGE_ICON_IDS: [&str; 16] = [
    "AchievementIconCoconutbounce",
    "AchievementIconHurricanezaps",
    "AchievementIconBirdbomb",
    "AchievementIconSharksnaps",
    "AchievementIconOogajump",
    "AchievementIconPiranhaFeed",
    "AchievementSharkBlasts",
    "AchievementIconTrex",
    "AchievementIconSpider",
    "AchievementIceMonster",
    "AchievementPainDrain",
    "AchievementTheRuns",
    "AchievementKonkeyDong",
    "AchievementMoronPests",
    "AchievementDanceExecution",
    "AchievementLongTimeNoPee",
];

/// `getChallengeName:`.
const CHALLENGE_NAMES: [&str; 16] = [
    "Coconut Bounce",
    "Hurricane Zap",
    "Bird Bomb",
    "Shark Snap",
    "Ooga Jump",
    "Bait Master",
    "Shark Blast",
    "T-Rex Hunt",
    "Spider Hunt",
    "Ice Monster Battle",
    "Pain Drain",
    "The Runs",
    "Konkey Dong",
    "Moron Pests",
    "Dance Dance Execution",
    "Call of Booty",
];

/// `getChallengeGameCenterGroupId:` (i32 table `0x10029d3a0`).
const CHALLENGE_GAME_CENTER_GROUP_IDS: [i32; 16] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];

/// `getChallengeGameCenterGroupId:` for a type outside 0…15.
const DEFAULT_GAME_CENTER_GROUP_ID: i32 = 3;

/// `getLowerCaseChallengeName:` (nothing calls it; `"ooga jump:"` is the original's).
const LOWER_CASE_CHALLENGE_NAMES: [&str; 16] = [
    "coconut bounce",
    "hurricane zap",
    "bird bomb",
    "shark snap",
    "ooga jump:",
    "bait master",
    "shark blast",
    "t-rex hunt",
    "spider hunt",
    "ice monster battle",
    "pain drain",
    "the runs",
    "konkey dong",
    "moron pests",
    "dance dance execution",
    "call of booty",
];

/// `getChallengeText:` (nothing calls it).
const CHALLENGE_TEXTS: [&str; 16] = [
    "How many Coconut Bounces can you get?",
    "How many Hurricane Zaps can you get?",
    "How many Bird Bombs can you get?",
    "How many Shark Snaps can you get?",
    "How high can you get on Ooga Jump?",
    "How many Piranha can you serve in Bait Master?",
    "How many Pygmies can you blast in Shark Blast?",
    "How many spears does it take to kill the T-Rex?",
    "How many spears does it take to kill the Spider?",
    "How many pygmies does it take to kill the Ice Monster?",
    "How deep can you go in the Pain Drain?",
    "How far can you go in The Runs?",
    "How high can you get in Konkey Dong?",
    "How far can you go in Moron Pests?",
    "How many points can get in Dance Dance Execution?",
    "How many can you collect in Call of Booty?",
];

/// `getChallengeUserMessage:` (nothing calls it).
const CHALLENGE_USER_MESSAGES: [&str; 16] = [
    "Just try to kick my butt at Coconut Bounces!",
    "Just try to beat me at Hurricane Zaps!",
    "Just try to kick my butt at Bird Bombs!",
    "Just try to kick my butt at Shark Snaps!",
    "Just try to kick my butt at Ooga Jump!",
    "Just try to kick my butt at Bait Master!",
    "Just try to kick my butt at Shark Blast!",
    "Just try to kick my butt at T-Rex Hunt!",
    "Just try to kick my butt at Spider Hunt!",
    "Just try to kick my butt at Ice Monster Battle!",
    "Just try to kick my butt in the Pain Drain!",
    "Just try to kick my butt in The Runs!",
    "Just try to kick my butt in Konkey Dong!",
    "Just try to kick my butt in Moron Pests!",
    "Just try to kick my butt in Dance Dance Execution!",
    "Just try to kick my butt in Call of Booty!",
];

/// `getChallengeStartDescription:`.
const CHALLENGE_START_DESCRIPTIONS: [&str; 16] = [
    "You have 1 minute to bounce as many coconuts as you can!",
    "You have 1 minute to zap as many pygmies in a hurricane as you can!",
    "You have 1 minute to poo on as many pygmies as you can!",
    "You have 1 minute to tease as many sharks as you can!",
    "You have 1 minute to jump as many Oogles as you can!",
    "You have 1 minute to serve as many piranhas as you can!",
    "You have 1 minute to blast as many pygmies as you can!",
    "You have 1 minute to hunt the T-Rex!",
    "You have 1 minute to hunt the Spider!",
    "You have 1 minute to battle the Ice Monster!",
    "You have 1 minute to dive as many Oogles as you can!",
    "You have 1 minute to travel as many Oogles as you can!",
    "You have 1 minute to score as high as you can!",
    "You have 1 minute to travel as many Oogles as you can!",
    "You have 1 minute to get as many points as you can!",
    "You have 1 minute to get as much booty as you can!",
];

/// `getChallengeRecapDescription:`: formats taking the attempt's best score.
const CHALLENGE_RECAP_DESCRIPTIONS: [&str; 16] = [
    "You've scored %d coconut bounces!",
    "You zapped %d pygmies!",
    "You pooed on %d pygmies!",
    "You teased %d sharks!",
    "You jumped %d Oogles!",
    "You served %d piranhas!",
    "You blasted %d pygmies!",
    "Used %d spears!",
    "Used %d spears!",
    "Used %d pygmies!",
    "You dove %d Oogles!",
    "You ran %d Oogles!",
    "You scored %d Points!",
    "You ran %d Oogles!",
    "You scored %d Points!",
    "You collected %d booties!",
];

/// The match data keys (`docs/re/audio-prefs-text.md` §2.6.5).
const GAME_CATEGORY_KEY: &str = "GameCategory";
const MINI_CHALLENGE_CATEGORY: &str = "MiniChallenge";
const GAME_ID_KEY: &str = "GameId";
const CHALLENGER_ALIAS_KEY: &str = "ChallengerAlias";
const CHALLENGER_ID_KEY: &str = "ChallengerId";
const CHALLENGER_GAME_SCORE_KEY: &str = "ChallengerGameScore";
const RECIPIENT_ALIAS_KEY: &str = "RecipientAlias";
const RECIPIENT_ID_KEY: &str = "RecipientId";
const RECIPIENT_GAME_SCORE_KEY: &str = "RecipientGameScore";

/// Sequences this part runs.
const OPEN_CHALLENGE_SCREEN_SEQUENCE: &str = "OpenChallengeScreen";
const CLOSE_CHALLENGE_SCREEN_SEQUENCE: &str = "CloseChallengeScreen";
const UPDATE_CHALLENGE_TYPE_SEQUENCE: &str = "UpdateChallengeType";
const SEND_CHALLENGE_DIALOG_SEQUENCE: &str = "SendChallengeDialog";
const RESPOND_TO_CHALLENGE_DIALOG_SEQUENCE: &str = "RespondToChallengeDialog";
const CHALLENGE_ERROR_SEQUENCE: &str = "ChallengeError";
const CHALLENGE_DIALOG_ERROR_SEQUENCE: &str = "ChallengeDialogError";
const SEND_CHALLENGE_CANCEL_SEQUENCE: &str = "SendChallengeCancel";
const SEND_CHALLENGE_START_SEQUENCE: &str = "SendChallengeStart";
const SUBMIT_CHALLENGE_CLOSE_SEQUENCE: &str = "SubmitChallengeClose";
const RESPOND_CHALLENGE_CANCEL_SEQUENCE: &str = "RespondChallengeCancel";
const RESPOND_CHALLENGE_START_SEQUENCE: &str = "RespondChallengeStart";
const SUBMIT_CHALLENGE_RESULT_CLOSE_SEQUENCE: &str = "SubmitChallengeResultClose";
const SUBMIT_CHALLENGE_RESULT_RECHALLENGE_SEQUENCE: &str = "SubmitChallengeResultRechallenge";
const CHALLENGE_CLOSE_ERROR_SEQUENCE: &str = "ChallengeCloseError";
const CHALLENGE_DIALOG_ERROR_CLOSE_SEQUENCE: &str = "ChallengeDialogErrorClose";

/// The display groups this part writes into, and their objects.
const CHALLENGE_ERROR_GROUP: &str = "ChallengeError";
const CHALLENGE_CREATE_PANEL_GROUP: &str = "ChallengeCreatePanel";
const CHALLENGE_START_SEND_SCREEN_GROUP: &str = "ChallengeStartSendScreen";
const CHALLENGE_CONFIRM_SEND_SCREEN_GROUP: &str = "ChallengeConfirmSendScreen";
const CHALLENGE_START_RESPOND_SCREEN_GROUP: &str = "ChallengeStartRespondScreen";
const CHALLENGE_RESULTS_SCREEN_GROUP: &str = "ChallengeResultsScreen";
const CHALLENGE_LOADING_SCREEN_GROUP: &str = "ChallengeLoadingScreen";
const CHALLENGE_TIMER_GROUP: &str = "ChallengeTimer";
const CHALLENGE_ALERT_GROUP: &str = "ChallengeAlert";
const MESSAGE1_OBJECT: &str = "Message1";
const MESSAGE2_OBJECT: &str = "Message2";
const MESSAGE3_OBJECT: &str = "Message3";
const CHALLENGE_TITLE_OBJECT: &str = "ChallengeTitle";
const CHALLENGE_ICON_OBJECT: &str = "ChallengeIcon";
const CHALLENGE_START_MESSAGE_OBJECT: &str = "ChallengeStartMessage";
const CHALLENGE_START_TITLE_OBJECT: &str = "ChallengeStartTitle";
const CHALLENGE_CONFIRM_MESSAGE_OBJECT: &str = "ChallengeConfirmMessage";
const CHALLENGE_CONFIRM_TITLE_OBJECT: &str = "ChallengeConfirmTitle";
const TIMER_OBJECT: &str = "Timer";
const ALERT_NUM_OBJECT: &str = "AlertNum";

/// Message behaviours.
const CANT_OPEN_CHALLENGE_SCREEN_MESSAGE: &str = "CantOpenChallengeScreen";
const SHOW_CHALLENGE_TIMER_MESSAGE: &str = "MessageShowChallengeTimer";

/// The episode event of opening the challenge screen.
const OPEN_CHALLENGE_SCREEN_EVENT: &str = "OpenChallengeScreen";

/// Opponent aliases set before the real one is known.
const UNKNOWN_ALIAS: &str = "Unknown";
const AUTO_MATCH_PLAYER_ALIAS: &str = "an Auto Match Player";

/// The error panel's texts.
const GAME_CENTER_NOT_AVAILABLE: &str = "Game Center not available.";
const CHECK_GAME_CENTER_LOGIN: &str = "Check that you are logged in to Game Center and try again.";
const QUIT_MATCH_CONFIRMED: &str = "Quit match confirmed.";
const GAME_CENTER_ERROR: &str = "Game Center Error.";
const VERIFY_GAME_CENTER_LOGIN: &str = "Verify that you are logged into Game Center.";
const INTERNET_CONNECTION: &str = "and have an internet connection.";
const NOT_A_MINI_CHALLENGE_MATCH: &str = "Not a Mini Game Challenge match.";
const LOAD_IN_BATTLE_OF_THE_GODS: &str = "Load this match in Battle of the Gods.";
const WAITING_FOR_OTHER_PLAYER: &str = "Waiting for other player.";
const OTHER_PLAYER_HAS_NOT_TAKEN_TURN: &str = "Other player has not taken their turn yet.";
const CHALLENGE_NOT_RECOGNIZED: &str = "Mini Game Challenge not recognized.";
const UPDATE_POCKET_GOD: &str = "Make sure you are updated to the latest version of Pocket God.";
const MATCH_HAS_ENDED: &str = "Match has ended.";
const MATCH_HAS_ALREADY_ENDED: &str = "Match has already ended.";
const FINDING_A_MATCH: &str = "Finding a match.";
const NEED_MORE_TIME: &str = "Need more time to find a match for game.";

/// `throwDialogError:`'s (ignored) messages.
const SUBMIT_CHALLENGE_FAIL: &str = "Submit Challenge Fail";
const SUBMIT_CHALLENGE_RESULT_FAIL: &str = "Submit Challenge Result Fail";

/// The results screen's outcome lines.
const RESULT_TIED: &str = "You tied! The results have been recorded.";
const RESULT_WON: &str = "You won! The results have been recorded.";
const RESULT_LOST: &str = "You lost! The results have been recorded.";

/// The confirm screen's title after a forfeit.
const QUIT_AND_FORFEITED: &str = "You quit and forfeited the game.";

/// `mChallengeScoreType`: the hunts (types 7–9) count spears, so the lower score is better.
const HIGHER_SCORE_WINS: i32 = 0;
const LOWER_SCORE_WINS: i32 = 1;

/// The three texts of `throwErrorMessage1:message2:message3:`.
type ErrorMessages = (&'static str, &'static str, &'static str);

// ---- GameKit -----------------------------------------------------------------------------------
//
// PORT: the port has no Game Center (docs/re/managers-platform.md §18). The match code below is
// kept faithful against this small stand-in for the GameKit objects it uses, so a local match
// service (§18.3) can serve it: the service authenticates the [`GkLocalPlayer`], creates the
// match and participant entities, installs [`GameKitService`] and calls the matchmaker delegate
// methods (`turn_based_matchmaker_view_controller_…`). Without one the player is never
// authenticated, `openMatchPanel` shows "Game Center not available." and nothing below runs.

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

/// `[GKLocalPlayer localPlayer]`, as the match service reports it (absent: not authenticated).
#[derive(Resource, Clone, Debug, Default)]
pub struct GkLocalPlayer {
    /// `isAuthenticated`
    pub authenticated: bool,

    /// `playerID`
    pub player_id: Option<String>,

    /// `alias`
    pub alias: Option<String>,
}

/// `GKTurnBasedMatch`: a match entity of the match service.
#[derive(Component, Clone, Debug, Default)]
pub struct GkTurnBasedMatch {
    /// `matchID`
    pub match_id: Option<String>,

    /// `status` (`GKTurnBasedMatchStatus`: 0 unknown, 1 open, 2 ended, 3 matching)
    pub status: i64,

    /// `participants`: `GKTurnBasedParticipant` entities
    pub participants: Vec<Entity>,

    /// `currentParticipant`
    pub current_participant: Option<Entity>,

    /// `message`
    pub message: Option<String>,
}

/// `GKTurnBasedParticipant`.
#[derive(Component, Clone, Debug, Default)]
pub struct GkTurnBasedParticipant {
    /// `playerID` (nil while an auto-match seat is empty)
    pub player_id: Option<String>,

    /// `matchOutcome` (`GKTurnBasedMatchOutcome`)
    pub match_outcome: i64,
}

/// `GKPlayer`, as `loadPlayersForIdentifiers:withCompletionHandler:` reports it.
#[derive(Clone, Debug, Default)]
pub struct GkPlayer {
    pub player_id: Option<String>,
    pub alias: Option<String>,
}

/// `GKMatchRequest`.
#[derive(Clone, Debug, Default)]
pub struct GkMatchRequest {
    pub player_group: u64,
    pub max_players: u64,
    pub min_players: u64,
    pub invite_message: Option<String>,
}

/// `GKTurnBasedMatchmakerViewController`: the matchmaker sheet handed to the listener's
/// `presentModalViewController:`.
#[derive(Component, Clone, Debug, Default)]
pub struct GkTurnBasedMatchmakerViewController {
    pub match_request: GkMatchRequest,

    /// `turnBasedMatchmakerDelegate`
    pub turn_based_matchmaker_delegate: Option<Entity>,
}

/// Completion of `loadMatchDataWithCompletionHandler:`: the match data and the error
/// (its description), `None` for nil.
pub type MatchDataHandler = Box<dyn FnOnce(&mut World, Option<Vec<u8>>, Option<String>) + Send + Sync>;

/// Completion of `loadPlayersForIdentifiers:withCompletionHandler:`: the players and the error.
pub type PlayersHandler = Box<dyn FnOnce(&mut World, Option<Vec<GkPlayer>>, Option<String>) + Send + Sync>;

/// Completion of the turn and match-end requests: the error, `None` for nil.
pub type ErrorHandler = Box<dyn FnOnce(&mut World, Option<String>) + Send + Sync>;

/// GameKit's asynchronous requests, served by the match service that installs this resource.
/// Each one runs its completion handler later, outside the call (GameKit calls back on its own
/// queue).
#[derive(Resource, Clone, Copy)]
pub struct GameKitService {
    /// `-[GKTurnBasedMatch loadMatchDataWithCompletionHandler:]`
    pub load_match_data: fn(&mut World, Entity, MatchDataHandler),

    /// `-[GKTurnBasedMatch endTurnWithNextParticipant:matchData:completionHandler:]`
    pub end_turn_with_next_participant: fn(&mut World, Entity, Entity, Option<Vec<u8>>, ErrorHandler),

    /// `-[GKTurnBasedMatch endMatchInTurnWithMatchData:completionHandler:]`
    pub end_match_in_turn_with_match_data: fn(&mut World, Entity, Option<Vec<u8>>, ErrorHandler),

    /// `+[GKPlayer loadPlayersForIdentifiers:withCompletionHandler:]`
    pub load_players_for_identifiers: fn(&mut World, Vec<String>, PlayersHandler),
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

/// `[match participants]`, copied for enumeration.
fn match_participants(world: &World, game_match: Entity) -> Vec<Entity> {
    match world.get::<GkTurnBasedMatch>(game_match) {
        Some(state) => state.participants.clone(),
        None => Vec::new(),
    }
}

/// `[match currentParticipant]`.
fn match_current_participant(world: &World, game_match: Entity) -> Option<Entity> {
    let state = world.get::<GkTurnBasedMatch>(game_match)?;
    state.current_participant
}

/// `[match setMessage:message]`.
fn set_match_message(world: &mut World, game_match: Entity, message: String) {
    if let Some(mut state) = world.get_mut::<GkTurnBasedMatch>(game_match) {
        state.message = Some(message);
    }
}

/// `[participant playerID]` (nil for a nil participant).
fn participant_player_id(world: &World, participant: Option<Entity>) -> Option<String> {
    let participant = participant?;
    let state = world.get::<GkTurnBasedParticipant>(participant)?;
    state.player_id.clone()
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

/// `[GKPlayer loadPlayersForIdentifiers:identifiers withCompletionHandler:handler]`.
fn load_players_for_identifiers(world: &mut World, identifiers: Vec<String>, handler: PlayersHandler) {
    let service = world.get_resource::<GameKitService>().copied();
    if let Some(service) = service {
        (service.load_players_for_identifiers)(world, identifiers, handler);
    }
}

/// `[GKTurnBasedMatchmakerViewController alloc] initWithMatchRequest:request]`, then
/// `setTurnBasedMatchmakerDelegate:delegate`.
fn spawn_matchmaker_view_controller(world: &mut World, match_request: GkMatchRequest, delegate: Entity) -> Entity {
    let view_controller = GkTurnBasedMatchmakerViewController {
        match_request,
        turn_based_matchmaker_delegate: Some(delegate),
    };
    world.spawn(view_controller).id()
}

// ---- Main-thread calls ---------------------------------------------------------------------------

/// A `performSelectorOnMainThread:withObject:waitUntilDone:NO` of this class that carries an
/// object (the GameKit completion handlers' calls).
#[derive(Clone, Debug)]
enum MainThreadCall {
    ParseMatchData(Option<Vec<u8>>),
    SetActiveMatch(Option<Entity>),
}

/// The objects of the queued [`MainThreadCall`]s, oldest first. Each one has its entry in the
/// controller's main-thread queue (`crate::controller::perform_selector_on_main_thread`), so
/// every main-thread call of the game keeps one order.
#[derive(Resource, Default)]
struct ChallengeManagerMainThreadCalls {
    calls: VecDeque<MainThreadCall>,
}

/// `[self performSelectorOnMainThread:selector withObject:object waitUntilDone:NO]`.
fn perform_on_main_thread(world: &mut World, this: Entity, call: MainThreadCall) {
    let mut main_thread_calls = world.get_resource_or_insert_with(ChallengeManagerMainThreadCalls::default);
    main_thread_calls.calls.push_back(call);
    crate::controller::perform_selector_on_main_thread(world, this, run_main_thread_call);
}

/// Runs the oldest queued [`MainThreadCall`].
fn run_main_thread_call(world: &mut World, this: Entity) {
    let call = match world.get_resource_mut::<ChallengeManagerMainThreadCalls>() {
        Some(mut main_thread_calls) => main_thread_calls.calls.pop_front(),
        None => None,
    };
    match call {
        Some(MainThreadCall::ParseMatchData(data)) => parse_match_data(world, this, data.as_deref()),
        Some(MainThreadCall::SetActiveMatch(game_match)) => set_active_match(world, this, game_match),
        None => {}
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

/// `[NSString stringWithFormat:format, value]` / `sprintf` of this class's one-`%d` formats.
pub(super) fn format_with_int(format: &str, value: i32) -> String {
    let number = value.to_string();
    format.replacen("%d", &number, 1)
}

/// The entry of a per-type table, `Undefined` outside 0…15 (the getters' `switch` default).
pub(super) fn challenge_table_entry(table: &[&'static str; 16], challenge_type: i32) -> &'static str {
    let entry = match usize::try_from(challenge_type) {
        Ok(index) => table.get(index),
        Err(_) => None,
    };
    match entry {
        Some(entry) => *entry,
        None => UNDEFINED,
    }
}

/// `mChallengeManagerListener` (always the controller).
pub(super) fn listener(world: &World, this: Entity) -> Option<Entity> {
    world.obj::<ChallengeManager>(this).challenge_manager_listener
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:id]]`.
pub(super) fn set_sequence(world: &mut World, this: Entity, id: &str) {
    let sequence = world.obj::<ChallengeManager>(this).sequence;
    let def = pg_engine::library::sequence_def(world, id);
    if let Some(sequence) = sequence {
        pg_engine::sequence::set_sequence_def(world, sequence, def);
    }
}

/// `[messageLogic showMessage:behavior]` (nothing for a nil logic).
pub(super) fn show_message(world: &mut World, message_logic: Option<Entity>, behavior: &str) {
    if let Some(message_logic) = message_logic {
        pg_engine::logic::message::show_message(world, message_logic, behavior);
    }
}

/// `[[[BCLibrary instance] getDisplayGroupById:group] getDisplayObject:object]`.
fn display_object_in_group(world: &World, group_id: &str, object_id: &str) -> Option<Entity> {
    let group = pg_engine::library::display_group(world, group_id)?;
    get_display_object(world, group, object_id)
}

/// `[text setText:value]` (nothing for a nil object).
fn set_text(world: &mut World, text: Option<Entity>, value: &str) {
    if let Some(text) = text {
        pg_engine::display::text::set_text(world, text, Some(value));
    }
}

/// `[icon setTextureDef:[[BCLibrary instance] getTextureDefById:[self getChallengeIconId:type]]]`.
fn set_challenge_icon(world: &mut World, this: Entity, icon: Option<Entity>, challenge_type: i32) {
    let icon_id = get_challenge_icon_id(world, this, challenge_type);
    let texture_def = world.resource::<Library>().texture_def_id(icon_id);
    if let Some(icon) = icon {
        pg_engine::display::set_texture_def(world, icon, texture_def);
    }
}

/// `[mMatchPreferences getStringPreference:key]` (nil without a document).
fn match_string_preference(world: &World, this: Entity, key: &str) -> Option<String> {
    let match_preferences = world.obj::<ChallengeManager>(this).match_preferences?;
    pg_engine::prefs::get_string_preference(world, match_preferences, key)
}

/// `[mMatchPreferences doesPreferenceExist:key]`.
fn match_preference_exists(world: &World, this: Entity, key: &str) -> bool {
    match world.obj::<ChallengeManager>(this).match_preferences {
        Some(match_preferences) => pg_engine::prefs::does_preference_exist(world, match_preferences, key),
        None => false,
    }
}

/// `[mMatchPreferences getIntPreference:key]`.
fn match_int_preference(world: &World, this: Entity, key: &str) -> i32 {
    match world.obj::<ChallengeManager>(this).match_preferences {
        Some(match_preferences) => pg_engine::prefs::get_int_preference(world, match_preferences, key),
        None => 0,
    }
}

/// `[mMatchPreferences setPreference:key asString:value]`.
fn set_match_string_preference(world: &mut World, this: Entity, key: &str, value: Option<&str>) {
    let match_preferences = world.obj::<ChallengeManager>(this).match_preferences;
    let Some(match_preferences) = match_preferences else {
        return;
    };

    // PORT: a nil value would raise in `setObject:forKey:`; the local player always has an id
    // and an alias once authenticated, so nothing is stored instead.
    if let Some(value) = value {
        pg_engine::prefs::set_preference_as_string(world, match_preferences, key, value);
    }
}

/// `[mMatchPreferences setPreference:key asInt:value]`.
fn set_match_int_preference(world: &mut World, this: Entity, key: &str, value: i32) {
    let match_preferences = world.obj::<ChallengeManager>(this).match_preferences;
    if let Some(match_preferences) = match_preferences {
        pg_engine::prefs::set_preference_as_int(world, match_preferences, key, value);
    }
}

/// `[mMatchPreferences dataFromPreferences]`.
fn match_preferences_data(world: &World, this: Entity) -> Option<Vec<u8>> {
    let match_preferences = world.obj::<ChallengeManager>(this).match_preferences?;
    pg_engine::prefs::data_from_preferences(world, match_preferences)
}

/// `[mChallengeManagerListener dismissModalViewController]`.
fn dismiss_listener_modal_view_controller(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::dismiss_modal_view_controller(world, listener);
    }
}

/// `[mChallengeManagerListener onChallengeScreenClosed]`.
fn notify_challenge_screen_closed(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::on_challenge_screen_closed(world, listener);
    }
}

/// `[mChallengeManagerListener onChallengeDialogOpen]`.
fn notify_challenge_dialog_open(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::on_challenge_dialog_open(world, listener);
    }
}

/// `[match setMessage:[NSString stringWithFormat:@"%@ Challenge!", [self getChallengeName:mChallengeType]]]`.
fn set_challenge_match_message(world: &mut World, this: Entity, game_match: Entity) {
    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    let challenge_name = get_challenge_name(world, this, challenge_type);
    let message = format!("{challenge_name} Challenge!");
    set_match_message(world, game_match, message);
}

/// `mChallengeType = [self getChallengeType:[mMatchPreferences getStringPreference:@"GameId"]]`.
/// An unknown id resets the type to 0, stops the start from the matchmaker and returns false.
fn read_match_challenge_type(world: &mut World, this: Entity) -> bool {
    let game_id = match_string_preference(world, this, GAME_ID_KEY);
    let challenge_type = get_challenge_type(world, this, game_id.as_deref());
    world.obj_mut::<ChallengeManager>(this).challenge_type = challenge_type;
    if challenge_type != CHALLENGE_TYPE_NOT_RECOGNIZED {
        return true;
    }

    let mut manager = world.obj_mut::<ChallengeManager>(this);
    manager.challenge_type = 0;
    manager.starting_match_from_view_controller = false;
    false
}

/// `if (mbSavingMatchData) { mbSavingMatchData = NO; return; }`: the turn event caused by our
/// own `saveMatchData` ends the save. Returns whether a save was pending.
fn finish_saving_match_data(world: &mut World, this: Entity) -> bool {
    let mut manager = world.obj_mut::<ChallengeManager>(this);
    if !manager.saving_match_data {
        return false;
    }
    manager.saving_match_data = false;
    true
}

/// `setActiveMatch:`'s send screen: the challenger plays the attempt.
fn show_send_challenge_dialog(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.challenge_screen_active = false;
        manager.challenge_start_send_screen_active = true;
    }
    set_sequence(world, this, SEND_CHALLENGE_DIALOG_SEQUENCE);
}

/// `setActiveMatch:`'s respond screen: the recipient tries to beat the challenger.
fn show_respond_to_challenge_dialog(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.challenge_screen_active = false;
        manager.challenge_start_respond_screen_active = true;
    }
    set_sequence(world, this, RESPOND_TO_CHALLENGE_DIALOG_SEQUENCE);
}

/// `mbStartingMatchFromViewController = NO`.
fn stop_starting_match_from_view_controller(world: &mut World, this: Entity) {
    world.obj_mut::<ChallengeManager>(this).starting_match_from_view_controller = false;
}

/// `[[BCButtonLogic alloc] initWithDisplayObject:]` with this object as its listener.
fn spawn_button_logic(world: &mut World, this: Entity, display_object: Entity) -> Entity {
    let button = pg_engine::logic::button::spawn(world, display_object);
    pg_engine::logic::button::set_button_logic_listener(world, button, Some(this));
    button
}

/// `[[MessageLogic alloc] initWithDisplayObject:]` with this object as its listener.
fn spawn_message_logic(world: &mut World, this: Entity, display_object: Entity) -> Entity {
    let message = pg_engine::logic::message::spawn(world, display_object);
    pg_engine::logic::message::set_message_logic_listener(world, message, Some(this));
    message
}

/// The Go buttons of the send and respond screens: start the 61 s attempt at the challenge.
fn start_challenge_attempt(world: &mut World, this: Entity, start_sequence: &str) {
    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    start_challenge_game(world, this, challenge_type);
    set_sequence(world, this, start_sequence);

    let challenge_timer_logic = world.obj::<ChallengeManager>(this).challenge_timer_logic;
    show_message(world, challenge_timer_logic, SHOW_CHALLENGE_TIMER_MESSAGE);
    update_challenge_timer(world, this);

    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::on_start_challenge(world, listener, challenge_type);
    }
}

/// `-[ChallengeManager init]` @ 0x100013ab8
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(ChallengeManager::default());

    let mut singletons = world.get_resource_or_insert_with(Singletons::default);
    singletons.challenge_manager = Some(this);

    let sequence = pg_engine::sequence::spawn(world, None, Some(this));
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.sequence = Some(sequence);
        manager.logic_array = Vec::with_capacity(5);
        manager.challenge_ui_logic_array = Vec::with_capacity(5);
        manager.challenge_create_ui_logic_array = Vec::with_capacity(5);
        manager.game_center_authentication_complete = false;
        manager.game_center_not_available = false;
    }

    let game_server = crate::managers::game_server::instance(world);
    if let Some(game_server) = game_server {
        crate::managers::game_server::add_game_server_authentication_listener(world, game_server, this);
    }
}

/// `-[ChallengeManager id]` @ 0x100013c18
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(CLASS_ID.to_owned())
}

/// `-[ChallengeManager canReceiveCallbacksNow]` @ 0x100013c24
pub fn can_receive_callbacks_now(_world: &mut World, _this: Entity) -> bool {
    true
}

/// `-[ChallengeManager setChallengeManagerListener:]` @ 0x100013c2c
pub fn set_challenge_manager_listener(world: &mut World, this: Entity, challenge_manager_listener: Entity) {
    world.obj_mut::<ChallengeManager>(this).challenge_manager_listener = Some(challenge_manager_listener);
}

/// `-[ChallengeManager getChallengeType:]` @ 0x100013dd4
pub fn get_challenge_type(_world: &mut World, _this: Entity, game_id: Option<&str>) -> i32 {
    let Some(game_id) = game_id else {
        return CHALLENGE_TYPE_NOT_RECOGNIZED;
    };

    let position = CHALLENGE_IDS.iter().position(|id| *id == game_id);
    match position {
        Some(index) => index as i32,
        None => CHALLENGE_TYPE_NOT_RECOGNIZED,
    }
}

/// `-[ChallengeManager getChallengeId:]` @ 0x100014000
pub fn get_challenge_id(_world: &mut World, _this: Entity, challenge_type: i32) -> Option<&'static str> {
    let index = usize::try_from(challenge_type).ok()?;
    let id = CHALLENGE_IDS.get(index)?;
    Some(*id)
}

/// `-[ChallengeManager getChallengeIconId:]` @ 0x100014128
pub fn get_challenge_icon_id(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&CHALLENGE_ICON_IDS, challenge_type)
}

/// `-[ChallengeManager getChallengeName:]` @ 0x100014254
pub fn get_challenge_name(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&CHALLENGE_NAMES, challenge_type)
}

/// `-[ChallengeManager getChallengeGameCenterGroupId:]` @ 0x100014380
pub fn get_challenge_game_center_group_id(_world: &mut World, _this: Entity, challenge_type: i32) -> i32 {
    let group_id = match usize::try_from(challenge_type) {
        Ok(index) => CHALLENGE_GAME_CENTER_GROUP_IDS.get(index),
        Err(_) => None,
    };
    match group_id {
        Some(group_id) => *group_id,
        None => DEFAULT_GAME_CENTER_GROUP_ID,
    }
}

/// `-[ChallengeManager getLowerCaseChallengeName:]` @ 0x1000144c0
pub fn get_lower_case_challenge_name(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&LOWER_CASE_CHALLENGE_NAMES, challenge_type)
}

/// `-[ChallengeManager getChallengeText:]` @ 0x100014614
pub fn get_challenge_text(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&CHALLENGE_TEXTS, challenge_type)
}

/// `-[ChallengeManager getChallengeUserMessage:]` @ 0x100014740
pub fn get_challenge_user_message(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&CHALLENGE_USER_MESSAGES, challenge_type)
}

/// `-[ChallengeManager getChallengeStartDescription:]` @ 0x10001486c
pub fn get_challenge_start_description(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&CHALLENGE_START_DESCRIPTIONS, challenge_type)
}

/// `-[ChallengeManager getChallengeRecapDescription:]` @ 0x10001498c
pub fn get_challenge_recap_description(_world: &mut World, _this: Entity, challenge_type: i32) -> &'static str {
    challenge_table_entry(&CHALLENGE_RECAP_DESCRIPTIONS, challenge_type)
}

/// `-[ChallengeManager onGameServerAuthenticationStarted]` @ 0x100014a94
pub fn on_game_server_authentication_started(world: &mut World, this: Entity) {
    let mut manager = world.obj_mut::<ChallengeManager>(this);
    manager.game_center_authentication_complete = false;
    manager.game_center_not_available = false;
}

/// `-[ChallengeManager onGameServerAuthenticationComplete]` @ 0x100014b08
pub fn on_game_server_authentication_complete(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.game_center_authentication_complete = false;
        manager.game_center_not_available = false;
    }

    let game_server = crate::managers::game_server::instance(world);
    let multiplayer_available = match game_server {
        Some(game_server) => crate::managers::game_server::is_turn_based_multiplayer_available(world, game_server),
        None => false,
    };
    if !multiplayer_available {
        world.obj_mut::<ChallengeManager>(this).game_center_not_available = true;
        return;
    }

    let same_user = if local_player_is_authenticated(world) {
        world.obj_mut::<ChallengeManager>(this).game_center_authentication_complete = true;
        let player_id = local_player_id(world);
        let active_player_id = world.obj::<ChallengeManager>(this).active_player_id.clone();
        let same_user = ns_string_equal(active_player_id.as_deref(), player_id.as_deref());
        if !same_user {
            world.obj_mut::<ChallengeManager>(this).active_player_id = player_id;
        }
        same_user
    } else {
        world.obj_mut::<ChallengeManager>(this).game_center_not_available = true;
        false
    };

    if same_user {
        crate::controller::perform_selector_on_main_thread(world, this, on_game_center_authentication_changed_same_user);
    } else {
        crate::controller::perform_selector_on_main_thread(world, this, on_game_center_authentication_changed);
    }
    crate::controller::perform_selector_on_main_thread(world, this, finish_game_center_authentication);

    let open_match_panel_queued = world.obj::<ChallengeManager>(this).open_match_panel_on_authentication_complete;
    if !open_match_panel_queued {
        return;
    }
    world.obj_mut::<ChallengeManager>(this).open_match_panel_on_authentication_complete = false;
    crate::controller::perform_selector_on_main_thread(world, this, open_match_panel);
}

/// `-[ChallengeManager finishGameCenterAuthentication]` @ 0x100014d04
pub fn finish_game_center_authentication(_world: &mut World, _this: Entity) {}

/// `-[ChallengeManager onGameCenterAuthenticationChangedSameUser]` @ 0x100014d08
pub fn on_game_center_authentication_changed_same_user(world: &mut World, this: Entity) {
    let mut manager = world.obj_mut::<ChallengeManager>(this);
    if manager.submitting_challenge_result {
        manager.submitting_challenge_result = false;
        manager.error = true;
    }
    if manager.submitting_challenge {
        manager.submitting_challenge = false;
        manager.error = true;
    }
}

/// `-[ChallengeManager onGameCenterAuthenticationChanged]` @ 0x100014dbc
pub fn on_game_center_authentication_changed(world: &mut World, this: Entity) {
    let active_match = world.obj::<ChallengeManager>(this).active_match;
    if active_match.is_some() {
        set_active_match(world, this, None);
    }

    let mut manager = world.obj_mut::<ChallengeManager>(this);
    if manager.submitting_challenge || manager.submitting_challenge_result {
        manager.error = true;
        manager.submitting_challenge = false;
        manager.submitting_challenge_result = false;
    }
}

/// `-[ChallengeManager openMatchPanel]` @ 0x100014e38
pub fn open_match_panel(world: &mut World, this: Entity) {
    let (game_center_not_available, game_center_authentication_complete) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.game_center_not_available, manager.game_center_authentication_complete)
    };
    if game_center_not_available || !game_center_authentication_complete {
        throw_error_message1(world, this, GAME_CENTER_NOT_AVAILABLE, CHECK_GAME_CENTER_LOGIN, "");
        return;
    }

    set_active_match(world, this, None);

    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    let player_group = get_challenge_game_center_group_id(world, this, challenge_type);

    // `[request respondsToSelector:@selector(inviteMessage)]`: GameKit of iOS 5 or later
    // (docs/re/managers-platform.md §18.1).
    let challenge_name = get_challenge_name(world, this, challenge_type);
    let invite_message = format!("Want to play Pocket God: {challenge_name}?");

    let match_request = GkMatchRequest {
        player_group: i64::from(player_group) as u64,
        max_players: 2,
        min_players: 2,
        invite_message: Some(invite_message),
    };
    let view_controller = spawn_matchmaker_view_controller(world, match_request, this);

    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::present_modal_view_controller(world, listener, view_controller);
    }
}

/// `FUN_1000151c0`: `turnBasedMatchmakerViewController:didFindMatch:`'s match data arrived.
/// The opponent is the participant that is not the local player.
fn did_find_match_data_loaded(world: &mut World, this: Entity, game_match: Entity, match_data: Option<Vec<u8>>) {
    let player_id = local_player_id(world);
    let participants = match_participants(world, game_match);
    for participant in participants {
        let participant_id = participant_player_id(world, Some(participant));
        if ns_string_equal(player_id.as_deref(), participant_id.as_deref()) {
            continue;
        }

        let Some(participant_id) = participant_id else {
            world.obj_mut::<ChallengeManager>(this).opponent_player_alias = Some(AUTO_MATCH_PLAYER_ALIAS.to_owned());
            perform_on_main_thread(world, this, MainThreadCall::ParseMatchData(match_data.clone()));
            perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(Some(game_match)));
            continue;
        };

        let identifiers = vec![participant_id];
        let players_match_data = match_data.clone();
        let completion: PlayersHandler = Box::new(move |world: &mut World, players: Option<Vec<GkPlayer>>, _error: Option<String>| {
            did_find_match_players_loaded(world, this, game_match, players_match_data, players);
        });
        load_players_for_identifiers(world, identifiers, completion);
    }
}

/// `FUN_1000154d0`: the opponent's player arrived; its alias names the opponent.
fn did_find_match_players_loaded(world: &mut World, this: Entity, game_match: Entity, match_data: Option<Vec<u8>>, players: Option<Vec<GkPlayer>>) {
    let first_player = match &players {
        Some(players) => players.first(),
        None => None,
    };
    if let Some(first_player) = first_player {
        let alias = first_player.alias.clone();
        world.obj_mut::<ChallengeManager>(this).opponent_player_alias = alias;
    }

    perform_on_main_thread(world, this, MainThreadCall::ParseMatchData(match_data));
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(Some(game_match)));
}

/// `-[ChallengeManager turnBasedMatchmakerViewController:didFindMatch:]` @ 0x1000150dc
pub fn turn_based_matchmaker_view_controller_did_find_match(world: &mut World, this: Entity, _view_controller: Entity, found_match: Entity) {
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.starting_match_from_view_controller = true;
        manager.opponent_player_alias = Some(UNKNOWN_ALIAS.to_owned());
    }

    let game_match = found_match;
    let completion: MatchDataHandler = Box::new(move |world: &mut World, match_data: Option<Vec<u8>>, _error: Option<String>| {
        did_find_match_data_loaded(world, this, game_match, match_data);
    });
    load_match_data(world, game_match, completion);

    dismiss_listener_modal_view_controller(world, this);
    // Then `[viewController release]`: the sheet's entity stays with the match service.
}

/// `-[ChallengeManager turnBasedMatchmakerViewController:didFailWithError:]` @ 0x1000156e4
pub fn turn_based_matchmaker_view_controller_did_fail_with_error(world: &mut World, this: Entity, _view_controller: Entity, _error: Entity) {
    set_active_match(world, this, None);
    throw_error_message1(world, this, GAME_CENTER_NOT_AVAILABLE, CHECK_GAME_CENTER_LOGIN, "");
    dismiss_listener_modal_view_controller(world, this);
}

/// `-[ChallengeManager turnBasedMatchmakerViewController:playerQuitForMatch:]` @ 0x1000157ac
pub fn turn_based_matchmaker_view_controller_player_quit_for_match(world: &mut World, this: Entity, _view_controller: Entity, game_match: Entity) {
    set_active_match(world, this, None);
    cancel_match(world, this, game_match);
    throw_error_message1(world, this, QUIT_MATCH_CONFIRMED, "", "");
    dismiss_listener_modal_view_controller(world, this);
}

/// `-[ChallengeManager turnBasedMatchmakerViewControllerWasCancelled:]` @ 0x100015884
pub fn turn_based_matchmaker_view_controller_was_cancelled(world: &mut World, this: Entity, _view_controller: Entity) {
    set_active_match(world, this, None);
    dismiss_listener_modal_view_controller(world, this);
}

/// `-[ChallengeManager parseMatchData:]` @ 0x1000158e4
pub fn parse_match_data(world: &mut World, this: Entity, data: Option<&[u8]>) {
    let previous_preferences = world.obj::<ChallengeManager>(this).match_preferences;
    if let Some(previous_preferences) = previous_preferences {
        world.despawn(previous_preferences);
    }

    let match_preferences = pg_engine::prefs::spawn_empty(world);
    world.obj_mut::<ChallengeManager>(this).match_preferences = Some(match_preferences);
    pg_engine::prefs::load_preferences_from_data(world, match_preferences, data);
}

/// `FUN_1000159cc`: `loadActiveMatchData`'s match data arrived.
fn active_match_data_loaded(world: &mut World, this: Entity, match_data: Option<Vec<u8>>) {
    perform_on_main_thread(world, this, MainThreadCall::ParseMatchData(match_data));
    let active_match = world.obj::<ChallengeManager>(this).active_match;
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(active_match));
}

/// `-[ChallengeManager loadActiveMatchData]` @ 0x100015964
pub fn load_active_match_data(world: &mut World, this: Entity) {
    let active_match = world.obj::<ChallengeManager>(this).active_match;
    let Some(active_match) = active_match else {
        return;
    };

    let completion: MatchDataHandler = Box::new(move |world: &mut World, match_data: Option<Vec<u8>>, _error: Option<String>| {
        active_match_data_loaded(world, this, match_data);
    });
    load_match_data(world, active_match, completion);
}

/// `setActiveMatch:` for an open match: shows the next screen, or returns the error to throw.
fn set_open_active_match(world: &mut World, this: Entity, game_match: Entity) -> Option<ErrorMessages> {
    let game_category = match_string_preference(world, this, GAME_CATEGORY_KEY);
    let mini_challenge = match game_category.as_deref() {
        Some(game_category) => game_category == MINI_CHALLENGE_CATEGORY,
        None => true,
    };
    if !mini_challenge {
        stop_starting_match_from_view_controller(world, this);
        return Some((NOT_A_MINI_CHALLENGE_MATCH, LOAD_IN_BATTLE_OF_THE_GODS, ""));
    }

    let current_participant = match_current_participant(world, game_match);
    let current_player_id = participant_player_id(world, current_participant);
    let player_id = local_player_id(world);
    if !ns_string_equal(current_player_id.as_deref(), player_id.as_deref()) {
        stop_starting_match_from_view_controller(world, this);
        return Some((WAITING_FOR_OTHER_PLAYER, OTHER_PLAYER_HAS_NOT_TAKEN_TURN, ""));
    }

    let not_recognized = Some((CHALLENGE_NOT_RECOGNIZED, UPDATE_POCKET_GOD, ""));

    let challenger_id = match_string_preference(world, this, CHALLENGER_ID_KEY);
    if challenger_id.is_none() {
        // A new match: I am the challenger.
        set_match_string_preference(world, this, GAME_CATEGORY_KEY, Some(MINI_CHALLENGE_CATEGORY));
        let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
        let game_id = get_challenge_id(world, this, challenge_type);
        set_match_string_preference(world, this, GAME_ID_KEY, game_id);

        let alias = local_player_alias(world);
        let player_id = local_player_id(world);
        set_match_string_preference(world, this, CHALLENGER_ALIAS_KEY, alias.as_deref());
        set_match_string_preference(world, this, CHALLENGER_ID_KEY, player_id.as_deref());
        set_challenge_match_message(world, this, game_match);
        save_match_data(world, this);
        show_send_challenge_dialog(world, this);
        return None;
    }

    let challenger_scored = match_preference_exists(world, this, CHALLENGER_GAME_SCORE_KEY);
    if !challenger_scored {
        // The challenger opened the match again before playing.
        if finish_saving_match_data(world, this) {
            return None;
        }
        if !read_match_challenge_type(world, this) {
            return not_recognized;
        }
        show_send_challenge_dialog(world, this);
        return None;
    }

    let recipient_id = match_string_preference(world, this, RECIPIENT_ID_KEY);
    if recipient_id.is_none() {
        // I am the recipient, opening the challenge for the first time.
        if !read_match_challenge_type(world, this) {
            return not_recognized;
        }

        let alias = local_player_alias(world);
        let player_id = local_player_id(world);
        set_match_string_preference(world, this, RECIPIENT_ALIAS_KEY, alias.as_deref());
        set_match_string_preference(world, this, RECIPIENT_ID_KEY, player_id.as_deref());
        set_challenge_match_message(world, this, game_match);
        save_match_data(world, this);
        show_respond_to_challenge_dialog(world, this);
        return None;
    }

    let recipient_scored = match_preference_exists(world, this, RECIPIENT_GAME_SCORE_KEY);
    if recipient_scored {
        return None;
    }
    if finish_saving_match_data(world, this) {
        return None;
    }
    if !read_match_challenge_type(world, this) {
        return not_recognized;
    }
    show_respond_to_challenge_dialog(world, this);
    None
}

/// `-[ChallengeManager setActiveMatch:]` @ 0x100015a58
pub fn set_active_match(world: &mut World, this: Entity, active_match: Option<Entity>) {
    world.obj_mut::<ChallengeManager>(this).active_match = active_match;
    let Some(game_match) = active_match else {
        return;
    };

    let status = match_status(world, game_match);
    let error = match status {
        MATCH_STATUS_UNKNOWN => {
            stop_starting_match_from_view_controller(world, this);
            Some((GAME_CENTER_ERROR, VERIFY_GAME_CENTER_LOGIN, INTERNET_CONNECTION))
        }
        MATCH_STATUS_OPEN => set_open_active_match(world, this, game_match),
        MATCH_STATUS_ENDED => {
            stop_starting_match_from_view_controller(world, this);
            Some((MATCH_HAS_ENDED, MATCH_HAS_ALREADY_ENDED, ""))
        }
        MATCH_STATUS_MATCHING => {
            stop_starting_match_from_view_controller(world, this);
            Some((FINDING_A_MATCH, NEED_MORE_TIME, ""))
        }
        _ => None,
    };
    let Some((message1, message2, message3)) = error else {
        return;
    };

    throw_error_message1(world, this, message1, message2, message3);
    world.obj_mut::<ChallengeManager>(this).active_match = None;
}

/// `FUN_100016530`: the quit match ended.
fn cancel_match_ended(world: &mut World, this: Entity) {
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(None));
}

/// `-[ChallengeManager cancelMatch:]` @ 0x100016358
pub fn cancel_match(world: &mut World, this: Entity, game_match: Entity) {
    let participants = match_participants(world, game_match);
    for participant in participants {
        set_participant_match_outcome(world, participant, MATCH_OUTCOME_QUIT);
    }

    let match_data = match_preferences_data(world, this);
    let match_data = match_data.unwrap_or_default();
    let completion: ErrorHandler = Box::new(move |world: &mut World, _error: Option<String>| {
        cancel_match_ended(world, this);
    });
    end_match_in_turn_with_match_data(world, game_match, Some(match_data), completion);
}

/// `FUN_100016848`: the turn passed to ourselves, which saves the match data.
fn save_match_data_completed(world: &mut World, this: Entity, error: Option<String>) {
    if error.is_some() {
        world.obj_mut::<ChallengeManager>(this).error = true;
    }
}

/// `-[ChallengeManager saveMatchData]` @ 0x1000165e0
pub fn save_match_data(world: &mut World, this: Entity) {
    let active_match = world.obj::<ChallengeManager>(this).active_match;
    let Some(game_match) = active_match else {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.error = true;
        manager.submitting_challenge = false;
        return;
    };

    world.obj_mut::<ChallengeManager>(this).saving_match_data = true;

    // The iOS 5 idiom: the turn goes to the local player itself, which stores the data.
    let player_id = local_player_id(world);
    let participants = match_participants(world, game_match);
    for participant in participants {
        let participant_id = participant_player_id(world, Some(participant));
        if !ns_string_equal(player_id.as_deref(), participant_id.as_deref()) {
            continue;
        }

        let match_data = match_preferences_data(world, this);
        let completion: ErrorHandler = Box::new(move |world: &mut World, error: Option<String>| {
            save_match_data_completed(world, this, error);
        });
        end_turn_with_next_participant(world, game_match, participant, match_data, completion);
    }
}

/// `FUN_100016b34`: the turn passed to the opponent.
fn end_turn_completed(world: &mut World, this: Entity, error: Option<String>) {
    let mut manager = world.obj_mut::<ChallengeManager>(this);
    if error.is_some() {
        manager.error = true;
    }
    manager.submitting_challenge = false;
}

/// `-[ChallengeManager endTurn]` @ 0x1000168dc
pub fn end_turn(world: &mut World, this: Entity) {
    let active_match = world.obj::<ChallengeManager>(this).active_match;
    let Some(game_match) = active_match else {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.error = true;
        manager.submitting_challenge = false;
        return;
    };

    let player_id = local_player_id(world);
    let participants = match_participants(world, game_match);
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

/// `FUN_10001704c`: the match ended.
fn end_match_completed(world: &mut World, this: Entity, error: Option<String>) {
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        if error.is_some() {
            manager.error = true;
        }
        manager.submitting_challenge_result = false;
    }
    perform_on_main_thread(world, this, MainThreadCall::SetActiveMatch(None));
}

/// `-[ChallengeManager endMatch]` @ 0x100016be4
pub fn end_match(world: &mut World, this: Entity) {
    let active_match = world.obj::<ChallengeManager>(this).active_match;
    let Some(game_match) = active_match else {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.error = true;
        manager.submitting_challenge_result = false;
        return;
    };

    let challenger_alias = match_string_preference(world, this, CHALLENGER_ALIAS_KEY);
    let challenger_score = match_int_preference(world, this, CHALLENGER_GAME_SCORE_KEY);
    let recipient_alias = match_string_preference(world, this, RECIPIENT_ALIAS_KEY);
    let recipient_score = match_int_preference(world, this, RECIPIENT_GAME_SCORE_KEY);

    // Original quirk, kept: Game Center hears that the higher score won, even for the hunts
    // (types 7–9) where the results screen honours the lower score.
    let (local_outcome, other_outcome) = if challenger_score > recipient_score {
        (MATCH_OUTCOME_LOST, MATCH_OUTCOME_WON)
    } else if challenger_score < recipient_score {
        (MATCH_OUTCOME_WON, MATCH_OUTCOME_LOST)
    } else {
        (MATCH_OUTCOME_TIED, MATCH_OUTCOME_TIED)
    };

    let participants = match_participants(world, game_match);
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

    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    let challenge_name = get_challenge_name(world, this, challenge_type);
    let challenger = ns_description(challenger_alias.as_deref());
    let recipient = ns_description(recipient_alias.as_deref());
    let message = if challenger_score == -1 && recipient_score == -1 {
        format!("{challenge_name} - {challenger}: forfeit to {recipient}: forfeit")
    } else if challenger_score == -1 {
        format!("{challenge_name} - {challenger}: forfeit to {recipient}: {recipient_score}")
    } else if recipient_score == -1 {
        format!("{challenge_name} - {challenger}: {challenger_score} to {recipient}: forfeit")
    } else {
        format!("{challenge_name} - {challenger}: {challenger_score} to {recipient}: {recipient_score}")
    };
    set_match_message(world, game_match, message);

    let completion: ErrorHandler = Box::new(move |world: &mut World, error: Option<String>| {
        end_match_completed(world, this, error);
    });
    end_match_in_turn_with_match_data(world, game_match, match_data, completion);
}

/// `-[ChallengeManager handleTurnEventForActiveMatch:]` @ 0x100017120
pub fn handle_turn_event_for_active_match(world: &mut World, this: Entity, game_match: Entity) -> bool {
    let active_match = world.obj::<ChallengeManager>(this).active_match;
    let Some(active_match) = active_match else {
        return false;
    };

    let active_match_id = match_id(world, active_match);
    let event_match_id = match_id(world, game_match);
    if !ns_string_equal(active_match_id.as_deref(), event_match_id.as_deref()) {
        return false;
    }

    load_active_match_data(world, this);
    true
}

/// `-[ChallengeManager handleTurnEventNotificationForInactiveMatch:]` @ 0x1000171f0
pub fn handle_turn_event_notification_for_inactive_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// `-[ChallengeManager handleMatchEndedForActiveMatch:]` @ 0x100017244
pub fn handle_match_ended_for_active_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// `-[ChallengeManager handleMatchEndedNotificationForInactiveMatch:]` @ 0x1000172a4
pub fn handle_match_ended_notification_for_inactive_match(_world: &mut World, _this: Entity, _game_match: Entity) -> bool {
    false
}

/// `-[ChallengeManager waitForSaveMatchDataComplete:::]` @ 0x100017304
pub fn seq_wait_for_save_match_data_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let saving_match_data = world.obj::<ChallengeManager>(this).saving_match_data;
    if saving_match_data {
        return ItemResult::Pending;
    }

    stop_starting_match_from_view_controller(world, this);
    ItemResult::Done
}

/// `-[ChallengeManager submitChallenge:::]` @ 0x100017330
pub fn seq_submit_challenge(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let best_challenge_game_score = {
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.error = false;
            manager.submitting_challenge = true;
            manager.best_challenge_game_score
        };
        set_match_int_preference(world, this, CHALLENGER_GAME_SCORE_KEY, best_challenge_game_score);
        end_turn(world, this);
        world.obj_mut::<ChallengeManager>(this).active_match = None;
    }

    let (submitting_challenge, error) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.submitting_challenge, manager.error)
    };
    if submitting_challenge {
        return ItemResult::Pending;
    }
    if error {
        throw_dialog_error(world, this, SUBMIT_CHALLENGE_FAIL);
    }
    ItemResult::Done
}

/// `-[ChallengeManager submitChallengeResult:::]` @ 0x100017440
pub fn seq_submit_challenge_result(world: &mut World, this: Entity, call: &mut SequenceCall) -> ItemResult {
    if call.first_time {
        let best_challenge_game_score = {
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.error = false;
            manager.submitting_challenge_result = true;
            manager.best_challenge_game_score
        };
        set_match_int_preference(world, this, RECIPIENT_GAME_SCORE_KEY, best_challenge_game_score);
        end_match(world, this);
        world.obj_mut::<ChallengeManager>(this).active_match = None;
    }

    let (submitting_challenge_result, error) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.submitting_challenge_result, manager.error)
    };
    if submitting_challenge_result {
        return ItemResult::Pending;
    }
    if error {
        throw_dialog_error(world, this, SUBMIT_CHALLENGE_RESULT_FAIL);
    }
    ItemResult::Done
}

/// `-[ChallengeManager throwErrorMessage1:message2:message3:]` @ 0x100017550
pub fn throw_error_message1(world: &mut World, this: Entity, message1: &str, message2: &str, message3: &str) {
    world.obj_mut::<ChallengeManager>(this).challenge_error_active = true;

    let message1_text = display_object_in_group(world, CHALLENGE_ERROR_GROUP, MESSAGE1_OBJECT);
    set_text(world, message1_text, message1);
    let message2_text = display_object_in_group(world, CHALLENGE_ERROR_GROUP, MESSAGE2_OBJECT);
    set_text(world, message2_text, message2);
    let message3_text = display_object_in_group(world, CHALLENGE_ERROR_GROUP, MESSAGE3_OBJECT);
    set_text(world, message3_text, message3);

    set_sequence(world, this, CHALLENGE_ERROR_SEQUENCE);
}

/// `-[ChallengeManager throwDialogError:]` @ 0x100017698
pub fn throw_dialog_error(world: &mut World, this: Entity, _message: &str) {
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.challenge_system_error_during_result_screen = manager.challenge_results_screen_active;
        manager.challenge_system_error_during_send_challenge = manager.challenge_confirm_send_screen_active;
        manager.challenge_results_screen_active = false;
        manager.challenge_confirm_send_screen_active = false;
        manager.challenge_system_error_active = true;
    }
    set_sequence(world, this, CHALLENGE_DIALOG_ERROR_SEQUENCE);
}

/// `-[ChallengeManager updateChallengeCreatePanel]` @ 0x100017748
pub fn update_challenge_create_panel(world: &mut World, this: Entity) {
    let title = display_object_in_group(world, CHALLENGE_CREATE_PANEL_GROUP, CHALLENGE_TITLE_OBJECT);
    let icon = display_object_in_group(world, CHALLENGE_CREATE_PANEL_GROUP, CHALLENGE_ICON_OBJECT);
    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;

    let challenge_name = get_challenge_name(world, this, challenge_type);
    set_text(world, title, challenge_name);
    set_challenge_icon(world, this, icon, challenge_type);
}

/// `-[ChallengeManager updateChallengeStartSendScreen]` @ 0x100017870
pub fn update_challenge_start_send_screen(world: &mut World, this: Entity) {
    let start_message = display_object_in_group(world, CHALLENGE_START_SEND_SCREEN_GROUP, CHALLENGE_START_MESSAGE_OBJECT);
    let start_title = display_object_in_group(world, CHALLENGE_START_SEND_SCREEN_GROUP, CHALLENGE_START_TITLE_OBJECT);
    let icon = display_object_in_group(world, CHALLENGE_START_SEND_SCREEN_GROUP, CHALLENGE_ICON_OBJECT);
    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;

    let start_description = get_challenge_start_description(world, this, challenge_type);
    set_text(world, start_message, start_description);

    let opponent_player_alias = world.obj::<ChallengeManager>(this).opponent_player_alias.clone();
    let opponent = ns_description(opponent_player_alias.as_deref());
    let title = format!("You've challenged {opponent}!");
    set_text(world, start_title, &title);

    set_challenge_icon(world, this, icon, challenge_type);
}

/// `-[ChallengeManager updateChallengeConfirmSendScreen]` @ 0x100017a04
pub fn update_challenge_confirm_send_screen(world: &mut World, this: Entity) {
    let confirm_message = display_object_in_group(world, CHALLENGE_CONFIRM_SEND_SCREEN_GROUP, CHALLENGE_CONFIRM_MESSAGE_OBJECT);
    let confirm_title = display_object_in_group(world, CHALLENGE_CONFIRM_SEND_SCREEN_GROUP, CHALLENGE_CONFIRM_TITLE_OBJECT);
    let icon = display_object_in_group(world, CHALLENGE_CONFIRM_SEND_SCREEN_GROUP, CHALLENGE_ICON_OBJECT);

    let opponent_player_alias = world.obj::<ChallengeManager>(this).opponent_player_alias.clone();
    let opponent = ns_description(opponent_player_alias.as_deref());
    let message = format!("Your challenge has been sent to {opponent}.");
    set_text(world, confirm_message, &message);

    let (best_challenge_game_score, challenge_type) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.best_challenge_game_score, manager.challenge_type)
    };
    let title = if best_challenge_game_score == -1 {
        QUIT_AND_FORFEITED.to_owned()
    } else {
        let recap_description = get_challenge_recap_description(world, this, challenge_type);
        format_with_int(recap_description, best_challenge_game_score)
    };
    set_text(world, confirm_title, &title);

    set_challenge_icon(world, this, icon, challenge_type);
}

/// `-[ChallengeManager updateChallengeStartRespondScreen]` @ 0x100017be8
pub fn update_challenge_start_respond_screen(world: &mut World, this: Entity) {
    let start_message = display_object_in_group(world, CHALLENGE_START_RESPOND_SCREEN_GROUP, CHALLENGE_START_MESSAGE_OBJECT);
    let start_title = display_object_in_group(world, CHALLENGE_START_RESPOND_SCREEN_GROUP, CHALLENGE_START_TITLE_OBJECT);
    let icon = display_object_in_group(world, CHALLENGE_START_RESPOND_SCREEN_GROUP, CHALLENGE_ICON_OBJECT);
    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;

    let start_description = get_challenge_start_description(world, this, challenge_type);
    set_text(world, start_message, start_description);

    let opponent_player_alias = world.obj::<ChallengeManager>(this).opponent_player_alias.clone();
    let opponent = ns_description(opponent_player_alias.as_deref());
    let title = format!("{opponent} has challenged you!");
    set_text(world, start_title, &title);

    set_challenge_icon(world, this, icon, challenge_type);
}

/// The results screen's line when both scores were played: the score type decides who won.
fn compared_challenge_result(challenge_score_type: i32, best: i32, challenger_score: i32) -> &'static str {
    let higher_won = challenge_score_type == HIGHER_SCORE_WINS && challenger_score < best;
    let lower_won = challenge_score_type == LOWER_SCORE_WINS && best < challenger_score;
    if higher_won || lower_won {
        return RESULT_WON;
    }

    let higher_tied = challenge_score_type != HIGHER_SCORE_WINS || challenger_score <= best;
    let lower_tied = challenge_score_type != LOWER_SCORE_WINS || best <= challenger_score;
    if higher_tied && lower_tied {
        return RESULT_TIED;
    }
    RESULT_LOST
}

/// `-[ChallengeManager updateChallengeResultsScreen]` @ 0x100017d7c
pub fn update_challenge_results_screen(world: &mut World, this: Entity) {
    let confirm_message = display_object_in_group(world, CHALLENGE_RESULTS_SCREEN_GROUP, CHALLENGE_CONFIRM_MESSAGE_OBJECT);
    let confirm_title = display_object_in_group(world, CHALLENGE_RESULTS_SCREEN_GROUP, CHALLENGE_CONFIRM_TITLE_OBJECT);
    let icon = display_object_in_group(world, CHALLENGE_RESULTS_SCREEN_GROUP, CHALLENGE_ICON_OBJECT);

    let challenger_score = match_int_preference(world, this, CHALLENGER_GAME_SCORE_KEY);
    let (best, challenge_score_type, opponent_player_alias) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.best_challenge_game_score, manager.challenge_score_type, manager.opponent_player_alias.clone())
    };
    let opponent = ns_description(opponent_player_alias.as_deref());

    let (result, title) = if challenger_score == -1 {
        let result = if best == -1 { RESULT_TIED } else { RESULT_WON };
        let title = if best == -1 {
            format!("you: forfeit, {opponent}: forfeit")
        } else {
            format!("you: {best}, {opponent}: forfeit")
        };
        (result, title)
    } else {
        let result = if best == -1 {
            RESULT_LOST
        } else {
            compared_challenge_result(challenge_score_type, best, challenger_score)
        };
        let title = if best == -1 {
            format!("you: forfeit, {opponent}: {challenger_score}")
        } else {
            format!("you: {best}, {opponent}: {challenger_score}")
        };
        (result, title)
    };
    set_text(world, confirm_message, result);
    set_text(world, confirm_title, &title);

    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    set_challenge_icon(world, this, icon, challenge_type);
}

/// `-[ChallengeManager updateLoadingScreen]` @ 0x1000180a0
pub fn update_loading_screen(world: &mut World, this: Entity) {
    let icon = display_object_in_group(world, CHALLENGE_LOADING_SCREEN_GROUP, CHALLENGE_ICON_OBJECT);
    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    set_challenge_icon(world, this, icon, challenge_type);
}

/// `-[ChallengeManager updateChallengeTimer]` @ 0x100018170
pub fn update_challenge_timer(world: &mut World, this: Entity) {
    let timer = display_object_in_group(world, CHALLENGE_TIMER_GROUP, TIMER_OBJECT);
    let icon = display_object_in_group(world, CHALLENGE_TIMER_GROUP, CHALLENGE_ICON_OBJECT);
    let challenge_type = world.obj::<ChallengeManager>(this).challenge_type;

    let challenge_name = get_challenge_name(world, this, challenge_type);
    set_text(world, timer, challenge_name);
    set_challenge_icon(world, this, icon, challenge_type);
}

/// `-[ChallengeManager updateChallengeAlert]` @ 0x100018298
pub fn update_challenge_alert(world: &mut World, this: Entity) {
    let alert_num = display_object_in_group(world, CHALLENGE_ALERT_GROUP, ALERT_NUM_OBJECT);
    let icon = display_object_in_group(world, CHALLENGE_ALERT_GROUP, CHALLENGE_ICON_OBJECT);

    let (new_challenge_count, new_challenge_type) = {
        let manager = world.obj::<ChallengeManager>(this);
        (manager.new_challenge_count, manager.new_challenge_type)
    };
    let count = format!("{new_challenge_count}");
    set_text(world, alert_num, &count);
    set_challenge_icon(world, this, icon, new_challenge_type);
}

/// `-[ChallengeManager updateChallengeCreatePanel:::]` @ 0x1000183e0
pub fn seq_update_challenge_create_panel(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_challenge_create_panel(world, this);
    ItemResult::Done
}

/// `-[ChallengeManager updateChallengeStartSendScreen:::]` @ 0x100018404
pub fn seq_update_challenge_start_send_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_loading_screen(world, this);
    update_challenge_start_send_screen(world, this);
    ItemResult::Done
}

/// `-[ChallengeManager showChallengeConfirmSendScreen:::]` @ 0x100018448
pub fn seq_show_challenge_confirm_send_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ChallengeManager>(this).challenge_confirm_send_screen_active = true;
    notify_challenge_dialog_open(world, this);
    update_loading_screen(world, this);
    update_challenge_confirm_send_screen(world, this);
    ItemResult::Done
}

/// `-[ChallengeManager updateChallengeStartRespondScreen:::]` @ 0x1000184bc
pub fn seq_update_challenge_start_respond_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    update_loading_screen(world, this);
    update_challenge_start_respond_screen(world, this);
    ItemResult::Done
}

/// `-[ChallengeManager showChallengeResultsScreen:::]` @ 0x100018500
pub fn seq_show_challenge_results_screen(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    world.obj_mut::<ChallengeManager>(this).challenge_results_screen_active = true;
    notify_challenge_dialog_open(world, this);
    update_loading_screen(world, this);
    update_challenge_results_screen(world, this);
    ItemResult::Done
}

/// `-[ChallengeManager loadLocationTextureAtlas:::]` @ 0x100018574
pub fn seq_load_location_texture_atlas(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let listener = listener(world, this);
    if let Some(listener) = listener {
        crate::controller::on_load_location_texture_atlas(world, listener);
    }
    ItemResult::Done
}

/// `-[ChallengeManager updateChallengeType:]` @ 0x1000185a4
pub fn update_challenge_type(world: &mut World, this: Entity, requested_type: i32) {
    let below_last = requested_type.min(LAST_CHALLENGE_TYPE);
    let challenge_type = below_last.max(0);
    let current_challenge_type = world.obj::<ChallengeManager>(this).challenge_type;
    if challenge_type == current_challenge_type {
        return;
    }

    world.obj_mut::<ChallengeManager>(this).challenge_type = challenge_type;
    set_sequence(world, this, UPDATE_CHALLENGE_TYPE_SEQUENCE);
}

/// `-[ChallengeManager cantOpenChallengeScreen]` @ 0x100018644
pub fn cant_open_challenge_screen(world: &mut World, this: Entity) {
    let challenge_in_progress_logic = world.obj::<ChallengeManager>(this).challenge_in_progress_logic;
    show_message(world, challenge_in_progress_logic, CANT_OPEN_CHALLENGE_SCREEN_MESSAGE);
}

/// `-[ChallengeManager openChallengeScreen]` @ 0x100018668
pub fn open_challenge_screen(world: &mut World, this: Entity) {
    let idol_manager = crate::managers::idol_manager::instance(world);
    if let Some(idol_manager) = idol_manager {
        crate::managers::idol_manager::delay_trigger_episode_event(world, idol_manager, OPEN_CHALLENGE_SCREEN_EVENT);
    }

    world.obj_mut::<ChallengeManager>(this).challenge_screen_active = true;
    set_sequence(world, this, OPEN_CHALLENGE_SCREEN_SEQUENCE);
}

/// `-[ChallengeManager closeChallengeScreen]` @ 0x100018718
pub fn close_challenge_screen(world: &mut World, this: Entity) {
    {
        let mut manager = world.obj_mut::<ChallengeManager>(this);
        manager.challenge_error_active = false;
        manager.challenge_screen_active = false;
    }
    set_sequence(world, this, CLOSE_CHALLENGE_SCREEN_SEQUENCE);
    notify_challenge_screen_closed(world, this);
}

/// The buttons `onButtonPress:touch:` handles, in the order it compares them.
#[derive(Clone, Copy)]
enum ChallengeButton {
    Close,
    NextChallenge,
    PrevChallenge,
    CreateChallenge,
    StartSendCancel,
    StartSendGo,
    ConfirmSendClose,
    StartRespondCancel,
    StartRespondGo,
    ResultsClose,
    ResultsRechallenge,
    ErrorClose,
    SystemErrorClose,
}

/// Which of this manager's buttons `button` is.
fn pressed_challenge_button(manager: &ChallengeManager, button: Entity) -> Option<ChallengeButton> {
    let pressed = Some(button);
    let buttons = [
        (manager.challenge_close_button_logic, ChallengeButton::Close),
        (manager.challenge_create_next_challenge_button_logic, ChallengeButton::NextChallenge),
        (manager.challenge_create_prev_challenge_button_logic, ChallengeButton::PrevChallenge),
        (manager.challenge_create_challenge_button_logic, ChallengeButton::CreateChallenge),
        (manager.challenge_start_send_cancel_button_logic, ChallengeButton::StartSendCancel),
        (manager.challenge_start_send_go_button_logic, ChallengeButton::StartSendGo),
        (manager.challenge_confirm_send_close_button_logic, ChallengeButton::ConfirmSendClose),
        (manager.challenge_start_respond_cancel_button_logic, ChallengeButton::StartRespondCancel),
        (manager.challenge_start_respond_go_button_logic, ChallengeButton::StartRespondGo),
        (manager.challenge_results_close_button_logic, ChallengeButton::ResultsClose),
        (manager.challenge_results_rechallenge_button_logic, ChallengeButton::ResultsRechallenge),
        (manager.challenge_error_close_button_logic, ChallengeButton::ErrorClose),
        (manager.challenge_system_error_close_button_logic, ChallengeButton::SystemErrorClose),
    ];
    for (button_logic, challenge_button) in buttons {
        if button_logic == pressed {
            return Some(challenge_button);
        }
    }
    None
}

/// `-[ChallengeManager onButtonPress:touch:]` @ 0x100019ab4
pub fn on_button_press(world: &mut World, this: Entity, button: Entity, _touch: Entity) {
    let (starting_match_from_view_controller, challenge_type, pressed) = {
        let manager = world.obj::<ChallengeManager>(this);
        let pressed = pressed_challenge_button(manager, button);
        (manager.starting_match_from_view_controller, manager.challenge_type, pressed)
    };
    if starting_match_from_view_controller {
        return;
    }
    let Some(pressed) = pressed else {
        return;
    };

    match pressed {
        ChallengeButton::Close => close_challenge_screen(world, this),
        ChallengeButton::NextChallenge => {
            let next_challenge_type = challenge_type.wrapping_add(1);
            update_challenge_type(world, this, next_challenge_type);
        }
        ChallengeButton::PrevChallenge => {
            let previous_challenge_type = challenge_type.wrapping_sub(1);
            update_challenge_type(world, this, previous_challenge_type);
        }
        ChallengeButton::CreateChallenge => open_match_panel(world, this),
        ChallengeButton::StartSendCancel => {
            world.obj_mut::<ChallengeManager>(this).challenge_start_send_screen_active = false;
            set_sequence(world, this, SEND_CHALLENGE_CANCEL_SEQUENCE);
            notify_challenge_screen_closed(world, this);
        }
        ChallengeButton::StartSendGo => {
            {
                let mut manager = world.obj_mut::<ChallengeManager>(this);
                manager.best_challenge_game_score = -1;
                manager.challenge_timer_active = true;
                manager.challenge_send_attempt_active = true;
                manager.challenge_start_send_screen_active = false;
            }
            start_challenge_attempt(world, this, SEND_CHALLENGE_START_SEQUENCE);
        }
        ChallengeButton::ConfirmSendClose => {
            world.obj_mut::<ChallengeManager>(this).challenge_confirm_send_screen_active = false;
            notify_challenge_screen_closed(world, this);
            set_sequence(world, this, SUBMIT_CHALLENGE_CLOSE_SEQUENCE);
        }
        ChallengeButton::StartRespondCancel => {
            world.obj_mut::<ChallengeManager>(this).challenge_start_respond_screen_active = false;
            set_sequence(world, this, RESPOND_CHALLENGE_CANCEL_SEQUENCE);
            notify_challenge_screen_closed(world, this);
        }
        ChallengeButton::StartRespondGo => {
            {
                let mut manager = world.obj_mut::<ChallengeManager>(this);
                manager.best_challenge_game_score = -1;
                manager.challenge_timer_active = true;
                manager.challenge_respond_attempt_active = true;
                manager.challenge_start_respond_screen_active = false;
            }
            start_challenge_attempt(world, this, RESPOND_CHALLENGE_START_SEQUENCE);
        }
        ChallengeButton::ResultsClose => {
            world.obj_mut::<ChallengeManager>(this).challenge_results_screen_active = false;
            notify_challenge_screen_closed(world, this);
            set_sequence(world, this, SUBMIT_CHALLENGE_RESULT_CLOSE_SEQUENCE);
        }
        ChallengeButton::ResultsRechallenge => {
            // `submitChallengeResult:::` already ended and released the match, so the send
            // screen's later `endTurn` fails with "Submit Challenge Fail" (original behaviour).
            {
                let mut manager = world.obj_mut::<ChallengeManager>(this);
                manager.challenge_results_screen_active = false;
                manager.challenge_start_send_screen_active = true;
            }
            set_sequence(world, this, SUBMIT_CHALLENGE_RESULT_RECHALLENGE_SEQUENCE);
        }
        ChallengeButton::ErrorClose => {
            world.obj_mut::<ChallengeManager>(this).challenge_error_active = false;
            set_sequence(world, this, CHALLENGE_CLOSE_ERROR_SEQUENCE);
        }
        ChallengeButton::SystemErrorClose => {
            {
                let mut manager = world.obj_mut::<ChallengeManager>(this);
                manager.challenge_system_error_active = false;
                manager.challenge_system_error_during_result_screen = false;
                manager.challenge_system_error_during_send_challenge = false;
            }
            set_sequence(world, this, CHALLENGE_DIALOG_ERROR_CLOSE_SEQUENCE);
            notify_challenge_screen_closed(world, this);
        }
    }
}

/// `-[ChallengeManager onButtonRelease:touch:]` @ 0x10001a0a4
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[ChallengeManager onButtonReleaseOutside:touch:]` @ 0x10001a0a8
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[ChallengeManager setLogic:]` @ 0x10001a238: the boot script's `setLogic` items for this
/// manager (`call` carries the `BCSequenceItemDef`).
pub fn set_logic(world: &mut World, this: Entity, call: &SequenceCall) {
    let display_group_id = call.param("displayGroupId");
    let logic_id = call.param("logicId");
    let display_object_id = call.param("displayObjectId");

    let display_group = match display_group_id {
        Some(display_group_id) => pg_engine::library::display_group(world, display_group_id),
        None => None,
    };
    let display_object = match (display_group, display_object_id) {
        (Some(display_group), Some(display_object_id)) => get_display_object(world, display_group, display_object_id),
        _ => None,
    };

    let Some(logic_id) = logic_id else {
        return;
    };

    // PORT: the original also creates the logic on a nil display object; the engine's logics
    // need one, so nothing is created. Every `setLogic` item of the boot script names an
    // existing object.
    let Some(display_object) = display_object else {
        return;
    };

    match logic_id {
        "ChallengeCloseButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_close_button_logic = Some(button);
            manager.challenge_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "ChallengeCreateNextChallengeButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_create_next_challenge_button_logic = Some(button);
            manager.challenge_create_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "ChallengeCreatePrevChallengeButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_create_prev_challenge_button_logic = Some(button);
            manager.challenge_create_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "ChallengeCreateChallengeButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_create_challenge_button_logic = Some(button);
            manager.challenge_create_ui_logic_array.push(button);
            manager.logic_array.push(button);
        }
        "ChallengeStartSendCancelButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_start_send_cancel_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeStartSendGoButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_start_send_go_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeConfirmSendCloseButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_confirm_send_close_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeStartRespondCancelButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_start_respond_cancel_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeStartRespondGoButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_start_respond_go_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeResultsCloseButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_results_close_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeResultsRechallengeButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_results_rechallenge_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeErrorCloseButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_error_close_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "ChallengeSystemErrorCloseButtonLogic" => {
            let button = spawn_button_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_system_error_close_button_logic = Some(button);
            manager.logic_array.push(button);
        }
        "AchievementMessageLogic" => {
            let message = spawn_message_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.achievement_message_logic = Some(message);
            manager.logic_array.push(message);
        }
        "CountingMessageLogic" => {
            let message = spawn_message_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.counting_message_logic = Some(message);
            manager.logic_array.push(message);
        }
        "ChallengeTimerLogic" => {
            let message = spawn_message_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_timer_logic = Some(message);
            manager.logic_array.push(message);
        }
        "ChallengeAlertLogic" => {
            let message = spawn_message_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_alert_logic = Some(message);
            manager.logic_array.push(message);
        }
        "ChallengeInProgressLogic" => {
            let message = spawn_message_logic(world, this, display_object);
            let mut manager = world.obj_mut::<ChallengeManager>(this);
            manager.challenge_in_progress_logic = Some(message);
            manager.logic_array.push(message);
        }
        _ => {}
    }
}

/// `+[ChallengeManager instance]` @ 0x100013aac
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().challenge_manager
}
