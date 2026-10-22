//! `BCDisplayGroup` / `BCDisplayMarker` tree operations and ref-counted hibernation
//! (`docs/re/rendering.md` §4.3, §4.4, §4.6, §9.4).

use bevy_ecs::prelude::*;
use pg_engine::display::{self, BC_DISPLAY_OBJECT, group};
use pg_engine::view::{self, BcView};

fn new_world() -> World {
    let mut world = World::new();
    world.init_resource::<BcView>();
    world
}

fn spawn_object(world: &mut World, id: &str) -> Entity {
    display::spawn(world, &BC_DISPLAY_OBJECT, Some(id))
}

fn ref_count(world: &World, entity: Entity) -> i32 {
    display::object(world, entity).hibernate_ref_count
}

fn hibernating(world: &World, entity: Entity) -> bool {
    display::object(world, entity).hibernate
}

#[test]
fn lookups_follow_the_child_array() {
    let mut world = new_world();
    let root = group::spawn_group(&mut world, Some("Root"));
    let first = spawn_object(&mut world, "First");
    let second = spawn_object(&mut world, "Second");
    let third = spawn_object(&mut world, "Second");
    group::add_display_object(&mut world, root, Some(first));
    group::add_display_object(&mut world, root, Some(second));
    group::add_display_object(&mut world, root, Some(third));
    group::add_display_object(&mut world, root, None);

    assert_eq!(group::display_object_count(&world, root), 3);
    assert_eq!(group::get_display_object(&world, root, "Second"), Some(second));
    assert_eq!(group::get_display_object(&world, root, "Missing"), None);
    assert_eq!(group::get_display_object_index(&world, root, "Second"), 1);
    assert_eq!(group::get_display_object_index(&world, root, "Missing"), -1);
    assert_eq!(group::index_of_display_object(&world, root, third), 2);
    assert_eq!(group::index_of_display_object(&world, root, root), -1);
    assert_eq!(group::get_display_object_at_index(&world, root, 0), Some(first));
    assert_eq!(group::get_display_object_at_index(&world, root, 3), None);
    assert_eq!(group::get_display_object_at_index(&world, root, -1), None);
    assert_eq!(display::object(&world, third).display_group, Some(root));
}

#[test]
fn insert_at_index_appends_past_the_end() {
    let mut world = new_world();
    let root = group::spawn_group(&mut world, Some("Root"));
    let a = spawn_object(&mut world, "A");
    let b = spawn_object(&mut world, "B");
    let c = spawn_object(&mut world, "C");
    let d = spawn_object(&mut world, "D");

    group::add_display_object_at_index(&mut world, root, Some(a), 5);
    group::add_display_object_at_index(&mut world, root, Some(b), 0);
    group::add_display_object_at_index(&mut world, root, Some(c), 1);
    group::add_display_object_at_index(&mut world, root, Some(d), 3);
    assert_eq!(group::children(&world, root), &[b, c, a, d]);

    let e = spawn_object(&mut world, "E");
    display::insert_after(&mut world, c, Some(e));
    assert_eq!(group::children(&world, root), &[b, c, e, a, d]);
    display::insert_before(&mut world, b, Some(d));
    assert_eq!(group::children(&world, root), &[d, b, c, e, a]);
}

#[test]
fn removal_drops_every_occurrence_and_unparents() {
    let mut world = new_world();
    let root = group::spawn_group(&mut world, Some("Root"));
    let other = group::spawn_group(&mut world, Some("Other"));
    let child = spawn_object(&mut world, "Child");
    let last = spawn_object(&mut world, "Last");

    group::add_display_object(&mut world, root, Some(child));
    group::add_display_object(&mut world, root, Some(child));
    group::add_display_object(&mut world, other, Some(child));
    group::add_display_object(&mut world, root, Some(last));
    assert_eq!(group::children(&world, root), &[child, child, last]);
    assert_eq!(display::object(&world, child).display_group, Some(other));

    // Removing from the first parent unparents the child although it is still in `other`.
    group::remove_display_object(&mut world, root, Some(child));
    assert_eq!(group::children(&world, root), &[last]);
    assert_eq!(group::children(&world, other), &[child]);
    assert_eq!(display::object(&world, child).display_group, None);

    group::remove_display_object_at_index(&mut world, root, 7);
    assert!(group::children(&world, root).is_empty());
    assert_eq!(display::object(&world, last).display_group, None);
    group::remove_display_object_at_index(&mut world, root, 0);
}

#[test]
fn a_marker_holds_one_child() {
    let mut world = new_world();
    let root = group::spawn_group(&mut world, Some("Root"));
    let marker = group::spawn_marker(&mut world, Some("Marker"));
    group::add_display_object(&mut world, root, Some(marker));
    view::set_active_display_group(&mut world, Some(root));

    let first = group::spawn_group(&mut world, Some("First"));
    let second = group::spawn_group(&mut world, Some("Second"));
    group::add_display_object(&mut world, marker, Some(first));
    assert!(!hibernating(&world, first));
    assert_eq!(ref_count(&world, first), 1);

    group::add_display_object_at_index(&mut world, marker, Some(second), 0);
    assert_eq!(group::children(&world, marker), &[second]);
    assert_eq!(display::object(&world, first).display_group, None);
    assert!(hibernating(&world, first));
    assert!(!hibernating(&world, second));

    group::add_display_object(&mut world, marker, None);
    assert!(group::children(&world, marker).is_empty());
    assert!(hibernating(&world, second));
}

#[test]
fn inserting_nil_next_to_a_marker_child_empties_the_marker() {
    let mut world = new_world();
    let marker = group::spawn_marker(&mut world, Some("Marker"));
    let child = spawn_object(&mut world, "Child");
    group::add_display_object(&mut world, marker, Some(child));

    display::insert_after(&mut world, child, None);
    assert!(group::children(&world, marker).is_empty());
    assert_eq!(display::object(&world, child).display_group, None);

    let root = group::spawn_group(&mut world, Some("Root"));
    let kept = spawn_object(&mut world, "Kept");
    group::add_display_object(&mut world, root, Some(kept));
    display::insert_before(&mut world, kept, None);
    assert_eq!(group::children(&world, root), &[kept]);
}

#[test]
fn hibernation_is_reference_counted_and_recursive() {
    let mut world = new_world();
    let root = group::spawn_group(&mut world, Some("Root"));
    let layer = group::spawn_group(&mut world, Some("Layer"));
    let shared = spawn_object(&mut world, "Shared");
    group::add_display_object(&mut world, root, Some(layer));
    group::add_display_object(&mut world, layer, Some(shared));
    group::add_display_object(&mut world, root, Some(shared));
    assert!(hibernating(&world, shared));
    assert_eq!(ref_count(&world, shared), 0);

    view::set_active_display_group(&mut world, Some(root));
    assert!(!hibernating(&world, root));
    assert!(!hibernating(&world, layer));
    assert_eq!(ref_count(&world, shared), 2);

    group::remove_display_object(&mut world, layer, Some(shared));
    assert_eq!(ref_count(&world, shared), 1);
    assert!(!hibernating(&world, shared));

    view::set_active_display_group(&mut world, None);
    assert!(hibernating(&world, root));
    assert!(hibernating(&world, layer));
    assert!(hibernating(&world, shared));
    assert_eq!(ref_count(&world, shared), 0);
    assert_eq!(world.resource::<BcView>().active_display_group, None);
}

#[test]
fn hibernating_below_zero_is_not_clamped() {
    let mut world = new_world();
    let object = spawn_object(&mut world, "Object");
    display::hibernate(&mut world, object);
    assert_eq!(ref_count(&world, object), -1);
    assert!(hibernating(&world, object));

    display::unhibernate(&mut world, object);
    assert_eq!(ref_count(&world, object), 0);
    assert!(hibernating(&world, object));

    display::unhibernate(&mut world, object);
    assert_eq!(ref_count(&world, object), 1);
    assert!(!hibernating(&world, object));
}

#[test]
fn setting_the_same_active_group_twice_does_nothing() {
    let mut world = new_world();
    let root = group::spawn_group(&mut world, Some("Root"));
    view::set_active_display_group(&mut world, Some(root));
    view::set_active_display_group(&mut world, Some(root));
    assert_eq!(ref_count(&world, root), 1);

    let overlay = group::spawn_group(&mut world, Some("Overlay"));
    view::set_wide_screen_overlay_display_group(&mut world, Some(overlay));
    view::set_tall_screen_overlay_display_group(&mut world, Some(overlay));
    assert_eq!(ref_count(&world, overlay), 2);
    view::set_wide_screen_overlay_display_group(&mut world, None);
    assert_eq!(ref_count(&world, overlay), 1);
    assert!(!hibernating(&world, overlay));
}

#[test]
fn selectable_logics_are_collected_in_pre_order() {
    let mut world = new_world();
    let root = group::spawn_group(&mut world, Some("Root"));
    let layer = group::spawn_group(&mut world, Some("Layer"));
    let marker = group::spawn_marker(&mut world, Some("Marker"));
    let hidden = spawn_object(&mut world, "Hidden");
    let plain = spawn_object(&mut world, "Plain");
    let last = spawn_object(&mut world, "Last");
    group::add_display_object(&mut world, root, Some(layer));
    group::add_display_object(&mut world, layer, Some(hidden));
    group::add_display_object(&mut world, layer, Some(plain));
    group::add_display_object(&mut world, root, Some(marker));
    group::add_display_object(&mut world, marker, Some(last));

    let logics: Vec<Entity> = (0..5).map(|_| world.spawn_empty().id()).collect();
    let owners = [layer, hidden, plain, last, root];
    for (owner, logic) in owners.iter().zip(&logics) {
        display::object_mut(&mut world, *owner).logic = Some(*logic);
    }
    for selectable in [layer, hidden, last] {
        display::object_mut(&mut world, selectable).is_selectable = true;
    }
    display::object_mut(&mut world, hidden).hidden = true;

    let mut selectable_logics = Vec::new();
    display::add_selectable_logics(&world, root, &mut selectable_logics);
    assert_eq!(selectable_logics, vec![logics[0], logics[1], logics[3]]);

    let mut all_logics = Vec::new();
    group::add_logics_to_array(&world, root, &mut all_logics);
    assert_eq!(all_logics, vec![logics[0], logics[1], logics[2], logics[3]]);
}
