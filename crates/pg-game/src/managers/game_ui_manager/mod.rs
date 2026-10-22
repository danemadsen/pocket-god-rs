//! `GameUiManager` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod game_ui_points;
pub use game_ui_points::*;

/// `GameUiManager` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct GameUiManager {
    /// `mGameUiManagerListener`: `@"<GameUiManagerListener>"`
    pub game_ui_manager_listener: Option<Entity>,

    /// `mSequence`: `@"BCSequence"`
    pub sequence: Option<Entity>,

    /// `mGamePointsLogicArray`: `@"NSMutableArray"`
    pub game_points_logic_array: Vec<Entity>,

    /// `mCustomGamePointsLogicArray`: `@"NSMutableArray"`
    pub custom_game_points_logic_array: Vec<Entity>,

    /// `mGamePointsTextArray`: `@"NSMutableArray"`
    pub game_points_text_array: Vec<Entity>,

    /// `mGamePointsLabelTextArray`: `@"NSMutableArray"`
    pub game_points_label_text_array: Vec<Entity>,

    /// `mPocketGodDisplayGroup`: `@"BCDisplayGroup"`
    pub pocket_god_display_group: Option<Entity>,

    /// `mTimeForNextGamePoints`: `f`
    pub time_for_next_game_points: f32,

    /// `mNextActiveGamePointsIndex`: `i`
    pub next_active_game_points_index: i32,

    /// `mNextFreeGamePointsIndex`: `i`
    pub next_free_game_points_index: i32,

    /// `mGamePointsArray`: `[30{GamePoints="mMessageLogic"@"MultiModelLogic""mGamePoints"i"mSequenceId"@"NSString""mLabel"@"NSString""mPos"{?="x"f"y"f"z"f"w"f}}]`
    pub game_points_array: [GamePoints; 30],
}

pub const GAME_UI_MANAGER: Class = Class {
    name: "GameUiManager",
    superclass: None,
    game_frame_order: 40,
    id,
    set_behavior,
    on_game_frame,
    on_multi_model_event,
    ..Class::NS_OBJECT
};
