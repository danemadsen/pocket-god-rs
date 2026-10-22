//! C structs used by game-class ivars (field names from the ObjC type encodings).

#![allow(dead_code)]

use crate::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct GamePoints {
    pub message_logic: Option<Entity>,
    pub game_points: i32,
    pub sequence_id: Option<String>,
    pub label: Option<String>,
    pub pos: Vec4,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HurricaneSpinState {
    pub stall: i32,
    pub radius: f32,
    pub radius_rot: f32,
    pub radius_rot_vel: f32,
    pub y_pos: f32,
    pub y_pos_rot: f32,
    pub y_pos_rot_vel: f32,
    pub angle: f32,
    pub angle_vel: f32,
    pub strength: f32,
    pub strength_vel: f32,
}

#[derive(Clone, Debug, Default)]
pub struct HurricaneWindPos {
    pub pos: Vec4,
    pub pos_vel: Vec4,
    pub wind_rot: f32,
    pub wind_rot_vel: f32,
    pub radius: f32,
    pub radius_vel: f32,
    pub draw_pos: f32,
    pub draw_pos_vel: f32,
    pub line_width: f32,
    pub segment_count: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct IdolBattleState {
    pub active_frame_count: i32,
    pub struct_xml_episode_node_def: Option<usize>,
    pub idol_battle_realm_effect_logic: Option<Entity>,
    pub damage: f32,
    pub health: f32,
    pub used_count: i32,
    pub experience: f32,
    pub frozen_ref_count: i32,
    pub freeze_idol: bool,
    pub frozen_idol_battle_state: Option<usize>,
    pub resurrect: bool,
    pub resurrect_health_per_level: f32,
    pub potential_attack_strength: f32,
    pub potential_realm_strength: f32,
    pub potential_strength: f32,
    pub enemy_attack_lose_count: i32,
    pub enemy_attack_win_count: i32,
    pub enemy_attack_tie_count: i32,
    pub enemy_realm_lose_count: i32,
    pub enemy_realm_win_count: i32,
    pub enemy_realm_tie_count: i32,
    pub attack_type_index: i32,
    pub realm_type_index: i32,
    pub choice_score: f32,
}

#[derive(Clone, Debug, Default)]
pub struct KonkeyDongLevel {
    pub display_group: Option<Entity>,
    pub platform_logic_array: Vec<Entity>,
    pub ladder_logic_array: Vec<Entity>,
    pub pick_logic_array: Vec<Entity>,
    pub beetle_logic_array: Vec<Entity>,
    pub vegetation_logic: Option<Entity>,
    pub level_width: f32,
    pub level_height: f32,
    pub kong_pos: Vec4,
    pub start_pygmy_pos: Vec4,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerBattleState {
    pub name: Option<String>,
    pub level: i32,
    pub health: f32,
    pub max_health: f32,
    pub player_struct_xml_episode_node_def: Option<usize>,
    pub battle_struct_xml_episode_node_def: Option<usize>,
    pub win_attack: bool,
    pub win_realm: bool,
    pub actual_attack_damage: f32,
    pub actual_reflect_damage: f32,
    pub actual_drain_health: f32,
    pub total_boost_damage: f32,
    pub attack_boost_damage: f32,
    pub attack_reflect_damage: f32,
    pub attack_drain_health: f32,
    pub realm_damage: f32,
    pub tie_damage: f32,
    pub total_defense: f32,
    pub attack_defense: f32,
    pub realm_health: f32,
    pub total_drain_level_points: f32,
    pub drain_level_points: f32,
    pub ignore_attack_defense: bool,
    pub cancel_enemy_power_ups: bool,
    pub unfreeze_idols: bool,
    pub average_damage: f32,
    pub has_freeze_idol: bool,
    pub attack_type_count_array: [i32; 3],
    pub realm_type_count_array: [i32; 5],
    pub idol_battle_state_array: [IdolBattleState; 4],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TheRunsPlatformEvent {
    pub the_runs_platform_event_type: i32,
    pub platform_count: i32,
    pub placed_platform_count: i32,
    pub off_screen_platform_count: i32,
    pub stalactite_count: i32,
    pub placed_stalactite_count: i32,
    pub turd_count: i32,
    pub height: f32,
    pub gap_width: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WhirlPoolSpinState {
    pub stall: i32,
    pub radius: f32,
    pub target_radius: f32,
    pub radius_rot: f32,
    pub radius_rot_vel: f32,
    pub angle: f32,
    pub angle_vel: f32,
    pub strength: f32,
    pub strength_vel: f32,
}

#[derive(Clone, Debug, Default)]
pub struct WhirlPoolWindPos {
    pub pos: Vec4,
    pub pos_vel: Vec4,
    pub wind_rot: f32,
    pub wind_rot_vel: f32,
    pub radius: f32,
    pub radius_vel: f32,
    pub draw_pos: f32,
    pub draw_pos_vel: f32,
    pub line_width: f32,
    pub segment_count: i32,
}
