//! `IdolIconLogic` (superclass `BCLogic`).
//!
//! A `BCStructXmlEpisodeNodeDef *` is an index into the idol manager's
//! [`EpisodeTable`](crate::managers::idol_manager::EpisodeTable); a `BCSkinTextureBitsDef *`
//! array is an index into the idol screen's `SKIN_TEXTURE_BITS_DEF_ARRAYS`. The listener is
//! always the `IdolScreenLogic` (`initLogicListeners`).

use pg_engine::display;
use pg_engine::display::text;
use pg_engine::library::Library;
use pg_engine::logic;

use crate::logic::idol_screen::idol_screen_logic;
use crate::managers::idol_manager;
use crate::prelude::*;

/// `IdolIconLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct IdolIconLogic {
    /// `mIdolIconLogicListener`: `@"<IdolIconLogicListener>"`
    pub idol_icon_logic_listener: Option<Entity>,

    /// `mbIsActive`: `B`
    pub is_active: bool,

    /// `mpStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub struct_xml_episode_node_def: Option<usize>,

    /// `mSkinTextureBitsDefArray`: `^{BCSkinTextureBitsDef=^{BCTextureDef}iiii}`
    pub skin_texture_bits_def_array: Option<usize>,

    /// `mLabelId`: `@"NSString"`
    pub label_id: Option<String>,

    /// `mHaloLogic`: `@"MultiModelLogic"`
    pub halo_logic: Option<Entity>,

    /// `mCheckMarkLogic`: `@"MultiModelLogic"`
    pub check_mark_logic: Option<Entity>,
}

pub const IDOL_ICON_LOGIC: Class = Class {
    name: "IdolIconLogic",
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

const IDOL_ICON_SHOW_HALO: &str = "IdolIconShowHalo";
const IDOL_ICON_HIDE_HALO: &str = "IdolIconHideHalo";
const IDOL_ICON_SHOW_CHECK_MARK: &str = "IdolIconShowCheckMark";
const IDOL_ICON_HIDE_CHECK_MARK: &str = "IdolIconHideCheckMark";

/// `[[IdolIconLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&IDOL_ICON_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[logic setBehavior:behavior]` on a logic ivar that may be nil.
fn set_logic_behavior(world: &mut World, logic: Option<Entity>, behavior: &str) {
    if let Some(logic) = logic {
        logic::set_behavior(world, logic, Some(behavior));
    }
}

/// `mSkinTextureBitsDefArray`, which `FUN_100038648` dereferences without a NULL check.
fn required_skin_texture_bits_def_array(world: &World, this: Entity) -> usize {
    let skin_texture_bits_def_array = world.obj::<IdolIconLogic>(this).skin_texture_bits_def_array;
    skin_texture_bits_def_array.expect("IdolIconLogic: the IdolScreenLogic sets mSkinTextureBitsDefArray before the icon is used")
}

/// `-[IdolIconLogic initWithDisplayObject:]` @ 0x100198ccc
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(IdolIconLogic::default());
    logic::init_with_display_object(world, this, display_object);

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.col_x_min = 0.0;
    object.col_x_max = 100.0;
    object.col_y_min = 0.0;
    object.col_y_max = 100.0;
    object.hidden = true;
}

/// `-[IdolIconLogic setIdolIconLogicListener:]` @ 0x100198e10
pub fn set_idol_icon_logic_listener(world: &mut World, this: Entity, idol_icon_logic_listener: Entity) {
    world.obj_mut::<IdolIconLogic>(this).idol_icon_logic_listener = Some(idol_icon_logic_listener);
}

/// `-[IdolIconLogic onGameFrame:]` @ 0x100198e6c
pub fn on_game_frame(_world: &mut World, _this: Entity, _dt: f32) {}

/// `-[IdolIconLogic isActive]` @ 0x100198e70
pub fn is_active(world: &mut World, this: Entity) -> bool {
    world.obj::<IdolIconLogic>(this).is_active
}

/// `-[IdolIconLogic setSkinTextureBitsDefArray:]` @ 0x100198e80
pub fn set_skin_texture_bits_def_array(world: &mut World, this: Entity, skin_texture_bits_def_array: Option<usize>) {
    world.obj_mut::<IdolIconLogic>(this).skin_texture_bits_def_array = skin_texture_bits_def_array;
}

/// `-[IdolIconLogic setLabelId:]` @ 0x100198e90
pub fn set_label_id(world: &mut World, this: Entity, label_id: &str) {
    world.obj_mut::<IdolIconLogic>(this).label_id = Some(label_id.to_owned());
}

/// `-[IdolIconLogic setHaloLogic:]` @ 0x100198ea0
pub fn set_halo_logic(world: &mut World, this: Entity, halo_logic: Entity) {
    world.obj_mut::<IdolIconLogic>(this).halo_logic = Some(halo_logic);
}

/// `-[IdolIconLogic setCheckMarkLogic:]` @ 0x100198eb0
pub fn set_check_mark_logic(world: &mut World, this: Entity, check_mark_logic: Entity) {
    world.obj_mut::<IdolIconLogic>(this).check_mark_logic = Some(check_mark_logic);
}

/// `-[IdolIconLogic activate:isSelected:]` @ 0x100198ec0
///
/// Shows the icon of episode node `activate` at its list position: the idol's icon image in
/// the icon's texture slot, `Episode <number>` as its label, the halo when it is the active
/// episode and the check mark when the idol is earned.
pub fn activate(world: &mut World, this: Entity, activate: Option<usize>, selected: bool) {
    world.obj_mut::<IdolIconLogic>(this).is_active = true;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).hidden = false;

    world.obj_mut::<IdolIconLogic>(this).struct_xml_episode_node_def = activate;
    let node = activate.expect("activate:isSelected: dereferences its episode node");
    let def = idol_manager::episode_table(world).def(node);

    let skin_texture_bits_def_array = required_skin_texture_bits_def_array(world, this);
    idol_screen_logic::activate_skin_texture_bits_def_array(world, skin_texture_bits_def_array, &def.icon_texture);

    let label_id = world.obj::<IdolIconLogic>(this).label_id.clone();
    let label = match label_id {
        Some(label_id) => world.resource::<Library>().dynamic_text(&label_id),
        None => None,
    };
    let episode_text = format!("Episode {}", def.episode_number);
    if let Some(label) = label {
        text::set_text(world, label, Some(&episode_text));
    }

    let idol_icon_pos = idol_manager::episode_table(world).nodes[node].idol_icon_pos;
    {
        let mut object = display::object_mut(world, display_object);
        object.pos.x = idol_icon_pos.x;
        object.pos.y = idol_icon_pos.y;
    }

    let halo_behavior = if selected { IDOL_ICON_SHOW_HALO } else { IDOL_ICON_HIDE_HALO };
    let halo_logic = world.obj::<IdolIconLogic>(this).halo_logic;
    set_logic_behavior(world, halo_logic, halo_behavior);

    let complete = idol_manager::episode_table(world).nodes[node].complete;
    let check_mark_behavior = if complete { IDOL_ICON_SHOW_CHECK_MARK } else { IDOL_ICON_HIDE_CHECK_MARK };
    let check_mark_logic = world.obj::<IdolIconLogic>(this).check_mark_logic;
    set_logic_behavior(world, check_mark_logic, check_mark_behavior);
}

/// `-[IdolIconLogic updateIdolIconTexture]` @ 0x10019905c
pub fn update_idol_icon_texture(world: &mut World, this: Entity) {
    let node = world.obj::<IdolIconLogic>(this).struct_xml_episode_node_def;
    let Some(node) = node else {
        return;
    };

    let skin_texture_bits_def_array = required_skin_texture_bits_def_array(world, this);
    let def = idol_manager::episode_table(world).def(node);
    idol_screen_logic::activate_skin_texture_bits_def_array(world, skin_texture_bits_def_array, &def.icon_texture);
}

/// `-[IdolIconLogic updateHalo:]` @ 0x10019908c
pub fn update_halo(world: &mut World, this: Entity, update_halo: bool) {
    let halo_logic = world.obj::<IdolIconLogic>(this).halo_logic;
    let halo_behavior = if update_halo { IDOL_ICON_SHOW_HALO } else { IDOL_ICON_HIDE_HALO };
    set_logic_behavior(world, halo_logic, halo_behavior);
}

/// `-[IdolIconLogic deactivate]` @ 0x1001990c0
pub fn deactivate(world: &mut World, this: Entity) {
    world.obj_mut::<IdolIconLogic>(this).is_active = false;

    let display_object = logic::display_object(world, this);
    display::object_mut(world, display_object).hidden = true;

    world.obj_mut::<IdolIconLogic>(this).struct_xml_episode_node_def = None;
}

/// `-[IdolIconLogic touchBeganInside:allTouches:]` @ 0x100199114
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let (node, listener) = {
        let icon = world.obj::<IdolIconLogic>(this);
        (icon.struct_xml_episode_node_def, icon.idol_icon_logic_listener)
    };
    if node.is_none() {
        return;
    }

    if let Some(listener) = listener {
        idol_screen_logic::on_idol_icon_select(world, listener, node);
    }
}

/// `-[IdolIconLogic touchBeganOutside:allTouches:]` @ 0x100199144
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolIconLogic touchMovedEntered:allTouches:]` @ 0x100199148
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolIconLogic touchMovedExited:allTouches:]` @ 0x10019914c
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolIconLogic touchMovedInside:allTouches:]` @ 0x100199150
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[IdolIconLogic touchMovedOutside:allTouches:]` @ 0x100199160
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolIconLogic touchEndedInside:allTouches:]` @ 0x100199164
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[IdolIconLogic touchEndedOutside:allTouches:]` @ 0x100199168
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
