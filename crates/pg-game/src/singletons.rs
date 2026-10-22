//! The `+instance` singletons. Each class keeps its instance in a static that its `init`
//! sets (`sInstance = self`); here that static is a field of [`Singletons`], and the class's
//! `instance(world)` reads it back (`None` until the object exists, like a nil static).

use pg_engine::prelude::*;

/// The static instance variable of every class with `+instance`.
#[derive(Resource, Default, Clone, Debug)]
pub struct Singletons {
    pub apocalypse_temple_fountain_logic: Option<Entity>,
    pub apocalypse_time_temple_platform_logic: Option<Entity>,
    pub bonus_pack_manager: Option<Entity>,
    pub challenge_manager: Option<Entity>,
    pub closet_temple_closet_door_logic: Option<Entity>,
    pub closet_temple_couch_logic: Option<Entity>,
    pub closet_temple_thetan_meter_logic: Option<Entity>,
    pub dance_dance_execution_screen_logic: Option<Entity>,
    pub debug_screen: Option<Entity>,
    pub fail_boat_logic: Option<Entity>,
    pub game_server: Option<Entity>,
    pub game_ui_manager: Option<Entity>,
    pub idol_manager: Option<Entity>,
    pub idol_screen_logic: Option<Entity>,
    pub locust_queen_logic: Option<Entity>,
    pub map_screen: Option<Entity>,
    pub moron_pests_question_screen_logic: Option<Entity>,
    pub moron_snake_logic: Option<Entity>,
    pub no_pee_logic: Option<Entity>,
    pub pocket_god_view_controller: Option<Entity>,
    pub skin_pack_manager: Option<Entity>,
    pub temple_door_logic: Option<Entity>,
    pub underwater_temple_logic: Option<Entity>,
    pub xenu_logic: Option<Entity>,
    pub xenu_temple_logic: Option<Entity>,
}
