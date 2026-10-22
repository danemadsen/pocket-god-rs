//! `GameServer` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod game_center_server;
pub use game_center_server::*;

/// `GameServer` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct GameServer {
    /// `mGameServerListener`: `@"<GameServerListener>"`
    pub game_server_listener: Option<Entity>,

    /// `mGameServerLeaderboardListener`: `@"<GameServerLeaderboardListener>"`
    pub game_server_leaderboard_listener: Option<Entity>,

    /// `mGameServerAchievementListener`: `@"<GameServerAchievementListener>"`
    pub game_server_achievement_listener: Option<Entity>,

    /// `mGameServerChallengeListener`: `@"<GameServerChallengeListener>"`
    pub game_server_challenge_listener: Option<Entity>,

    /// `mGameServerAuthenticationListenerArray`: `@"NSMutableArray"`
    pub game_server_authentication_listener_array: Vec<Entity>,

    /// `mbGameCenterNotAvailable`: `B`
    pub game_center_not_available: bool,

    /// `mbStartupCheckShowAuthenticationViewController`: `B`
    pub startup_check_show_authentication_view_controller: bool,

    /// `mAuthenticationViewController`: `@"UIViewController"`
    pub authentication_view_controller: Option<Entity>,

    /// `mbSubmittingScore`: `B`
    pub submitting_score: bool,

    /// `mLeaderboardDictionary`: `@"NSMutableDictionary"`
    pub leaderboard_dictionary: HashMap<String, Entity>,

    /// `mFlushFrameCounter`: `i`
    pub flush_frame_counter: i32,
}

pub const GAME_SERVER: Class = Class {
    name: "GameServer",
    superclass: None,
    game_frame_order: 70,
    id,
    on_game_frame,
    ..Class::NS_OBJECT
};
