//! `IdolBattleRealmEffectLogic` (superclass `BCLogic`).
//!
//! A `BCStructXmlEpisodeNodeDef *` is an index into the idol manager's
//! [`EpisodeTable`](crate::managers::idol_manager::EpisodeTable); an `IdolBattleState *` is the
//! idol screen's `idol_battle_state_pointer` encoding. The listener is always the
//! `IdolScreenLogic` (`initLogicListeners`).

use pg_engine::display;
use pg_engine::display::model::SPRITE_Z;
use pg_engine::logic;

use crate::logic::idol_screen::idol_screen_logic;
use crate::prelude::*;

/// `IdolBattleRealmEffectLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IdolBattleRealmEffectLogic {
    /// `mIdolBattleRealmEffectLogicListener`: `@"<IdolBattleRealmEffectLogicListener>"`
    pub idol_battle_realm_effect_logic_listener: Option<Entity>,

    /// `mbIsActive`: `B`
    pub is_active: bool,

    /// `mpStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub struct_xml_episode_node_def: Option<usize>,

    /// `mpIdolBattleState`: `^{IdolBattleState=i^{BCStructXmlEpisodeNodeDef}@ffifiB^{IdolBattleState}Bffffiiiiiiiif}`
    pub idol_battle_state: Option<usize>,

    /// `mBuffSequenceId`: `@"NSString"`
    pub buff_sequence_id: Option<String>,

    /// `mDebuffSequenceId`: `@"NSString"`
    pub debuff_sequence_id: Option<String>,

    /// `mTargetPos`: `{?="x"f"y"f"z"f"w"f}`
    pub target_pos: Vec4,

    /// `mbIsBuffEffect`: `B`
    pub is_buff_effect: bool,
}

pub const IDOL_BATTLE_REALM_EFFECT_LOGIC: Class = Class {
    name: "IdolBattleRealmEffectLogic",
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

/// `[[IdolBattleRealmEffectLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&IDOL_BATTLE_REALM_EFFECT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `-[IdolBattleRealmEffectLogic initWithDisplayObject:]` @ 0x10017e5d8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IdolBattleRealmEffectLogic::default());
    logic::init_with_display_object(world, this, display_object);

    world.obj_mut::<IdolBattleRealmEffectLogic>(this).target_pos.z = SPRITE_Z;

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.col_x_min = 0.0;
    object.col_x_max = 25.0;
    object.col_y_min = -25.0;
    object.col_y_max = 0.0;
    object.hidden = true;
}

/// `-[IdolBattleRealmEffectLogic setIdolBattleRealmEffectLogicListener:]` @ 0x10017e724
pub fn set_idol_battle_realm_effect_logic_listener(world: &mut World, this: Entity, idol_battle_realm_effect_logic_listener: Entity) {
    let listener = Some(idol_battle_realm_effect_logic_listener);
    world.obj_mut::<IdolBattleRealmEffectLogic>(this).idol_battle_realm_effect_logic_listener = listener;
}

/// `-[IdolBattleRealmEffectLogic onGameFrame:]` @ 0x10017e780
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[IdolBattleRealmEffectLogic isBuffEffect]` @ 0x10017e784
pub fn is_buff_effect(world: &mut World, this: Entity) -> bool {
    world.obj::<IdolBattleRealmEffectLogic>(this).is_buff_effect
}

/// `-[IdolBattleRealmEffectLogic isActive]` @ 0x10017e794
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<IdolBattleRealmEffectLogic>(this).is_active
}

/// `-[IdolBattleRealmEffectLogic setTargetXPos:yPos:]` @ 0x10017e7a4
pub fn set_target_x_pos(world: &mut World, this: Entity, target_x_pos: f32, y_pos: f32) {
    let target_pos = {
        let mut effect = world.obj_mut::<IdolBattleRealmEffectLogic>(this);
        effect.target_pos.x = target_x_pos;
        effect.target_pos.y = y_pos;
        effect.target_pos
    };

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).pos = target_pos;
}

/// `-[IdolBattleRealmEffectLogic setBuffSequenceId:debuffSequenceId:]` @ 0x10017e7fc
pub fn set_buff_sequence_id(world: &mut World, this: Entity, buff_sequence_id: &str, debuff_sequence_id: &str) {
    let mut effect = world.obj_mut::<IdolBattleRealmEffectLogic>(this);
    effect.buff_sequence_id = Some(buff_sequence_id.to_owned());
    effect.debuff_sequence_id = Some(debuff_sequence_id.to_owned());
}

/// `-[IdolBattleRealmEffectLogic structXmlEpisodeNodeDef]` @ 0x10017e818
pub fn struct_xml_episode_node_def(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<IdolBattleRealmEffectLogic>(this).struct_xml_episode_node_def
}

/// `-[IdolBattleRealmEffectLogic setIdolBattleState:]` @ 0x10017e828
pub fn set_idol_battle_state(world: &mut World, this: Entity, idol_battle_state: Option<usize>) {
    world.obj_mut::<IdolBattleRealmEffectLogic>(this).idol_battle_state = idol_battle_state;
}

/// `-[IdolBattleRealmEffectLogic idolBattleState]` @ 0x10017e838
pub fn idol_battle_state(world: &mut World, this: Entity) -> Option<usize> {
    world.obj::<IdolBattleRealmEffectLogic>(this).idol_battle_state
}

/// `-[IdolBattleRealmEffectLogic activate:isBuffEffect:]` @ 0x10017e848
///
/// Shows the icon at its target position with its buff or debuff behaviour.
pub fn activate(world: &mut World, this: Entity, activate: Option<usize>, buff_effect: bool) {
    world.obj_mut::<IdolBattleRealmEffectLogic>(this).is_active = true;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).hidden = false;

    let sequence_id = {
        let mut effect = world.obj_mut::<IdolBattleRealmEffectLogic>(this);
        effect.struct_xml_episode_node_def = activate;
        effect.is_buff_effect = buff_effect;
        if buff_effect {
            effect.buff_sequence_id.clone()
        } else {
            effect.debuff_sequence_id.clone()
        }
    };
    logic::set_behavior(world, this, sequence_id.as_deref());

    let target_pos = world.obj::<IdolBattleRealmEffectLogic>(this).target_pos;
    display::object_mut(world, display_object).pos = target_pos;
}

/// `-[IdolBattleRealmEffectLogic deactivate]` @ 0x10017e910
pub fn deactivate(world: &mut World, this: Entity) {
    world.obj_mut::<IdolBattleRealmEffectLogic>(this).is_active = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).hidden = true;

    world.obj_mut::<IdolBattleRealmEffectLogic>(this).struct_xml_episode_node_def = None;
}

/// `-[IdolBattleRealmEffectLogic touchBeganInside:allTouches:]` @ 0x10017e964
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let (node, listener) = {
        let effect = world.obj::<IdolBattleRealmEffectLogic>(this);
        (effect.struct_xml_episode_node_def, effect.idol_battle_realm_effect_logic_listener)
    };
    if node.is_none() {
        return;
    }

    if let Some(listener) = listener {
        idol_screen_logic::on_idol_battle_realm_effect_select(world, listener, this, node);
    }
}

/// `-[IdolBattleRealmEffectLogic touchBeganOutside:allTouches:]` @ 0x10017e99c
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolBattleRealmEffectLogic touchMovedEntered:allTouches:]` @ 0x10017e9a0
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolBattleRealmEffectLogic touchMovedExited:allTouches:]` @ 0x10017e9a4
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolBattleRealmEffectLogic touchMovedInside:allTouches:]` @ 0x10017e9a8
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[IdolBattleRealmEffectLogic touchMovedOutside:allTouches:]` @ 0x10017e9b8
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolBattleRealmEffectLogic touchEndedInside:allTouches:]` @ 0x10017e9bc
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolBattleRealmEffectLogic touchEndedOutside:allTouches:]` @ 0x10017e9c0
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
