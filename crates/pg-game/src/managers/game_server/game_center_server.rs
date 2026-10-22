//! `GameServer`: methods in this part of the class.
//!
//! The Game Center wrapper (`docs/re/managers-platform.md` §3) over the port's GameKit stand-in
//! (§18.1 rows 13–15). Sign-in completes at once, without a sign-in sheet, and every
//! authentication listener still gets `onGameServerAuthenticationComplete`; the player counts as
//! authenticated only when a match service (§18.3) set up `GkLocalPlayer`. Leaderboard scores are
//! kept on the device ([`LocalLeaderboards`]). A match service delivers its turn events to the
//! [`GkTurnBasedEventHandler`] delegate, which forwards them to the listeners.

use std::collections::VecDeque;

use crate::controller::perform_selector_on_main_thread;
use crate::managers::challenge_manager::GameKitService;
use crate::prelude::*;
use super::*;

/// `GameServer`'s file-scope globals, and the notification it observes.
#[derive(Resource, Default, Clone, Debug)]
pub struct GameServerStatics {
    /// `DAT_1007e50a0`: one end of the dead score queue (§3.4). Nothing writes it.
    pub score_queue_head: i32,

    /// `DAT_1007e50a4`: the other end. Nothing writes it either, so the queue is always empty.
    pub score_queue_tail: i32,

    /// The `UIApplicationDidBecomeActiveNotification` observers with the selector
    /// `applicationDidBecomeActive` (`init`'s `addObserver:selector:name:object:`), in order.
    pub application_did_become_active_observers: Vec<Entity>,
}

/// `[GKTurnBasedEventHandler sharedTurnBasedEventHandler]`. A match service delivers its turn
/// events to `delegate` (set once authentication completed) through
/// [`handle_turn_event_for_match_did_become_active`], [`handle_turn_event_for_match`],
/// [`handle_match_ended`] and [`handle_invite_from_game_center`].
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct GkTurnBasedEventHandler {
    /// `delegate` (`GKTurnBasedEventHandlerDelegate`)
    pub delegate: Option<Entity>,
}

/// Game Center's leaderboards as the port keeps them on the device (§3.5): the best value
/// reported to each leaderboard id (the highest, as Game Center's sort order for these ids is
/// unknown). Nothing in the game reads them back, as in the original.
#[derive(Resource, Default, Clone, Debug)]
pub struct LocalLeaderboards {
    pub best_scores: HashMap<String, i64>,
}

/// A `performSelectorOnMainThread:withObject:waitUntilDone:NO` of this class that carries an
/// object: the match of a turn event.
#[derive(Clone, Copy, Debug)]
enum MainThreadCall {
    MainHandleTurnEventForMatch(Entity),
    MainHandleMatchEnded(Entity),
}

/// The objects of the queued [`MainThreadCall`]s, oldest first. Each one has its entry in the
/// controller's main-thread queue ([`perform_selector_on_main_thread`]), so every main-thread
/// call of the game keeps one order.
#[derive(Resource, Default)]
struct GameServerMainThreadCalls {
    calls: VecDeque<MainThreadCall>,
}

/// `[self performSelectorOnMainThread:selector withObject:match waitUntilDone:NO]`.
fn perform_on_main_thread(world: &mut World, this: Entity, call: MainThreadCall) {
    let mut main_thread_calls = world.get_resource_or_insert_with(GameServerMainThreadCalls::default);
    main_thread_calls.calls.push_back(call);
    perform_selector_on_main_thread(world, this, run_main_thread_call);
}

/// Runs the oldest queued [`MainThreadCall`].
fn run_main_thread_call(world: &mut World, this: Entity) {
    let call = match world.get_resource_mut::<GameServerMainThreadCalls>() {
        Some(mut main_thread_calls) => main_thread_calls.calls.pop_front(),
        None => None,
    };
    match call {
        Some(MainThreadCall::MainHandleTurnEventForMatch(game_match)) => {
            main_handle_turn_event_for_match(world, this, game_match);
        }
        Some(MainThreadCall::MainHandleMatchEnded(game_match)) => {
            main_handle_match_ended(world, this, game_match);
        }
        None => {}
    }
}

/// `mGameServerAuthenticationListenerArray`, copied for enumeration.
fn authentication_listeners(world: &World, this: Entity) -> Vec<Entity> {
    world.obj::<GameServer>(this).game_server_authentication_listener_array.clone()
}

/// UIKit's `UIApplicationDidBecomeActiveNotification`, posted right after the app delegate's
/// `applicationDidBecomeActive:`: sends `applicationDidBecomeActive` to the observers [`init`]
/// registered. The app wiring calls it after
/// `crate::managers::pocket_god_app_delegate::application_did_become_active`.
pub fn post_application_did_become_active_notification(world: &mut World) {
    let observers = match world.get_resource::<GameServerStatics>() {
        Some(statics) => statics.application_did_become_active_observers.clone(),
        None => Vec::new(),
    };
    for observer in observers {
        application_did_become_active(world, observer);
    }
}

/// `-[GameServer init]` @ 0x10008a918
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(GameServer::default());

    world.get_resource_or_insert_with(Singletons::default).game_server = Some(this);
    world.obj_mut::<GameServer>(this).leaderboard_dictionary = HashMap::with_capacity(5);
    pg_engine::view::add_game_frame_listener(world, this);
    world.obj_mut::<GameServer>(this).game_server_authentication_listener_array = Vec::with_capacity(5);

    // `[[NSNotificationCenter defaultCenter] addObserver:self
    // selector:@selector(applicationDidBecomeActive)
    // name:UIApplicationDidBecomeActiveNotification object:nil]`.
    let mut statics = world.get_resource_or_insert_with(GameServerStatics::default);
    statics.application_did_become_active_observers.push(this);

    authenticate_user(world, this);
}

/// `-[GameServer applicationDidBecomeActive]` @ 0x10008aa68
pub fn application_did_become_active(world: &mut World, this: Entity) {
    authenticate_user(world, this);
}

/// `-[GameServer id]` @ 0x10008aa78
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some("GameServer".to_owned())
}

/// `-[GameServer setGameServerListener:]` @ 0x10008aa84
pub fn set_game_server_listener(world: &mut World, this: Entity, game_server_listener: Entity) {
    world.obj_mut::<GameServer>(this).game_server_listener = Some(game_server_listener);
}

/// `-[GameServer setGameServerLeaderboardListener:]` @ 0x10008aae0
pub fn set_game_server_leaderboard_listener(world: &mut World, this: Entity, game_server_leaderboard_listener: Entity) {
    world.obj_mut::<GameServer>(this).game_server_leaderboard_listener = Some(game_server_leaderboard_listener);
}

/// `-[GameServer setGameServerAchievementListener:]` @ 0x10008ab3c
pub fn set_game_server_achievement_listener(world: &mut World, this: Entity, game_server_achievement_listener: Entity) {
    world.obj_mut::<GameServer>(this).game_server_achievement_listener = Some(game_server_achievement_listener);
}

/// `-[GameServer setGameServerChallengeListener:]` @ 0x10008ab98
pub fn set_game_server_challenge_listener(world: &mut World, this: Entity, game_server_challenge_listener: Entity) {
    world.obj_mut::<GameServer>(this).game_server_challenge_listener = Some(game_server_challenge_listener);
}

/// `-[GameServer addGameServerAuthenticationListener:]` @ 0x10008abf4
pub fn add_game_server_authentication_listener(world: &mut World, this: Entity, game_server_authentication_listener: Entity) {
    let mut server = world.obj_mut::<GameServer>(this);
    server.game_server_authentication_listener_array.push(game_server_authentication_listener);
}

/// `-[GameServer removeGameServerAuthenticationListener:]` @ 0x10008ac10
pub fn remove_game_server_authentication_listener(world: &mut World, this: Entity, game_server_authentication_listener: Entity) {
    // `removeObject:` removes every occurrence.
    let mut server = world.obj_mut::<GameServer>(this);
    let listeners = &mut server.game_server_authentication_listener_array;
    listeners.retain(|listener| *listener != game_server_authentication_listener);
}

/// `-[GameServer onGameCenterAuthenticationComplete]` @ 0x10008ad18
pub fn on_game_center_authentication_complete(world: &mut World, this: Entity) {
    let listeners = authentication_listeners(world, this);
    for listener in listeners {
        crate::send::on_game_server_authentication_complete(world, Some(listener));
    }

    // `[[GKTurnBasedEventHandler sharedTurnBasedEventHandler] setDelegate:self]`.
    let mut event_handler = world.get_resource_or_insert_with(GkTurnBasedEventHandler::default);
    event_handler.delegate = Some(this);

    let listener = world.obj::<GameServer>(this).game_server_listener;
    if let Some(listener) = listener {
        crate::controller::check_all_achievements_and_leaderboards(world, listener);
    }
}

/// `-[GameServer mainHandleTurnEventForMatch:]` @ 0x10008ae78
///
/// The first listener that claims the match handles it.
pub fn main_handle_turn_event_for_match(world: &mut World, this: Entity, game_match: Entity) {
    let listeners = authentication_listeners(world, this);
    for listener in listeners {
        let handled = crate::send::handle_turn_event_for_active_match(world, Some(listener), game_match);
        if handled {
            break;
        }
    }
}

/// `-[GameServer handleInviteFromGameCenter:]` @ 0x10008af98
pub fn handle_invite_from_game_center(_world: &mut World, _this: Entity, _players_to_invite: Entity) {}

/// `-[GameServer handleTurnEventForMatch:]` @ 0x10008af9c
pub fn handle_turn_event_for_match(world: &mut World, this: Entity, game_match: Entity) {
    let call = MainThreadCall::MainHandleTurnEventForMatch(game_match);
    perform_on_main_thread(world, this, call);
}

/// `-[GameServer handleTurnEventForMatch:didBecomeActive:]` @ 0x10008afc4
pub fn handle_turn_event_for_match_did_become_active(world: &mut World, this: Entity, game_match: Entity, _did_become_active: bool) {
    let call = MainThreadCall::MainHandleTurnEventForMatch(game_match);
    perform_on_main_thread(world, this, call);
}

/// `-[GameServer mainHandleMatchEnded:]` @ 0x10008afec
///
/// The first listener that claims the match handles it.
pub fn main_handle_match_ended(world: &mut World, this: Entity, game_match: Entity) {
    let listeners = authentication_listeners(world, this);
    for listener in listeners {
        let handled = crate::send::handle_match_ended_for_active_match(world, Some(listener), game_match);
        if handled {
            break;
        }
    }
}

/// `-[GameServer handleMatchEnded:]` @ 0x10008b10c
pub fn handle_match_ended(world: &mut World, this: Entity, game_match: Entity) {
    let call = MainThreadCall::MainHandleMatchEnded(game_match);
    perform_on_main_thread(world, this, call);
}

/// `-[GameServer isTurnBasedMultiplayerAvailable]` @ 0x10008b134
pub fn is_turn_based_multiplayer_available(world: &mut World, _this: Entity) -> bool {
    // `NSClassFromString(@"GKTurnBasedMatch")`. PORT: turn-based matches exist only while a
    // match service (`docs/re/managers-platform.md` §18.3) serves GameKit's requests.
    let turn_based_match_class = world.contains_resource::<GameKitService>();

    // `[[[UIDevice currentDevice] systemVersion] compare:@"5.0" options:NSNumericSearch]` is
    // never `NSOrderedAscending` (iOS ≥ 5, §18.1 row 12).
    let system_version_at_least_5 = true;

    system_version_at_least_5 && turn_based_match_class
}

/// `-[GameServer checkShowAuthenticationViewController]` @ 0x10008b1b4
pub fn check_show_authentication_view_controller(world: &mut World, this: Entity) -> bool {
    world.obj_mut::<GameServer>(this).startup_check_show_authentication_view_controller = true;

    let sheet_pending = world.obj::<GameServer>(this).authentication_view_controller.is_some();
    if sheet_pending {
        perform_selector_on_main_thread(world, this, show_authentication_view_controller);
    }
    sheet_pending
}

/// `-[GameServer showAuthenticationViewController]` @ 0x10008b214
pub fn show_authentication_view_controller(world: &mut World, this: Entity) {
    let server = world.obj::<GameServer>(this);
    let listener = server.game_server_listener;
    let sheet = server.authentication_view_controller;

    // PORT: with no sheet stored the original presents nil, which UIKit rejects with an
    // exception; the port presents nothing. (The GameKit stand-in never hands over a sheet.)
    if let Some(listener) = listener
        && let Some(sheet) = sheet
    {
        crate::controller::present_modal_view_controller(world, listener, sheet);
    }
}

/// `-[GameServer hideAuthenticationViewController]` @ 0x10008b240
pub fn hide_authentication_view_controller(world: &mut World, this: Entity) {
    let listener = world.obj::<GameServer>(this).game_server_listener;
    if let Some(listener) = listener {
        crate::controller::dismiss_modal_view_controller(world, listener);
    }
    world.obj_mut::<GameServer>(this).authentication_view_controller = None;
}

/// `-[GameServer authenticateUser]` @ 0x10008b298
pub fn authenticate_user(world: &mut World, this: Entity) {
    // `[GKLocalPlayer localPlayer]`.
    let listeners = authentication_listeners(world, this);
    for listener in listeners {
        crate::send::on_game_server_authentication_started(world, Some(listener));
    }

    // `[localPlayer respondsToSelector:@selector(authenticateHandler)]` holds on every iOS the
    // arm64 build runs on (7 and later), so the iOS 5 branch (`authenticateWithCompletionHandler:`
    // with the block `FUN_10008b678`) never runs. `[localPlayer setAuthenticateHandler:block]`:
    // PORT: the port's GameKit stand-in (`docs/re/managers-platform.md` §18.1 row 13) signs in at
    // once, without a sign-in sheet, and runs the block.
    authenticate_handler(world, this, None);
}

/// `FUN_10008b48c`: the `authenticateHandler` block of `authenticateUser`, with the sign-in
/// sheet GameKit hands over (`None` for nil). `[localPlayer isAuthenticated]` and the error are
/// only logged.
fn authenticate_handler(world: &mut World, this: Entity, view_controller: Option<Entity>) {
    if let Some(view_controller) = view_controller {
        world.obj_mut::<GameServer>(this).authentication_view_controller = Some(view_controller);

        let startup_check = world.obj::<GameServer>(this).startup_check_show_authentication_view_controller;
        if startup_check {
            perform_selector_on_main_thread(world, this, show_authentication_view_controller);
        }
        return;
    }

    let sheet = world.obj::<GameServer>(this).authentication_view_controller;
    if sheet.is_some() {
        perform_selector_on_main_thread(world, this, hide_authentication_view_controller);
    }
    perform_selector_on_main_thread(world, this, on_game_center_authentication_complete);
}

/// `-[GameServer canReceiveCallbacksNow]` @ 0x10008b70c
pub fn can_receive_callbacks_now(_world: &mut World, _this: Entity) -> bool {
    true
}

/// `DAT_1007e50a0 == DAT_1007e50a4`: the dead score queue is empty (always, as nothing fills it).
fn score_queue_empty(world: &World) -> bool {
    match world.get_resource::<GameServerStatics>() {
        Some(statics) => statics.score_queue_head == statics.score_queue_tail,
        None => true,
    }
}

/// `-[GameServer onScoreSuccess]` @ 0x10008b714
pub fn on_score_success(world: &mut World, this: Entity) {
    if score_queue_empty(world) {
        world.obj_mut::<GameServer>(this).submitting_score = false;
        return;
    }
    submit_next_score(world, this);
}

/// `-[GameServer onScoreFailure]` @ 0x10008b74c
pub fn on_score_failure(world: &mut World, this: Entity) {
    on_score_success(world, this);
}

/// `-[GameServer onPendingHighScoreSuccess]` @ 0x10008b75c
pub fn on_pending_high_score_success(_world: &mut World, _this: Entity) {}

/// `-[GameServer onPendingHighScoreFailure]` @ 0x10008b794
pub fn on_pending_high_score_failure(_world: &mut World, _this: Entity) {}

/// `-[GameServer onPendingAchievementSuccess]` @ 0x10008b7cc
pub fn on_pending_achievement_success(_world: &mut World, _this: Entity) {}

/// `-[GameServer onPendingAchievementFailure]` @ 0x10008b804
pub fn on_pending_achievement_failure(_world: &mut World, _this: Entity) {}

/// `-[GameServer onUnlockAchievementsSuccess]` @ 0x10008b83c
pub fn on_unlock_achievements_success(world: &mut World, this: Entity) {
    if score_queue_empty(world) {
        world.obj_mut::<GameServer>(this).submitting_score = false;
        return;
    }
    submit_next_score(world, this);
}

/// `-[GameServer onUnlockAchievementsFailure]` @ 0x10008b874
pub fn on_unlock_achievements_failure(world: &mut World, this: Entity) {
    on_score_success(world, this);
}

/// `-[GameServer submitNextScore]` @ 0x10008b884
pub fn submit_next_score(_world: &mut World, _this: Entity) {}

/// `-[GameServer sendPendingData]` @ 0x10008b888
pub fn send_pending_data(_world: &mut World, _this: Entity) {}

/// `-[GameServer checkLeaderboardQueue]` @ 0x10008b88c
pub fn check_leaderboard_queue(_world: &mut World, _this: Entity) {}

/// `-[GameServer checkLeaderboardQueuePool]` @ 0x10008b890
pub fn check_leaderboard_queue_pool(_world: &mut World, _this: Entity) {}

/// `-[GameServer submitToLeaderboard:score:]` @ 0x10008b894
pub fn submit_to_leaderboard(world: &mut World, _this: Entity, leaderboard_id: &str, score: i32) {
    // `[[GKScore alloc] initWithCategory:leaderboardId]`, `value = (int64_t)score`, `context = 0`,
    // then `reportScoreWithCompletionHandler:`; its block (`FUN_10008b928`) only logs an error.
    let value = i64::from(score);
    report_score(world, leaderboard_id, value);
}

/// `-[GKScore reportScoreWithCompletionHandler:]`. PORT: there is no Game Center; the port keeps
/// the best value of each leaderboard on the device (`docs/re/managers-platform.md` §3.5).
fn report_score(world: &mut World, category: &str, value: i64) {
    let mut leaderboards = world.get_resource_or_insert_with(LocalLeaderboards::default);
    let best_score = leaderboards.best_scores.entry(category.to_owned()).or_insert(value);
    if value > *best_score {
        *best_score = value;
    }
}

/// `-[GameServer submitAchievement:]` @ 0x10008b970
pub fn submit_achievement(_world: &mut World, _this: Entity, _achievement_id: &str) {
    // `[[GKAchievement alloc] initWithIdentifier:achievementId]` (never nil here) with
    // `percentComplete = 100.0`, then `reportAchievementWithCompletionHandler:`; its block
    // (`FUN_10008ba00`) only logs an error.
    // PORT: there is no Game Center, and the port keeps no achievement record of its own: the
    // controller already stores the tier achievements in the System document
    // (`docs/re/managers-platform.md` §3.5).
}

/// `-[GameServer onGameFrame:]` @ 0x10008ba50
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `+[GameServer instance]` @ 0x10008a90c
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().game_server
}
