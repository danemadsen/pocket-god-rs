//! Origin-in-place setters, bounding boxes, view ↔ display-object transforms and hit testing
//! (`docs/re/rendering.md` §4.1, §9.3, §9.4).

use bevy_ecs::prelude::*;
use bevy_math::Vec4;
use pg_engine::display::{self, BC_DISPLAY_OBJECT, group};
use pg_engine::view::{self, BcView};

fn new_world(width: i32, height: i32) -> World {
    let mut world = World::new();
    world.init_resource::<BcView>();
    view::configure_screen(&mut world, width, height, width as f32, height as f32);
    world
}

/// A root group with the standard transform (`xPos = -160, yPos = 240, zRot = -90`).
fn spawn_root(world: &mut World) -> Entity {
    let root = group::spawn_group(world, Some("Root"));
    let mut display = display::object_mut(world, root);
    display.pos.x = -160.0;
    display.pos.y = 240.0;
    display.rot.z = -90.0;
    root
}

fn spawn_child(world: &mut World, parent: Entity, x: f32, y: f32) -> Entity {
    let child = display::spawn(world, &BC_DISPLAY_OBJECT, Some("Child"));
    group::add_display_object(world, parent, Some(child));
    let mut display = display::object_mut(world, child);
    display.pos.x = x;
    display.pos.y = y;
    child
}

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-3, "{actual} != {expected}");
}

#[test]
fn origin_in_place_keeps_the_geometry_still() {
    let mut world = new_world(480, 320);
    let object = display::spawn(&mut world, &BC_DISPLAY_OBJECT, None);
    {
        let mut display = display::object_mut(&mut world, object);
        display.pos = Vec4::new(100.0, 50.0, -277.0, 1.0);
        display.scale = Vec4::new(-1.0, 2.0, 1.0, 1.0);
        display.origin_pos = Vec4::new(-10.0, -20.0, 0.0, 1.0);
    }

    display::set_origin_x_pos_in_place(&mut world, object, -30.0);
    display::set_origin_y_pos_in_place(&mut world, object, -25.0);
    let display_state = display::object(&world, object).clone();
    assert_eq!(display_state.pos.x, 80.0);
    assert_eq!(display_state.pos.y, 55.0);
    assert_eq!(display_state.origin_pos.x, -30.0);
    assert_eq!(display_state.origin_pos.y, -25.0);

    // The z variant tests the sign of scale.y (original bug).
    display::object_mut(&mut world, object).scale.y = -2.0;
    display::set_origin_z_pos_in_place(&mut world, object, 5.0);
    assert_eq!(display::object(&world, object).pos.z, -272.0);

    let origin = Vec4::new(-40.0, -15.0, 6.0, 0.5);
    display::set_origin_pos_in_place(&mut world, object, origin);
    let display_state = display::object(&world, object).clone();
    assert_eq!(display_state.pos.x, 70.0);
    assert_eq!(display_state.pos.y, 65.0);
    assert_eq!(display_state.pos.z, -273.0);
    assert_eq!(display_state.origin_pos, origin);
}

#[test]
fn bounding_boxes_follow_the_scale_sign() {
    let mut world = new_world(480, 320);
    let object = display::spawn(&mut world, &BC_DISPLAY_OBJECT, None);
    {
        let mut display = display::object_mut(&mut world, object);
        display.pos = Vec4::new(100.0, 50.0, -277.0, 1.0);
        display.scale = Vec4::new(-2.0, 1.0, 1.0, 1.0);
        display.origin_pos = Vec4::new(-5.0, -10.0, 0.0, 1.0);
        display.col_x_min = -3.0;
        display.col_x_max = 7.0;
        display.col_y_min = -1.0;
        display.col_y_max = 4.0;
        display.rot.z = 90.0;
    }

    let min = display::min_pos(&mut world, object);
    assert_eq!(min, Vec4::new(96.0, 39.0, -277.0, 1.0));
    let max = display::max_pos(&mut world, object);
    assert_eq!(max, Vec4::new(116.0, 44.0, -277.0, 1.0));
    assert_eq!(display::object(&world, object).min_pos, min);

    // Local min corner (-4, -11) rotated by +90° about pos.
    let rotated_min = display::rotated_min_pos(&mut world, object);
    assert_close(rotated_min.x, 111.0);
    assert_close(rotated_min.y, 46.0);
    assert_eq!(rotated_min.z, 0.0);
    assert_eq!(rotated_min.w, 1.0);

    // Local max corner (16, -6).
    let rotated_max = display::rotated_max_pos(&mut world, object);
    assert_close(rotated_max.x, 106.0);
    assert_close(rotated_max.y, 66.0);
}

#[test]
fn view_points_map_to_game_coordinates() {
    let mut world = new_world(480, 320);
    let root = spawn_root(&mut world);
    let child = spawn_child(&mut world, root, 0.0, 0.0);

    let top_left = display::view_to_display_object(&world, child, Vec4::new(0.0, 0.0, 0.0, 1.0));
    assert_close(top_left.x, 0.0);
    assert_close(top_left.y, 320.0);
    let bottom_right = display::view_to_display_object(&world, child, Vec4::new(480.0, 320.0, 0.0, 1.0));
    assert_close(bottom_right.x, 480.0);
    assert_close(bottom_right.y, 0.0);

    let mut wide_world = new_world(568, 320);
    let wide_root = spawn_root(&mut wide_world);
    let wide_child = spawn_child(&mut wide_world, wide_root, 0.0, 0.0);
    let wide_corner = display::view_to_display_object(&wide_world, wide_child, Vec4::new(0.0, 0.0, 0.0, 1.0));
    assert_close(wide_corner.x, -44.0);
    assert_close(wide_corner.y, 320.0);
}

#[test]
fn display_points_map_back_to_view_points() {
    let mut world = new_world(568, 320);
    let root = spawn_root(&mut world);
    let child = spawn_child(&mut world, root, 0.0, 0.0);

    let view_point = display::display_object_to_view(&world, child, Vec4::new(240.0, 160.0, 9.0, 3.0));
    assert_close(view_point.x, 284.0);
    assert_close(view_point.y, 160.0);
    assert_eq!(view_point.w, 1.0);

    let game_point = display::view_to_display_object(&world, child, view_point);
    assert_close(game_point.x, 240.0);
    assert_close(game_point.y, 160.0);
}

#[test]
fn hit_test_is_strict_and_mirrors_with_the_scale() {
    let mut world = new_world(480, 320);
    let root = spawn_root(&mut world);
    let target = spawn_child(&mut world, root, 100.0, 100.0);
    {
        let mut display = display::object_mut(&mut world, target);
        display.col_x_min = -10.0;
        display.col_x_max = 20.0;
        display.col_y_min = -5.0;
        display.col_y_max = 5.0;
        display.scale.x = 3.0;
    }

    // Game (x, y) is view (x, 320 - y) on 480 × 320.
    let inside = Vec4::new(115.0, 220.0, 0.0, 1.0);
    let left_of_box = Vec4::new(85.0, 220.0, 0.0, 1.0);
    let above_box = Vec4::new(100.0, 205.0, 0.0, 1.0);

    assert!(!display::does_point_intersect(&world, target, inside, true));
    assert!(display::does_point_intersect(&world, target, inside, false));
    display::object_mut(&mut world, target).is_selectable = true;
    assert!(display::does_point_intersect(&world, target, inside, true));
    assert!(!display::does_point_intersect(&world, target, left_of_box, true));
    assert!(!display::does_point_intersect(&world, target, above_box, true));

    // A mirrored box covers -20 … 10.
    display::object_mut(&mut world, target).scale.x = -1.0;
    assert!(!display::does_point_intersect(&world, target, inside, true));
    assert!(display::does_point_intersect(&world, target, left_of_box, true));
}
