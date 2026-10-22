//! `IdolSelectionIconLogic` (superclass `BCLogic`).
//!
//! A `BCStructXmlEpisodeNodeDef *` is an index into the idol manager's
//! [`EpisodeTable`](crate::managers::idol_manager::EpisodeTable); a `BCSkinTextureBitsDef *`
//! array is an index into the idol screen's `SKIN_TEXTURE_BITS_DEF_ARRAYS`. The listener is
//! always the `IdolScreenLogic` (`initLogicListeners`).

use pg_assets::data::episodes::{Attack, Realm};
use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::display::text;
use pg_engine::library::Library;
use pg_engine::logic;

use crate::logic::idol_screen::idol_screen_logic;
use crate::managers::idol_manager;
use crate::prelude::*;

/// `IdolSelectionIconLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IdolSelectionIconLogic {
    /// `mIdolSelectionIconLogicListener`: `@"<IdolSelectionIconLogicListener>"`
    pub idol_selection_icon_logic_listener: Option<Entity>,

    /// `mbIsActive`: `B`
    pub is_active: bool,

    /// `mbInBattle`: `B`
    pub in_battle: bool,

    /// `mbIsEnemyIdol`: `B`
    pub is_enemy_idol: bool,

    /// `mpStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub struct_xml_episode_node_def: Option<usize>,

    /// `mIdolSelectionIconType`: `i`
    pub idol_selection_icon_type: i32,

    /// `mSkinTextureBitsDefArray`: `^{BCSkinTextureBitsDef=^{BCTextureDef}iiii}`
    pub skin_texture_bits_def_array: Option<usize>,

    /// `mLabelId`: `@"NSString"`
    pub label_id: Option<String>,

    /// `mStatsLabelId`: `@"NSString"`
    pub stats_label_id: Option<String>,

    /// `mNameLabelId`: `@"NSString"`
    pub name_label_id: Option<String>,

    /// `mEnemyLevel`: `i`
    pub enemy_level: i32,

    /// `mEnemyHealth`: `f`
    pub enemy_health: f32,

    /// `mAttackTypeLogic`: `@"MultiModelLogic"`
    pub attack_type_logic: Option<Entity>,

    /// `mRealmTypeLogic`: `@"MultiModelLogic"`
    pub realm_type_logic: Option<Entity>,

    /// `mBattleSelectedLogic`: `@"MultiModelLogic"`
    pub battle_selected_logic: Option<Entity>,

    /// `mbIsFrozen`: `B`
    pub is_frozen: bool,

    /// `mSequenceId`: `@"NSString"`
    pub sequence_id: Option<String>,

    /// `mBattleEnemySequenceId`: `@"NSString"`
    pub battle_enemy_sequence_id: Option<String>,

    /// `mBattleEnemyHighlightSequenceId`: `@"NSString"`
    pub battle_enemy_highlight_sequence_id: Option<String>,

    /// `mBattleEnemyLockedSequenceId`: `@"NSString"`
    pub battle_enemy_locked_sequence_id: Option<String>,

    /// `mBattleSequenceId`: `@"NSString"`
    pub battle_sequence_id: Option<String>,

    /// `mBattleFrozenSequenceId`: `@"NSString"`
    pub battle_frozen_sequence_id: Option<String>,

    /// `mBattleHighlightSequenceId`: `@"NSString"`
    pub battle_highlight_sequence_id: Option<String>,

    /// `mHomePos`: `{?="x"f"y"f"z"f"w"f}`
    pub home_pos: Vec4,

    /// `mBattleHomePos`: `{?="x"f"y"f"z"f"w"f}`
    pub battle_home_pos: Vec4,

    /// `mDragXPos`: `f`
    pub drag_x_pos: f32,

    /// `mDragYPos`: `f`
    pub drag_y_pos: f32,
}

pub const IDOL_SELECTION_ICON_LOGIC: Class = Class {
    name: "IdolSelectionIconLogic",
    superclass: Some(&pg_engine::logic::BC_LOGIC),
    on_game_frame,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..pg_engine::logic::BC_LOGIC
};

/// `@"BlankAnim"`: clears the battle selection badge.
const BLANK_ANIM: &str = "BlankAnim";

/// `@"IdolSelectionIconSelected"`: the battle selection badge.
const IDOL_SELECTION_ICON_SELECTED: &str = "IdolSelectionIconSelected";

const IDOL_ATTACK_TYPE_STRONG: &str = "IdolAttackTypeStrong";
const IDOL_ATTACK_TYPE_STRONG_WIN: &str = "IdolAttackTypeStrongWin";
const IDOL_ATTACK_TYPE_STRONG_LOSE: &str = "IdolAttackTypeStrongLose";
const IDOL_ATTACK_TYPE_COUNTER: &str = "IdolAttackTypeCounter";
const IDOL_ATTACK_TYPE_COUNTER_WIN: &str = "IdolAttackTypeCounterWin";
const IDOL_ATTACK_TYPE_COUNTER_LOSE: &str = "IdolAttackTypeCounterLose";
const IDOL_ATTACK_TYPE_FAST: &str = "IdolAttackTypeFast";
const IDOL_ATTACK_TYPE_FAST_WIN: &str = "IdolAttackTypeFastWin";
const IDOL_ATTACK_TYPE_FAST_LOSE: &str = "IdolAttackTypeFastLose";

const IDOL_REALM_TYPE_OCEANUS: &str = "IdolRealmTypeOceanus";
const IDOL_REALM_TYPE_OCEANUS_WIN: &str = "IdolRealmTypeOceanusWin";
const IDOL_REALM_TYPE_OCEANUS_LOSE: &str = "IdolRealmTypeOceanusLose";
const IDOL_REALM_TYPE_AERIS: &str = "IdolRealmTypeAeris";
const IDOL_REALM_TYPE_AERIS_WIN: &str = "IdolRealmTypeAerisWin";
const IDOL_REALM_TYPE_AERIS_LOSE: &str = "IdolRealmTypeAerisLose";
const IDOL_REALM_TYPE_INFERNIS: &str = "IdolRealmTypeInfernis";
const IDOL_REALM_TYPE_INFERNIS_WIN: &str = "IdolRealmTypeInfernisWin";
const IDOL_REALM_TYPE_INFERNIS_LOSE: &str = "IdolRealmTypeInfernisLose";
const IDOL_REALM_TYPE_ANIMUS: &str = "IdolRealmTypeAnimus";
const IDOL_REALM_TYPE_ANIMUS_WIN: &str = "IdolRealmTypeAnimusWin";
const IDOL_REALM_TYPE_ANIMUS_LOSE: &str = "IdolRealmTypeAnimusLose";
const IDOL_REALM_TYPE_PAX: &str = "IdolRealmTypePax";
const IDOL_REALM_TYPE_PAX_WIN: &str = "IdolRealmTypePaxWin";
const IDOL_REALM_TYPE_PAX_LOSE: &str = "IdolRealmTypePaxLose";

/// `[[IdolSelectionIconLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&IDOL_SELECTION_ICON_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[logic setBehavior:behavior]` on a logic ivar that may be nil.
fn set_logic_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    if let Some(logic) = logic {
        logic::set_behavior(world, logic, Some(behavior));
    }
}

/// `[[[BCLibrary instance] getDynamicTextById:id] setText:text]`; a nil id finds no text.
fn set_dynamic_text(world: &mut World, id: Option<&str>, text: &str) {
    let dynamic_text = match id {
        Some(id) => world.resource::<Library>().dynamic_text(id),
        None => None,
    };
    if let Some(dynamic_text) = dynamic_text {
        text::set_text(world, dynamic_text, Some(text));
    }
}

/// `mpStructXmlEpisodeNodeDef`, where the original dereferences it without a NULL check.
fn required_node(world: &World, this: Entity) -> usize {
    let node = world.obj::<IdolSelectionIconLogic>(this).struct_xml_episode_node_def;
    node.expect("IdolSelectionIconLogic: mpStructXmlEpisodeNodeDef is NULL (the original dereferences it)")
}

/// `FUN_100038648(mSkinTextureBitsDefArray, node->idolTexture, @"raw")`: loads the idol's image
/// into this icon's texture slot.
fn load_idol_texture(world: &mut World, this: Entity, node: usize) {
    let skin_texture_bits_def_array = world.obj::<IdolSelectionIconLogic>(this).skin_texture_bits_def_array;
    let skin_texture_bits_def_array = skin_texture_bits_def_array
        .expect("IdolSelectionIconLogic: the IdolScreenLogic sets mSkinTextureBitsDefArray before the icon is used");
    let def = idol_manager::episode_table(world).def(node);
    idol_screen_logic::activate_skin_texture_bits_def_array(world, skin_texture_bits_def_array, &def.idol_texture);
}

/// `[[[BCLibrary instance] getDynamicTextById:mLabelId] setText:[NSString
/// stringWithFormat:@"L%02d", node->level + 1]]`.
fn set_level_label(world: &mut World, this: Entity, node: usize) {
    let label_id = world.obj::<IdolSelectionIconLogic>(this).label_id.clone();
    let level = idol_manager::episode_table(world).nodes[node].level;
    let level_text = format!("L{:02}", level + 1);
    set_dynamic_text(world, label_id.as_deref(), &level_text);
}

/// `-[IdolSelectionIconLogic initWithDisplayObject:]` @ 0x100111880
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IdolSelectionIconLogic::default());
    logic::init_with_display_object(world, this, display_object);

    {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.home_pos.x = 40.0;
        icon.home_pos.y = 252.0;
        icon.home_pos.z = SPRITE_Z;
        icon.battle_home_pos.x = 50.0;
        icon.battle_home_pos.y = 115.0;
        icon.battle_home_pos.z = SPRITE_Z;
    }

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.col_x_min = 0.0;
    object.col_x_max = 88.0;
    object.col_y_min = -88.0;
    object.col_y_max = 0.0;
    object.hidden = true;
}

/// `-[IdolSelectionIconLogic setIdolSelectionIconLogicListener:]` @ 0x100111a00
pub fn set_idol_selection_icon_logic_listener(world: &mut World, this: Entity, idol_selection_icon_logic_listener: Entity) {
    let listener = Some(idol_selection_icon_logic_listener);
    world.obj_mut::<IdolSelectionIconLogic>(this).idol_selection_icon_logic_listener = listener;
}

/// `-[IdolSelectionIconLogic onGameFrame:]` @ 0x100111a5c
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[IdolSelectionIconLogic isActive]` @ 0x100111a60
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<IdolSelectionIconLogic>(this).is_active
}

/// `-[IdolSelectionIconLogic setSkinTextureBitsDefArray:]` @ 0x100111a70
pub fn set_skin_texture_bits_def_array(world: &mut World, this: Entity, skin_texture_bits_def_array: Option<usize>) {
    world.obj_mut::<IdolSelectionIconLogic>(this).skin_texture_bits_def_array = skin_texture_bits_def_array;
}

/// `-[IdolSelectionIconLogic setLabelId:]` @ 0x100111a80
pub fn set_label_id(world: &mut World, this: Entity, label_id: &str) {
    world.obj_mut::<IdolSelectionIconLogic>(this).label_id = Some(label_id.to_owned());
}

/// `-[IdolSelectionIconLogic setBattleNameLabelId:statsLabelId:]` @ 0x100111a90
pub fn set_battle_name_label_id(world: &mut World, this: Entity, battle_name_label_id: &str, stats_label_id: &str) {
    let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
    icon.name_label_id = Some(battle_name_label_id.to_owned());
    icon.stats_label_id = Some(stats_label_id.to_owned());
}

/// `-[IdolSelectionIconLogic setAttackTypeLogic:]` @ 0x100111aac
pub fn set_attack_type_logic(world: &mut World, this: Entity, attack_type_logic: Entity) {
    world.obj_mut::<IdolSelectionIconLogic>(this).attack_type_logic = Some(attack_type_logic);
}

/// `-[IdolSelectionIconLogic setRealmTypeLogic:]` @ 0x100111abc
pub fn set_realm_type_logic(world: &mut World, this: Entity, realm_type_logic: Entity) {
    world.obj_mut::<IdolSelectionIconLogic>(this).realm_type_logic = Some(realm_type_logic);
}

/// `-[IdolSelectionIconLogic setBattleSelectedLogic:]` @ 0x100111acc
pub fn set_battle_selected_logic(world: &mut World, this: Entity, battle_selected_logic: Entity) {
    world.obj_mut::<IdolSelectionIconLogic>(this).battle_selected_logic = Some(battle_selected_logic);
}

/// `-[IdolSelectionIconLogic idolSelectionIconType]` @ 0x100111adc
pub fn idol_selection_icon_type(world: &mut World, this: Entity) -> i32 {
    world.obj::<IdolSelectionIconLogic>(this).idol_selection_icon_type
}

/// `-[IdolSelectionIconLogic setIdolSelectionIconType:]` @ 0x100111aec
pub fn set_idol_selection_icon_type(world: &mut World, this: Entity, idol_selection_icon_type: i32) {
    world.obj_mut::<IdolSelectionIconLogic>(this).idol_selection_icon_type = idol_selection_icon_type;
}

/// `-[IdolSelectionIconLogic setHomeXPos:yPos:]` @ 0x100111afc
pub fn set_home_x_pos(world: &mut World, this: Entity, home_x_pos: f32, y_pos: f32) {
    let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
    icon.home_pos.x = home_x_pos;
    icon.home_pos.y = y_pos;
}

/// `-[IdolSelectionIconLogic setBattleHomeXPos:yPos:]` @ 0x100111b10
pub fn set_battle_home_x_pos(world: &mut World, this: Entity, battle_home_x_pos: f32, y_pos: f32) {
    let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
    icon.battle_home_pos.x = battle_home_x_pos;
    icon.battle_home_pos.y = y_pos;
}

/// `-[IdolSelectionIconLogic setSequenceId:battleEnemySequenceId:battleEnemyHighlightSequenceId:battleEnemyLockedSequenceId:battleSequenceId:battleHighlightSequenceId:battleFrozenSequenceId:]` @ 0x100111b24
#[allow(clippy::too_many_arguments)]
pub fn set_sequence_id(world: &mut World, this: Entity, sequence_id: &str, battle_enemy_sequence_id: &str, battle_enemy_highlight_sequence_id: &str, battle_enemy_locked_sequence_id: &str, battle_sequence_id: &str, battle_highlight_sequence_id: &str, battle_frozen_sequence_id: &str) {
    let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
    icon.sequence_id = Some(sequence_id.to_owned());
    icon.battle_enemy_sequence_id = Some(battle_enemy_sequence_id.to_owned());
    icon.battle_enemy_highlight_sequence_id = Some(battle_enemy_highlight_sequence_id.to_owned());
    icon.battle_enemy_locked_sequence_id = Some(battle_enemy_locked_sequence_id.to_owned());
    icon.battle_sequence_id = Some(battle_sequence_id.to_owned());
    icon.battle_highlight_sequence_id = Some(battle_highlight_sequence_id.to_owned());
    icon.battle_frozen_sequence_id = Some(battle_frozen_sequence_id.to_owned());
}

/// `-[IdolSelectionIconLogic setIsEnemyIdol:]` @ 0x100111b80
pub fn set_is_enemy_idol(world: &mut World, this: Entity, is_enemy_idol: bool) {
    world.obj_mut::<IdolSelectionIconLogic>(this).is_enemy_idol = is_enemy_idol;
}

/// `-[IdolSelectionIconLogic isEnemyIdol]` @ 0x100111b90
pub fn is_enemy_idol(world: &mut World, this: Entity) -> bool {
    world.obj::<IdolSelectionIconLogic>(this).is_enemy_idol
}

/// `-[IdolSelectionIconLogic setEnemyStatsLevel:health:]` @ 0x100111ba0
pub fn set_enemy_stats_level(world: &mut World, this: Entity, enemy_stats_level: i32, health: f32) {
    let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
    icon.enemy_level = enemy_stats_level;
    icon.enemy_health = health;
}

/// `-[IdolSelectionIconLogic structXmlEpisodeNodeDef]` @ 0x100111bbc
pub fn struct_xml_episode_node_def(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<IdolSelectionIconLogic>(this).struct_xml_episode_node_def
}

/// `-[IdolSelectionIconLogic resetBattleIcons]` @ 0x100111bcc
///
/// Shows the plain attack type and realm badges of the icon's idol.
pub fn reset_battle_icons(world: &mut World, this: Entity) {
    let node = required_node(world, this);
    let def = idol_manager::episode_table(world).def(node);

    let attack_behavior = match def.attack {
        Attack::Fast => IDOL_ATTACK_TYPE_FAST,
        Attack::Counter => IDOL_ATTACK_TYPE_COUNTER,
        Attack::Strong => IDOL_ATTACK_TYPE_STRONG,
    };
    let attack_type_logic = world.obj::<IdolSelectionIconLogic>(this).attack_type_logic;
    set_logic_behavior(world, attack_type_logic, attack_behavior);

    let realm_behavior = match def.realm {
        Realm::Oceanus => IDOL_REALM_TYPE_OCEANUS,
        Realm::Aeris => IDOL_REALM_TYPE_AERIS,
        Realm::Infernis => IDOL_REALM_TYPE_INFERNIS,
        Realm::Animus => IDOL_REALM_TYPE_ANIMUS,
        Realm::Pax => IDOL_REALM_TYPE_PAX,
    };
    let realm_type_logic = world.obj::<IdolSelectionIconLogic>(this).realm_type_logic;
    set_logic_behavior(world, realm_type_logic, realm_behavior);
}

/// `-[IdolSelectionIconLogic activate:]` @ 0x100111d6c
///
/// Shows the card of episode node `activate` at its home position (full size for the player's
/// idols, 0.8 for enemy idols) with its level and badges.
pub fn activate(world: &mut World, this: Entity, activate: Option<usize>) {
    {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.is_active = true;
        icon.in_battle = false;
        icon.is_frozen = false;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.hidden = false;
        object.col_x_max = 88.0;
        object.col_y_min = -88.0;
    }

    let sequence_id = {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.struct_xml_episode_node_def = activate;
        icon.sequence_id.clone()
    };
    logic::set_behavior(world, this, sequence_id.as_deref());

    let (home_pos, is_enemy_idol) = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        (icon.home_pos, icon.is_enemy_idol)
    };
    {
        let mut object = display::object_mut(world, display_object);
        object.pos = home_pos;
        object.scale.x = 0.8;
        object.scale.y = 0.8;
        if !is_enemy_idol {
            object.scale.x = 1.0;
            object.scale.y = 1.0;
        }
    }

    let node = required_node(world, this);
    load_idol_texture(world, this, node);

    let label_node = activate.expect("activate: reads the level of its episode node");
    set_level_label(world, this, label_node);

    reset_battle_icons(world, this);
}

/// `-[IdolSelectionIconLogic activateForBattleEnemy:]` @ 0x100111f58
///
/// Shows the card of an opponent's idol on the battle-enemy panel: locked while that battle is
/// not unlocked, with the opponent's name and stats when the icon has those labels.
pub fn activate_for_battle_enemy(world: &mut World, this: Entity, activate_for_battle_enemy: Option<usize>) {
    {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.is_active = true;
        icon.in_battle = false;
        icon.is_frozen = false;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.hidden = false;
        object.col_x_max = 88.0;
        object.col_y_min = -88.0;
    }

    world.obj_mut::<IdolSelectionIconLogic>(this).struct_xml_episode_node_def = activate_for_battle_enemy;
    let enemy_node = activate_for_battle_enemy.expect("activateForBattleEnemy: dereferences its episode node");

    let battle_unlocked = idol_manager::episode_table(world).nodes[enemy_node].battle_unlocked;
    let sequence_id = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        if !battle_unlocked && icon.is_enemy_idol {
            icon.battle_enemy_locked_sequence_id.clone()
        } else {
            icon.battle_enemy_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());

    let (battle_home_pos, is_enemy_idol) = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        (icon.battle_home_pos, icon.is_enemy_idol)
    };
    {
        let mut object = display::object_mut(world, display_object);
        object.scale.x = 0.8;
        object.scale.y = 0.8;
        if !is_enemy_idol {
            object.pos = battle_home_pos;
            object.scale.x = 1.0;
            object.scale.y = 1.0;
        }
    }

    let node = required_node(world, this);
    load_idol_texture(world, this, node);

    set_level_label(world, this, enemy_node);

    let name_label_id = world.obj::<IdolSelectionIconLogic>(this).name_label_id.clone();
    if name_label_id.is_some() {
        let enemy_def = idol_manager::episode_table(world).def(enemy_node);
        set_dynamic_text(world, name_label_id.as_deref(), &enemy_def.idol_name);

        let (stats_label_id, enemy_level, enemy_health) = {
            let icon = world.obj::<IdolSelectionIconLogic>(this);
            (icon.stats_label_id.clone(), icon.enemy_level, icon.enemy_health)
        };
        let stats_text = format!("Level: {} Health: {:.0}", enemy_level + 1, enemy_health);
        set_dynamic_text(world, stats_label_id.as_deref(), &stats_text);
    }

    reset_battle_icons(world, this);
}

/// `-[IdolSelectionIconLogic activateForBattle:isFrozen:]` @ 0x100112238
///
/// Shows the card of one of the player's idols on the battle selection panel.
pub fn activate_for_battle(world: &mut World, this: Entity, activate_for_battle: Option<usize>, frozen: bool) {
    {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.is_active = true;
        icon.in_battle = true;
        icon.is_frozen = frozen;
    }

    let display_object = logic::display_object(world, this);
    {
        let mut object = display::object_mut(world, display_object);
        object.hidden = false;
        object.col_x_max = 50.0;
        object.col_y_min = -88.0;
    }

    let battle_selected_logic = world.obj::<IdolSelectionIconLogic>(this).battle_selected_logic;
    set_logic_behavior(world, battle_selected_logic, BLANK_ANIM);

    let sequence_id = {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.struct_xml_episode_node_def = activate_for_battle;
        if icon.is_frozen {
            icon.battle_frozen_sequence_id.clone()
        } else {
            icon.battle_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());

    let node = required_node(world, this);
    load_idol_texture(world, this, node);

    let label_node = activate_for_battle.expect("activateForBattle:isFrozen: reads the level of its episode node");
    set_level_label(world, this, label_node);

    reset_battle_icons(world, this);
}

/// `-[IdolSelectionIconLogic selectForBattle]` @ 0x1001123fc
pub fn select_for_battle(world: &mut World, this: Entity) {
    let battle_selected_logic = world.obj::<IdolSelectionIconLogic>(this).battle_selected_logic;
    set_logic_behavior(world, battle_selected_logic, IDOL_SELECTION_ICON_SELECTED);
}

/// `-[IdolSelectionIconLogic highlightBattleEnemy]` @ 0x100112420
pub fn highlight_battle_enemy(world: &mut World, this: Entity) {
    let sequence_id = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        if icon.is_frozen {
            icon.battle_frozen_sequence_id.clone()
        } else {
            icon.battle_enemy_highlight_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());
}

/// `-[IdolSelectionIconLogic unhighlightBattleEnemy]` @ 0x100112458
pub fn unhighlight_battle_enemy(world: &mut World, this: Entity) {
    let sequence_id = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        if icon.is_frozen {
            icon.battle_frozen_sequence_id.clone()
        } else {
            icon.battle_enemy_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());
}

/// `-[IdolSelectionIconLogic highlightForBattle]` @ 0x100112490
pub fn highlight_for_battle(world: &mut World, this: Entity) {
    reset_battle_icons(world, this);

    let sequence_id = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        if icon.is_frozen {
            icon.battle_frozen_sequence_id.clone()
        } else {
            icon.battle_highlight_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());
}

/// `-[IdolSelectionIconLogic unhighlightForBattle]` @ 0x1001124f4
pub fn unhighlight_for_battle(world: &mut World, this: Entity) {
    reset_battle_icons(world, this);

    let sequence_id = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        if icon.is_frozen {
            icon.battle_frozen_sequence_id.clone()
        } else {
            icon.battle_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());
}

/// `-[IdolSelectionIconLogic highlightBasedOnAttackEpisodeNodeDef:]` @ 0x100112558
///
/// Shows whether this idol wins or loses against the attacking idol, for the attack type
/// (fast beats strong, strong beats counter, counter beats fast) and for the realm.
pub fn highlight_based_on_attack_episode_node_def(world: &mut World, this: Entity, highlight_based_on_attack_episode_node_def: Option<usize>) {
    let sequence_id = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        if icon.is_frozen {
            icon.battle_frozen_sequence_id.clone()
        } else {
            icon.battle_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());

    let node = required_node(world, this);
    let def = idol_manager::episode_table(world).def(node);
    let attack_node = highlight_based_on_attack_episode_node_def
        .expect("highlightBasedOnAttackEpisodeNodeDef: dereferences the attacking episode node");
    let attack_def = idol_manager::episode_table(world).def(attack_node);

    let attack_behavior = match def.attack {
        Attack::Fast => match attack_def.attack {
            Attack::Counter => IDOL_ATTACK_TYPE_FAST_LOSE,
            Attack::Strong => IDOL_ATTACK_TYPE_FAST_WIN,
            Attack::Fast => IDOL_ATTACK_TYPE_FAST,
        },
        Attack::Counter => match attack_def.attack {
            Attack::Strong => IDOL_ATTACK_TYPE_COUNTER_LOSE,
            Attack::Fast => IDOL_ATTACK_TYPE_COUNTER_WIN,
            Attack::Counter => IDOL_ATTACK_TYPE_COUNTER,
        },
        Attack::Strong => match attack_def.attack {
            Attack::Fast => IDOL_ATTACK_TYPE_STRONG_LOSE,
            Attack::Counter => IDOL_ATTACK_TYPE_STRONG_WIN,
            Attack::Strong => IDOL_ATTACK_TYPE_STRONG,
        },
    };
    let attack_type_logic = world.obj::<IdolSelectionIconLogic>(this).attack_type_logic;
    set_logic_behavior(world, attack_type_logic, attack_behavior);

    let realm_behavior = match def.realm {
        Realm::Oceanus => match attack_def.realm {
            Realm::Aeris | Realm::Infernis => IDOL_REALM_TYPE_OCEANUS_WIN,
            Realm::Animus | Realm::Pax => IDOL_REALM_TYPE_OCEANUS_LOSE,
            Realm::Oceanus => IDOL_REALM_TYPE_OCEANUS,
        },
        Realm::Aeris => match attack_def.realm {
            Realm::Oceanus | Realm::Animus => IDOL_REALM_TYPE_AERIS_LOSE,
            Realm::Infernis | Realm::Pax => IDOL_REALM_TYPE_AERIS_WIN,
            Realm::Aeris => IDOL_REALM_TYPE_AERIS,
        },
        Realm::Infernis => match attack_def.realm {
            Realm::Animus | Realm::Pax => IDOL_REALM_TYPE_INFERNIS_WIN,
            Realm::Oceanus | Realm::Aeris => IDOL_REALM_TYPE_INFERNIS_LOSE,
            Realm::Infernis => IDOL_REALM_TYPE_INFERNIS,
        },
        Realm::Animus => match attack_def.realm {
            Realm::Oceanus | Realm::Aeris => IDOL_REALM_TYPE_ANIMUS_WIN,
            Realm::Infernis | Realm::Pax => IDOL_REALM_TYPE_ANIMUS_LOSE,
            Realm::Animus => IDOL_REALM_TYPE_ANIMUS,
        },
        Realm::Pax => match attack_def.realm {
            Realm::Animus | Realm::Oceanus => IDOL_REALM_TYPE_PAX_WIN,
            Realm::Aeris | Realm::Infernis => IDOL_REALM_TYPE_PAX_LOSE,
            Realm::Pax => IDOL_REALM_TYPE_PAX,
        },
    };
    let realm_type_logic = world.obj::<IdolSelectionIconLogic>(this).realm_type_logic;
    set_logic_behavior(world, realm_type_logic, realm_behavior);
}

/// `-[IdolSelectionIconLogic setScrollXPos:]` @ 0x100112a0c
///
/// Moves the home position to the idol's library slot scrolled by `scroll_x_pos`; the icon
/// keeps its drag offset from it.
pub fn set_scroll_x_pos(world: &mut World, this: Entity, scroll_x_pos: f32) {
    let display_object = logic::display_object(world, this);
    let node = required_node(world, this);
    let slot_pos = idol_manager::episode_table(world).nodes[node].idol_selection_icon_pos;

    let home_x_pos = slot_pos.x + scroll_x_pos;
    let home_y_pos = slot_pos.y;
    let (drag_x_pos, drag_y_pos) = {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.home_pos.x = home_x_pos;
        icon.home_pos.y = home_y_pos;
        (icon.drag_x_pos, icon.drag_y_pos)
    };

    let mut object = display::object_mut(world, display_object);
    object.pos.x = home_x_pos + drag_x_pos;
    object.pos.y = home_y_pos + drag_y_pos;
}

/// `-[IdolSelectionIconLogic dragXPos:yPos:]` @ 0x100112aac
pub fn drag_x_pos(world: &mut World, this: Entity, drag_x_pos: f32, y_pos: f32) {
    let (home_pos, total_drag_x_pos, total_drag_y_pos) = {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.drag_x_pos += drag_x_pos;
        icon.drag_y_pos += y_pos;
        (icon.home_pos, icon.drag_x_pos, icon.drag_y_pos)
    };

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.pos.x = home_pos.x + total_drag_x_pos;
    object.pos.y = home_pos.y + total_drag_y_pos;
}

/// `-[IdolSelectionIconLogic stopDrag]` @ 0x100112b44
pub fn stop_drag(world: &mut World, this: Entity) {
    let home_pos = {
        let mut icon = world.obj_mut::<IdolSelectionIconLogic>(this);
        icon.drag_x_pos = 0.0;
        icon.drag_y_pos = 0.0;
        icon.home_pos
    };

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos = home_pos;
}

/// `-[IdolSelectionIconLogic updateIdolIconTexture]` @ 0x100112ba8
pub fn update_idol_icon_texture(world: &mut World, this: Entity) {
    let node = world.obj::<IdolSelectionIconLogic>(this).struct_xml_episode_node_def;
    if let Some(node) = node {
        load_idol_texture(world, this, node);
    }
}

/// `-[IdolSelectionIconLogic deactivate]` @ 0x100112bd8
pub fn deactivate(world: &mut World, this: Entity) {
    world.obj_mut::<IdolSelectionIconLogic>(this).is_active = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).hidden = true;

    world.obj_mut::<IdolSelectionIconLogic>(this).struct_xml_episode_node_def = None;
}

/// `-[IdolSelectionIconLogic touchBeganInside:allTouches:]` @ 0x100112c2c
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let (node, listener) = {
        let icon = world.obj::<IdolSelectionIconLogic>(this);
        (icon.struct_xml_episode_node_def, icon.idol_selection_icon_logic_listener)
    };
    if node.is_none() {
        return;
    }

    if let Some(listener) = listener {
        idol_screen_logic::on_idol_selection_icon_select(world, listener, this, node);
    }
}

/// `-[IdolSelectionIconLogic touchBeganOutside:allTouches:]` @ 0x100112c64
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolSelectionIconLogic touchMovedEntered:allTouches:]` @ 0x100112c68
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolSelectionIconLogic touchMovedExited:allTouches:]` @ 0x100112c6c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolSelectionIconLogic touchMovedInside:allTouches:]` @ 0x100112c70
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[IdolSelectionIconLogic touchMovedOutside:allTouches:]` @ 0x100112c80
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolSelectionIconLogic touchEndedInside:allTouches:]` @ 0x100112c84
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolSelectionIconLogic touchEndedOutside:allTouches:]` @ 0x100112c88
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
