//! `BCView`: screen configuration, scissor, accelerometer filters, game-frame listeners and the
//! draw of the active group and overlays (`docs/re/rendering.md` §1.4–§1.7, §2, §10).

use bevy_ecs::prelude::*;
use bevy_math::{Vec3, Vec4};
use pg_engine::display::{self, group};
use pg_engine::gl::GlContext;
use pg_engine::object::Class;
use pg_engine::view::{self, BcView, MotionInput};

fn new_world() -> World {
    let mut world = World::new();
    world.init_resource::<BcView>();
    world
}

#[test]
fn screen_flags_and_scissor_follow_the_device_table() {
    let mut world = new_world();
    view::configure_screen(&mut world, 480, 320, 480.0, 320.0);
    {
        let view = world.resource::<BcView>();
        assert!(!view.wide_screen && !view.tall_screen);
        assert_eq!((view.screen_width, view.screen_height), (480.0, 320.0));
        assert_eq!(view::game_area_scissor(view), None);
        assert_eq!(view::frustum_center_x_pos(view), 240.0);
    }

    view::configure_screen(&mut world, 568, 320, 568.0, 320.0);
    {
        let view = world.resource::<BcView>();
        assert!(view.wide_screen && !view.tall_screen);
        assert_eq!((view.screen_width, view.screen_height), (568.0, 320.0));
        assert_eq!((view.scissor_x_pos, view.scissor_y_pos), (44.0, 0.0));
        assert_eq!((view.scissor_width, view.scissor_height), (480.0, 320.0));
        let scissor = view::game_area_scissor(view).expect("wide screens are scissored");
        assert!((scissor.min_x - (88.0 / 568.0 - 1.0)).abs() < 1e-6);
        assert!((scissor.max_x - (1048.0 / 568.0 - 1.0)).abs() < 1e-6);
        assert_eq!((scissor.min_y, scissor.max_y), (-1.0, 1.0));
    }

    view::configure_screen(&mut world, 1024, 768, 1024.0, 768.0);
    {
        let view = world.resource::<BcView>();
        assert!(!view.wide_screen && view.tall_screen);
        assert_eq!(view.screen_width, 480.0);
        assert!((view.screen_height - 360.0).abs() < 1e-3);
        // Rounded scissor: (0, 42, 1024, 683).
        let scissor = view::game_area_scissor(view).expect("tall screens are scissored");
        assert_eq!((scissor.min_x, scissor.max_x), (-1.0, 1.0));
        assert!((scissor.min_y - (84.0 / 768.0 - 1.0)).abs() < 1e-6);
        assert!((scissor.max_y - (1450.0 / 768.0 - 1.0)).abs() < 1e-6);
    }

    assert_eq!(idle_gl(&world).units_per_pixel, 480.0 / 1024.0);
}

#[test]
fn accelerometer_filters_match_the_original() {
    let mut world = new_world();
    world.insert_resource(MotionInput {
        acceleration: Vec3::new(-1.0, 0.5, 0.25),
    });

    view::read_accelerometer(&mut world);
    {
        let view = world.resource::<BcView>();
        let expected_raw_x = (-f64::from(0.7_f32)) as f32;
        assert_eq!(view.raw_accel_force.x, expected_raw_x);
        assert_eq!(view.accel_force.x, expected_raw_x);
        assert_eq!(view.raw_accel_force.w, 1.0);
        let raw = view.raw_accel_force;
        let expected_total = (raw.x * raw.x + raw.y * raw.y + raw.z * raw.z).sqrt();
        assert_eq!(view.total_raw_accel_force, expected_total);
        let expected_degrees = raw.y.atan2(raw.x) / std::f32::consts::PI * 180.0;
        assert_eq!(view.raw_accel_rot_degrees.z, expected_degrees);
    }

    // A rising x force is smoothed at 0.1 (for all three components); the raw one is not.
    world.insert_resource(MotionInput {
        acceleration: Vec3::new(1.0, 0.0, 0.0),
    });
    let before = world.resource::<BcView>().accel_force;
    view::read_accelerometer(&mut world);
    let view = world.resource::<BcView>();
    let factor = f64::from(0.1_f32);
    let expected_x = (f64::from(before.x) + (1.0 - f64::from(before.x)) * factor) as f32;
    let expected_y = (f64::from(before.y) + (0.0 - f64::from(before.y)) * factor) as f32;
    assert_eq!(view.accel_force.x, expected_x);
    assert_eq!(view.accel_force.y, expected_y);
    assert!(view.raw_accel_force.x > view.accel_force.x);
}

#[test]
fn no_motion_input_means_no_tilt() {
    let mut world = new_world();
    view::read_accelerometer(&mut world);
    let view = world.resource::<BcView>();
    assert_eq!(view.raw_accel_force, Vec4::new(0.0, 0.0, 0.0, 1.0));
    assert_eq!(view.total_raw_accel_force, 0.0);
    assert_eq!(view.raw_accel_rot_degrees.x, 0.0);
}

#[derive(Resource, Default)]
struct FrameLog(Vec<Entity>);

#[derive(Resource)]
struct Plan {
    removed: Entity,
    added: Entity,
}

fn record(world: &mut World, this: Entity, _dt: f32) {
    world.resource_mut::<FrameLog>().0.push(this);
}

fn record_and_reschedule(world: &mut World, this: Entity, dt: f32) {
    record(world, this, dt);
    let Some(plan) = world.get_resource::<Plan>() else {
        return;
    };
    let removed = plan.removed;
    let added = plan.added;
    world.remove_resource::<Plan>();
    view::remove_game_frame_listener(world, removed);
    view::add_game_frame_listener(world, added);
}

const EARLY: Class = Class {
    name: "Early",
    game_frame_order: 10,
    on_game_frame: record,
    ..Class::NS_OBJECT
};

const EARLY_RESCHEDULING: Class = Class {
    name: "EarlyRescheduling",
    game_frame_order: 10,
    on_game_frame: record_and_reschedule,
    ..Class::NS_OBJECT
};

const LATE: Class = Class {
    name: "Late",
    game_frame_order: 40,
    on_game_frame: record,
    ..Class::NS_OBJECT
};

#[test]
fn listeners_run_in_stable_order_from_a_snapshot() {
    let mut world = new_world();
    world.init_resource::<FrameLog>();
    let late_a = world.spawn(pg_engine::object::Isa(&LATE)).id();
    let early_a = world.spawn(pg_engine::object::Isa(&EARLY_RESCHEDULING)).id();
    let late_b = world.spawn(pg_engine::object::Isa(&LATE)).id();
    let early_b = world.spawn(pg_engine::object::Isa(&EARLY)).id();
    let newcomer = world.spawn(pg_engine::object::Isa(&EARLY)).id();
    for listener in [late_a, early_a, late_b, early_b] {
        view::add_game_frame_listener(&mut world, listener);
    }
    world.insert_resource(Plan {
        removed: late_b,
        added: newcomer,
    });

    view::game_loop(&mut world, 1.0 / 60.0);
    assert_eq!(world.resource::<FrameLog>().0, vec![early_a, early_b, late_a, late_b]);

    world.resource_mut::<FrameLog>().0.clear();
    view::game_loop(&mut world, 1.0 / 60.0);
    assert_eq!(world.resource::<FrameLog>().0, vec![early_a, early_b, newcomer, late_a]);
    assert_eq!(world.resource::<BcView>().game_frame_count, 2);
}

/// A drawable covering far more than the screen, in the current GL colour.
fn draw_big_quad(_world: &mut World, _this: Entity, gl: &mut GlContext) {
    let vertices = [[-1000.0, 1000.0, -1.0], [1000.0, 1000.0, -1.0], [-1000.0, -1000.0, -1.0], [1000.0, -1000.0, -1.0]];
    gl.draw_strip(&vertices, &[]);
}

fn draw_red_quad(world: &mut World, this: Entity, gl: &mut GlContext) {
    gl.set_color(1.0, 0.0, 0.0, 1.0);
    draw_big_quad(world, this, gl);
}

fn draw_green_quad(world: &mut World, this: Entity, gl: &mut GlContext) {
    gl.set_color(0.0, 1.0, 0.0, 1.0);
    draw_big_quad(world, this, gl);
}

const RED_QUAD: Class = Class {
    name: "RedQuad",
    draw: draw_red_quad,
    ..display::BC_DISPLAY_OBJECT
};

const GREEN_QUAD: Class = Class {
    name: "GreenQuad",
    draw: draw_green_quad,
    ..display::BC_DISPLAY_OBJECT
};

fn spawn_root_with(world: &mut World, class: &'static Class) -> Entity {
    let root = group::spawn_group(world, None);
    let quad = display::spawn(world, class, None);
    group::add_display_object(world, root, Some(quad));
    root
}

fn idle_gl(world: &World) -> &GlContext {
    let view = world.resource::<BcView>();
    view.gl.as_ref().expect("the GL context is idle")
}

fn horizontal_extent(world: &World, color: [f32; 4]) -> (f32, f32) {
    let gl = idle_gl(world);
    let mut extent = (f32::MAX, f32::MIN);
    for batch in &gl.list.batches {
        for vertex in batch.vertices.iter().filter(|vertex| vertex.color == color) {
            let ndc_x = vertex.position[0] / vertex.position[3];
            extent.0 = extent.0.min(ndc_x);
            extent.1 = extent.1.max(ndc_x);
        }
    }
    extent
}

#[test]
fn the_active_group_is_scissored_and_the_overlay_is_not() {
    let mut world = new_world();
    view::configure_screen(&mut world, 568, 320, 568.0, 320.0);
    let root = spawn_root_with(&mut world, &RED_QUAD);
    let overlay = spawn_root_with(&mut world, &GREEN_QUAD);
    view::set_active_display_group(&mut world, Some(root));
    view::set_wide_screen_overlay_display_group(&mut world, Some(overlay));
    view::set_tall_screen_overlay_display_group(&mut world, Some(overlay));

    world.resource_mut::<BcView>().clear_color = [0.0, 0.0, 1.0, 1.0];
    view::game_loop(&mut world, 1.0 / 60.0);

    let (red_min, red_max) = horizontal_extent(&world, [1.0, 0.0, 0.0, 1.0]);
    assert!((red_min + 480.0 / 568.0).abs() < 1e-4, "{red_min}");
    assert!((red_max - 480.0 / 568.0).abs() < 1e-4, "{red_max}");
    // Unclipped: the quad spans ±1000 eye units = ±1000/284 in NDC.
    let (green_min, green_max) = horizontal_extent(&world, [0.0, 1.0, 0.0, 1.0]);
    assert!(green_min < -3.5 && green_max > 3.5, "{green_min} {green_max}");

    let gl = idle_gl(&world);
    assert_eq!(gl.list.clear_color, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(gl.color(), [0.0, 1.0, 0.0, 1.0]);
}

fn set_clear_color_during_update(world: &mut World, _this: Entity, _dt: f32) {
    world.resource_mut::<BcView>().clear_color = [0.5, 0.5, 0.5, 1.0];
}

const SKY: Class = Class {
    name: "Sky",
    on_game_frame: set_clear_color_during_update,
    ..Class::NS_OBJECT
};

#[test]
fn a_clear_colour_set_during_a_tick_shows_on_the_next_one() {
    let mut world = new_world();
    let sky = world.spawn(pg_engine::object::Isa(&SKY)).id();
    view::add_game_frame_listener(&mut world, sky);

    view::game_loop(&mut world, 1.0 / 60.0);
    assert_eq!(idle_gl(&world).list.clear_color, [1.0, 0.0, 0.0, 1.0]);

    view::game_loop(&mut world, 1.0 / 60.0);
    assert_eq!(idle_gl(&world).list.clear_color, [0.5, 0.5, 0.5, 1.0]);
}

#[test]
fn hidden_and_empty_groups_draw_nothing() {
    let mut world = new_world();
    let root = spawn_root_with(&mut world, &RED_QUAD);
    view::set_active_display_group(&mut world, Some(root));
    display::object_mut(&mut world, root).hidden = true;
    view::draw(&mut world);
    assert!(idle_gl(&world).list.batches.is_empty());

    let empty = group::spawn_group(&mut world, None);
    view::set_active_display_group(&mut world, Some(empty));
    view::draw(&mut world);
    assert!(idle_gl(&world).list.batches.is_empty());

    // A marker draws its child even when the marker itself is hidden.
    let marker_root = group::spawn_group(&mut world, None);
    let marker = group::spawn_marker(&mut world, None);
    let quad = display::spawn(&mut world, &RED_QUAD, None);
    group::add_display_object(&mut world, marker_root, Some(marker));
    group::add_display_object(&mut world, marker, Some(quad));
    display::object_mut(&mut world, marker).hidden = true;
    view::set_active_display_group(&mut world, Some(marker_root));
    view::draw(&mut world);
    assert!(!idle_gl(&world).list.batches.is_empty());
}

#[test]
fn starting_the_render_loop_restarts_game_time_once() {
    let mut world = new_world();
    world.resource_mut::<BcView>().time = 12.5;

    view::start_render_loop(&mut world);
    {
        let view = world.resource::<BcView>();
        assert!(view.render_loop_active && view.render_loop_restarted && view.accelerometer_enabled);
        assert_eq!(view.time, 0.0);
    }

    // A second start while running changes nothing (the original returns early).
    let mut view = world.resource_mut::<BcView>();
    view.render_loop_restarted = false;
    view.time = 3.0;
    view::start_render_loop(&mut world);
    assert!(!world.resource::<BcView>().render_loop_restarted);
    assert_eq!(world.resource::<BcView>().time, 3.0);

    view::stop_render_loop(&mut world);
    assert!(!view::is_render_loop_active(&world));
    assert!(!world.resource::<BcView>().accelerometer_enabled);
}
