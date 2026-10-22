//! `BCTouch` creation, the 1000-slot event ring and dispatch to the view delegate
//! (`docs/re/rendering.md` §9.1–§9.2, `docs/re/logic.md` §9.1).

use bevy_ecs::prelude::*;
use bevy_math::Vec4;
use pg_engine::hooks::EngineHooks;
use pg_engine::touch::{self, BcTouch, PlatformTouch, PlatformTouches, TouchPhase};
use pg_engine::view::{self, BcView};

#[derive(Resource, Default)]
struct DelegateLog(Vec<(&'static str, Entity, Vec<Entity>)>);

#[derive(Resource, Default)]
struct PreStartLog(Vec<(f32, f32)>);

fn log_call(world: &mut World, name: &'static str, touch: Entity) {
    let all_touches = touch::all_touches(world);
    world.resource_mut::<DelegateLog>().0.push((name, touch, all_touches));
}

fn began(world: &mut World, touch: Entity) {
    log_call(world, "began", touch);
}

fn moved(world: &mut World, touch: Entity) {
    log_call(world, "moved", touch);
}

fn ended(world: &mut World, touch: Entity) {
    log_call(world, "ended", touch);
}

fn cancelled(world: &mut World, touch: Entity) {
    log_call(world, "cancelled", touch);
}

fn pre_start(world: &mut World, x: f32, y: f32) {
    world.resource_mut::<PreStartLog>().0.push((x, y));
}

fn new_world() -> World {
    let mut world = World::new();
    world.init_resource::<BcView>();
    world.init_resource::<DelegateLog>();
    world.init_resource::<PreStartLog>();
    let hooks = EngineHooks {
        touch_began: began,
        touch_moved: moved,
        touch_ended: ended,
        touch_cancelled: cancelled,
        pre_start_touch_began: pre_start,
        ..EngineHooks::default()
    };
    world.insert_resource(hooks);
    world
}

fn platform_touch(id: u64, phase: TouchPhase, x: f32, y: f32) -> PlatformTouch {
    PlatformTouch {
        id,
        phase,
        time_stamp: 1000.25,
        tap_count: 1,
        x,
        y,
    }
}

fn calls(world: &World) -> Vec<(&'static str, Entity, Vec<Entity>)> {
    world.resource::<DelegateLog>().0.clone()
}

#[test]
fn inactive_touch_only_reports_beginnings() {
    let mut world = new_world();
    let began_touch = platform_touch(1, TouchPhase::Began, 10.0, 20.0);
    let moved_touch = platform_touch(1, TouchPhase::Moved, 11.0, 21.0);
    touch::create_touch_event(&mut world, began_touch);
    touch::create_touch_event(&mut world, moved_touch);
    touch::process_touch_events(&mut world, 0.5);

    assert_eq!(world.resource::<PreStartLog>().0, vec![(10.0, 20.0)]);
    assert!(calls(&world).is_empty());
    assert!(world.resource::<BcView>().ui_touches.is_empty());
}

#[test]
fn one_finger_keeps_its_touch_object() {
    let mut world = new_world();
    world.resource_mut::<BcView>().touch_active = true;

    touch::create_touch_event(&mut world, platform_touch(7, TouchPhase::Began, 10.0, 20.0));
    touch::create_touch_event(&mut world, platform_touch(7, TouchPhase::Moved, 30.0, 40.0));
    touch::create_touch_event(&mut world, platform_touch(7, TouchPhase::Ended, 50.0, 60.0));
    assert!(world.resource::<BcView>().ui_touches.is_empty());

    touch::process_touch_events(&mut world, 2.5);
    let log = calls(&world);
    assert_eq!(log.len(), 3);
    let finger = log[0].1;
    assert_eq!(log[0], ("began", finger, vec![finger]));
    assert_eq!(log[1], ("moved", finger, vec![finger]));
    assert_eq!(log[2], ("ended", finger, vec![finger]));
    assert!(touch::all_touches(&world).is_empty());

    let state = world.get::<BcTouch>(finger).expect("touches are never despawned");
    assert_eq!(state.touch_phase, TouchPhase::Ended);
    assert_eq!(state.pos, Vec4::new(50.0, 60.0, 0.0, 1.0));
    assert_eq!(state.game_time, 2.5);
    assert_eq!(state.time_stamp, 1000.25);
    assert_eq!(state.tap_count, 1);
    assert_eq!(state.ui_touch, 7);

    // The same platform id after the end is a new finger.
    touch::create_touch_event(&mut world, platform_touch(7, TouchPhase::Began, 1.0, 2.0));
    touch::process_touch_events(&mut world, 3.0);
    let log = calls(&world);
    assert_ne!(log[3].1, finger);
}

#[test]
fn all_touches_lists_every_finger_down() {
    let mut world = new_world();
    world.resource_mut::<BcView>().touch_active = true;

    touch::create_touch_event(&mut world, platform_touch(1, TouchPhase::Began, 0.0, 0.0));
    touch::create_touch_event(&mut world, platform_touch(2, TouchPhase::Began, 5.0, 5.0));
    touch::create_touch_event(&mut world, platform_touch(1, TouchPhase::Stationary, 0.0, 0.0));
    touch::create_touch_event(&mut world, platform_touch(1, TouchPhase::Cancelled, 0.0, 0.0));
    touch::create_touch_event(&mut world, platform_touch(2, TouchPhase::Moved, 6.0, 6.0));
    touch::process_touch_events(&mut world, 0.0);

    let log = calls(&world);
    let first = log[0].1;
    let second = log[1].1;
    assert_eq!(log[0], ("began", first, vec![first]));
    assert_eq!(log[1], ("began", second, vec![first, second]));
    assert_eq!(log[2], ("cancelled", first, vec![first, second]));
    assert_eq!(log[3], ("moved", second, vec![second]));
    assert_eq!(touch::all_touches(&world), vec![second]);

    touch::clear_touch_array_for_modal_view_controller(&mut world);
    assert!(touch::all_touches(&world).is_empty());
}

#[test]
fn platform_touches_are_drained_by_the_game_loop() {
    let mut world = new_world();
    world.resource_mut::<BcView>().touch_active = true;
    let delivered = vec![
        platform_touch(3, TouchPhase::Began, 1.0, 1.0),
        platform_touch(3, TouchPhase::Moved, 2.0, 2.0),
    ];
    world.insert_resource(PlatformTouches(delivered));

    view::game_loop(&mut world, 1.0 / 60.0);
    let names: Vec<&str> = calls(&world).iter().map(|call| call.0).collect();
    assert_eq!(names, vec!["began", "moved"]);
    assert!(world.resource::<PlatformTouches>().0.is_empty());
    let finger = calls(&world)[0].1;
    assert_eq!(world.get::<BcTouch>(finger).map(|state| state.game_time), Some(1.0 / 60.0));
}

#[test]
fn a_full_ring_looks_empty() {
    let mut world = new_world();
    world.resource_mut::<BcView>().touch_active = true;

    for _ in 0..touch::TOUCH_EVENT_RING_SIZE {
        touch::create_touch_event(&mut world, platform_touch(9, TouchPhase::Moved, 0.0, 0.0));
    }
    {
        let ring = &world.resource::<BcView>().touch_events;
        assert_eq!(ring.write_index, ring.read_index);
    }
    touch::process_touch_events(&mut world, 0.0);
    assert!(calls(&world).is_empty());

    // The next event makes the stale slots reachable again.
    touch::create_touch_event(&mut world, platform_touch(9, TouchPhase::Moved, 0.0, 0.0));
    touch::process_touch_events(&mut world, 0.0);
    assert_eq!(calls(&world).len(), 1);
}
