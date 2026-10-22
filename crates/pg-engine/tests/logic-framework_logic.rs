//! The logic framework (`docs/re/logic.md`): game-frame and hibernation flags, the
//! `setBehavior:` quirk, root motion, `BCLogic` item handlers, buttons, multi-model events,
//! screen layers and messages. Each world holds only what the case needs; the logic objects
//! are built without an animation player (as during `initWithDisplayObject:`) unless the case
//! needs one.

use std::sync::Arc;

use bevy_ecs::prelude::*;

use pg_engine::animation::{self, AnimationDefId, BcAnimation, FrameRef, TimelineAnimation, Timelines};
use pg_engine::display::group;
use pg_engine::display::model::Model;
use pg_engine::display::multi_model::MultiModel;
use pg_engine::display::text::Text;
use pg_engine::display::{self, BC_DISPLAY_OBJECT, DisplayObject};
use pg_engine::hooks::EngineHooks;
use pg_engine::library::Library;
use pg_engine::logic::button::{self, BC_BUTTON_LOGIC, BcButtonLogic};
use pg_engine::logic::message::{self, MESSAGE_LOGIC, MessageLogic};
use pg_engine::logic::multi_model::{self, MULTI_MODEL_LOGIC, MultiModelLogic};
use pg_engine::logic::screen::{self, BC_SCREEN_LOGIC, BcScreenLogic};
use pg_engine::logic::{self, BC_LOGIC, BcLogic, NO_ANIMATION};
use pg_engine::math::VEC4_ORIGIN;
use pg_engine::object::{Class, Isa, isa};
use pg_engine::random::Random;
use pg_engine::sequence::{BcSequence, ItemResult, SequenceCall, SequenceDef, SequenceItemDef};
use pg_engine::texture::TextureDefId;
use pg_engine::view::{self, BcView};

const RANDOM_SEED: u64 = 7;

fn new_world() -> World {
    let mut world = World::new();
    world.insert_resource(Library::default());
    world.insert_resource(BcView::default());
    world.insert_resource(Random::with_seed(RANDOM_SEED));
    world.insert_resource(EngineHooks::default());
    world
}

/// A logic of `class` bound to `display_object` as `initWithDisplayObject:` leaves it on an
/// awake display object, but without its animation player.
fn bind_logic(world: &mut World, class: &'static Class, display_object: Entity) -> Entity {
    let state = BcLogic {
        shutdown: false,
        display_object,
        game_frame_running: false,
        animation: NO_ANIMATION,
        sequence: None,
        hibernate: false,
        hibernate_game_frame_running: false,
        pos_vel: VEC4_ORIGIN,
        pos_force: VEC4_ORIGIN,
        rot_vel: VEC4_ORIGIN,
        rot_force: VEC4_ORIGIN,
        stall_time: 0.0,
        delta_value: 0.0,
        last_move_x_pos: 0.0,
        last_move_y_pos: 0.0,
    };
    let logic = world.spawn((Isa(class), state)).id();
    object_mut(world, display_object).logic = Some(logic);
    logic
}

/// A logic of `class` bound to a new plain display object.
fn bare_logic(world: &mut World, class: &'static Class) -> (Entity, Entity) {
    let display_object = display::spawn(world, &BC_DISPLAY_OBJECT, Some("Object"));
    let logic = bind_logic(world, class, display_object);
    (logic, display_object)
}

fn state(world: &World, logic: Entity) -> &BcLogic {
    world.get::<BcLogic>(logic).expect("a logic object")
}

fn state_mut(world: &mut World, logic: Entity) -> Mut<'_, BcLogic> {
    world.get_mut::<BcLogic>(logic).expect("a logic object")
}

fn object(world: &World, display_object: Entity) -> &DisplayObject {
    world.get::<DisplayObject>(display_object).expect("a display object")
}

fn object_mut(world: &mut World, display_object: Entity) -> Mut<'_, DisplayObject> {
    world.get_mut::<DisplayObject>(display_object).expect("a display object")
}

fn listeners(world: &World) -> Vec<Entity> {
    world.resource::<BcView>().game_frame_listeners.clone()
}

fn add_sequence(world: &mut World, id: &str) {
    let placeholder_item = item("trace", &[]);
    let def = SequenceDef {
        id: id.to_owned(),
        items: vec![placeholder_item],
    };
    let mut library = world.resource_mut::<Library>();
    library.sequence_defs.insert(id.to_owned(), Arc::new(def));
}

fn top_sequence_id(world: &World, sequence: Entity) -> Option<String> {
    let state = world.get::<BcSequence>(sequence).expect("a sequence");
    let top = &state.stack[state.stack_index as usize];
    let def = top.def.as_ref()?;
    Some(def.id.clone())
}

fn item(method: &str, params: &[(&str, &str)]) -> SequenceItemDef {
    let mut owned_params = Vec::new();
    for (name, value) in params {
        owned_params.push((name.to_string(), value.to_string()));
    }
    SequenceItemDef {
        method: method.to_owned(),
        params: owned_params,
    }
}

/// Runs `item` through the logic's class, as the sequence runner does for its delegate.
fn run_item(world: &mut World, logic: Entity, item: &SequenceItemDef, first_time: bool, delta_time: f32) -> ItemResult {
    let class = isa(world, logic);
    let mut call = SequenceCall {
        sequence: Entity::PLACEHOLDER,
        first_time,
        delta_time,
        item,
    };
    let result = (class.sequence_item)(world, logic, &mut call);
    result.expect("the logic class handles the item")
}

#[test]
fn random_scaled_time_uses_the_original_integer_formula() {
    assert_eq!(logic::random_scaled_time(0.5, 1.5, 1000.0, 0), 0.5);
    assert_eq!(logic::random_scaled_time(0.5, 1.5, 1000.0, 1000), 1.5);
    assert_eq!(logic::random_scaled_time(0.5, 1.5, 1000.0, 1001), 0.5);

    let largest_offset = u32::MAX % 1001 + 500;
    let largest = largest_offset as f32 / 1000.0;
    assert_eq!(logic::random_scaled_time(0.5, 1.5, 1000.0, u32::MAX), largest);

    // A negative minimum goes through the u32 sum and back to a signed value.
    assert_eq!(logic::random_scaled_time(-0.5, 0.5, 1000.0, 0), -0.5);
    assert_eq!(logic::random_scaled_time(-0.5, 0.5, 1000.0, 1000), 0.5);

    // max one step below min: the range is 0 and r is not reduced.
    assert_eq!(logic::random_scaled_time(0.002, 0.001, 1000.0, 5), 0.007);

    // The products are single precision: 0.29f32 · 1000 rounds to 290 before truncation
    // (a double product would truncate to 289).
    assert_eq!(logic::random_scaled_time(0.29, 0.29, 1000.0, 12345), 0.29);
}

#[test]
fn start_and_stop_game_frame_register_the_logic_once() {
    let mut world = new_world();
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);

    logic::start_game_frame(&mut world, logic);
    logic::start_game_frame(&mut world, logic);
    assert_eq!(listeners(&world), vec![logic]);
    assert!(state(&world, logic).game_frame_running);

    logic::stop_game_frame(&mut world, logic);
    logic::stop_game_frame(&mut world, logic);
    assert!(listeners(&world).is_empty());
    assert!(!state(&world, logic).game_frame_running);
}

#[test]
fn game_frame_requests_are_deferred_while_hibernating() {
    let mut world = new_world();
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);

    logic::hibernate(&mut world, logic);
    assert!(logic::is_hibernating(&world, logic));
    assert!(!state(&world, logic).hibernate_game_frame_running);

    logic::start_game_frame(&mut world, logic);
    assert!(state(&world, logic).hibernate_game_frame_running);
    assert!(listeners(&world).is_empty());

    logic::unhibernate(&mut world, logic);
    assert_eq!(listeners(&world), vec![logic]);
    assert!(state(&world, logic).game_frame_running);
    // unhibernate does not clear the remembered flag.
    assert!(state(&world, logic).hibernate_game_frame_running);

    logic::hibernate(&mut world, logic);
    assert!(listeners(&world).is_empty());
    assert!(!state(&world, logic).game_frame_running);
    assert!(state(&world, logic).hibernate_game_frame_running);

    logic::stop_game_frame(&mut world, logic);
    assert!(!state(&world, logic).hibernate_game_frame_running);
    logic::unhibernate(&mut world, logic);
    assert!(listeners(&world).is_empty());
}

#[test]
fn first_behavior_while_hibernating_also_requests_the_logic_game_frame() {
    let mut world = new_world();
    add_sequence(&mut world, "Wander");
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    logic::hibernate(&mut world, logic);

    logic::set_behavior(&mut world, logic, Some("Wander"));
    let behavior = state(&world, logic).sequence.expect("the behaviour sequence");
    assert!(state(&world, logic).hibernate_game_frame_running, "setBehavior: quirk");
    assert!(listeners(&world).is_empty());

    let sequence = world.get::<BcSequence>(behavior).expect("a sequence");
    assert!(sequence.hibernate);
    assert!(sequence.hibernate_game_frame_running);
    assert!(!sequence.is_running);
    assert_eq!(sequence.delegate, Some(logic));

    // Waking up starts the logic's own game frame first, then the behaviour.
    logic::unhibernate(&mut world, logic);
    assert_eq!(listeners(&world), vec![logic, behavior]);
}

#[test]
fn later_behaviors_reuse_the_sequence_and_unknown_ids_stop_it() {
    let mut world = new_world();
    add_sequence(&mut world, "Wander");
    add_sequence(&mut world, "Dance");
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);

    logic::set_behavior(&mut world, logic, Some("Wander"));
    let behavior = state(&world, logic).sequence.expect("the behaviour sequence");
    assert!(!state(&world, logic).hibernate_game_frame_running);
    assert_eq!(listeners(&world), vec![behavior]);

    logic::set_behavior(&mut world, logic, Some("Dance"));
    assert_eq!(state(&world, logic).sequence, Some(behavior));
    assert_eq!(top_sequence_id(&world, behavior).as_deref(), Some("Dance"));
    assert_eq!(listeners(&world), vec![behavior]);

    logic::set_behavior(&mut world, logic, Some("Missing"));
    assert_eq!(top_sequence_id(&world, behavior), None);
    assert!(listeners(&world).is_empty());

    logic::set_behavior(&mut world, logic, None);
    assert_eq!(state(&world, logic).sequence, Some(behavior));
    assert_eq!(top_sequence_id(&world, behavior), None);
}

#[test]
fn shutdown_marks_the_logic_dead_and_stops_its_behavior() {
    let mut world = new_world();
    add_sequence(&mut world, "Wander");
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    logic::start_game_frame(&mut world, logic);
    logic::set_behavior(&mut world, logic, Some("Wander"));
    assert_eq!(listeners(&world).len(), 2);

    logic::shutdown(&mut world, logic);
    assert!(state(&world, logic).shutdown);
    assert!(!state(&world, logic).game_frame_running);
    assert!(listeners(&world).is_empty());
}

#[test]
fn apply_move_x_is_computed_in_double_precision() {
    let mut world = new_world();
    let (logic, display_object) = bare_logic(&mut world, &BC_LOGIC);
    let scale_x = f32::from_bits(0x3f33_3333);
    let start_x = f32::from_bits(0x43d4_eaf1);
    let last_move_x = f32::from_bits(0x4111_8eb7);
    let move_x = f32::from_bits(0x415e_a527);
    {
        let mut moved = object_mut(&mut world, display_object);
        moved.scale.x = scale_x;
        moved.scale.y = 2.0;
        moved.pos.x = start_x;
        moved.pos.y = 10.0;
    }
    {
        let mut moving = state_mut(&mut world, logic);
        moving.last_move_x_pos = last_move_x;
        moving.last_move_y_pos = 1.0;
    }

    logic::apply_move_pos(&mut world, logic, move_x, 4.0);

    let single_precision_step = scale_x * (move_x - last_move_x) / scale_x.abs();
    let single_precision_x = single_precision_step + start_x;
    assert_eq!(single_precision_x.to_bits(), 0x43d7_53a4, "the case must tell f32 from f64");
    assert_eq!(object(&world, display_object).pos.x.to_bits(), 0x43d7_53a5);
    assert_eq!(object(&world, display_object).pos.y, 16.0);
    assert_eq!(state(&world, logic).last_move_x_pos, move_x);
    assert_eq!(state(&world, logic).last_move_y_pos, 4.0);
}

#[test]
fn a_mirrored_sprite_moves_back_by_the_unscaled_step() {
    let mut world = new_world();
    let (logic, display_object) = bare_logic(&mut world, &BC_LOGIC);
    object_mut(&mut world, display_object).scale.x = -2.0;

    logic::apply_move_pos(&mut world, logic, 5.0, 0.0);
    assert_eq!(object(&world, display_object).pos.x, -5.0);

    logic::on_animation_move_enabled(&mut world, logic);
    assert_eq!(state(&world, logic).last_move_x_pos, 0.0);
}

#[test]
fn binary_frames_only_reach_multi_models() {
    let mut world = new_world();
    let player = animation::spawn(&mut world);

    let (plain_logic, _) = bare_logic(&mut world, &BC_LOGIC);
    logic::set_binary_frame_def(&mut world, plain_logic, Some(FrameRef(3)), Some(player));

    let model = display::multi_model::spawn(&mut world, Some("Model"));
    world.get_mut::<Model>(model).expect("a model").texture_def = Some(TextureDefId(1));
    let model_logic = bind_logic(&mut world, &MULTI_MODEL_LOGIC, model);
    logic::set_binary_frame_def(&mut world, model_logic, Some(FrameRef(3)), Some(player));

    let multi_model = world.get::<MultiModel>(model).expect("a multi model");
    assert_eq!(multi_model.frame, Some(FrameRef(3)));
    assert_eq!(multi_model.channels_of, Some(player));
    assert_eq!(world.get::<Model>(model).expect("a model").texture_def, None);
}

#[test]
fn transform_items_read_their_positional_parameter_names() {
    let mut world = new_world();
    let (logic, display_object) = bare_logic(&mut world, &BC_LOGIC);
    {
        let mut start = object_mut(&mut world, display_object);
        start.pos.y = 7.0;
        start.pivot_pos.x = 9.0;
        start.pivot_pos.y = 9.0;
        start.pivot_pos.z = 9.0;
        start.origin_pos.x = 9.0;
        start.origin_pos.y = 9.0;
        start.origin_pos.z = 9.0;
    }

    let set_pos = item("setPos", &[("xPos", "12.5"), ("zPos", "-277.12")]);
    assert_eq!(run_item(&mut world, logic, &set_pos, true, 0.0), ItemResult::Done);
    let pos = object(&world, display_object).pos;
    assert_eq!((pos.x, pos.y, pos.z), (12.5, 7.0, -277.12));

    let offset_pos = item("offsetPos", &[("xPos", "1"), ("yPos", "-2")]);
    run_item(&mut world, logic, &offset_pos, true, 0.0);
    let pos = object(&world, display_object).pos;
    assert_eq!((pos.x, pos.y, pos.z), (13.5, 5.0, -277.12));

    let set_scale = item("setScale", &[("zScale", "4"), ("yScale", "2")]);
    run_item(&mut world, logic, &set_scale, true, 0.0);
    let scale = object(&world, display_object).scale;
    assert_eq!((scale.x, scale.y, scale.z), (1.0, 2.0, 4.0));

    let set_rot = item("setRot", &[("zRot", "90")]);
    run_item(&mut world, logic, &set_rot, true, 0.0);
    let rot = object(&world, display_object).rot;
    assert_eq!((rot.x, rot.y, rot.z), (0.0, 0.0, 90.0));

    // setPivot and setOrigin write all three coordinates; absent ones read as 0.
    let set_pivot = item("setPivot", &[("xPos", "5")]);
    run_item(&mut world, logic, &set_pivot, true, 0.0);
    let pivot = object(&world, display_object).pivot_pos;
    assert_eq!((pivot.x, pivot.y, pivot.z), (5.0, 0.0, 0.0));

    let set_origin = item("setOrigin", &[("xPos", "1"), ("yPos", "2")]);
    run_item(&mut world, logic, &set_origin, true, 0.0);
    let origin = object(&world, display_object).origin_pos;
    assert_eq!((origin.x, origin.y, origin.z), (1.0, 2.0, 0.0));

    let set_vel = item("setVel", &[("zVel", "-1"), ("xVel", "3")]);
    run_item(&mut world, logic, &set_vel, true, 0.0);
    let velocity = state(&world, logic).pos_vel;
    assert_eq!((velocity.x, velocity.y, velocity.z, velocity.w), (3.0, 0.0, -1.0, 1.0));

    let set_alpha = item("setAlpha", &[("alpha", "0.25"), ("id", "Ignored")]);
    run_item(&mut world, logic, &set_alpha, true, 0.0);
    assert_eq!(object(&world, display_object).alpha, 0.25);
    assert!(object(&world, display_object).color);
}

#[test]
fn rotate_advances_two_degrees_and_wraps_past_360() {
    let mut world = new_world();
    let (logic, display_object) = bare_logic(&mut world, &BC_LOGIC);
    let rotate = item("rotate", &[]);

    object_mut(&mut world, display_object).rot.z = 357.0;
    assert_eq!(run_item(&mut world, logic, &rotate, true, 0.0), ItemResult::Pending);
    assert_eq!(object(&world, display_object).rot.z, 359.0);
    assert_eq!(run_item(&mut world, logic, &rotate, false, 0.0), ItemResult::Done);
    assert_eq!(object(&world, display_object).rot.z, 0.0);

    object_mut(&mut world, display_object).rot.z = 358.0;
    assert_eq!(run_item(&mut world, logic, &rotate, true, 0.0), ItemResult::Pending);
    assert_eq!(object(&world, display_object).rot.z, 360.0);

    object_mut(&mut world, display_object).rot.z = f32::NAN;
    assert_eq!(run_item(&mut world, logic, &rotate, true, 0.0), ItemResult::Pending);
    assert!(object(&world, display_object).rot.z.is_nan());
}

#[test]
fn stall_completes_on_the_call_after_the_time_ran_out() {
    let mut world = new_world();
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    let stall = item("stall", &[("time", "0.05")]);

    assert_eq!(run_item(&mut world, logic, &stall, true, 0.02), ItemResult::Pending);
    assert_eq!(run_item(&mut world, logic, &stall, false, 0.02), ItemResult::Pending);
    assert_eq!(run_item(&mut world, logic, &stall, false, 0.02), ItemResult::Pending);
    assert!(state(&world, logic).stall_time < 0.0);
    assert_eq!(run_item(&mut world, logic, &stall, false, 0.02), ItemResult::Done);

    let no_stall = item("stall", &[("time", "0")]);
    assert_eq!(run_item(&mut world, logic, &no_stall, true, 0.02), ItemResult::Done);
}

#[test]
fn random_stall_draws_one_arc4random() {
    let mut world = new_world();
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    let stall = item("stall", &[("minTime", "0.5"), ("maxTime", "1.5")]);

    assert_eq!(run_item(&mut world, logic, &stall, true, 0.0), ItemResult::Pending);

    let mut reference = Random::with_seed(RANDOM_SEED);
    let draw = reference.arc4random();
    let expected = logic::random_scaled_time(0.5, 1.5, 1000.0, draw);
    assert_eq!(state(&world, logic).stall_time, expected);
}

#[test]
fn tween_alpha_fades_the_own_object_and_lands_on_the_target() {
    let mut world = new_world();
    let (logic, display_object) = bare_logic(&mut world, &BC_LOGIC);
    let params = [("displayGroupId", "Other"), ("time", "0.1"), ("target", "0"), ("id", "Other")];
    let tween = item("tweenAlpha", &params);

    // The first call re-sets the current alpha.
    assert_eq!(run_item(&mut world, logic, &tween, true, 0.05), ItemResult::Pending);
    let first_alpha = object(&world, display_object).alpha;
    assert!((first_alpha - 1.0).abs() < 1e-6, "{first_alpha}");

    assert_eq!(run_item(&mut world, logic, &tween, false, 0.05), ItemResult::Pending);
    let second_alpha = object(&world, display_object).alpha;
    assert!((second_alpha - 0.5).abs() < 1e-6, "{second_alpha}");

    assert_eq!(run_item(&mut world, logic, &tween, false, 0.05), ItemResult::Done);
    assert_eq!(object(&world, display_object).alpha, 0.0);
    assert!(object(&world, display_object).color);
}

#[test]
fn wait_for_animation_waits_for_completion() {
    let mut world = new_world();
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    let player = animation::spawn(&mut world);
    state_mut(&mut world, logic).animation = player;
    let wait = item("waitForAnimation", &[]);

    assert_eq!(run_item(&mut world, logic, &wait, true, 0.0), ItemResult::Pending);
    world.get_mut::<BcAnimation>(player).expect("a player").is_complete = true;
    assert_eq!(run_item(&mut world, logic, &wait, false, 0.0), ItemResult::Done);
}

#[derive(Resource, Default)]
struct CallLog(Vec<(&'static str, Entity, Entity, String)>);

fn log_call(world: &mut World, name: &'static str, listener: Entity, sender: Entity, detail: String) {
    world.resource_mut::<CallLog>().0.push((name, listener, sender, detail));
}

fn record_press(world: &mut World, listener: Entity, button: Entity, _touch: Entity) {
    log_call(world, "press", listener, button, String::new());
}

fn record_release(world: &mut World, listener: Entity, button: Entity, _touch: Entity) {
    log_call(world, "release", listener, button, String::new());
}

fn record_release_outside(world: &mut World, listener: Entity, button: Entity, _touch: Entity) {
    log_call(world, "release outside", listener, button, String::new());
}

fn record_multi_model_event(world: &mut World, listener: Entity, logic: Entity, event_id: &str) {
    log_call(world, "multi model event", listener, logic, event_id.to_owned());
}

fn record_message_complete(world: &mut World, listener: Entity, message: Entity) {
    log_call(world, "message complete", listener, message, String::new());
}

fn record_episode_event(world: &mut World, event: &str) {
    let sender = Entity::PLACEHOLDER;
    log_call(world, "episode event", sender, sender, event.to_owned());
}

const LISTENER: Class = Class {
    name: "TestListener",
    on_button_press: record_press,
    on_button_release: record_release,
    on_button_release_outside: record_release_outside,
    on_multi_model_event: record_multi_model_event,
    on_message_complete: record_message_complete,
    ..Class::NS_OBJECT
};

fn logged_names(world: &World) -> Vec<&'static str> {
    let mut names = Vec::new();
    for (name, _, _, _) in &world.resource::<CallLog>().0 {
        names.push(*name);
    }
    names
}

fn new_button(world: &mut World, listener: Option<Entity>) -> Entity {
    let (button_logic, _) = bare_logic(world, &BC_BUTTON_LOGIC);
    world.entity_mut(button_logic).insert(BcButtonLogic::default());
    button::set_button_logic_listener(world, button_logic, listener);
    button_logic
}

fn armed(world: &World, button_logic: Entity) -> bool {
    world.get::<BcButtonLogic>(button_logic).expect("a button").touch_began_inside
}

#[test]
fn a_button_clicks_only_when_the_press_began_on_it() {
    let mut world = new_world();
    world.init_resource::<CallLog>();
    let listener = world.spawn(Isa(&LISTENER)).id();
    let button_logic = new_button(&mut world, Some(listener));
    let touch = world.spawn_empty().id();
    let class = isa(&world, button_logic);

    (class.touch_began_inside)(&mut world, button_logic, touch);
    assert!(armed(&world, button_logic));
    (class.touch_moved_inside)(&mut world, button_logic, touch);
    (class.touch_ended_inside)(&mut world, button_logic, touch);
    assert!(!armed(&world, button_logic));
    assert_eq!(logged_names(&world), vec!["press", "release"]);
    let (_, logged_listener, logged_button, _) = world.resource::<CallLog>().0[1].clone();
    assert_eq!((logged_listener, logged_button), (listener, button_logic));

    // Sliding off and back on cancels the click.
    world.resource_mut::<CallLog>().0.clear();
    (class.touch_began_inside)(&mut world, button_logic, touch);
    (class.touch_moved_exited)(&mut world, button_logic, touch);
    (class.touch_moved_entered)(&mut world, button_logic, touch);
    assert!(!armed(&world, button_logic));
    (class.touch_ended_inside)(&mut world, button_logic, touch);
    assert_eq!(logged_names(&world), vec!["press", "release outside"]);

    // A finger beginning elsewhere disarms the button; any finger lifting elsewhere reports.
    world.resource_mut::<CallLog>().0.clear();
    (class.touch_began_inside)(&mut world, button_logic, touch);
    (class.touch_began_outside)(&mut world, button_logic, touch);
    assert!(!armed(&world, button_logic));
    (class.touch_ended_outside)(&mut world, button_logic, touch);
    assert_eq!(logged_names(&world), vec!["press", "release outside"]);
}

#[test]
fn a_button_without_listener_still_tracks_its_press() {
    let mut world = new_world();
    world.init_resource::<CallLog>();
    let button_logic = new_button(&mut world, None);
    let touch = world.spawn_empty().id();

    button::touch_began_inside(&mut world, button_logic, touch);
    assert!(armed(&world, button_logic));
    button::touch_ended_inside(&mut world, button_logic, touch);
    assert!(!armed(&world, button_logic));
    assert!(world.resource::<CallLog>().0.is_empty());
}

#[test]
fn multi_model_items_reach_the_listener_and_the_idol_manager() {
    let mut world = new_world();
    world.init_resource::<CallLog>();
    let hooks = EngineHooks {
        trigger_episode_event: record_episode_event,
        ..EngineHooks::default()
    };
    world.insert_resource(hooks);
    let listener = world.spawn(Isa(&LISTENER)).id();
    let (model_logic, display_object) = bare_logic(&mut world, &MULTI_MODEL_LOGIC);
    world.entity_mut(model_logic).insert(MultiModelLogic::default());
    multi_model::set_multi_model_logic_listener(&mut world, model_logic, Some(listener));

    let event = item("multiModelEvent", &[("eventId", "RainbowGlowComplete")]);
    assert_eq!(run_item(&mut world, model_logic, &event, true, 0.0), ItemResult::Done);
    let trigger = item("triggerEpisodeEvent", &[("event", "PygmyEatenByShark")]);
    assert_eq!(run_item(&mut world, model_logic, &trigger, true, 0.0), ItemResult::Done);

    let log = world.resource::<CallLog>().0.clone();
    assert_eq!(log[0], ("multi model event", listener, model_logic, "RainbowGlowComplete".to_owned()));
    assert_eq!(log[1].0, "episode event");
    assert_eq!(log[1].3, "PygmyEatenByShark");

    // BCLogic's handlers stay reachable; unknown items fall through to BCLibrary.
    let set_alpha = item("setAlpha", &[("alpha", "0.5")]);
    run_item(&mut world, model_logic, &set_alpha, true, 0.0);
    assert_eq!(object(&world, display_object).alpha, 0.5);
    let library_item = item("playSound", &[]);
    let mut call = SequenceCall {
        sequence: Entity::PLACEHOLDER,
        first_time: true,
        delta_time: 0.0,
        item: &library_item,
    };
    assert_eq!(multi_model::sequence_item(&mut world, model_logic, &mut call), None);
}

#[test]
fn die_shuts_down_and_deallocates_the_logic() {
    let mut world = new_world();
    add_sequence(&mut world, "Effect");
    let (model_logic, display_object) = bare_logic(&mut world, &MULTI_MODEL_LOGIC);
    world.entity_mut(model_logic).insert(MultiModelLogic::default());
    let player = give_player(&mut world, model_logic);
    logic::start_game_frame(&mut world, model_logic);
    logic::set_behavior(&mut world, model_logic, Some("Effect"));
    let behavior = state(&world, model_logic).sequence.expect("the behaviour sequence");

    let die = item("die", &[]);
    assert_eq!(run_item(&mut world, model_logic, &die, true, 0.0), ItemResult::Done);
    assert!(world.get::<BcLogic>(model_logic).is_none());
    assert!(world.get::<BcAnimation>(player).is_none());
    assert!(listeners(&world).is_empty());

    // The running sequence outlives the item (leaked in the original), stopped; the display
    // object stays for its other owners.
    let sequence = world.get::<BcSequence>(behavior).expect("the sequence is kept");
    assert!(!sequence.is_running);
    assert!(world.get::<DisplayObject>(display_object).is_some());
}

/// Gives a bare logic its animation player, as the initialiser does.
fn give_player(world: &mut World, logic: Entity) -> Entity {
    let player = animation::spawn(world);
    animation::set_animation_listener(world, player, Some(logic));
    state_mut(world, logic).animation = player;
    player
}

fn player(world: &World, player: Entity) -> &BcAnimation {
    world.get::<BcAnimation>(player).expect("an animation player")
}

#[test]
fn init_hibernates_a_logic_bound_to_a_sleeping_object_before_creating_its_animation() {
    let mut world = new_world();
    let display_object = display::spawn(&mut world, &BC_DISPLAY_OBJECT, Some("Asleep"));
    let logic = logic::spawn(&mut world, &BC_LOGIC, display_object);

    assert_eq!(object(&world, display_object).logic, Some(logic));
    let created = state(&world, logic);
    assert!(created.hibernate);
    assert!(!created.hibernate_game_frame_running);
    assert!(!created.game_frame_running);
    assert_eq!(created.sequence, None);
    assert_eq!(created.pos_vel, VEC4_ORIGIN);
    assert_eq!(created.rot_force, VEC4_ORIGIN);

    let animation_player = created.animation;
    assert_ne!(animation_player, NO_ANIMATION);
    assert!(player(&world, animation_player).hibernate);
    assert_eq!(player(&world, animation_player).listener, Some(logic));

    display::unhibernate(&mut world, display_object);
    assert!(!state(&world, logic).hibernate);
    assert!(!player(&world, animation_player).hibernate);
    assert!(listeners(&world).is_empty());
}

#[test]
fn init_leaves_a_logic_bound_to_an_awake_object_awake() {
    let mut world = new_world();
    let display_object = display::spawn(&mut world, &BC_DISPLAY_OBJECT, Some("Awake"));
    {
        let mut awake = object_mut(&mut world, display_object);
        awake.hibernate = false;
        awake.hibernate_ref_count = 1;
    }

    let button_logic = button::spawn(&mut world, display_object);
    assert!(!state(&world, button_logic).hibernate);
    let animation_player = state(&world, button_logic).animation;
    assert!(!player(&world, animation_player).hibernate);
    assert!(!armed(&world, button_logic));
}

#[test]
fn a_new_multi_model_given_a_behavior_while_asleep_runs_its_game_frame_on_waking() {
    let mut world = new_world();
    add_sequence(&mut world, "OneShotEffect");
    let model = display::multi_model::spawn(&mut world, Some("OneShot"));

    let model_logic = multi_model::spawn(&mut world, model);
    assert!(world.get::<MultiModelLogic>(model_logic).is_some());
    logic::set_behavior(&mut world, model_logic, Some("OneShotEffect"));
    let behavior = state(&world, model_logic).sequence.expect("the behaviour sequence");
    assert!(listeners(&world).is_empty());

    // Moving the model into an awake group wakes it: the quirk starts the logic's own game
    // frame although MultiModelLogic stopped it.
    display::unhibernate(&mut world, model);
    assert_eq!(listeners(&world), vec![model_logic, behavior]);
}

#[test]
fn a_new_message_is_hidden_and_has_no_game_frame() {
    let mut world = new_world();
    let display_object = display::spawn(&mut world, &BC_DISPLAY_OBJECT, Some("Message"));
    let message_logic = message::spawn(&mut world, display_object);

    let created = world.get::<MessageLogic>(message_logic).expect("a message");
    assert!(!created.active);
    assert_eq!(created.listener, None);
    assert_eq!(object(&world, display_object).display_group, None);
    assert!(!state(&world, message_logic).hibernate_game_frame_running);
}

#[test]
fn shutdown_stops_the_animation_and_breaks_the_listener_cycle() {
    let mut world = new_world();
    install_animations(&mut world, &["WalkAnim"]);
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    let animation_player = give_player(&mut world, logic);
    logic::play_animation_id(&mut world, logic, Some("WalkAnim"));
    assert!(player(&world, animation_player).is_running);

    logic::shutdown(&mut world, logic);
    assert!(!player(&world, animation_player).is_running);
    assert_eq!(player(&world, animation_player).listener, None);
    assert!(listeners(&world).is_empty());
}

/// Registers one-frame, non-looping timeline animations under their names, in order.
fn install_animations(world: &mut World, names: &[&str]) {
    let mut timelines = Timelines::default();
    for (index, name) in names.iter().enumerate() {
        let record = TimelineAnimation {
            name: name.to_string(),
            fps: 30,
            length: 1,
            looping: false,
            move_enabled: false,
            move_x: 0.0,
            move_y: 0.0,
            next_animation: None,
            first_frame: 0,
            frame_count: 0,
        };
        timelines.animations.push(record);
        let id = AnimationDefId(index as u32);
        world.resource_mut::<Library>().animation_defs.insert(name.to_string(), id);
    }
    world.insert_resource(timelines);
}

#[test]
fn set_animation_plays_and_scrubs_to_the_percent() {
    let mut world = new_world();
    install_animations(&mut world, &["IdleAnim", "HealthAnim"]);
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    let animation_player = give_player(&mut world, logic);

    let params = [("displayGroupId", "Other"), ("animation", "HealthAnim"), ("percent", "0.5")];
    let set_animation = item("setAnimation", &params);
    assert_eq!(run_item(&mut world, logic, &set_animation, true, 0.0), ItemResult::Done);
    assert_eq!(player(&world, animation_player).def, Some(AnimationDefId(1)));
    assert!(player(&world, animation_player).use_frame_percent);
    assert_eq!(player(&world, animation_player).frame_percent, 0.5);
}

#[test]
fn choose_animation_plays_the_entry_picked_by_arc4random() {
    let mut world = new_world();
    install_animations(&mut world, &["AAnim", "BAnim", "CAnim"]);
    let (logic, _) = bare_logic(&mut world, &BC_LOGIC);
    let animation_player = give_player(&mut world, logic);

    let choose = item("chooseAnimation", &[("list", "AAnim,BAnim,CAnim")]);
    assert_eq!(run_item(&mut world, logic, &choose, true, 0.0), ItemResult::Done);

    let mut reference = Random::with_seed(RANDOM_SEED);
    let draw = reference.arc4random();
    let expected = AnimationDefId(draw % 3);
    assert_eq!(player(&world, animation_player).def, Some(expected));
}

#[test]
fn set_texture_def_stops_the_animation_and_shows_the_texture() {
    let mut world = new_world();
    install_animations(&mut world, &["LeafAnim"]);
    world.resource_mut::<Library>().texture_def_ids.insert("Leaf".to_owned(), TextureDefId(5));
    let model = display::multi_model::spawn(&mut world, Some("Leaf"));
    let model_logic = bind_logic(&mut world, &MULTI_MODEL_LOGIC, model);
    world.entity_mut(model_logic).insert(MultiModelLogic::default());
    let animation_player = give_player(&mut world, model_logic);
    logic::play_animation_id(&mut world, model_logic, Some("LeafAnim"));
    world.get_mut::<MultiModel>(model).expect("a multi model").frame = Some(FrameRef(0));

    let set_texture = item("setTextureDef", &[("texture", "Leaf")]);
    assert_eq!(run_item(&mut world, model_logic, &set_texture, true, 0.0), ItemResult::Done);
    assert!(!player(&world, animation_player).is_running);
    assert_eq!(world.get::<Model>(model).expect("a model").texture_def, Some(TextureDefId(5)));
    assert_eq!(world.get::<MultiModel>(model).expect("a multi model").frame, None);

    let unknown_texture = item("setTextureDef", &[("texture", "Missing")]);
    run_item(&mut world, model_logic, &unknown_texture, true, 0.0);
    assert_eq!(world.get::<Model>(model).expect("a model").texture_def, None);
}

fn new_screen(world: &mut World) -> Entity {
    let (screen_logic, _) = bare_logic(world, &BC_SCREEN_LOGIC);
    let components = (MultiModelLogic::default(), BcScreenLogic::default());
    world.entity_mut(screen_logic).insert(components);
    screen_logic
}

#[test]
fn screen_layers_hold_logics_and_behaviors_go_to_the_screen() {
    let mut world = new_world();
    add_sequence(&mut world, "IdolScreenOpen");
    let screen_logic = new_screen(&mut world);
    let (panel_logic, _) = bare_logic(&mut world, &MULTI_MODEL_LOGIC);

    screen::set_screen_layer_logic(&mut world, screen_logic, 1, Some(panel_logic));
    let layers = world.get::<BcScreenLogic>(screen_logic).expect("a screen").layers;
    assert_eq!(layers[1].logic, Some(panel_logic));

    // Layer 2 has no logic: the animation message goes to nil.
    let layer_animation = item("setLayerAnimation", &[("layer", "2"), ("animation", "PanelOpenAnim")]);
    assert_eq!(run_item(&mut world, screen_logic, &layer_animation, true, 0.0), ItemResult::Done);
    let disable_buttons = item("disableButtonLayer", &[("layer", "1")]);
    assert_eq!(run_item(&mut world, screen_logic, &disable_buttons, true, 0.0), ItemResult::Done);

    screen::set_screen_layer_behavior(&mut world, screen_logic, 1, Some("IdolScreenOpen"));
    assert!(state(&world, screen_logic).sequence.is_some());
    assert_eq!(state(&world, panel_logic).sequence, None);
}

fn register_group(world: &mut World, id: &str) -> Entity {
    let created = group::spawn_group(world, Some(id));
    world.resource_mut::<Library>().display_groups.insert(id.to_owned(), created);
    created
}

fn children_of(world: &World, parent: Entity) -> Vec<Entity> {
    group::children(world, parent).to_vec()
}

fn screen_state(world: &World, screen_logic: Entity) -> &BcScreenLogic {
    world.get::<BcScreenLogic>(screen_logic).expect("a screen")
}

fn active_group(world: &World) -> Option<Entity> {
    world.resource::<BcView>().active_display_group
}

#[test]
fn a_screen_opens_over_the_game_takes_over_the_root_and_closes() {
    let mut world = new_world();
    let game_root = register_group(&mut world, "IslandDisplayGroup");
    let scenery = display::spawn(&mut world, &BC_DISPLAY_OBJECT, Some("Island"));
    group::add_display_object(&mut world, game_root, Some(scenery));
    {
        let mut root = object_mut(&mut world, game_root);
        root.pos.x = -160.0;
        root.pos.y = 240.0;
        root.rot.z = -90.0;
    }
    view::set_active_display_group(&mut world, Some(game_root));

    let screen_group = register_group(&mut world, "ScreenDisplayGroup");
    {
        let mut moved = object_mut(&mut world, screen_group);
        moved.pos.x = 12.0;
        moved.pos.y = 34.0;
        moved.rot.z = 56.0;
    }
    let buttons = register_group(&mut world, "IdolDetailButtons");

    let screen_model = display::multi_model::spawn(&mut world, Some("IdolScreen"));
    {
        let mut placed = object_mut(&mut world, screen_model);
        placed.pos.x = 100.0;
        placed.pos.y = 50.0;
    }
    let screen_logic = world.spawn(Isa(&BC_SCREEN_LOGIC)).id();
    screen::init_with_display_object(&mut world, screen_logic, screen_model);
    assert_eq!(screen_state(&world, screen_logic).screen_display_group, Some(screen_group));
    assert!(state(&world, screen_logic).hibernate);

    // openScreen: the screen group goes on top of the game root, the screen into it.
    screen::open_screen(&mut world, screen_logic);
    assert_eq!(screen_state(&world, screen_logic).active_display_group, Some(game_root));
    assert_eq!(children_of(&world, game_root), vec![scenery, screen_group]);
    assert_eq!(children_of(&world, screen_group), vec![screen_model]);
    assert!(!state(&world, screen_logic).hibernate);
    let opened = object(&world, screen_group);
    assert_eq!((opened.pos.x, opened.pos.y, opened.rot.z), (0.0, 0.0, 0.0));

    // A button layer sits at its layer logic's x/y inside the screen group.
    screen::set_screen_layer_logic(&mut world, screen_logic, 0, Some(screen_logic));
    let button_params = [("layer", "0"), ("displayGroup", "IdolDetailButtons")];
    let enable_buttons = item("enableButtonLayer", &button_params);
    assert_eq!(run_item(&mut world, screen_logic, &enable_buttons, true, 0.0), ItemResult::Done);
    assert_eq!(children_of(&world, screen_group), vec![screen_model, buttons]);
    let button_pos = object(&world, buttons).pos;
    assert_eq!((button_pos.x, button_pos.y), (100.0, 50.0));
    assert_eq!(screen_state(&world, screen_logic).layers[0].button_display_group, Some(buttons));

    // disableGameLayer: the screen group becomes the root with the game root's transform.
    let disable_game = item("disableGameLayer", &[]);
    assert_eq!(run_item(&mut world, screen_logic, &disable_game, true, 0.0), ItemResult::Done);
    assert_eq!(active_group(&world), Some(screen_group));
    assert_eq!(children_of(&world, game_root), vec![scenery]);
    assert_eq!(object(&world, screen_group).pos, object(&world, game_root).pos);
    assert_eq!(object(&world, screen_group).rot, object(&world, game_root).rot);
    assert!(object(&world, scenery).hibernate);
    assert!(!state(&world, screen_logic).hibernate);

    // enableGameLayer: back on top of the game root, which is the root again.
    let enable_game = item("enableGameLayer", &[]);
    assert_eq!(run_item(&mut world, screen_logic, &enable_game, true, 0.0), ItemResult::Done);
    assert_eq!(active_group(&world), Some(game_root));
    assert_eq!(children_of(&world, game_root), vec![scenery, screen_group]);
    let returned = object(&world, screen_group);
    assert_eq!((returned.pos.x, returned.pos.y, returned.rot.z), (0.0, 0.0, 0.0));
    assert!(!object(&world, scenery).hibernate);
    assert!(!state(&world, screen_logic).hibernate);

    let disable_buttons = item("disableButtonLayer", &[("layer", "0")]);
    assert_eq!(run_item(&mut world, screen_logic, &disable_buttons, true, 0.0), ItemResult::Done);
    assert_eq!(children_of(&world, screen_group), vec![screen_model]);
    assert_eq!(screen_state(&world, screen_logic).layers[0].button_display_group, None);

    // closeScreenComplete → closeScreenNow takes everything down again.
    run_item(&mut world, screen_logic, &enable_buttons, true, 0.0);
    let close = item("closeScreenComplete", &[]);
    assert_eq!(run_item(&mut world, screen_logic, &close, true, 0.0), ItemResult::Done);
    assert_eq!(active_group(&world), Some(game_root));
    assert_eq!(children_of(&world, game_root), vec![scenery]);
    assert!(children_of(&world, screen_group).is_empty());
    assert_eq!(screen_state(&world, screen_logic).layers[0].button_display_group, None);
    assert!(state(&world, screen_logic).hibernate);
}

#[test]
fn a_message_is_shown_in_the_pocket_god_group_until_it_completes() {
    let mut world = new_world();
    add_sequence(&mut world, "ShowMessage");
    let pocket_god = register_group(&mut world, "PocketGod");
    view::set_active_display_group(&mut world, Some(pocket_god));
    let display_object = display::spawn(&mut world, &BC_DISPLAY_OBJECT, Some("Message"));
    group::add_display_object(&mut world, pocket_god, Some(display_object));

    let message_logic = message::spawn(&mut world, display_object);
    assert!(children_of(&world, pocket_god).is_empty());
    assert!(state(&world, message_logic).hibernate);

    message::show_message(&mut world, message_logic, "ShowMessage");
    assert_eq!(children_of(&world, pocket_god), vec![display_object]);
    assert!(!state(&world, message_logic).hibernate);
    let behavior = state(&world, message_logic).sequence.expect("the behaviour sequence");
    assert_eq!(listeners(&world), vec![behavior]);

    let complete = item("onMessageComplete", &[]);
    run_item(&mut world, message_logic, &complete, true, 0.0);
    assert!(children_of(&world, pocket_god).is_empty());
    assert!(state(&world, message_logic).hibernate);
}

#[test]
#[should_panic(expected = "outside mScreenLayerArray")]
fn screen_layer_indices_past_the_array_panic() {
    let mut world = new_world();
    let screen_logic = new_screen(&mut world);
    let disable_buttons = item("disableButtonLayer", &[("layer", "10")]);
    run_item(&mut world, screen_logic, &disable_buttons, true, 0.0);
}

#[test]
#[should_panic(expected = "outside mScreenLayerArray")]
fn negative_screen_layer_indices_panic() {
    let mut world = new_world();
    let screen_logic = new_screen(&mut world);
    let disable_buttons = item("disableButtonLayer", &[("layer", "-1")]);
    run_item(&mut world, screen_logic, &disable_buttons, true, 0.0);
}

fn new_message(world: &mut World, listener: Option<Entity>) -> Entity {
    let (message_logic, _) = bare_logic(world, &MESSAGE_LOGIC);
    world.entity_mut(message_logic).insert(MessageLogic::default());
    message::set_message_logic_listener(world, message_logic, listener);
    message_logic
}

#[test]
fn messages_show_run_their_behavior_and_report_completion() {
    let mut world = new_world();
    world.init_resource::<CallLog>();
    add_sequence(&mut world, "ShowMessage");
    add_sequence(&mut world, "ShowOtherMessage");
    let listener = world.spawn(Isa(&LISTENER)).id();
    let message_logic = new_message(&mut world, Some(listener));

    message::show_message(&mut world, message_logic, "ShowMessage");
    assert!(world.get::<MessageLogic>(message_logic).expect("a message").active);
    let behavior = state(&world, message_logic).sequence.expect("the behaviour sequence");
    assert_eq!(top_sequence_id(&world, behavior).as_deref(), Some("ShowMessage"));

    message::show_message(&mut world, message_logic, "ShowOtherMessage");
    assert_eq!(top_sequence_id(&world, behavior).as_deref(), Some("ShowOtherMessage"));

    let complete = item("onMessageComplete", &[]);
    assert_eq!(run_item(&mut world, message_logic, &complete, true, 0.0), ItemResult::Done);
    assert!(!world.get::<MessageLogic>(message_logic).expect("a message").active);
    let log = world.resource::<CallLog>().0.clone();
    assert_eq!(log, vec![("message complete", listener, message_logic, String::new())]);
}

fn shown_text(world: &World, text_object: Entity) -> Option<String> {
    let shown = world.get::<Text>(text_object).expect("a text object");
    shown.text.clone()
}

#[test]
fn count_down_timer_counts_like_stall_and_shows_whole_seconds_left() {
    let mut world = new_world();
    let timer_group = register_group(&mut world, "ChallengeTimer");
    let timer = display::text::spawn(&mut world, Some("Timer"), Some("Font"));
    group::add_display_object(&mut world, timer_group, Some(timer));
    let message_logic = new_message(&mut world, None);
    let count_down = item("countDownTimer", &[("time", "1.5")]);

    // The text shows the time truncated toward zero, before this frame's decrement.
    assert_eq!(run_item(&mut world, message_logic, &count_down, true, 1.0), ItemResult::Pending);
    assert_eq!(shown_text(&world, timer).as_deref(), Some("1"));
    assert_eq!(state(&world, message_logic).stall_time, 0.5);
    assert_eq!(run_item(&mut world, message_logic, &count_down, false, 1.0), ItemResult::Pending);
    assert_eq!(shown_text(&world, timer).as_deref(), Some("0"));
    assert_eq!(run_item(&mut world, message_logic, &count_down, false, 1.0), ItemResult::Done);
    assert_eq!(shown_text(&world, timer).as_deref(), Some("0"));

    let negative = item("countDownTimer", &[("time", "-1.5")]);
    assert_eq!(run_item(&mut world, message_logic, &negative, true, 1.0), ItemResult::Done);
    assert_eq!(shown_text(&world, timer).as_deref(), Some("-1"));
}

#[test]
fn count_down_timer_without_a_timer_object_only_counts() {
    let mut world = new_world();
    let message_logic = new_message(&mut world, None);
    let count_down = item("countDownTimer", &[("time", "0.5")]);

    assert_eq!(run_item(&mut world, message_logic, &count_down, true, 1.0), ItemResult::Pending);
    assert_eq!(run_item(&mut world, message_logic, &count_down, false, 1.0), ItemResult::Done);
}
