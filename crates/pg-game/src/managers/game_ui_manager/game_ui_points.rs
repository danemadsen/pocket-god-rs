//! `GameUiManager`: methods in this part of the class.

use pg_engine::display::model::SPRITE_Z;
use pg_engine::display::text::{self, Justification};
use pg_engine::display::{self, group, multi_model};
use pg_engine::library;
use pg_engine::logic::multi_model as multi_model_logic;
use pg_engine::sequence;

use crate::prelude::*;
use super::*;

/// `@"GameUiManager"` (`0x1003ee3b0`).
const CLASS_ID: &str = "GameUiManager";

/// `@"PocketGod"` (`0x10038a270`): the group the pop-ups are shown in.
const POCKET_GOD_DISPLAY_GROUP: &str = "PocketGod";

/// `@"GamePoints"` (`0x1003ee3d0`): the multi-model of a pooled pop-up.
const GAME_POINTS_MULTI_MODEL: &str = "GamePoints";

/// `@"AnimatedGothicLite20"` (`0x1003ee3f0`): the font of both pooled texts.
const GAME_POINTS_FONT: &str = "AnimatedGothicLite20";

// `@"PointsText"` (`0x1003ee410`) and `@"PointsLabelText"` (`0x1003ee430`).
const POINTS_TEXT: &str = "PointsText";
const POINTS_LABEL_TEXT: &str = "PointsLabelText";

// The animation channels the pooled texts are bound to.
const POINTS_TEXT_CHANNEL: usize = 3;
const POINTS_LABEL_TEXT_CHANNEL: usize = 2;

/// The event a pop-up's behaviour sends when it is over.
const GAME_POINTS_COMPLETE_EVENT: &str = "GamePointsComplete";

/// The last index of the 30-entry `mGamePointsArray` ring.
const LAST_GAME_POINTS_INDEX: i32 = 29;

/// `0x3e2aaaab`: the pause before the next pop-up starts (1/6 s).
const TIME_BETWEEN_GAME_POINTS: f32 = 1.0 / 6.0;

/// `-[GameUiManager init]` @ 0x100120bd0
pub fn init(world: &mut World, this: Entity) {
    world.entity_mut(this).insert(GameUiManager::default());

    world.get_resource_or_insert_with(Singletons::default).game_ui_manager = Some(this);

    let pocket_god_display_group = library::display_group(world, POCKET_GOD_DISPLAY_GROUP);
    let points_sequence = sequence::spawn(world, None, Some(this));
    {
        let mut manager = world.obj_mut::<GameUiManager>(this);
        manager.game_points_logic_array = Vec::with_capacity(5);
        manager.custom_game_points_logic_array = Vec::with_capacity(5);
        manager.game_points_text_array = Vec::with_capacity(5);
        manager.game_points_label_text_array = Vec::with_capacity(5);
        manager.pocket_god_display_group = pocket_god_display_group;
        manager.sequence = Some(points_sequence);
    }

    pg_engine::view::add_game_frame_listener(world, this);
}

/// `-[GameUiManager id]` @ 0x100120d74
pub fn id(_world: &World, _this: Entity) -> Option<String> {
    Some(CLASS_ID.to_owned())
}

/// `-[GameUiManager setGameUiManagerListener:]` @ 0x100120d80
pub fn set_game_ui_manager_listener(world: &mut World, this: Entity, game_ui_manager_listener: Entity) {
    world.obj_mut::<GameUiManager>(this).game_ui_manager_listener = Some(game_ui_manager_listener);
}

/// `-[GameUiManager setBehavior:]` @ 0x100120ddc
pub fn set_behavior(world: &mut World, this: Entity, behavior: Option<&str>) {
    let points_sequence = world.obj::<GameUiManager>(this).sequence;

    // `getSequenceDefById:` finds nothing for a nil id.
    let sequence_def = match behavior {
        Some(behavior) => library::sequence_def(world, behavior),
        None => None,
    };
    if let Some(points_sequence) = points_sequence {
        sequence::set_sequence_def(world, points_sequence, sequence_def);
    }
}

/// `-[GameUiManager dealloc]` @ 0x10012198c
///
/// The view's game-frame listener array keeps the manager alive for good, so this never runs in
/// the game.
pub fn dealloc(world: &mut World, this: Entity) {
    let game_points_logics = world.obj::<GameUiManager>(this).game_points_logic_array.clone();
    for game_points_logic in game_points_logics.iter().copied() {
        remove_logic_from_display_group(world, game_points_logic);
        pg_engine::logic::shutdown(world, game_points_logic);
    }

    let custom_game_points_logics = world.obj::<GameUiManager>(this).custom_game_points_logic_array.clone();
    for custom_game_points_logic in custom_game_points_logics {
        remove_logic_from_display_group(world, custom_game_points_logic);
    }

    // Releasing the pool arrays frees the pooled sets: each array held the last reference to its
    // logics or texts, and each logic the last one to its multi-model. The custom logics belong
    // to their creators.
    for game_points_logic in game_points_logics {
        let game_points_multi_model = pg_engine::logic::display_object(world, game_points_logic);
        pg_engine::logic::dealloc(world, game_points_logic);
        world.despawn(game_points_multi_model);
    }

    let (points_texts, points_label_texts, points_sequence) = {
        let manager = world.obj::<GameUiManager>(this);
        let points_texts = manager.game_points_text_array.clone();
        let points_label_texts = manager.game_points_label_text_array.clone();
        (points_texts, points_label_texts, manager.sequence)
    };
    for points_text in points_texts {
        world.despawn(points_text);
    }
    for points_label_text in points_label_texts {
        world.despawn(points_label_text);
    }

    if let Some(points_sequence) = points_sequence {
        sequence::stop(world, points_sequence);
        world.despawn(points_sequence);
    }

    world.despawn(this);
}

/// `-[GameUiManager removeAllGamePointsDisplayObjects]` @ 0x100120e44
pub fn remove_all_game_points_display_objects(world: &mut World, this: Entity) {
    let game_points_logics = world.obj::<GameUiManager>(this).game_points_logic_array.clone();
    for game_points_logic in game_points_logics {
        remove_logic_from_display_group(world, game_points_logic);
    }

    let custom_game_points_logics = world.obj::<GameUiManager>(this).custom_game_points_logic_array.clone();
    for custom_game_points_logic in custom_game_points_logics {
        remove_logic_from_display_group(world, custom_game_points_logic);
    }
}

/// `-[GameUiManager activateNextGamePoints]` @ 0x10012102c
pub fn activate_next_game_points(world: &mut World, this: Entity) {
    let entry_index = {
        let mut manager = world.obj_mut::<GameUiManager>(this);
        let entry_index = manager.next_active_game_points_index;
        manager.next_active_game_points_index = next_game_points_index(entry_index);
        entry_index as usize
    };

    let message_logic = world.obj::<GameUiManager>(this).game_points_array[entry_index].message_logic;
    let (game_points_logic, texts) = match message_logic {
        Some(message_logic) => {
            let known = world.obj::<GameUiManager>(this).custom_game_points_logic_array.contains(&message_logic);
            if !known {
                multi_model_logic::set_multi_model_logic_listener(world, message_logic, Some(this));
                world.obj_mut::<GameUiManager>(this).custom_game_points_logic_array.push(message_logic);
            }

            // A custom logic shows its own content: no texts are filled in.
            (message_logic, None)
        }
        None => {
            let free_set = find_free_game_points_set(world, this);
            let (game_points_logic, points_text, points_label_text) = match free_set {
                Some(free_set) => free_set,
                None => create_game_points_set(world, this),
            };
            (game_points_logic, Some((points_text, points_label_text)))
        }
    };

    let pocket_god_display_group = world.obj::<GameUiManager>(this).pocket_god_display_group;
    let game_points_object = pg_engine::logic::display_object(world, game_points_logic);
    if let Some(pocket_god_display_group) = pocket_god_display_group {
        group::add_display_object(world, pocket_god_display_group, Some(game_points_object));
    }
    pg_engine::logic::set_binary_frame_def(world, game_points_logic, None, None);

    let sequence_id = world.obj::<GameUiManager>(this).game_points_array[entry_index].sequence_id.clone();
    pg_engine::logic::set_behavior(world, game_points_logic, sequence_id.as_deref());

    let game_points_object = pg_engine::logic::display_object(world, game_points_logic);
    let mut pos = world.obj::<GameUiManager>(this).game_points_array[entry_index].pos;
    pos.z = SPRITE_Z;
    display::object_mut(world, game_points_object).pos = pos;

    let label = world.obj::<GameUiManager>(this).game_points_array[entry_index].label.clone();
    let (Some(label), Some((points_text, points_label_text))) = (label, texts) else {
        return;
    };

    let game_points = world.obj::<GameUiManager>(this).game_points_array[entry_index].game_points;
    let points = format!("{game_points}");
    text::set_text(world, points_text, Some(&points));
    text::set_text(world, points_label_text, Some(&label));

    // `[label release]` balances the retain of `showPoints:…`. The entry keeps the stale pointer,
    // but `showPoints:…` always rewrites an entry before it is activated again.
}

/// `-[GameUiManager checkForNextGamePoints]` @ 0x10012158c
pub fn check_for_next_game_points(world: &mut World, this: Entity) {
    let manager = world.obj::<GameUiManager>(this);
    let pause_over = manager.time_for_next_game_points <= 0.0;
    let queued = manager.next_free_game_points_index != manager.next_active_game_points_index;
    if !(pause_over && queued) {
        return;
    }

    world.obj_mut::<GameUiManager>(this).time_for_next_game_points = TIME_BETWEEN_GAME_POINTS;
    activate_next_game_points(world, this);
}

/// `-[GameUiManager showPoints:label:sequenceId:pos:messageLogic:]` @ 0x1001215e0
///
/// The queue has no overflow check: 30 entries queued ahead of activation bring the write index
/// back to the read index, and the queue looks empty.
pub fn show_points(
    world: &mut World,
    this: Entity,
    show_points: i32,
    label: Option<&str>,
    sequence_id: &str,
    pos: Vec4,
    message_logic: Option<Entity>,
) {
    {
        let mut manager = world.obj_mut::<GameUiManager>(this);
        let entry_index = manager.next_free_game_points_index as usize;

        let entry = &mut manager.game_points_array[entry_index];
        entry.message_logic = message_logic;
        entry.game_points = show_points;
        entry.label = label.map(str::to_owned);
        entry.pos = pos;
        entry.sequence_id = Some(sequence_id.to_owned());

        let next_free_index = next_game_points_index(manager.next_free_game_points_index);
        manager.next_free_game_points_index = next_free_index;
    }

    check_for_next_game_points(world, this);
}

/// `-[GameUiManager showMessage:sequenceId:pos:messageLogic:]` @ 0x10012169c
pub fn show_message_sequence_id_pos_message_logic(world: &mut World, this: Entity, show_message: &str, sequence_id: &str, pos: Vec4, message_logic: Entity) {
    show_points(world, this, 0, Some(show_message), sequence_id, pos, Some(message_logic));
}

/// `-[GameUiManager showMessage:sequenceId:pos:]` @ 0x1001216c8
pub fn show_message_sequence_id_pos(world: &mut World, this: Entity, show_message: &str, sequence_id: &str, pos: Vec4) {
    show_points(world, this, 0, Some(show_message), sequence_id, pos, None);
}

/// `-[GameUiManager allPointsComplete]` @ 0x1001216f0
///
/// Queued entries that have not started yet do not count.
pub fn all_points_complete(world: &mut World, this: Entity) -> bool {
    let manager = world.obj::<GameUiManager>(this);
    let game_points_logics = manager.game_points_logic_array.clone();
    let custom_game_points_logics = manager.custom_game_points_logic_array.clone();

    let mut all_complete = true;
    for game_points_logic in game_points_logics {
        if is_logic_in_display_group(world, game_points_logic) {
            all_complete = false;
            break;
        }
    }

    for custom_game_points_logic in custom_game_points_logics {
        if is_logic_in_display_group(world, custom_game_points_logic) {
            all_complete = false;
            break;
        }
    }

    all_complete
}

/// `-[GameUiManager onGameFrame:]` @ 0x100121900
pub fn on_game_frame(world: &mut World, this: Entity, dt: f32) {
    world.obj_mut::<GameUiManager>(this).time_for_next_game_points -= dt;
    check_for_next_game_points(world, this);
}

/// `-[GameUiManager onMultiModelEvent:eventId:]` @ 0x100121924
///
/// Leaving the display group frees a pooled set for the next pop-up.
pub fn on_multi_model_event(world: &mut World, _this: Entity, logic: Entity, event_id: &str) {
    if event_id == GAME_POINTS_COMPLETE_EVENT {
        remove_logic_from_display_group(world, logic);
    }
}

/// `+[GameUiManager instance]` @ 0x100120bc4
pub fn instance(world: &World) -> Option<Entity> {
    world.resource::<Singletons>().game_ui_manager
}

/// The ring index after `index`: `index < 29 ? index + 1 : 0`.
fn next_game_points_index(index: i32) -> i32 {
    if index < LAST_GAME_POINTS_INDEX {
        index + 1
    } else {
        0
    }
}

/// The first pooled logic whose display object is in no display group, with the texts at the
/// same index.
fn find_free_game_points_set(world: &World, this: Entity) -> Option<(Entity, Entity, Entity)> {
    let manager = world.obj::<GameUiManager>(this);
    for (index, game_points_logic) in manager.game_points_logic_array.iter().copied().enumerate() {
        if is_logic_in_display_group(world, game_points_logic) {
            continue;
        }
        let points_text = manager.game_points_text_array[index];
        let points_label_text = manager.game_points_label_text_array[index];
        return Some((game_points_logic, points_text, points_label_text));
    }
    None
}

/// A new pooled set: a `GamePoints` multi-model logic listened to by this manager, and its
/// points and label texts bound to animation channels 3 and 2.
fn create_game_points_set(world: &mut World, this: Entity) -> (Entity, Entity, Entity) {
    let game_points_multi_model = multi_model::spawn(world, Some(GAME_POINTS_MULTI_MODEL));
    let game_points_logic = multi_model_logic::spawn(world, game_points_multi_model);
    multi_model_logic::set_multi_model_logic_listener(world, game_points_logic, Some(this));
    world.obj_mut::<GameUiManager>(this).game_points_logic_array.push(game_points_logic);

    // `getFontDefById:` is looked up once for both texts; the engine resolves a text's font by
    // id, and a missing font is nil either way.
    let points_text = text::spawn(world, Some(POINTS_TEXT), Some(GAME_POINTS_FONT));
    text::set_font_justification(world, points_text, Justification::Center);
    world.obj_mut::<GameUiManager>(this).game_points_text_array.push(points_text);

    let points_label_text = text::spawn(world, Some(POINTS_LABEL_TEXT), Some(GAME_POINTS_FONT));
    text::set_font_justification(world, points_label_text, Justification::Center);
    world.obj_mut::<GameUiManager>(this).game_points_label_text_array.push(points_label_text);

    let animation = pg_engine::logic::animation(world, game_points_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(points_text), POINTS_TEXT_CHANNEL);
    let animation = pg_engine::logic::animation(world, game_points_logic);
    pg_engine::animation::set_tweenable(world, animation, Some(points_label_text), POINTS_LABEL_TEXT_CHANNEL);

    (game_points_logic, points_text, points_label_text)
}

/// `[[logic displayObject] displayGroup] != nil`.
fn is_logic_in_display_group(world: &World, logic: Entity) -> bool {
    let display_object = pg_engine::logic::display_object(world, logic);
    display::object(world, display_object).display_group.is_some()
}

/// `[[logic displayObject] removeFromDisplayGroup]`.
fn remove_logic_from_display_group(world: &mut World, logic: Entity) {
    let display_object = pg_engine::logic::display_object(world, logic);
    display::remove_from_display_group(world, display_object);
}
