//! `EpisodeAlertLogic` (superclass `SpriteLogic`).
//!
//! The listener is always the view controller (`initLogicListeners`), so its protocol methods
//! are direct calls into `crate::controller`.

use pg_engine::display;
use pg_engine::library;
use pg_engine::logic;
use pg_engine::prefs;

use crate::controller;
use crate::logic::island_core::sprite_logic;
use crate::prelude::*;

/// `EpisodeAlertLogic` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct EpisodeAlertLogic {
    /// `mEpisodeAlertLogicListener`: `@"<EpisodeAlertLogicListener>"`
    pub episode_alert_logic_listener: Option<Entity>,

    /// `mbActive`: `B`
    pub active: bool,

    /// `mbShowEpisodeIcon`: `B`
    pub show_episode_icon: bool,

    /// `mpStructXmlEpisodeNodeDef`: `^{BCStructXmlEpisodeNodeDef=i^^{BCStructXmlEpisodeEnemyIdolNodeDef}^^{BCStructXmlEpisodeRealmPowerNodeDef}^^{BCStructXmlEpisodeChallengeNodeDef}^^{BCStructXmlEpisodeLevelUpNodeDef}@@@@@@@@@iii@iBiiiiiBffi@{?=ffff}@{?=ffff}}`
    pub struct_xml_episode_node_def: Option<usize>,
}

pub const EPISODE_ALERT_LOGIC: Class = Class {
    name: "EpisodeAlertLogic",
    superclass: Some(&crate::logic::island_core::sprite_logic::SPRITE_LOGIC),
    sequence_item,
    touch_began_inside,
    touch_began_outside,
    touch_moved_entered,
    touch_moved_exited,
    touch_moved_inside,
    touch_moved_outside,
    touch_ended_inside,
    touch_ended_outside,
    ..crate::logic::island_core::sprite_logic::SPRITE_LOGIC
};

/// `@"PocketGod"`: the display group the alert shows in.
const POCKET_GOD_DISPLAY_GROUP: &str = "PocketGod";

/// `@"ShowEpisodeIcon"`: the option preference that keeps the alert on screen.
const SHOW_EPISODE_ICON_KEY: &str = "ShowEpisodeIcon";

const EPISODE_ALERT_ACTIVATE: &str = "EpisodeAlertActivate";
const EPISODE_ALERT_DEACTIVATE: &str = "EpisodeAlertDeactivate";
const EPISODE_ALERT_EVENT: &str = "EpisodeAlertEvent";
const EPISODE_ALERT_FADE_OUT: &str = "EpisodeAlertFadeOut";

/// Sequence-item handlers implemented by `EpisodeAlertLogic` (`respondsToSelector:` order).
pub fn sequence_item(world: &mut World, this: Entity, call: &mut SequenceCall) -> Option<ItemResult> {
    let result = match call.method() {
        "checkHiddenAlert" => seq_check_hidden_alert(world, this, call),
        "onAlertFadeOutComplete" => seq_on_alert_fade_out_complete(world, this, call),
        _ => return (crate::logic::island_core::sprite_logic::SPRITE_LOGIC.sequence_item)(world, this, call),
    };
    Some(result)
}

/// `[[EpisodeAlertLogic alloc] initWithDisplayObject:]`.
pub fn spawn(world: &mut World, display_object: Entity) -> Entity {
    let this = world.spawn(Isa(&EPISODE_ALERT_LOGIC)).id();
    init_with_display_object(world, this, display_object);
    this
}

/// `[mDisplayObject moveToDisplayGroup:[[BCLibrary instance] getDisplayGroupById:@"PocketGod"]]`.
fn move_to_pocket_god_display_group(world: &mut World, this: Entity) {
    let pocket_god_display_group = library::display_group(world, POCKET_GOD_DISPLAY_GROUP);
    let display_object = logic::display_object(world, this);
    display::move_to_display_group(world, display_object, pocket_god_display_group);
}

/// `[mDisplayObject removeFromDisplayGroup]`.
fn remove_from_display_group(world: &mut World, this: Entity) {
    let display_object = logic::display_object(world, this);
    display::remove_from_display_group(world, display_object);
}

/// `-[EpisodeAlertLogic initWithDisplayObject:]` @ 0x1001a3eb8
pub fn init_with_display_object(world: &mut World, this: Entity, display_object: Entity) {
    world.entity_mut(this).insert(EpisodeAlertLogic::default());
    sprite_logic::init_with_display_object(world, this, display_object);

    let display_object = logic::display_object(world, this);
    let mut object = display::object_mut(world, display_object);
    object.col_x_min = 0.0;
    object.col_x_max = 42.0;
    object.col_y_min = 0.0;
    object.col_y_max = 42.0;
    object.is_selectable = true;
}

/// `-[EpisodeAlertLogic setEpisodeAlertLogicListener:]` @ 0x1001a3f9c
pub fn set_episode_alert_logic_listener(world: &mut World, this: Entity, episode_alert_logic_listener: Entity) {
    world.obj_mut::<EpisodeAlertLogic>(this).episode_alert_logic_listener = Some(episode_alert_logic_listener);
}

/// `-[EpisodeAlertLogic activate:]` @ 0x1001a3ff8
pub fn activate(world: &mut World, this: Entity, activate: Option<usize>) {
    let listener = world.obj::<EpisodeAlertLogic>(this).episode_alert_logic_listener;
    let option_preferences = match listener {
        Some(listener) => controller::option_preferences(world, listener),
        None => None,
    };
    let show_episode_icon_preference = match option_preferences {
        Some(option_preferences) => prefs::get_int_preference(world, option_preferences, SHOW_EPISODE_ICON_KEY),
        None => 0,
    };

    {
        let mut alert = world.obj_mut::<EpisodeAlertLogic>(this);
        alert.show_episode_icon = show_episode_icon_preference != 0;
        alert.struct_xml_episode_node_def = activate;
    }

    let active = world.obj::<EpisodeAlertLogic>(this).active;
    if active {
        return;
    }
    world.obj_mut::<EpisodeAlertLogic>(this).active = true;

    let show_episode_icon = world.obj::<EpisodeAlertLogic>(this).show_episode_icon;
    if show_episode_icon {
        move_to_pocket_god_display_group(world, this);
        logic::set_behavior(world, this, Some(EPISODE_ALERT_ACTIVATE));
    }
}

/// `-[EpisodeAlertLogic deactivate]` @ 0x1001a4104
pub fn deactivate(world: &mut World, this: Entity) {
    let active = world.obj::<EpisodeAlertLogic>(this).active;
    if !active {
        return;
    }
    world.obj_mut::<EpisodeAlertLogic>(this).active = false;

    remove_from_display_group(world, this);
    logic::set_behavior(world, this, Some(EPISODE_ALERT_DEACTIVATE));
}

/// `-[EpisodeAlertLogic onEpisodeIconPrefChanged:]` @ 0x1001a4178
pub fn on_episode_icon_pref_changed(world: &mut World, this: Entity, episode_icon_pref_changed: bool) {
    world.obj_mut::<EpisodeAlertLogic>(this).show_episode_icon = episode_icon_pref_changed;

    let active = world.obj::<EpisodeAlertLogic>(this).active;
    if !active {
        return;
    }

    let show_episode_icon = world.obj::<EpisodeAlertLogic>(this).show_episode_icon;
    let behavior = if show_episode_icon {
        move_to_pocket_god_display_group(world, this);
        EPISODE_ALERT_ACTIVATE
    } else {
        remove_from_display_group(world, this);
        EPISODE_ALERT_DEACTIVATE
    };
    logic::set_behavior(world, this, Some(behavior));
}

/// `-[EpisodeAlertLogic sparkle]` @ 0x1001a4268
///
/// Shows the alert for an episode event, even while the player has hidden it.
pub fn sparkle(world: &mut World, this: Entity) {
    let (active, show_episode_icon) = {
        let alert = world.obj::<EpisodeAlertLogic>(this);
        (alert.active, alert.show_episode_icon)
    };
    if !active {
        return;
    }

    if !show_episode_icon {
        move_to_pocket_god_display_group(world, this);
    }
    logic::set_behavior(world, this, Some(EPISODE_ALERT_EVENT));
}

/// `-[EpisodeAlertLogic checkHiddenAlert:::]` @ 0x1001a431c
pub fn seq_check_hidden_alert(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let show_episode_icon = world.obj::<EpisodeAlertLogic>(this).show_episode_icon;
    if !show_episode_icon {
        logic::set_behavior(world, this, Some(EPISODE_ALERT_FADE_OUT));
    }
    ItemResult::Done
}

/// `-[EpisodeAlertLogic onAlertFadeOutComplete:::]` @ 0x1001a4358
pub fn seq_on_alert_fade_out_complete(world: &mut World, this: Entity, _call: &mut SequenceCall) -> ItemResult {
    let show_episode_icon = world.obj::<EpisodeAlertLogic>(this).show_episode_icon;
    if !show_episode_icon {
        remove_from_display_group(world, this);
        logic::set_behavior(world, this, Some(EPISODE_ALERT_DEACTIVATE));
    }
    ItemResult::Done
}

/// `-[EpisodeAlertLogic touchBeganInside:allTouches:]` @ 0x1001a4428
pub fn touch_began_inside(world: &mut World, this: Entity, _touch: Entity) {
    let listener = world.obj::<EpisodeAlertLogic>(this).episode_alert_logic_listener;
    let Some(listener) = listener else {
        return;
    };

    if controller::is_story_active(world, listener) {
        return;
    }
    if controller::is_screen_open(world, listener) {
        return;
    }

    let episode_node = world.obj::<EpisodeAlertLogic>(this).struct_xml_episode_node_def;
    controller::on_episode_alert_open_challenge_detail(world, listener, episode_node);
}

/// `-[EpisodeAlertLogic touchBeganOutside:allTouches:]` @ 0x1001a44a4
pub fn touch_began_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[EpisodeAlertLogic touchMovedEntered:allTouches:]` @ 0x1001a44a8
pub fn touch_moved_entered(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[EpisodeAlertLogic touchMovedExited:allTouches:]` @ 0x1001a44ac
pub fn touch_moved_exited(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[EpisodeAlertLogic touchMovedInside:allTouches:]` @ 0x1001a44b0
pub fn touch_moved_inside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_moved_outside)(world, this, touch);
}

/// `-[EpisodeAlertLogic touchMovedOutside:allTouches:]` @ 0x1001a44c0
pub fn touch_moved_outside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[EpisodeAlertLogic touchEndedInside:allTouches:]` @ 0x1001a44c4
pub fn touch_ended_inside(_world: &mut World, _this: Entity, _touch: Entity) {}

/// `-[EpisodeAlertLogic touchEndedOutside:allTouches:]` @ 0x1001a44c8
pub fn touch_ended_outside(world: &mut World, this: Entity, touch: Entity) {
    let class = isa(world, this);
    (class.touch_ended_inside)(world, this, touch);
}
