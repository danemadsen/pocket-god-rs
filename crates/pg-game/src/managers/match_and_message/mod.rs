//! `MatchAndMessage` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod idol_multiplayer;
pub use idol_multiplayer::*;

/// `MatchAndMessage` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct MatchAndMessage {
    /// `mMatch`: `@"GKTurnBasedMatch"`
    pub match_: Option<Entity>,

    /// `mMessage`: `@"NSString"`
    pub message: Option<String>,
}

pub const MATCH_AND_MESSAGE: Class = Class {
    name: "MatchAndMessage",
    superclass: None,
    ..Class::NS_OBJECT
};
