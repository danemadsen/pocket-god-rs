//! `IdolManager`: methods in this part of the class.
//!
//! This part also owns the runtime state of the episode list ([`EpisodeTable`]): the members
//! the original writes into the static `BCStructXmlEpisodeNodeDef`, `…ChallengeNodeDef` and
//! `…LevelUpNodeDef` structs of the list `PTR_DAT_1007306f8` (`docs/re/struct-xml.md` §8), plus
//! `IdolManager`'s file-scope globals ([`IdolManagerStatics`]).

use std::ops::Range;
use std::sync::OnceLock;

use pg_assets::data::episodes::{self, Challenge, Episode, EpisodesFile, LevelUp};
use pg_engine::texture::{Patch, Textures};

use crate::controller::ControllerWorldRegistry;
use crate::logic::battle::{god_idol_logic, temple_door_logic};
use crate::logic::idol_screen::idol_screen_logic;
use crate::logic::idol_ui_widgets::episode_alert_logic;
use crate::managers::game_server;
use crate::prelude::*;

use super::*;

/// Episodes in the list `PTR_DAT_1007306f8` (`episodes.json`, Episode1 … Episode48 with 31B).
pub const EPISODE_LIST_LEN: usize = 49;

/// First of the four battle copies of the player's selected idols (`DAT_1007cdc78 + i * 0xe8`,
/// filled by `initBattleOfTheGodsEnemy` and `initBattleOfTheGodsMultiPlayer`).
pub const PLAYER_BATTLE_EPISODE_COPIES: usize = EPISODE_LIST_LEN;

/// First of the four battle copies of the enemy's idols (`DAT_1007ce018 + i * 0xe8`).
pub const ENEMY_BATTLE_EPISODE_COPIES: usize = EPISODE_LIST_LEN + 4;

/// Episode nodes: the list, then the eight battle copies.
pub const EPISODE_NODE_COUNT: usize = EPISODE_LIST_LEN + 8;

/// `@"IdolManagerCheckAutoUnlockChallenges"`: waits until challenges may trigger, then runs
/// `checkAutoUnlockChallenges:::`.
const CHECK_AUTO_UNLOCK_SEQUENCE: &str = "IdolManagerCheckAutoUnlockChallenges";

/// `@"ActiveEpisode"`: the option preference holding the active episode's id.
const ACTIVE_EPISODE_KEY: &str = "ActiveEpisode";

/// `DAT_10071b8b0`, an episode `IdolManager` names by address.
const EPISODE_1: &str = "Episode1";

/// `DAT_10071bb68`
const EPISODE_2: &str = "Episode2";

/// `DAT_100728a18`
const EPISODE_39: &str = "Episode39";

/// The episode list's data (`episodes.json`, list order), parsed once.
pub fn episode_defs() -> &'static [Episode] {
    static FILE: OnceLock<EpisodesFile> = OnceLock::new();
    let file = FILE.get_or_init(load_episode_file);
    &file.episodes
}

fn load_episode_file() -> EpisodesFile {
    episodes::load().expect("assets/data/episodes.json matches its schema")
}

/// The list index of the episode with id `id`.
pub fn episode_index(id: &str) -> Option<usize> {
    let defs = episode_defs();
    defs.iter().position(|episode| episode.id == id)
}

/// The list index of an episode `IdolManager` refers to by address.
fn list_episode(id: &str) -> usize {
    let index = episode_index(id);
    index.expect("the episode list holds every episode IdolManager names")
}

/// The runtime members of one `BCStructXmlEpisodeNodeDef` (a list episode or a battle copy).
/// A `BCStructXmlEpisodeNodeDef *` is an index into [`EpisodeTable::nodes`]: `0..49` are the
/// list episodes in list order, then the player's ([`PLAYER_BATTLE_EPISODE_COPIES`]) and the
/// enemy's ([`ENEMY_BATTLE_EPISODE_COPIES`]) battle copies. A copy (`memcpy` of a node) clones
/// the node; it keeps sharing the copied episode's challenge and level-up nodes.
#[derive(Clone, Debug, Default)]
pub struct EpisodeNode {
    /// Index in [`episode_defs`] of the episode whose data this node holds (+0x08 … +0x80,
    /// +0xa8 … +0xb0). The never-filled battle copies (zero in the original) point at the
    /// first episode.
    pub data: usize,

    /// +0x18: the challenge list, indices into [`EpisodeTable::challenges`].
    pub challenges: Range<usize>,

    /// +0x20: the level-up list, indices into [`EpisodeTable::level_ups`].
    pub level_ups: Range<usize>,

    /// +0x88: idol level (the lowest level-up level, 0 without level-ups).
    pub level: i32,

    /// +0x8c: every challenge is complete (the idol is earned).
    pub complete: bool,

    /// +0x90
    pub completed_challenge_count: i32,

    /// +0x94
    pub challenge_count: i32,

    /// +0x98
    pub level_up_count: i32,

    /// +0x9c: challenges whose hint is activated.
    pub activated_hint_count: i32,

    /// +0xa0: selection slot 0–3, −1 when not selected.
    pub selection_slot: i32,

    /// +0xa4
    pub battle_unlocked: bool,

    /// +0xb8: the `IdolIconLogic` of the idol list panel.
    pub idol_icon_logic: Option<Entity>,

    /// +0xc0: its scroll position.
    pub idol_icon_pos: Vec4,

    /// +0xd0: the `IdolSelectionIconLogic` of the selection and battle-enemy panels.
    pub idol_selection_icon_logic: Option<Entity>,

    /// +0xd8: its scroll position.
    pub idol_selection_icon_pos: Vec4,
}

/// The runtime members of one `BCStructXmlEpisodeChallengeNodeDef`. A
/// `BCStructXmlEpisodeChallengeNodeDef *` is an index into [`EpisodeTable::challenges`].
#[derive(Clone, Debug, Default)]
pub struct ChallengeNode {
    /// List index of the episode that lists this challenge.
    pub episode: usize,

    /// Index of the challenge in that episode's `challenges`.
    pub index: usize,

    /// +0x38
    pub completed: bool,

    /// +0x39
    pub hint_activated: bool,

    /// +0x50: preference key `"%@%@Challenge"` (episode id, event id); nil until
    /// `loadEpisodeState`.
    pub preference_key: Option<String>,

    /// +0x58: preference key `"%@%@ChallengeHint"`.
    pub hint_preference_key: Option<String>,
}

impl ChallengeNode {
    /// The challenge's data (`event_id` +0x08 … `hint` +0x30).
    pub fn def(&self) -> &'static Challenge {
        let episode = &episode_defs()[self.episode];
        &episode.challenges[self.index]
    }
}

/// The runtime members of one `BCStructXmlEpisodeLevelUpNodeDef`, an index into
/// [`EpisodeTable::level_ups`].
#[derive(Clone, Debug, Default)]
pub struct LevelUpNode {
    /// List index of the episode that lists this level-up.
    pub episode: usize,

    /// Index of the level-up in that episode's `level_ups`.
    pub index: usize,

    /// +0x2c: points (preference `"%@%@LevelUp"`).
    pub points: i32,

    /// +0x30: points where the current level starts.
    pub level_start_points: i32,

    /// +0x34: points of the next level (0 at the cap).
    pub next_level_points: i32,

    /// +0x38: level 0–19.
    pub level: i32,

    /// +0x48: preference key `"%@%@LevelUp"` (episode id, event id); nil until
    /// `loadEpisodeState`.
    pub preference_key: Option<String>,
}

impl LevelUpNode {
    /// The level-up's data (`event_id` +0x08 … `max_points` +0x28).
    pub fn def(&self) -> &'static LevelUp {
        let episode = &episode_defs()[self.episode];
        &episode.level_ups[self.index]
    }
}

/// The runtime state of the episode list and its sub-lists (see [`EpisodeNode`]).
#[derive(Resource, Clone, Debug)]
pub struct EpisodeTable {
    pub nodes: Vec<EpisodeNode>,
    pub challenges: Vec<ChallengeNode>,
    pub level_ups: Vec<LevelUpNode>,
}

impl Default for EpisodeTable {
    /// The table as the binary holds it: runtime members zero, selection slots −1.
    fn default() -> Self {
        let defs = episode_defs();
        assert_eq!(defs.len(), EPISODE_LIST_LEN, "episodes.json holds the whole episode list");

        let mut nodes = Vec::with_capacity(EPISODE_NODE_COUNT);
        let mut challenges = Vec::new();
        let mut level_ups = Vec::new();

        for (episode, def) in defs.iter().enumerate() {
            let first_challenge = challenges.len();
            for index in 0..def.challenges.len() {
                let challenge = ChallengeNode {
                    episode,
                    index,
                    ..Default::default()
                };
                challenges.push(challenge);
            }

            let first_level_up = level_ups.len();
            for index in 0..def.level_ups.len() {
                let level_up = LevelUpNode {
                    episode,
                    index,
                    ..Default::default()
                };
                level_ups.push(level_up);
            }

            let node = EpisodeNode {
                data: episode,
                challenges: first_challenge..challenges.len(),
                level_ups: first_level_up..level_ups.len(),
                selection_slot: def.initial_selection_slot,
                ..Default::default()
            };
            nodes.push(node);
        }

        let battle_copy = EpisodeNode::default();
        nodes.resize(EPISODE_NODE_COUNT, battle_copy);

        Self {
            nodes,
            challenges,
            level_ups,
        }
    }
}

impl EpisodeTable {
    /// The data of episode node `episode`.
    pub fn def(&self, episode: usize) -> &'static Episode {
        let data = self.nodes[episode].data;
        &episode_defs()[data]
    }
}

/// Read access to the episode table, created on first use.
pub fn episode_table(world: &mut World) -> &EpisodeTable {
    world.init_resource::<EpisodeTable>();
    world.resource::<EpisodeTable>()
}

/// Write access to the episode table, created on first use.
pub fn episode_table_mut(world: &mut World) -> Mut<'_, EpisodeTable> {
    world.get_resource_or_insert_with(EpisodeTable::default)
}

/// `IdolManager`'s file-scope globals (the battle copies are in [`EpisodeTable`]).
#[derive(Resource, Clone, Debug, Default)]
pub struct IdolManagerStatics {
    /// `DAT_1007cdc28` … `DAT_1007cdc74`: points of each idol level on a 190000-point track
    /// (`1000·i(i+1)/2`), filled by `init`.
    pub level_thresholds: [i32; 20],
}

/// A `BCSkinTextureBitsDef` record: a destination rect (texels) in a texture def's slot.
pub(crate) struct SkinTextureBitsDef {
    pub texture: &'static str,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// `PTR_PTR_100569a68`: the episode alert's icon slot.
const EPISODE_ALERT_ICON_SLOT: SkinTextureBitsDef = SkinTextureBitsDef {
    texture: "EpisodeAlertIconSlot",
    x: 128,
    y: 164,
    width: 84,
    height: 84,
};

/// `PTR_PTR_100569a98`: the episode alert's idol slot.
const EPISODE_ALERT_IDOL_SLOT: SkinTextureBitsDef = SkinTextureBitsDef {
    texture: "EpisodeAlertIdolSlot",
    x: 0,
    y: 66,
    width: 128,
    height: 128,
};

/// `FUN_100038648(records, file, @"raw")` (`activateSkinTextureBitsBatchDef`) for an episode
/// bits slot: uploads `<file>.raw` into the record's rect of its def's slot. Every episode slot
/// holds a single record, so the data starts at the image's origin.
pub(crate) fn activate_episode_bits_slot(world: &mut World, record: &SkinTextureBitsDef, file: &str) {
    let Some(def) = pg_engine::library::texture_def_id(world, record.texture) else {
        return;
    };
    let atlas = world.resource::<Textures>().def(def).atlas;
    let Some(atlas) = atlas else {
        return;
    };

    let patch = Patch {
        image: file,
        src_x: 0,
        src_y: 0,
        width: record.width,
        height: record.height,
        dst_x: record.x,
        dst_y: record.y,
    };
    pg_engine::texture::patch_atlas(world, atlas, patch);
}

/// `mIdolManagerListener`: always the view controller (`setIdolManagerListener:` right after
/// `init` in `loadGame`).
fn listener(world: &World, this: Entity) -> Entity {
    let listener = world.obj::<IdolManager>(this).idol_manager_listener;
    listener.expect("loadGame sets the IdolManager listener")
}

/// `[prefs setPreference:key asInt:value]`. A nil document ignores it; so does a nil key, which
/// only exists before `loadEpisodeState` (the original's dictionary would raise).
fn set_int_preference(world: &mut World, prefs: Option<Entity>, key: Option<&str>, value: i32) {
    let (Some(prefs), Some(key)) = (prefs, key) else {
        return;
    };
    pg_engine::prefs::set_preference_as_int(world, prefs, key, value);
}

/// `[prefs getIntPreference:key]`: 0 for a nil document or key.
fn get_int_preference(world: &World, prefs: Option<Entity>, key: Option<&str>) -> i32 {
    let (Some(prefs), Some(key)) = (prefs, key) else {
        return 0;
    };
    pg_engine::prefs::get_int_preference(world, prefs, key)
}

/// `[prefs setDefaultPreference:key asInt:value]`.
fn set_default_int_preference(world: &mut World, prefs: Option<Entity>, key: &str, value: i32) {
    if let Some(prefs) = prefs {
        pg_engine::prefs::set_default_preference_as_int(world, prefs, key, value);
    }
}

/// `[prefs setPreference:key asString:value]`.
fn set_string_preference(world: &mut World, prefs: Option<Entity>, key: &str, value: &str) {
    if let Some(prefs) = prefs {
        pg_engine::prefs::set_preference_as_string(world, prefs, key, value);
    }
}

/// `[prefs getStringPreference:key]`.
fn get_string_preference(world: &World, prefs: Option<Entity>, key: &str) -> Option<String> {
    let prefs = prefs?;
    pg_engine::prefs::get_string_preference(world, prefs, key)
}

/// `[prefs flush]`.
fn flush_preferences(world: &mut World, prefs: Option<Entity>) {
    if let Some(prefs) = prefs {
        pg_engine::prefs::flush(world, prefs);
    }
}

/// `"%@SelectionSlot"` with the episode's id.
fn selection_slot_key(def: &Episode) -> String {
    format!("{}SelectionSlot", def.id)
}

/// `"%@BattleUnlock"` with the episode's id.
fn battle_unlock_key(def: &Episode) -> String {
    format!("{}BattleUnlock", def.id)
}

/// `[mSequence setSequenceDef:[[BCLibrary instance] getSequenceDefById:
/// @"IdolManagerCheckAutoUnlockChallenges"]]`.
fn restart_check_auto_unlock_sequence(world: &mut World, this: Entity) {
    let sequence = world.obj::<IdolManager>(this).sequence;
    let def = pg_engine::library::sequence_def(world, CHECK_AUTO_UNLOCK_SEQUENCE);
    if let Some(sequence) = sequence {
        pg_engine::sequence::set_sequence_def(world, sequence, def);
    }
}

/// `[[mIdolManagerListener godIdolLogicArray] objectAtIndex:slot]`.
fn god_idol_logic_in_slot(world: &World, this: Entity, slot: i32) -> Entity {
    let listener = listener(world, this);
    let registry = world.obj::<ControllerWorldRegistry>(listener);
    registry.god_idol_logic_array[slot as usize]
}

/// `[mEpisodeAlertLogic sparkle]`.
fn sparkle_episode_alert(world: &mut World, this: Entity) {
    let episode_alert_logic = world.obj::<IdolManager>(this).episode_alert_logic;
    if let Some(episode_alert_logic) = episode_alert_logic {
        episode_alert_logic::sparkle(world, episode_alert_logic);
    }
}

/// `-[IdolManager init]` @ 0x10002a1c4
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(IdolManager::default());

    // The episode list is static data in the original: make its state exist from the start.
    world.init_resource::<EpisodeTable>();

    let mut singletons = world.get_resource_or_insert_with(Singletons::default);
    singletons.idol_manager = Some(this);

    let sequence = pg_engine::sequence::spawn(world, None, Some(this));
    world.obj_mut::<IdolManager>(this).sequence = Some(sequence);

    let idol_screen_logic = pg_engine::library::logic(world, crate::logic_table::IDOL_SCREEN);
    world.obj_mut::<IdolManager>(this).idol_screen_logic = Some(idol_screen_logic);

    let episode_alert_logic = pg_engine::library::logic(world, crate::logic_table::EPISODE_ALERT);
    world.obj_mut::<IdolManager>(this).episode_alert_logic = Some(episode_alert_logic);

    let battle_alert_logic = pg_engine::library::logic(world, crate::logic_table::BATTLE_ALERT);
    world.obj_mut::<IdolManager>(this).battle_alert_logic = Some(battle_alert_logic);

    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.gem_id_dictionary = HashMap::with_capacity(5);
        manager.delayed_episode_event_array = Vec::with_capacity(5);
        manager.delayed_episode_gift_event_array = Vec::with_capacity(5);
        manager.delayed_episode_sacrifice_event_array = Vec::with_capacity(5);
        manager.delayed_award_screen_array = Vec::with_capacity(5);
    }

    let mut level_thresholds = [0; 20];
    let mut points = 0;
    let mut step = 1000;
    for threshold in level_thresholds.iter_mut() {
        *threshold = points;
        points += step;
        step += 1000;
    }
    let mut statics = world.get_resource_or_insert_with(IdolManagerStatics::default);
    statics.level_thresholds = level_thresholds;

    {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.game_center_authentication_complete = false;
        manager.game_center_not_available = false;
    }

    let game_server = game_server::instance(world);
    if let Some(game_server) = game_server {
        game_server::add_game_server_authentication_listener(world, game_server, this);
    }
}

/// `-[IdolManager id]` @ 0x10002a420
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some("IdolManager".to_owned())
}

/// `-[IdolManager setIdolManagerListener:]` @ 0x10002a42c
pub fn set_idol_manager_listener(world: &mut World, this: Entity, idol_manager_listener: Entity) {
    world.obj_mut::<IdolManager>(this).idol_manager_listener = Some(idol_manager_listener);
}

/// `-[IdolManager dealloc]` @ 0x10002a488
pub fn dealloc(world: &mut World, this: Entity) {
    let game_server = game_server::instance(world);
    if let Some(game_server) = game_server {
        game_server::remove_game_server_authentication_listener(world, game_server, this);
    }

    {
        let mut table = episode_table_mut(world);
        for episode in 0..EPISODE_LIST_LEN {
            let node = table.nodes[episode].clone();
            for challenge in node.challenges {
                let state = &mut table.challenges[challenge];
                state.preference_key = None;
                state.hint_preference_key = None;
            }
            for level_up in node.level_ups {
                table.level_ups[level_up].preference_key = None;
            }
        }
    }

    let sequence = world.obj::<IdolManager>(this).sequence;
    if let Some(sequence) = sequence {
        pg_engine::sequence::stop(world, sequence);
        world.despawn(sequence);
    }

    world.despawn(this);
}

/// `-[IdolManager episodeAlertIcon]` @ 0x10002a694
pub fn episode_alert_icon(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<IdolManager>(this).episode_alert_logic
}

/// `-[IdolManager battleAlertIcon]` @ 0x10002a6a4
pub fn battle_alert_icon(world: &mut World, this: Entity) -> Option<Entity> {
    world.obj::<IdolManager>(this).battle_alert_logic
}

/// `-[IdolManager canTriggerChallenge]` @ 0x10002a6b4
pub fn can_trigger_challenge(world: &mut World, this: Entity) -> bool {
    let listener = listener(world, this);

    let screen_open = crate::controller::is_screen_open(world, listener);
    if screen_open {
        let temple_door = temple_door_logic::instance(world);
        let battle_active = match temple_door {
            Some(temple_door) => temple_door_logic::is_battle_of_the_gods_active(world, temple_door),
            None => false,
        };
        if !battle_active {
            return false;
        }
    }

    let story_active = crate::controller::is_story_active(world, listener);
    !story_active
}

/// `-[IdolManager canShowAwardScreen]` @ 0x10002a73c
pub fn can_show_award_screen(world: &mut World, this: Entity) -> bool {
    let listener = listener(world, this);

    let screen_open = crate::controller::is_screen_open(world, listener);
    if screen_open {
        return false;
    }
    let story_active = crate::controller::is_story_active(world, listener);
    if story_active {
        return false;
    }
    let any_game_active = crate::controller::is_any_game_active(world, listener);
    if any_game_active {
        return false;
    }

    let dialog_active = crate::controller::is_dialog_active(world, listener);
    !dialog_active
}

/// `-[IdolManager canShowIdolScreen]` @ 0x10002a7c8
pub fn can_show_idol_screen(world: &mut World, this: Entity) -> bool {
    let listener = listener(world, this);

    let screen_open = crate::controller::is_screen_open(world, listener);
    if screen_open {
        return false;
    }
    let story_active = crate::controller::is_story_active(world, listener);
    if story_active {
        return false;
    }
    let any_game_active = crate::controller::is_any_game_active(world, listener);
    if any_game_active {
        return false;
    }

    let dialog_active = crate::controller::is_dialog_active(world, listener);
    !dialog_active
}

/// `-[IdolManager checkEmptyIdolSlot:]` @ 0x10002a854
pub fn check_empty_idol_slot(world: &mut World, this: Entity, check_empty_idol_slot: Option<usize>) {
    for slot in 0..4 {
        let selected = world.obj::<IdolManager>(this).selected_struct_xml_episode_node_def_array[slot];
        if selected.is_none() {
            on_episode_idol_selected(world, this, check_empty_idol_slot, slot as i32);
            return;
        }
    }
}

/// `-[IdolManager checkImmediateUnlocks:]` @ 0x10002a88c
pub fn check_immediate_unlocks(world: &mut World, this: Entity, check_immediate_unlocks: Option<usize>) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    // Completing Episode1 completes Episode2's first challenge.
    let episode_1 = list_episode(EPISODE_1);
    if check_immediate_unlocks == Some(episode_1) {
        let episode_2 = list_episode(EPISODE_2);
        let first_challenge = episode_table(world).nodes[episode_2].challenges.start;
        let key = episode_table(world).challenges[first_challenge].preference_key.clone();
        set_int_preference(world, challenge_preferences, key.as_deref(), 1);
        episode_table_mut(world).challenges[first_challenge].completed = true;
        update_episode_status(world, this, Some(episode_2), true);
        check_empty_idol_slot(world, this, Some(episode_2));
    }

    // Once every episode before Episode39 is complete, Episode39's last challenge completes.
    let episode_39 = list_episode(EPISODE_39);
    for episode in 0..EPISODE_LIST_LEN {
        if episode == episode_39 {
            break;
        }
        let complete = episode_table(world).nodes[episode].complete;
        if !complete {
            return;
        }
    }

    let node = episode_table(world).nodes[episode_39].clone();
    let last_index = (node.challenge_count - 1) as usize;
    let last_challenge = node.challenges.start + last_index;
    let key = episode_table(world).challenges[last_challenge].preference_key.clone();
    set_int_preference(world, challenge_preferences, key.as_deref(), 1);
    episode_table_mut(world).challenges[last_challenge].completed = true;
    update_episode_status(world, this, Some(episode_39), true);

    let complete = episode_table(world).nodes[episode_39].complete;
    if complete {
        check_empty_idol_slot(world, this, Some(episode_39));
    }
}

/// `-[IdolManager updateEpisodeStatus:startup:]` @ 0x10002aa10
pub fn update_episode_status(world: &mut World, this: Entity, update_episode_status: Option<usize>, startup: bool) {
    let episode = update_episode_status.expect("updateEpisodeStatus: needs an episode");
    let statics = world.get_resource::<IdolManagerStatics>();
    let thresholds = statics.map(|statics| statics.level_thresholds).unwrap_or_default();
    let cap_points = thresholds[19] as f32;

    let newly_complete = {
        let mut table = episode_table_mut(world);
        let node = table.nodes[episode].clone();

        let mut challenge_count = 0;
        let mut completed_challenge_count = 0;
        let mut activated_hint_count = 0;
        let mut complete = true;
        for challenge in node.challenges.clone() {
            let state = &table.challenges[challenge];
            completed_challenge_count += state.completed as i32;
            complete &= state.completed;
            activated_hint_count += state.hint_activated as i32;
            challenge_count += 1;
        }

        let mut level_up_count = 0;
        let mut level = 1_000_000;
        for level_up in node.level_ups.clone() {
            let max_points = table.level_ups[level_up].def().max_points;
            let state = &mut table.level_ups[level_up];
            let track_level = update_level_up_level(state, max_points, cap_points, &thresholds);
            if track_level < level {
                level = track_level;
            }
            level_up_count += 1;
        }
        if level_up_count == 0 {
            level = 0;
        }

        let node = &mut table.nodes[episode];
        let was_complete = node.complete;
        node.level = level;
        node.complete = complete;
        node.completed_challenge_count = completed_challenge_count;
        node.challenge_count = challenge_count;
        node.level_up_count = level_up_count;
        node.activated_hint_count = activated_hint_count;

        complete && complete != was_complete
    };

    if !newly_complete || startup {
        return;
    }
    open_idol_award_screen(world, this, update_episode_status);
    check_empty_idol_slot(world, this, update_episode_status);
    check_immediate_unlocks(world, this, update_episode_status);
}

/// One level-up track of `updateEpisodeStatus:startup:`: its level from its points, scaling the
/// thresholds to the track's maximum. Returns the level.
fn update_level_up_level(state: &mut LevelUpNode, max_points: i32, cap_points: f32, thresholds: &[i32; 20]) -> i32 {
    if state.points < max_points {
        let scale = max_points as f32 / cap_points;
        for (index, threshold) in thresholds.iter().enumerate() {
            let scaled_threshold = scale * *threshold as f32;
            let next_level_points = scaled_threshold as i32;
            if state.points < next_level_points {
                // PORT: index 0 (negative points only) reads the word before the table, the
                // high half of the instance pointer (`DAT_1007cdc24`); the port reads 0.
                let previous_threshold = match index {
                    0 => 0,
                    _ => thresholds[index - 1],
                };
                let scaled_previous = scale * previous_threshold as f32;
                let level = index as i32 - 1;
                state.level = level;
                state.level_start_points = scaled_previous as i32;
                state.next_level_points = next_level_points;
                return level;
            }
        }
    }

    state.points = max_points;
    state.next_level_points = 0;
    state.level = 19;
    19
}

/// `-[IdolManager updateEpisodeHintStatus]` @ 0x10002abbc
pub fn update_episode_hint_status(world: &mut World, this: Entity) {
    world.obj_mut::<IdolManager>(this).activated_hint_count = 0;

    let mut total = 0;
    {
        let mut table = episode_table_mut(world);
        for episode in 0..EPISODE_LIST_LEN {
            let node = table.nodes[episode].clone();
            let mut count = 0;
            for challenge in node.challenges {
                if table.challenges[challenge].hint_activated {
                    total += 1;
                    count += 1;
                }
            }
            table.nodes[episode].activated_hint_count = count;
        }
    }

    world.obj_mut::<IdolManager>(this).activated_hint_count = total;
}

/// `-[IdolManager resetAllChallenges]` @ 0x10002ac2c
pub fn reset_all_challenges(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);
    let episode_1 = list_episode(EPISODE_1);

    for episode in 0..EPISODE_LIST_LEN {
        let node = episode_table(world).nodes[episode].clone();
        if node.selection_slot != -1 {
            on_episode_idol_deselected(world, this, Some(episode));
        }

        if episode != episode_1 {
            let def = episode_table(world).def(episode);
            let key = battle_unlock_key(def);
            set_int_preference(world, challenge_preferences, Some(&key), 0);
            episode_table_mut(world).nodes[episode].battle_unlocked = false;
        }

        for challenge in node.challenges {
            let state = episode_table(world).challenges[challenge].clone();
            set_int_preference(world, challenge_preferences, state.preference_key.as_deref(), 0);
            set_int_preference(world, challenge_preferences, state.hint_preference_key.as_deref(), 0);

            let mut table = episode_table_mut(world);
            table.challenges[challenge].completed = false;
            table.challenges[challenge].hint_activated = false;
        }

        for level_up in node.level_ups {
            let key = episode_table(world).level_ups[level_up].preference_key.clone();
            set_int_preference(world, challenge_preferences, key.as_deref(), 0);
            episode_table_mut(world).level_ups[level_up].points = 0;
        }

        update_episode_status(world, this, Some(episode), true);
    }

    update_episode_hint_status(world, this);
}

/// `-[IdolManager resetEpisodeState]` @ 0x10002ae14
pub fn reset_episode_state(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let option_preferences = crate::controller::option_preferences(world, listener);
    set_string_preference(world, option_preferences, ACTIVE_EPISODE_KEY, "");

    for episode in 0..EPISODE_LIST_LEN {
        let def = episode_table(world).def(episode);
        let key = selection_slot_key(def);
        set_int_preference(world, option_preferences, Some(&key), -1);
    }

    flush_preferences(world, option_preferences);
}

/// `-[IdolManager resetBattleState]` @ 0x10002af40
pub fn reset_battle_state(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    for episode in 0..EPISODE_LIST_LEN {
        let def = episode_table(world).def(episode);
        let key = battle_unlock_key(def);
        set_int_preference(world, challenge_preferences, Some(&key), 0);
    }

    flush_preferences(world, challenge_preferences);
}

/// `-[IdolManager unlockBattleState]` @ 0x10002b04c
pub fn unlock_battle_state(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    for episode in 0..EPISODE_LIST_LEN {
        let def = episode_table(world).def(episode);
        let key = battle_unlock_key(def);
        set_int_preference(world, challenge_preferences, Some(&key), 1);
    }

    flush_preferences(world, challenge_preferences);
}

/// `-[IdolManager setIdolLevels:]` @ 0x10002b158
pub fn set_idol_levels(world: &mut World, this: Entity, idol_levels: i32) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);
    let statics = world.get_resource::<IdolManagerStatics>();
    let thresholds = statics.map(|statics| statics.level_thresholds).unwrap_or_default();
    let cap_points = thresholds[19] as f32;
    let level_points = thresholds[(idol_levels - 1) as usize] as f32;

    for episode in 0..EPISODE_LIST_LEN {
        let level_ups = episode_table(world).nodes[episode].level_ups.clone();
        for level_up in level_ups {
            let max_points = episode_table(world).level_ups[level_up].def().max_points;
            let scale = max_points as f32 / cap_points;
            let scaled_points = scale * level_points;
            let points = scaled_points as i32;

            let key = {
                let mut table = episode_table_mut(world);
                let state = &mut table.level_ups[level_up];
                state.points = points;
                state.preference_key.clone()
            };
            set_int_preference(world, challenge_preferences, key.as_deref(), points);
        }

        update_episode_status(world, this, Some(episode), true);
    }

    update_episode_hint_status(world, this);
    flush_preferences(world, challenge_preferences);
}

/// `-[IdolManager resetIdolLevels]` @ 0x10002b28c
pub fn reset_idol_levels(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    for episode in 0..EPISODE_LIST_LEN {
        let level_ups = episode_table(world).nodes[episode].level_ups.clone();
        for level_up in level_ups {
            let key = {
                let mut table = episode_table_mut(world);
                let state = &mut table.level_ups[level_up];
                state.points = 0;
                state.preference_key.clone()
            };
            set_int_preference(world, challenge_preferences, key.as_deref(), 0);
        }
    }

    flush_preferences(world, challenge_preferences);
}

/// `-[IdolManager resetIdolChallenges]` @ 0x10002b338
pub fn reset_idol_challenges(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);
    let option_preferences = crate::controller::option_preferences(world, listener);
    set_string_preference(world, option_preferences, ACTIVE_EPISODE_KEY, "");

    for episode in 0..EPISODE_LIST_LEN {
        let def = episode_table(world).def(episode);
        let key = selection_slot_key(def);
        set_int_preference(world, option_preferences, Some(&key), -1);

        let node = episode_table(world).nodes[episode].clone();
        for challenge in node.challenges {
            let state = episode_table(world).challenges[challenge].clone();
            set_int_preference(world, challenge_preferences, state.preference_key.as_deref(), 0);
            set_int_preference(world, challenge_preferences, state.hint_preference_key.as_deref(), 0);
        }
        for level_up in node.level_ups {
            let key = episode_table(world).level_ups[level_up].preference_key.clone();
            set_int_preference(world, challenge_preferences, key.as_deref(), 0);
        }
    }

    flush_preferences(world, challenge_preferences);
    flush_preferences(world, option_preferences);
}

/// `-[IdolManager unlockIdolChallenges]` @ 0x10002b4fc
pub fn unlock_idol_challenges(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    for episode in 0..EPISODE_LIST_LEN {
        let challenges = episode_table(world).nodes[episode].challenges.clone();
        for challenge in challenges {
            let key = episode_table(world).challenges[challenge].preference_key.clone();
            set_int_preference(world, challenge_preferences, key.as_deref(), 1);
            episode_table_mut(world).challenges[challenge].completed = true;
        }

        update_episode_status(world, this, Some(episode), true);
    }

    flush_preferences(world, challenge_preferences);
    update_episode_hint_status(world, this);
}

/// `-[IdolManager onPreferencesChanged:]` @ 0x10002b5fc
pub fn on_preferences_changed(world: &mut World, this: Entity, _preferences_changed: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    for episode in 0..EPISODE_LIST_LEN {
        let def = episode_table(world).def(episode);
        let key = battle_unlock_key(def);
        let battle_unlock = get_int_preference(world, challenge_preferences, Some(&key));
        if battle_unlock == 1 {
            episode_table_mut(world).nodes[episode].battle_unlocked = true;
        }

        let node = episode_table(world).nodes[episode].clone();
        for challenge in node.challenges {
            let state = episode_table(world).challenges[challenge].clone();

            let completed = get_int_preference(world, challenge_preferences, state.preference_key.as_deref());
            if completed == 1 {
                episode_table_mut(world).challenges[challenge].completed = true;
            }

            let hint_activated = get_int_preference(world, challenge_preferences, state.hint_preference_key.as_deref());
            if hint_activated == 1 {
                episode_table_mut(world).challenges[challenge].hint_activated = true;
            }
        }

        for level_up in node.level_ups {
            let key = episode_table(world).level_ups[level_up].preference_key.clone();
            let points = get_int_preference(world, challenge_preferences, key.as_deref());
            episode_table_mut(world).level_ups[level_up].points = points;
        }

        update_episode_status(world, this, Some(episode), true);
    }

    update_episode_hint_status(world, this);
}

/// `-[IdolManager loadEpisodeState]` @ 0x10002b7b8
pub fn load_episode_state(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);
    let option_preferences = crate::controller::option_preferences(world, listener);
    if let Some(challenge_preferences) = challenge_preferences {
        pg_engine::prefs::set_preferences_listener(world, challenge_preferences, Some(this));
    }
    let active_episode_id = get_string_preference(world, option_preferences, ACTIVE_EPISODE_KEY);
    let episode_1 = list_episode(EPISODE_1);

    for episode in 0..EPISODE_LIST_LEN {
        let def = episode_table(world).def(episode);

        let active = active_episode_id.as_deref() == Some(def.id.as_str());
        if active {
            on_episode_activated(world, this, Some(episode));
        }

        let key = selection_slot_key(def);
        set_default_int_preference(world, option_preferences, &key, -1);
        let selection_slot = get_int_preference(world, option_preferences, Some(&key));
        episode_table_mut(world).nodes[episode].selection_slot = selection_slot;

        if selection_slot != -1 {
            let slot_index = selection_slot as usize;
            world.obj_mut::<IdolManager>(this).selected_struct_xml_episode_node_def_array[slot_index] = Some(episode);

            let god_idol_logic = god_idol_logic_in_slot(world, this, selection_slot);
            god_idol_logic::set_struct_xml_episode_node_def(world, god_idol_logic, Some(episode));
        }

        let key = battle_unlock_key(def);
        if episode == episode_1 {
            set_int_preference(world, challenge_preferences, Some(&key), 1);
            episode_table_mut(world).nodes[episode_1].battle_unlocked = true;
        } else {
            let battle_unlock = get_int_preference(world, challenge_preferences, Some(&key));
            if battle_unlock == 1 {
                episode_table_mut(world).nodes[episode].battle_unlocked = true;
            }
        }

        let node = episode_table(world).nodes[episode].clone();
        for challenge in node.challenges {
            let challenge_def = episode_table(world).challenges[challenge].def();
            if challenge_def.gem_id != 0 {
                let gem_id = challenge_def.gem_id;
                let mut manager = world.obj_mut::<IdolManager>(this);
                let gem_ids = &mut manager.gem_id_dictionary;
                gem_ids.insert(challenge_def.event_id.clone(), gem_id);
                if let Some(sacrifice_id) = &challenge_def.sacrifice_id {
                    gem_ids.insert(sacrifice_id.clone(), gem_id);
                }
                if let Some(gift_id) = &challenge_def.gift_id {
                    gem_ids.insert(gift_id.clone(), gem_id);
                }
            }

            let key = format!("{}{}Challenge", def.id, challenge_def.event_id);
            episode_table_mut(world).challenges[challenge].preference_key = Some(key.clone());
            let completed = get_int_preference(world, challenge_preferences, Some(&key));
            if completed == 1 {
                episode_table_mut(world).challenges[challenge].completed = true;
            }

            let hint_key = format!("{}{}ChallengeHint", def.id, challenge_def.event_id);
            episode_table_mut(world).challenges[challenge].hint_preference_key = Some(hint_key.clone());
            let hint_activated = get_int_preference(world, challenge_preferences, Some(&hint_key));
            if hint_activated == 1 {
                episode_table_mut(world).challenges[challenge].hint_activated = true;
            }
        }

        for level_up in node.level_ups {
            let level_up_def = episode_table(world).level_ups[level_up].def();
            let key = format!("{}{}LevelUp", def.id, level_up_def.event_id);
            episode_table_mut(world).level_ups[level_up].preference_key = Some(key.clone());
            let points = get_int_preference(world, challenge_preferences, Some(&key));
            episode_table_mut(world).level_ups[level_up].points = points;
        }

        update_episode_status(world, this, Some(episode), true);
    }

    update_episode_hint_status(world, this);
}

/// Whether the active episode's challenge `event_id` already holds in the game world
/// (`checkAutoUnlockChallenges`: weather and island location challenges).
fn is_auto_unlock_event_active(world: &mut World, listener: Entity, event_id: &str) -> bool {
    let location = match event_id {
        "StormClouds" => return crate::controller::is_storm_cloud_enabled(world, listener),
        "Night" => return crate::controller::is_night_active(world, listener),
        "Sunset" => return crate::controller::is_sunset_active(world, listener),
        "RockIsland" => 1,
        "GraveyardIsland" => 3,
        "IceIsland" => 4,
        "ApeMountain" => 5,
        "MisfitIsland" => 6,
        "GodIsland" => 7,
        "Underwater" => 2,
        "UnderwaterCoralReef" => 9,
        "ApocalypseIsland" => 10,
        "TimeTemple" => 11,
        "LocustTemple" => 12,
        "DanceTemple" => 13,
        "PlagueTemple" => 14,
        "PurificationTemple" => 15,
        "ClosetTemple" => 16,
        "XenuTemple" => 17,
        _ => return false,
    };

    let island_location_state = crate::controller::island_location_state(world, listener);
    island_location_state == location
}

/// `-[IdolManager checkAutoUnlockChallenges]` @ 0x10002bcd8
pub fn check_auto_unlock_challenges(world: &mut World, this: Entity) {
    let active = world.obj::<IdolManager>(this).active_struct_xml_episode_node_def;
    let mut incomplete_active = None;
    if let Some(active) = active {
        let complete = episode_table(world).nodes[active].complete;
        if !complete {
            incomplete_active = Some(active);
        }
    }

    if let Some(active) = incomplete_active {
        let listener = listener(world, this);
        let challenges = episode_table(world).nodes[active].challenges.clone();
        for challenge in challenges {
            let event_id = &episode_table(world).challenges[challenge].def().event_id;
            let event_active = is_auto_unlock_event_active(world, listener, event_id);
            if event_active {
                trigger_episode_event(world, this, event_id);
            }
        }

        // PORT: an event re-queued while the queue is enumerated (an award screen opened by an
        // earlier event) makes the original raise a mutation exception; the port finishes the
        // snapshot and the clear drops the re-queued events.
        let delayed_events = world.obj::<IdolManager>(this).delayed_episode_event_array.clone();
        for event in &delayed_events {
            trigger_episode_event(world, this, event);
        }
        world.obj_mut::<IdolManager>(this).delayed_episode_event_array.clear();

        let delayed_gift_events = world.obj::<IdolManager>(this).delayed_episode_gift_event_array.clone();
        for event in &delayed_gift_events {
            trigger_episode_event_by_gift_id(world, this, event);
        }
        world.obj_mut::<IdolManager>(this).delayed_episode_gift_event_array.clear();

        let delayed_sacrifice_events = world.obj::<IdolManager>(this).delayed_episode_sacrifice_event_array.clone();
        for event in &delayed_sacrifice_events {
            trigger_episode_event_by_sacrifice_id(world, this, event);
        }
        world.obj_mut::<IdolManager>(this).delayed_episode_sacrifice_event_array.clear();
    }

    let delayed_award_count = world.obj::<IdolManager>(this).delayed_award_screen_array.len();
    if delayed_award_count != 0 {
        let can_show = can_show_award_screen(world, this);
        if can_show {
            let episode = world.obj::<IdolManager>(this).delayed_award_screen_array[0];
            open_idol_award_screen(world, this, episode);
            world.obj_mut::<IdolManager>(this).delayed_award_screen_array.remove(0);
        }

        let remaining = world.obj::<IdolManager>(this).delayed_award_screen_array.len();
        if remaining != 0 {
            restart_check_auto_unlock_sequence(world, this);
        }
    }
}

/// `-[IdolManager waitForScreenClosed:::]` @ 0x10002c490
pub fn seq_wait_for_screen_closed(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let can_trigger = can_trigger_challenge(world, this);
    if can_trigger {
        ItemResult::Done
    } else {
        ItemResult::Pending
    }
}

/// `-[IdolManager checkAutoUnlockChallenges:::]` @ 0x10002c4bc
pub fn seq_check_auto_unlock_challenges(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let can_trigger = can_trigger_challenge(world, this);
    if !can_trigger {
        restart_check_auto_unlock_sequence(world, this);
        return ItemResult::Replaced;
    }

    check_auto_unlock_challenges(world, this);
    ItemResult::Done
}

/// `-[IdolManager onEpisodeIdolDeselected:]` @ 0x10002c560
pub fn on_episode_idol_deselected(world: &mut World, this: Entity, episode_idol_deselected: Option<usize>) {
    let episode = episode_idol_deselected.expect("onEpisodeIdolDeselected: needs an episode");
    let slot = episode_table(world).nodes[episode].selection_slot;
    if slot as u32 > 3 {
        return;
    }
    let selected = world.obj::<IdolManager>(this).selected_struct_xml_episode_node_def_array[slot as usize];
    if selected != Some(episode) {
        return;
    }

    let def = episode_table(world).def(episode);
    let key = selection_slot_key(def);
    let listener = listener(world, this);
    let option_preferences = crate::controller::option_preferences(world, listener);
    set_int_preference(world, option_preferences, Some(&key), -1);

    let slot = episode_table(world).nodes[episode].selection_slot;
    let god_idol_logic = god_idol_logic_in_slot(world, this, slot);
    god_idol_logic::set_struct_xml_episode_node_def(world, god_idol_logic, None);

    let slot = episode_table(world).nodes[episode].selection_slot;
    world.obj_mut::<IdolManager>(this).selected_struct_xml_episode_node_def_array[slot as usize] = None;
    episode_table_mut(world).nodes[episode].selection_slot = -1;
}

/// `-[IdolManager onEpisodeIdolSelected:slot:]` @ 0x10002c6a0
pub fn on_episode_idol_selected(world: &mut World, this: Entity, episode_idol_selected: Option<usize>, slot: i32) {
    if slot as u32 > 3 {
        return;
    }
    let episode = episode_idol_selected.expect("onEpisodeIdolSelected:slot: needs an episode");

    let current_slot = episode_table(world).nodes[episode].selection_slot;
    if current_slot != -1 {
        on_episode_idol_deselected(world, this, episode_idol_selected);
    }

    let previous = world.obj::<IdolManager>(this).selected_struct_xml_episode_node_def_array[slot as usize];
    if previous.is_some() {
        on_episode_idol_deselected(world, this, previous);
    }

    let def = episode_table(world).def(episode);
    let key = selection_slot_key(def);
    let listener = listener(world, this);
    let option_preferences = crate::controller::option_preferences(world, listener);
    set_int_preference(world, option_preferences, Some(&key), slot);

    episode_table_mut(world).nodes[episode].selection_slot = slot;
    world.obj_mut::<IdolManager>(this).selected_struct_xml_episode_node_def_array[slot as usize] = episode_idol_selected;

    let god_idol_logic = god_idol_logic_in_slot(world, this, slot);
    god_idol_logic::set_struct_xml_episode_node_def(world, god_idol_logic, episode_idol_selected);
}

/// `-[IdolManager selectedEpisodeIdolInSlot:]` @ 0x10002c81c
pub fn selected_episode_idol_in_slot(world: &mut World, this: Entity, selected_episode_idol_in_slot: i32) -> Option<usize> {
    let manager = world.obj::<IdolManager>(this);
    manager.selected_struct_xml_episode_node_def_array[selected_episode_idol_in_slot as usize]
}

/// `-[IdolManager activeEpisodeNodeDef]` @ 0x10002f2a4
pub fn active_episode_node_def(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<IdolManager>(this).active_struct_xml_episode_node_def
}

/// `-[IdolManager openIdolStartScreen]` @ 0x10002f2b4
pub fn open_idol_start_screen(world: &mut World, this: Entity) {
    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = true;
        manager.idol_screen_logic
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_start_screen(world, idol_screen_logic);
    }
}

/// `-[IdolManager openIdolListScreen]` @ 0x10002f2ec
pub fn open_idol_list_screen(world: &mut World, this: Entity) {
    let (idol_screen_logic, active) = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = true;
        (manager.idol_screen_logic, manager.active_struct_xml_episode_node_def)
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_list_screen(world, idol_screen_logic, active);
    }
}

/// `-[IdolManager openIdolSelectionScreen]` @ 0x10002f334
pub fn open_idol_selection_screen(world: &mut World, this: Entity) {
    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = true;
        manager.idol_screen_logic
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_selection_screen(world, idol_screen_logic);
    }
}

/// `-[IdolManager openIdolBattleEnemyScreen]` @ 0x10002f36c
pub fn open_idol_battle_enemy_screen(world: &mut World, this: Entity) {
    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = true;
        manager.idol_screen_logic
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_battle_enemy_screen(world, idol_screen_logic);
    }
}

/// `-[IdolManager openIdolBattleScreen]` @ 0x10002f3a4
pub fn open_idol_battle_screen(world: &mut World, this: Entity) {
    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = true;
        manager.idol_screen_logic
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_battle_screen(world, idol_screen_logic);
    }
}

/// `-[IdolManager openIdolBattleResultsScreen]` @ 0x10002f3dc
pub fn open_idol_battle_results_screen(world: &mut World, this: Entity) {
    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        manager.idol_screen_active = true;
        manager.idol_screen_logic
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_battle_results_screen(world, idol_screen_logic);
    }
}

/// `-[IdolManager openIdolDetailScreen:]` @ 0x10002f408
pub fn open_idol_detail_screen(world: &mut World, this: Entity, open_idol_detail_screen: Option<usize>) {
    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = true;
        manager.idol_screen_logic
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_detail_screen(world, idol_screen_logic, open_idol_detail_screen);
    }
}

/// `-[IdolManager closeIdolScreen]` @ 0x10002f440
pub fn close_idol_screen(world: &mut World, this: Entity) {
    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if !manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = false;
        manager.idol_screen_logic
    };
    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::close_screen(world, idol_screen_logic);
    }
}

/// `-[IdolManager onIdolScreenClosed]` @ 0x10002f474
pub fn on_idol_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<IdolManager>(this).idol_screen_active = false;
    let listener = listener(world, this);
    crate::controller::on_idol_screen_closed(world, listener);
}

/// `-[IdolManager onIdolScreenLaunchStoreScreen]` @ 0x10002f49c
pub fn on_idol_screen_launch_store_screen(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    crate::controller::on_idol_screen_launch_store_screen(world, listener);
}

/// `-[IdolManager openIdolAwardScreen:]` @ 0x10002f4b8
pub fn open_idol_award_screen(world: &mut World, this: Entity, open_idol_award_screen: Option<usize>) {
    let can_show = can_show_award_screen(world, this);
    if !can_show {
        world.obj_mut::<IdolManager>(this).delayed_award_screen_array.push(open_idol_award_screen);
        restart_check_auto_unlock_sequence(world, this);
        return;
    }

    let idol_screen_logic = {
        let mut manager = world.obj_mut::<IdolManager>(this);
        if manager.idol_screen_active {
            return;
        }
        manager.idol_screen_active = true;
        manager.idol_screen_logic
    };
    let listener = listener(world, this);
    crate::controller::on_idol_award_screen_opened(world, listener);

    let episode = open_idol_award_screen.expect("openIdolAwardScreen: needs an episode");
    let idol_texture = &episode_table(world).def(episode).idol_texture;
    activate_episode_bits_slot(world, &EPISODE_ALERT_IDOL_SLOT, idol_texture);

    if let Some(idol_screen_logic) = idol_screen_logic {
        idol_screen_logic::open_idol_award_screen(world, idol_screen_logic, open_idol_award_screen);
    }
}

/// `FUN_100038648(EpisodeAlertIdolSlot, <active episode's idol>, @"raw")`: puts the active
/// episode's idol back into the episode alert after the award screen.
fn restore_active_idol_texture(world: &mut World, this: Entity) {
    let active = world.obj::<IdolManager>(this).active_struct_xml_episode_node_def;
    let active = active.expect("the award screen shows while an episode is active");
    let idol_texture = &episode_table(world).def(active).idol_texture;
    activate_episode_bits_slot(world, &EPISODE_ALERT_IDOL_SLOT, idol_texture);
}

/// `-[IdolManager onIdolAwardScreenClosed]` @ 0x10002f600
pub fn on_idol_award_screen_closed(world: &mut World, this: Entity) {
    world.obj_mut::<IdolManager>(this).idol_screen_active = false;
    restore_active_idol_texture(world, this);
    let listener = listener(world, this);
    crate::controller::on_idol_award_screen_closed(world, listener);
}

/// `-[IdolManager onIdolAwardScreenOpenChallenges]` @ 0x10002f664
pub fn on_idol_award_screen_open_challenges(world: &mut World, this: Entity) {
    restore_active_idol_texture(world, this);
    let listener = listener(world, this);
    crate::controller::on_idol_award_screen_open_challenges(world, listener);
}

/// `-[IdolManager onIdolAwardScreenChangeIslandLocationState:]` @ 0x10002f6bc
pub fn on_idol_award_screen_change_island_location_state(world: &mut World, this: Entity, idol_award_screen_change_island_location_state: i32) {
    restore_active_idol_texture(world, this);
    let listener = listener(world, this);
    crate::controller::on_idol_award_screen_change_island_location_state(world, listener, idol_award_screen_change_island_location_state);
}

/// `-[IdolManager onEpisodeActivated:]` @ 0x10002f71c
pub fn on_episode_activated(world: &mut World, this: Entity, episode_activated: Option<usize>) {
    world.obj_mut::<IdolManager>(this).active_struct_xml_episode_node_def = episode_activated;

    let listener = listener(world, this);
    let option_preferences = crate::controller::option_preferences(world, listener);
    let active = episode_activated.expect("onEpisodeActivated: needs an episode");
    let def = episode_table(world).def(active);
    set_string_preference(world, option_preferences, ACTIVE_EPISODE_KEY, &def.id);

    activate_episode_bits_slot(world, &EPISODE_ALERT_ICON_SLOT, &def.icon_texture);
    activate_episode_bits_slot(world, &EPISODE_ALERT_IDOL_SLOT, &def.idol_texture);

    let episode_alert_logic = world.obj::<IdolManager>(this).episode_alert_logic;
    if let Some(episode_alert_logic) = episode_alert_logic {
        episode_alert_logic::activate(world, episode_alert_logic, episode_activated);
    }

    restart_check_auto_unlock_sequence(world, this);
}

/// `-[IdolManager activatedHintCount]` @ 0x10002f830
pub fn activated_hint_count(world: &mut World, this: Entity) -> i32 {
    world.obj::<IdolManager>(this).activated_hint_count
}

/// `-[IdolManager onHintActivated:]` @ 0x10002f840
pub fn on_hint_activated(world: &mut World, this: Entity, hint_activated: Option<usize>) {
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    let challenge = hint_activated.expect("onHintActivated: needs a challenge");
    let key = episode_table(world).challenges[challenge].hint_preference_key.clone();
    set_int_preference(world, challenge_preferences, key.as_deref(), 1);
    episode_table_mut(world).challenges[challenge].hint_activated = true;

    update_episode_hint_status(world, this);
}

/// `-[IdolManager checkGemEvent:]` @ 0x10002f8b4
pub fn check_gem_event(world: &mut World, this: Entity, check_gem_event: &str) {
    let gem_id = world.obj::<IdolManager>(this).gem_id_dictionary.get(check_gem_event).copied();
    let Some(gem_id) = gem_id else {
        return;
    };
    if gem_id == 0 {
        return;
    }

    let listener = listener(world, this);
    crate::controller::on_apocalypse_gem_event_triggered(world, listener, gem_id);
}

/// The id of a challenge or level-up that a kind of episode event names.
#[derive(Clone, Copy)]
enum EpisodeEventKind {
    /// `triggerEpisodeEvent:`: `event_id` (+0x08).
    Event,

    /// `triggerEpisodeEventBySacrificeId:`: `sacrifice_id` (+0x10).
    Sacrifice,

    /// `triggerEpisodeEventByGiftId:`: `gift_id` (+0x18).
    Gift,
}

impl EpisodeEventKind {
    fn challenge_id(self, challenge: &'static Challenge) -> Option<&'static str> {
        match self {
            Self::Event => Some(challenge.event_id.as_str()),
            Self::Sacrifice => challenge.sacrifice_id.as_deref(),
            Self::Gift => challenge.gift_id.as_deref(),
        }
    }

    fn level_up_id(self, level_up: &'static LevelUp) -> Option<&'static str> {
        match self {
            Self::Event => Some(level_up.event_id.as_str()),
            Self::Sacrifice => level_up.sacrifice_id.as_deref(),
            Self::Gift => level_up.gift_id.as_deref(),
        }
    }
}

/// The body the three `triggerEpisodeEvent…:` methods share once the event may trigger:
/// completes the active episode's matching open challenges, then, if its idol is earned, adds a
/// point to each matching level-up, and updates the episode.
fn trigger_active_episode_event(world: &mut World, this: Entity, kind: EpisodeEventKind, event: &str) {
    let active = world.obj::<IdolManager>(this).active_struct_xml_episode_node_def;
    let Some(episode) = active else {
        return;
    };
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    let node = episode_table(world).nodes[episode].clone();
    for challenge in node.challenges {
        let state = episode_table(world).challenges[challenge].clone();
        if state.completed {
            continue;
        }
        let challenge_id = kind.challenge_id(state.def());
        if challenge_id != Some(event) {
            continue;
        }

        sparkle_episode_alert(world, this);
        set_int_preference(world, challenge_preferences, state.preference_key.as_deref(), 1);
        episode_table_mut(world).challenges[challenge].completed = true;
    }

    let complete = episode_table(world).nodes[episode].complete;
    if complete {
        for level_up in node.level_ups {
            let level_up_def = episode_table(world).level_ups[level_up].def();
            let level_up_id = kind.level_up_id(level_up_def);
            if level_up_id != Some(event) {
                continue;
            }

            sparkle_episode_alert(world, this);
            let max_points = level_up_def.max_points;
            let (points, key) = {
                let mut table = episode_table_mut(world);
                let state = &mut table.level_ups[level_up];
                if state.points < max_points {
                    state.points += 1;
                } else {
                    state.points = max_points;
                }
                (state.points, state.preference_key.clone())
            };
            set_int_preference(world, challenge_preferences, key.as_deref(), points);
        }
    }

    update_episode_status(world, this, Some(episode), false);
}

/// `-[IdolManager triggerEpisodeEventBySacrificeId:]` @ 0x10002f930
pub fn trigger_episode_event_by_sacrifice_id(world: &mut World, this: Entity, trigger_episode_event_by_sacrifice_id: &str) {
    let can_trigger = can_trigger_challenge(world, this);
    if !can_trigger {
        delay_trigger_episode_sacrifice_event(world, this, trigger_episode_event_by_sacrifice_id);
        return;
    }

    check_gem_event(world, this, trigger_episode_event_by_sacrifice_id);
    trigger_active_episode_event(world, this, EpisodeEventKind::Sacrifice, trigger_episode_event_by_sacrifice_id);
}

/// `-[IdolManager triggerEpisodeEventByGiftId:]` @ 0x10002fb60
pub fn trigger_episode_event_by_gift_id(world: &mut World, this: Entity, trigger_episode_event_by_gift_id: &str) {
    let can_trigger = can_trigger_challenge(world, this);
    if !can_trigger {
        delay_trigger_episode_gift_event(world, this, trigger_episode_event_by_gift_id);
        return;
    }

    check_gem_event(world, this, trigger_episode_event_by_gift_id);
    trigger_active_episode_event(world, this, EpisodeEventKind::Gift, trigger_episode_event_by_gift_id);
}

/// `-[IdolManager triggerEpisodeEvent:]` @ 0x10002fd90
pub fn trigger_episode_event(world: &mut World, this: Entity, trigger_episode_event: &str) {
    let can_trigger = can_trigger_challenge(world, this);
    if !can_trigger {
        delay_trigger_episode_event(world, this, trigger_episode_event);
        return;
    }

    check_gem_event(world, this, trigger_episode_event);
    trigger_active_episode_event(world, this, EpisodeEventKind::Event, trigger_episode_event);
}

/// `-[IdolManager triggerEpisodeGameScoreEvent:score:]` @ 0x10002ffb8
pub fn trigger_episode_game_score_event(world: &mut World, this: Entity, trigger_episode_game_score_event: &str, score: i32) {
    let active = world.obj::<IdolManager>(this).active_struct_xml_episode_node_def;
    let Some(episode) = active else {
        return;
    };
    let node = episode_table(world).nodes[episode].clone();
    if !node.complete {
        return;
    }
    let listener = listener(world, this);
    let challenge_preferences = crate::controller::challenge_preferences(world, listener);

    for level_up in node.level_ups {
        let level_up_def = episode_table(world).level_ups[level_up].def();
        if level_up_def.event_id != trigger_episode_game_score_event {
            continue;
        }

        sparkle_episode_alert(world, this);
        let max_points = level_up_def.max_points;
        let (points, key) = {
            let mut table = episode_table_mut(world);
            let state = &mut table.level_ups[level_up];
            let scored_points = state.points.wrapping_add(score);
            if scored_points > max_points {
                state.points = max_points;
            } else {
                state.points = scored_points;
            }
            (state.points, state.preference_key.clone())
        };
        set_int_preference(world, challenge_preferences, key.as_deref(), points);
    }

    update_episode_status(world, this, Some(episode), false);
}

/// `-[IdolManager delayTriggerEpisodeEvent:]` @ 0x1000300fc
pub fn delay_trigger_episode_event(world: &mut World, this: Entity, delay_trigger_episode_event: &str) {
    let event = delay_trigger_episode_event.to_owned();
    world.obj_mut::<IdolManager>(this).delayed_episode_event_array.push(event);
    restart_check_auto_unlock_sequence(world, this);
}

/// `-[IdolManager delayTriggerEpisodeGiftEvent:]` @ 0x100030184
pub fn delay_trigger_episode_gift_event(world: &mut World, this: Entity, delay_trigger_episode_gift_event: &str) {
    let event = delay_trigger_episode_gift_event.to_owned();
    world.obj_mut::<IdolManager>(this).delayed_episode_gift_event_array.push(event);
    restart_check_auto_unlock_sequence(world, this);
}

/// `-[IdolManager delayTriggerEpisodeSacrificeEvent:]` @ 0x10003020c
pub fn delay_trigger_episode_sacrifice_event(world: &mut World, this: Entity, delay_trigger_episode_sacrifice_event: &str) {
    let event = delay_trigger_episode_sacrifice_event.to_owned();
    world.obj_mut::<IdolManager>(this).delayed_episode_sacrifice_event_array.push(event);
    restart_check_auto_unlock_sequence(world, this);
}

/// `-[IdolManager optionPreferences]` @ 0x100030294
pub fn option_preferences(world: &mut World, this: Entity) -> Option<Entity> {
    let listener = listener(world, this);
    crate::controller::option_preferences(world, listener)
}

/// `-[IdolManager onLoadLocationTextureAtlas]` @ 0x1000302b0
pub fn on_load_location_texture_atlas(world: &mut World, this: Entity) {
    let listener = listener(world, this);
    crate::controller::on_load_location_texture_atlas(world, listener);
}

/// `-[IdolManager onButtonPress:touch:]` @ 0x1000302cc
pub fn on_button_press(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[IdolManager onButtonRelease:touch:]` @ 0x1000302d0
pub fn on_button_release(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `-[IdolManager onButtonReleaseOutside:touch:]` @ 0x1000302d4
pub fn on_button_release_outside(_world: &mut World, _this: Entity, _button: Entity, _touch: Entity) {}

/// `+[IdolManager instance]` @ 0x10002a1b4
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().idol_manager
}
